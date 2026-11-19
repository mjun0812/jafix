use std::cell::OnceCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;

use crate::filter::Filters;
use crate::rules::builtin::Builtin;
use crate::rules::general::UrlChecker;

/// 組み込みの既定値に共通設定とプロジェクト設定を適用した結果
#[derive(Default)]
pub struct Config {
    /// symlinkを解決した実体の絶対パスと比べるglob
    exclude: GlobSet,
    /// `exclude` と同じく実体の絶対パスと比べるglob。当たるファイルでは指摘ルールの指摘を出さない
    report_exclude: GlobSet,
    /// `[builtin]` で決めた、組み込みのルールの有効・無効とオプション
    builtin: Builtin,
    /// `[filters]` で決めた、filterの有効・無効とオプション
    filters: Filters,
    /// `no-dead-link` の外部URLの確認結果は、同じ設定を使うファイル間で共有する
    urls: OnceCell<UrlChecker>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    exclude: Option<Vec<String>>,
    #[serde(rename = "report-exclude")]
    report_exclude: Option<Vec<String>>,
    builtin: Option<toml::Table>,
    filters: Option<toml::Table>,
}

struct ConfigFile {
    path: PathBuf,
    raw: RawConfig,
}

/// 共通設定を読み、同じプロジェクトに属するファイル間で設定を共有する
pub struct Configs {
    home: Option<PathBuf>,
    global: Option<ConfigFile>,
    resolved: HashMap<Option<PathBuf>, Result<Config, String>>,
}

impl Configs {
    pub fn load() -> Result<Self, String> {
        let home = std::env::home_dir();
        let config_dir = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|home| home.join(".config")));
        let global = match config_dir {
            Some(dir) => ConfigFile::read(dir.join("jafix/config.toml"))?,
            None => None,
        };
        Ok(Self {
            home,
            global,
            resolved: HashMap::new(),
        })
    }

    pub fn for_path(&mut self, path: &Path) -> Result<&Config, String> {
        let canonical = path.canonicalize().ok();
        self.for_project(canonical.as_deref().and_then(project_config_path))
    }

    /// インデックスのファイルは、作業ツリーで削除されたりsymlinkに置き換わったりしていてもGit上のパスで探す。
    pub fn for_staged_path(&mut self, path: &Path) -> Result<&Config, String> {
        self.for_project(project_config_path(path))
    }

    fn for_project(&mut self, project_config: Option<PathBuf>) -> Result<&Config, String> {
        self.resolved
            .entry(project_config)
            .or_insert_with_key(|project| {
                let project = match project {
                    Some(path) => ConfigFile::read(path.clone())?,
                    None => None,
                };
                let mut config = Config::default();
                if let Some(global) = &self.global {
                    global.apply(&mut config, self.home.as_deref(), false)?;
                }
                if let Some(project) = &project {
                    project.apply(&mut config, self.home.as_deref(), true)?;
                }
                config.builtin.validate().map_err(|reason| {
                    let file = project.as_ref().or(self.global.as_ref());
                    match file {
                        Some(file) => format!("{}: [builtin]: {reason}", file.path.display()),
                        None => format!("[builtin]: {reason}"),
                    }
                })?;
                Ok(config)
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

fn project_config_path(path: &Path) -> Option<PathBuf> {
    for dir in path.ancestors().skip(1) {
        let config = dir.join(".jafix.toml");
        match config.symlink_metadata() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Some(config),
        }
        // 通常のリポジトリではディレクトリ、worktreeではファイルになる。
        if dir.join(".git").exists() {
            break;
        }
    }
    None
}

impl ConfigFile {
    fn read(path: PathBuf) -> Result<Option<Self>, String> {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let raw = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Some(Self { path, raw }))
    }

    fn apply(&self, config: &mut Config, home: Option<&Path>, project: bool) -> Result<(), String> {
        let dir = self.path.parent().unwrap();
        let project_dir = project.then_some(dir);
        let at_path = |reason| format!("{}: {reason}", self.path.display());
        if let Some(values) = &self.raw.exclude {
            config.exclude = glob_set("exclude", values, home, project_dir).map_err(at_path)?;
        }
        if let Some(values) = &self.raw.report_exclude {
            config.report_exclude =
                glob_set("report-exclude", values, home, project_dir).map_err(at_path)?;
        }
        if let Some(table) = &self.raw.builtin {
            config
                .builtin
                .apply_table(table, dir)
                .map_err(|reason| at_path(format!("[builtin]: {reason}")))?;
        }
        if let Some(table) = &self.raw.filters {
            config
                .filters
                .apply_table(table, dir)
                .map_err(|reason| at_path(format!("[filters]: {reason}")))?;
        }
        Ok(())
    }
}

/// 設定の `key` の値をglobとして読み、1つの `GlobSet` にまとめる
fn glob_set(
    key: &str,
    values: &[String],
    home: Option<&Path>,
    project_dir: Option<&Path>,
) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for value in values {
        let pattern = resolve_literal_prefix(value, home, project_dir)
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
fn resolve_literal_prefix(
    value: &str,
    home: Option<&Path>,
    project_dir: Option<&Path>,
) -> Result<String, String> {
    #[cfg(windows)]
    let normalized = value.replace('\\', "/");
    #[cfg(windows)]
    let value = normalized.as_str();

    let path = Path::new(value);
    let (base, rest) = if let Some(rest) = value.strip_prefix("~/") {
        let home = home.ok_or("ホームディレクトリが分からないため `~/` を展開できない")?;
        (home.to_path_buf(), rest)
    } else if path.is_absolute() {
        let root = path.ancestors().last().unwrap();
        let rest = path.strip_prefix(root).unwrap().to_str().unwrap();
        (root.to_path_buf(), rest)
    } else if let Some(dir) = project_dir {
        (dir.to_path_buf(), value)
    } else if value.starts_with("**/") {
        return Ok(value.to_owned());
    } else {
        return Err("絶対パスか、`~/`・`**/` で始まるパターンを指定する".to_owned());
    };
    let components: Vec<&str> = rest.split('/').collect();
    let literal_len = components
        .iter()
        .position(|component| component.contains(['*', '?', '[', '{']))
        .unwrap_or(components.len());
    let literal = base.join(components[..literal_len].join("/"));
    // まだ無いパスは、照らし合わせる実体が無いのでそのまま使う
    let literal = literal.canonicalize().unwrap_or(literal);
    let literal = literal.to_string_lossy();
    #[cfg(windows)]
    let literal = literal.replace('\\', "/");
    let mut pattern = globset::escape(&literal);
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

    pub fn is_staged_path_excluded(&self, path: &Path) -> bool {
        self.exclude.is_match(path)
    }

    pub fn is_staged_report_excluded(&self, path: &Path) -> bool {
        self.report_exclude.is_match(path)
    }

    pub fn builtin(&self) -> &Builtin {
        &self.builtin
    }

    pub fn filters(&self) -> &Filters {
        &self.filters
    }

    pub fn url_checker(&self) -> Option<&UrlChecker> {
        self.builtin
            .no_dead_link
            .as_ref()
            .map(|_| self.urls.get_or_init(UrlChecker::default))
    }
}

fn matches_canonical(globs: &GlobSet, path: &Path) -> bool {
    path.canonicalize()
        .is_ok_and(|canonical| globs.is_match(canonical))
}
