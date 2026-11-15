//! 文体の3ルール (`1.1.1`・`1.1.3`・`1.1.5`) と見出しの `1.1.2`。敬体と常体の判定は、`analyze-desumasu-dearu` 2.1.5 の
//! 4つの正規表現を写す。`1.1.1` は文書の全体の本文、`1.1.3` はリストごとの直下の項目、`1.1.5` は文書のすべての
//! インラインの画像のaltで数える

use std::ops::Range;

use crate::prose::Document;

use super::super::Report;
use super::super::builtin::JtfStyle;

/// 4ルールの指摘
pub fn report(src: &str, document: &Document, rules: &JtfStyle) -> Vec<Report> {
    let _ = (src, document, rules, style_matches, minority);
    todo!("skeleton: not implemented")
}

/// 文体
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    /// 敬体 (ですます調)
    Desumasu,
    /// 常体 (である調)
    Dearu,
}

#[allow(dead_code)] // skeleton: not implemented
/// 文体の1つの一致。`range` は元のソース上のバイト範囲
struct StyleMatch {
    range: Range<usize>,
    style: Style,
}

/// 元のソースの `range` の文字列の中の、接続の形と文末の形の一致を、それぞれ重ねずにすべて返す
fn style_matches(src: &str, range: Range<usize>) -> Vec<StyleMatch> {
    let _ = (src, range);
    todo!("skeleton: not implemented")
}

/// 多数決で指摘する側の文体。敬体と常体の両方が1件以上あるときだけ返し、多いほうの逆を、同数なら常体を返す
fn minority(matches: &[StyleMatch]) -> Option<Style> {
    let _ = (matches, Style::Desumasu, Style::Dearu);
    todo!("skeleton: not implemented")
}
