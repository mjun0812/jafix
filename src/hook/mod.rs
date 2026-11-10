pub mod claude;
pub mod codex;

use std::path::{Path, PathBuf};

use crate::apply::TargetLines;

/// 直すファイルと、その対象行
#[derive(Debug, PartialEq, Eq)]
pub struct Target {
    pub path: PathBuf,
    pub lines: TargetLines,
}

/// hookの入力から決めた、直すファイルの一覧と、決められなかった理由の一覧
#[derive(Debug, Default)]
pub struct Targets {
    pub files: Vec<Target>,
    pub errors: Vec<String>,
}

pub fn is_markdown(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
}
