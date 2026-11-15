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
    #[allow(dead_code)] // skeleton: not implemented
    pub fn overlaps(&self, lines: RangeInclusive<usize>) -> bool {
        let _ = lines;
        todo!("skeleton: not implemented")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::RuleId;

    fn delete(range: std::ops::Range<usize>) -> Finding {
        Finding {
            position: range.start,
            range,
            replacement: "".into(),
            rule: RuleId::SpaceBetweenJaAndAlnum,
            message: "".into(),
        }
    }

    #[test]
    fn line_col_counts_columns_in_characters() {
        let index = LineIndex::new("日本語 Rust\nあ b\n");
        assert_eq!(index.line_col(0), (1, 1));
        assert_eq!(index.line_col("日本語".len()), (1, 4));
        assert_eq!(index.line_col("日本語 Rust\n".len()), (2, 1));
        assert_eq!(index.line_col("日本語 Rust\nあ".len()), (2, 2));
    }

    #[test]
    fn applies_every_finding_for_all_lines() {
        let src = "語 A\n語 B\n";
        let findings = [delete(3..4), delete(9..10)];
        assert_eq!(apply(src, &findings, &TargetLines::All), "語A\n語B\n");
    }

    #[test]
    fn applies_only_findings_that_start_on_target_lines() {
        let src = "語 A\n語 B\n語 C";
        let findings = [delete(3..4), delete(9..10), delete(15..16)];
        let targets = TargetLines::Lines(BTreeSet::from([2]));
        assert_eq!(apply(src, &findings, &targets), "語 A\n語B\n語 C");
    }

    #[test]
    fn replaces_with_the_replacement_text() {
        let src = "（a）";
        let findings = [
            Finding {
                range: 0..3,
                position: 0,
                replacement: "(".into(),
                rule: RuleId::FullwidthParen,
                message: "".into(),
            },
            Finding {
                range: 4..7,
                position: 4,
                replacement: ")".into(),
                rule: RuleId::FullwidthParen,
                message: "".into(),
            },
        ];
        assert_eq!(apply(src, &findings, &TargetLines::All), "(a)");
    }

    #[test]
    fn returns_the_source_unchanged_without_findings() {
        let src = "語 A\n";
        assert_eq!(apply(src, &[], &TargetLines::All), src);
    }
}
