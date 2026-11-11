use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;

use crate::rules::{CustomRule, RuleId};

/// `$XDG_CONFIG_HOME/jafix/config.toml` (未設定なら `~/.config/jafix/config.toml`) の内容
#[derive(Debug, Default)]
pub struct Config {
    /// symlinkを解決した実体の絶対パスと比べるglob
    exclude: GlobSet,
    /// `exclude` と同じく実体の絶対パスと比べるglob。当たるファイルでは指摘ルールの指摘を出さない
    report_exclude: GlobSet,
    /// `[[rules]]` で足した指摘ルール
    rules: Vec<CustomRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default, rename = "report-exclude")]
    report_exclude: Vec<String>,
    #[serde(default)]
    rules: Vec<RawRule>,
}

/// `[[rules]]` の1つの表。不正の理由にその表の `id` を含めるため、serdeの必須のフィールドと
/// `deny_unknown_fields` に任せず、どのキーも省略可として読んでから `custom_rule` で確かめる
#[derive(Deserialize)]
struct RawRule {
    id: Option<String>,
    message: Option<String>,
    pattern: Option<String>,
    words: Option<Vec<String>>,
    #[serde(flatten)]
    unknown: toml::Table,
}

/// 設定ファイルを読む。無ければ空の設定を返し、TOMLとして読めないか値が不正なら理由を返す
pub fn load() -> Result<Config, String> {
    let home = std::env::home_dir();
    let config_dir = match std::env::var_os("XDG_CONFIG_HOME").filter(|dir| !dir.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => match &home {
            Some(home) => home.join(".config"),
            None => return Ok(Config::default()),
        },
    };
    let path = config_dir.join("jafix/config.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let raw: RawConfig = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let exclude = glob_set("exclude", &raw.exclude, home.as_deref())
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let report_exclude = glob_set("report-exclude", &raw.report_exclude, home.as_deref())
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rules: Vec<CustomRule> = Vec::new();
    for raw_rule in raw.rules {
        let table = match &raw_rule.id {
            Some(id) => format!("[[rules]] の `{id}`"),
            None => "[[rules]]".to_owned(),
        };
        let rule = custom_rule(raw_rule, &rules)
            .map_err(|reason| format!("{}: {table}: {reason}", path.display()))?;
        rules.push(rule);
    }
    Ok(Config {
        exclude,
        report_exclude,
        rules,
    })
}

/// `[[rules]]` の1つの表を検証して指摘ルールにする。`defined` はそれより前の表から作った指摘ルール
fn custom_rule(raw: RawRule, defined: &[CustomRule]) -> Result<CustomRule, String> {
    if !raw.unknown.is_empty() {
        let keys: Vec<String> = raw.unknown.keys().map(|key| format!("`{key}`")).collect();
        return Err(format!("未知のキー {}", keys.join("、")));
    }
    let id = raw.id.ok_or("`id` が無い")?;
    let message = raw.message.ok_or("`message` が無い")?;
    if RuleId::ALL.iter().any(|rule| rule.name() == id) {
        return Err("`id` が組み込みのルールと同じ".to_owned());
    }
    if defined.iter().any(|rule| rule.id() == id) {
        return Err("`id` が他の `[[rules]]` と同じ".to_owned());
    }
    match (raw.pattern, raw.words) {
        (Some(pattern), None) => CustomRule::pattern(id, message, &pattern),
        (None, Some(words)) => CustomRule::words(id, message, &words),
        _ => Err("`pattern` と `words` のどちらか一方だけを書く".to_owned()),
    }
}

/// 設定の `key` の値をglobとして読み、1つの `GlobSet` にまとめる
fn glob_set(key: &str, values: &[String], home: Option<&Path>) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for value in values {
        let pattern = resolve_literal_prefix(value, home)
            .map_err(|reason| format!("{key} の `{value}`: {reason}"))?;
        let glob = GlobBuilder::new(&pattern)
            .literal_separator(true)
            .build()
            .map_err(|e| format!("{key} の `{value}` をglobとして読めない: {e}"))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| format!("{key} をglobとして読めない: {e}"))
}

/// 値のうちglobの特殊文字を含む最初の要素より前を、実体のパスに置き換えて特殊文字を文字そのものにしたglobを返す。
/// 照らし合わせる相手は実体のパスなので、symlinkを通る書き方の値も実体に当たるようにする
fn resolve_literal_prefix(value: &str, home: Option<&Path>) -> Result<String, String> {
    let (base, rest) = if let Some(rest) = value.strip_prefix("~/") {
        let home = home.ok_or("ホームディレクトリが分からないため `~/` を展開できない")?;
        (home.to_path_buf(), rest)
    } else if let Some(rest) = value.strip_prefix('/') {
        (PathBuf::from("/"), rest)
    } else if value.starts_with("**/") {
        return Ok(value.to_owned());
    } else {
        return Err("`/`・`~/`・`**/` のどれかで始める".to_owned());
    };
    let components: Vec<&str> = rest.split('/').collect();
    let literal_len = components
        .iter()
        .position(|component| component.contains(['*', '?', '[', '{']))
        .unwrap_or(components.len());
    let literal = base.join(components[..literal_len].join("/"));
    // まだ無いパスは、照らし合わせる実体が無いのでそのまま使う
    let literal = literal.canonicalize().unwrap_or(literal);
    let mut pattern = globset::escape(&literal.to_string_lossy());
    let glob_part = components[literal_len..].join("/");
    if !glob_part.is_empty() {
        if !pattern.ends_with('/') {
            pattern.push('/');
        }
        pattern.push_str(&glob_part);
    }
    Ok(pattern)
}

impl Config {
    /// `path` の実体の絶対パスが `exclude` のglobのいずれかに当たればtrue
    pub fn is_excluded(&self, path: &Path) -> bool {
        path.canonicalize()
            .is_ok_and(|canonical| self.exclude.is_match(canonical))
    }

    /// `path` の実体が `report-exclude` のglobのいずれかに当たればtrue。当たるファイルでは指摘ルールの指摘を出さない
    pub fn is_report_excluded(&self, path: &Path) -> bool {
        path.canonicalize()
            .is_ok_and(|canonical| self.report_exclude.is_match(canonical))
    }

    pub fn rules(&self) -> &[CustomRule] {
        &self.rules
    }
}
