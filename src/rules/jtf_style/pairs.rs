//! 段落ごとに対を数える5ルール (`4.3.3`〜`4.3.7`)。引用の中にない段落の中の本文の範囲 (リンクの文字列・太字・
//! 強調・取り消し線の中を含み、インラインコードの中身と画像のaltを含まない) を順に見る

use std::borrow::Cow;

use crate::prose::Document;

use super::super::Report;
use super::super::builtin::JtfStyle;
use super::{JtfRule, texts_in};

/// 開きと閉じが違う記号の4ルール
const DISTINCT_PAIRS: [(JtfRule, char, char); 4] = [
    (JtfRule::R4_3_3, '「', '」'),
    (JtfRule::R4_3_4, '『', '』'),
    (JtfRule::R4_3_6, '{', '}'),
    (JtfRule::R4_3_7, '<', '>'),
];

/// 5ルールの指摘。開きと閉じが違う記号は、開きを積み、閉じで直前の開きと対にして、対にならないものを指摘する。
/// `4.3.5` の `"` は、段落の中の数が奇数なら最後の `"` を指摘する
pub fn report(src: &str, document: &Document, rules: &JtfStyle) -> Vec<Report> {
    let rules_here = [
        JtfRule::R4_3_3,
        JtfRule::R4_3_4,
        JtfRule::R4_3_5,
        JtfRule::R4_3_6,
        JtfRule::R4_3_7,
    ];
    if !rules_here.into_iter().any(|rule| rules.enabled(rule)) {
        return Vec::new();
    }
    let pairs: Vec<(JtfRule, char, char)> = DISTINCT_PAIRS
        .into_iter()
        .filter(|(rule, _, _)| rules.enabled(*rule))
        .collect();
    let mut reports = Vec::new();
    for paragraph in document.paragraphs.iter().filter(|p| !p.blockquote) {
        // 記号ごとの、まだ対になっていない開きの位置
        let mut openings: Vec<Vec<usize>> = vec![Vec::new(); pairs.len()];
        let mut last_quote = None;
        let mut quotes = 0;
        for prose in texts_in(&document.texts, &paragraph.range) {
            for (offset, c) in src[prose.range.clone()].char_indices() {
                let position = prose.range.start + offset;
                if c == '"' {
                    quotes += 1;
                    last_quote = Some(position);
                    continue;
                }
                for (&(rule, opening, closing), stack) in pairs.iter().zip(&mut openings) {
                    if c == opening {
                        stack.push(position);
                    } else if c == closing && stack.pop().is_none() {
                        reports.push(unmatched(rule, position, closing, opening));
                    }
                }
            }
        }
        for (&(rule, opening, closing), stack) in pairs.iter().zip(&openings) {
            reports.extend(
                stack
                    .iter()
                    .map(|&position| unmatched(rule, position, opening, closing)),
            );
        }
        if rules.enabled(JtfRule::R4_3_5)
            && quotes % 2 == 1
            && let Some(position) = last_quote
        {
            reports.push(unmatched(JtfRule::R4_3_5, position, '"', '"'));
        }
    }
    reports
}

/// `position` の `found` に、対になる `missing` が無いことの指摘
fn unmatched(rule: JtfRule, position: usize, found: char, missing: char) -> Report {
    Report {
        range: position..position + found.len_utf8(),
        rule: rule.name(),
        message: Cow::Owned(format!("{found} に対応する {missing} が無い")),
    }
}
