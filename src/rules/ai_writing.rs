//! `@textlint-ja/preset-ai-writing` の5ルール

// skeleton: not implemented (実装したら、この行と次の行を消す)
#![allow(dead_code)]

mod ai_tech_writing_guideline;
mod no_ai_colon_continuation;
mod no_ai_emphasis_patterns;
mod no_ai_hype_expressions;
mod no_ai_list_formatting;
mod table;

use std::ops::Range;

use regex::Regex;

use crate::prose::{Document, Paragraph};

use super::Report;
use super::builtin::Builtin;

/// 有効なルールの指摘を返す。順は問わない (`rules::report` が位置の順に並べる)
pub fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report<'static>> {
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
    let _ = (allows, text);
    todo!("skeleton: not implemented")
}

/// 元のソースの範囲 `range` が、段落の子孫のインラインコードかHTMLのタグと重なるか。
/// 元の文字列に正規表現を当てるルールは、重なる一致を捨てる
fn overlaps_code_or_html(paragraph: &Paragraph, range: Range<usize>) -> bool {
    let _ = (paragraph, range);
    todo!("skeleton: not implemented")
}
