use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use crate::rules::Finding;

/// 修正ルールを適用してよい行。行番号は1始まり
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetLines {
    All,
    Lines(BTreeSet<usize>),
}

impl TargetLines {
    pub fn contains(&self, line: usize) -> bool {
        match self {
            TargetLines::All => true,
            TargetLines::Lines(lines) => lines.contains(&line),
        }
    }

    /// 行の範囲 `lines` のどれかの行が対象行か
    pub fn overlaps(&self, lines: RangeInclusive<usize>) -> bool {
        match self {
            TargetLines::All => true,
            TargetLines::Lines(targets) => targets.range(lines).next().is_some(),
        }
    }
}

/// バイト位置から1始まりの行と列 (文字で数える) を求める
pub struct LineIndex<'a> {
    src: &'a str,
    /// 各行の始まりのバイト位置
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub fn new(src: &'a str) -> Self {
        let line_starts = std::iter::once(0)
            .chain(src.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        Self { src, line_starts }
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_starts.partition_point(|&start| start <= offset);
        let column = self.src[self.line_starts[line - 1]..offset].chars().count() + 1;
        (line, column)
    }
}

/// 始まりの位置が対象行にある箇所だけを置き換えた内容を返す
pub fn apply(src: &str, findings: &[Finding], targets: &TargetLines) -> String {
    let index = LineIndex::new(src);
    let mut out = String::with_capacity(src.len());
    let mut copied = 0;
    for finding in findings {
        let (line, _) = index.line_col(finding.range.start);
        if !targets.contains(line) {
            continue;
        }
        out.push_str(&src[copied..finding.range.start]);
        out.push_str(&finding.replacement);
        copied = finding.range.end;
    }
    out.push_str(&src[copied..]);
    out
}
