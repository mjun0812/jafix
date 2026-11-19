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

impl Targets {
    fn includes(
        &mut self,
        path: &Path,
        is_excluded: &mut impl FnMut(&Path) -> Result<bool, String>,
    ) -> bool {
        if path.extension().is_none_or(|ext| ext != "md") {
            return false;
        }
        match is_excluded(path) {
            Ok(excluded) => !excluded,
            Err(reason) => {
                self.errors.push(reason);
                false
            }
        }
    }
}
