//! `@textlint-ja/preset-ai-writing` の5ルール

mod ai_tech_writing_guideline;
mod no_ai_colon_continuation;
mod no_ai_emphasis_patterns;
mod no_ai_hype_expressions;
mod no_ai_list_formatting;
mod table;

use std::ops::Range;

use regress::Regex;

use crate::prose::{Document, InlineKind, Paragraph};

use super::Report;
use super::builtin::Builtin;
use super::morph::inside;

/// 有効なルールの指摘を返す。呼び出し側が位置の順に並べる
pub fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    if let Some(rule) = &builtin.no_ai_list_formatting {
        reports.extend(no_ai_list_formatting::report(src, document, rule));
    }
    if let Some(rule) = &builtin.no_ai_hype_expressions {
        reports.extend(no_ai_hype_expressions::report(src, &document.texts, rule));
    }
    if let Some(rule) = &builtin.no_ai_emphasis_patterns {
        reports.extend(no_ai_emphasis_patterns::report(src, document, rule));
    }
    if let Some(rule) = &builtin.no_ai_colon_continuation {
        reports.extend(no_ai_colon_continuation::report(
            src,
            &document.paragraphs,
            rule,
        ));
    }
    if let Some(rule) = &builtin.ai_tech_writing_guideline {
        reports.extend(ai_tech_writing_guideline::report(
            src,
            &document.paragraphs,
            rule,
        ));
    }
    reports
}

/// `allows` のどれかが `text` のどこかに当たるか。当たれば、ルールはその要素 (本文の範囲・段落・見出し・リスト項目)
/// からの指摘をすべて出さない
fn allowed(allows: &[Regex], text: &str) -> bool {
    allows.iter().any(|allow| allow.find(text).is_some())
}

/// 元のソースの空でない範囲 `range` の先頭か末尾の文字が、段落の子孫のインラインコードかHTMLのタグの中にあるか。
/// 元の文字列に正規表現を当てるルールは、そういう一致を捨てる。先頭と末尾だけを見るので、
/// `**`name`**` のように太字がコードを含む一致は残る
fn overlaps_code_or_html(paragraph: &Paragraph, range: Range<usize>) -> bool {
    let code_or_html = |kind: &InlineKind| matches!(kind, InlineKind::Code | InlineKind::Html);
    // インラインの要素の範囲は文字の境界にあるので、末尾の文字の最後のバイトで末尾の文字を確かめられる
    [range.start, range.end - 1]
        .into_iter()
        .any(|position| inside(&paragraph.children, position, code_or_html))
}
