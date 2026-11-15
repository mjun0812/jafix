//! `terminology` (textlint-rule-terminology 5.2.16)。利用者の `terms` の項目と組み込みの辞書の項目を同じ照合で当て、
//! 利用者の項目の箇所を利用者の置き換えに、組み込みの辞書の項目の箇所を指摘にする

use std::borrow::Cow;
use std::ops::Range;

use crate::prose::Prose;
use crate::rules::builtin::Terminology;
use crate::rules::{Finding, Report, RuleId, UserReplacement, one_line};

/// 項目が当たり、置き換え後の語が一致と違う1箇所
struct TermMatch {
    /// 元のソース上のバイト範囲。一致の全体である
    range: Range<usize>,
    /// 置き換え後の語
    replacement: String,
    /// 置き換え後の語を決めた項目が、利用者の `terms` の項目か
    user: bool,
    /// 見出しの中の本文の範囲の箇所
    heading: bool,
}

/// 本文の範囲のうち `skip` の型の中にないものに、文字列の項目をまとめた正規表現、配列の項目の正規表現の順に、
/// 1つずつ重ならない一致をすべて探し、置き換え後の語が一致と違う箇所を返す。本文の範囲ごとに、正規表現の順、位置の順に並ぶ。
/// 置き換え後の語を求められない文字列の項目の一致は捨てる
fn matches(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<TermMatch> {
    let _ = (src, texts, rule);
    todo!("skeleton: not implemented")
}

/// 利用者の `terms` の項目の箇所を、利用者の置き換えとして返す
pub fn replacements(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<UserReplacement> {
    matches(src, texts, rule)
        .into_iter()
        .filter(|m| m.user)
        .map(|m| UserReplacement {
            finding: Finding {
                range: m.range.clone(),
                message: Cow::Owned(message(&src[m.range.clone()], &m.replacement)),
                replacement: Cow::Owned(m.replacement),
                rule: RuleId::Terminology,
                position: m.range.start,
            },
            heading: m.heading,
        })
        .collect()
}

/// 組み込みの辞書の項目の箇所を、指摘として返す
pub fn report(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<Report> {
    matches(src, texts, rule)
        .into_iter()
        .filter(|m| !m.user)
        .map(|m| Report {
            message: Cow::Owned(message(&src[m.range.clone()], &m.replacement)),
            range: m.range,
            rule: RuleId::Terminology.name(),
        })
        .collect()
}

/// `prh` と同じ `「<一致した文字列>」を「<置き換え後>」にする` の文面
fn message(matched: &str, replacement: &str) -> String {
    format!(
        "「{}」を「{}」にする",
        one_line(matched),
        one_line(replacement)
    )
}
