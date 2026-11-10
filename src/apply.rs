use std::collections::BTreeSet;

use crate::rules::Finding;

/// 修正ルールを適用してよい行。行番号は1始まり
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetLines {
    All,
    Lines(BTreeSet<usize>),
}

impl TargetLines {
    pub fn contains(&self, line: usize) -> bool {
        let _ = line;
        todo!("skeleton: not implemented")
    }
}

/// バイト位置から1始まりの行と列 (文字で数える) を求める
pub struct LineIndex<'a> {
    src: &'a str,
}

impl<'a> LineIndex<'a> {
    pub fn new(src: &'a str) -> Self {
        let _ = src;
        todo!("skeleton: not implemented")
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let _ = (self.src, offset);
        todo!("skeleton: not implemented")
    }
}

/// 始まりの位置が対象行にある箇所だけを置き換えた内容を返す
pub fn apply(src: &str, findings: &[Finding], targets: &TargetLines) -> String {
    let _ = (src, findings, targets);
    todo!("skeleton: not implemented")
}
