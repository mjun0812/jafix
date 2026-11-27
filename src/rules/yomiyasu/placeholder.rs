//! 本文に残った仮置きの文字。yomiyasu 3.4.1の `references/slop-catalog.md` が「仮置きの残り」に挙げる4つの形を指摘する。
//! yomiyasuのlintには無い検査である

use std::sync::LazyLock;

use crate::prose::{Document, Prose};
use crate::rules::{Report, RuleId};

use super::{finding, paired_ranges, prose_in};

/// 見出し・段落・リスト・表のセルの本文で、仮置きの文字の並びごとに指摘する。会話や引用の中の伏せ字は残すので、
/// かぎ括弧の中と引用は見ない
pub(super) fn report(src: &str, document: &Document) -> Vec<Report> {
    static PLACEHOLDER: LazyLock<regress::Regex> = LazyLock::new(|| {
        regress::Regex::new(
            r"[〇○]{2,}|(?<![A-Za-z0-9])X{3,}(?![A-Za-z0-9])|\[ここに[^\[\]\n]*\]|【要確認】",
        )
        .unwrap()
    });
    let blocks = document
        .paragraphs
        .iter()
        .chain(&document.headings)
        .map(|paragraph| &paragraph.range)
        .chain(&document.table_cells);
    let mut reports = Vec::new();
    for block in blocks {
        let prose: Vec<&Prose> = prose_in(document, block)
            .iter()
            .filter(|part| !part.context.blockquote)
            .collect();
        let quoted = paired_ranges(
            prose.iter().flat_map(|part| {
                src[part.range.clone()]
                    .char_indices()
                    .map(|(i, c)| (part.range.start + i, c))
            }),
            &[('「', '」'), ('『', '』')],
        );
        for part in prose {
            for hit in PLACEHOLDER.find_iter(&src[part.range.clone()]) {
                let range = part.range.start + hit.start()..part.range.start + hit.end();
                if quoted.iter().any(|quote| quote.contains(&range.start)) {
                    continue;
                }
                let message = format!(
                    "仮置きの文字「{}」が本文に残っていないか確認する",
                    &src[range.clone()]
                );
                reports.push(finding(range, RuleId::YomiyasuPlaceholderResidue, message));
            }
        }
    }
    reports
}
