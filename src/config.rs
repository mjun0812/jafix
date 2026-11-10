use std::path::{Path, PathBuf};

use serde::Deserialize;

/// `$XDG_CONFIG_HOME/jafix/config.toml` (未設定なら `~/.config/jafix/config.toml`) の内容
#[derive(Debug, Default)]
pub struct Config {
    /// symlinkを解決した実体のパス
    exclude: Vec<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    exclude: Vec<String>,
}

/// 設定ファイルを読む。無ければ空の設定を返し、TOMLとして読めないか未知のキーがあれば理由を返す
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
    let mut exclude = Vec::new();
    for value in raw.exclude {
        let expanded = match value.strip_prefix("~/") {
            Some(rest) => match &home {
                Some(home) => home.join(rest),
                None => {
                    return Err(format!(
                        "{}: ホームディレクトリが分からないため `{value}` を展開できない",
                        path.display()
                    ));
                }
            },
            None => PathBuf::from(value),
        };
        // まだ無いファイルは、照らし合わせる実体が無いので外す
        if let Ok(canonical) = expanded.canonicalize() {
            exclude.push(canonical);
        }
    }
    Ok(Config { exclude })
}

impl Config {
    /// `path` の実体が `exclude` のいずれかと同じならtrue
    pub fn is_excluded(&self, path: &Path) -> bool {
        path.canonicalize()
            .is_ok_and(|canonical| self.exclude.contains(&canonical))
    }
}
