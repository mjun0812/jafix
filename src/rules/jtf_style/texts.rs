//! 段落の直下の本文の範囲 (`Context::plain`) に当てる24ルール。`1.2.1`・`1.2.2`・`2.1.2`・`2.1.5` (半角カナの部分)・
//! `2.1.8`・`2.1.9`・`2.1.10`・`2.2.2`・`3.1.1`・`3.1.2`・`3.2`・`3.3`・`4.1.1`・`4.1.3`・`4.2.1`・`4.2.2`・`4.2.4`〜`4.2.9`・
//! `4.3.1`・`4.3.2`。前後の文字は同じ本文の範囲の中の文字で、すべての位置で判定する

use crate::prose::Prose;

use super::super::builtin::JtfStyle;
use super::super::{FixRuleFinding, Report};

/// 修正ルールになる7ルールの箇所。空白を消す箇所の位置は、消す空白の並びの先頭である
pub fn find(src: &str, texts: &[Prose], rules: &JtfStyle) -> Vec<FixRuleFinding> {
    let _ = (src, texts, rules, to_zenkaku);
    todo!("skeleton: not implemented")
}

/// 指摘ルールになる側の指摘。`3.3` は `requireOutsideHalfParentheses = true` の側、`4.2.1`・`4.2.2` は両方の部分である
pub fn report(src: &str, texts: &[Prose], rules: &JtfStyle) -> Vec<Report> {
    let _ = (src, texts, rules, is_joyo_kanji);
    todo!("skeleton: not implemented")
}

/// 半角カナの続き `run` を、`hankaku::HANKAKU_TO_ZENKAKU` で全角にする。表にある濁点・半濁点の付く2文字の組を
/// 先に1文字にし、残りを1文字ずつ変える
fn to_zenkaku(run: &str) -> String {
    let _ = (run, super::hankaku::HANKAKU_TO_ZENKAKU);
    todo!("skeleton: not implemented")
}

/// `c` が常用漢字か
fn is_joyo_kanji(c: char) -> bool {
    let _ = (c, super::joyo::JOYO_KANJI);
    todo!("skeleton: not implemented")
}
