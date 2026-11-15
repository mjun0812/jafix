use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;

use crate::rules::builtin::Builtin;

/// `$XDG_CONFIG_HOME/jafix/config.toml` (未設定なら `~/.config/jafix/config.toml`) の内容
#[derive(Debug, Default)]
pub struct Config {
    /// symlinkを解決した実体の絶対パスと比べるglob
    exclude: GlobSet,
    /// `exclude` と同じく実体の絶対パスと比べるglob。当たるファイルでは指摘ルールの指摘を出さない
    report_exclude: GlobSet,
    /// `[builtin]` で決めた、組み込みのルールの有効・無効とオプション
    builtin: Builtin,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default, rename = "report-exclude")]
    report_exclude: Vec<String>,
    builtin: Option<toml::Table>,
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
    let dir = config_dir.join("jafix");
    let path = dir.join("config.toml");
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
    let builtin = match &raw.builtin {
        Some(table) => Builtin::from_table(table, &dir)
            .map_err(|reason| format!("{}: [builtin]: {reason}", path.display()))?,
        None => Builtin::default(),
    };
    Ok(Config {
        exclude,
        report_exclude,
        builtin,
    })
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
        matches_canonical(&self.exclude, path)
    }

    /// `path` の実体が `report-exclude` のglobのいずれかに当たればtrue。当たるファイルでは指摘ルールの指摘を出さない
    pub fn is_report_excluded(&self, path: &Path) -> bool {
        matches_canonical(&self.report_exclude, path)
    }

    pub fn builtin(&self) -> &Builtin {
        &self.builtin
    }
}

fn matches_canonical(globs: &GlobSet, path: &Path) -> bool {
    path.canonicalize()
        .is_ok_and(|canonical| globs.is_match(canonical))
}
