use std::path::{Path, PathBuf};

/// `$XDG_CONFIG_HOME/jafix/config.toml` (未設定なら `~/.config/jafix/config.toml`) の内容
#[derive(Debug, Default)]
pub struct Config {
    /// symlinkを解決した実体のパス
    exclude: Vec<PathBuf>,
}

/// 設定ファイルを読む。無ければ空の設定を返し、TOMLとして読めないか未知のキーがあれば理由を返す
pub fn load() -> Result<Config, String> {
    todo!("skeleton: not implemented")
}

impl Config {
    /// `path` の実体が `exclude` のいずれかと同じならtrue
    pub fn is_excluded(&self, path: &Path) -> bool {
        let _ = (&self.exclude, path);
        todo!("skeleton: not implemented")
    }
}
