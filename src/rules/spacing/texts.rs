//! 本文の範囲に当てる7ルール。`ja-space-between-half-and-full-width` (`lintStyledNode` が真ならすべての本文の範囲、
//! 偽なら `Context::plain` の本文の範囲)、`ja-no-space-around-parentheses`、`ja-no-space-around-slash`、
//! `ja-no-space-between-full-width`、`ja-nakaguro-or-halfwidth-space-between-katakana`、
//! `ja-space-after-exclamation`、`ja-space-after-question` (どれも `Context::plain` の本文の範囲)

use crate::prose::Prose;

use super::super::builtin::Builtin;
use super::super::{FixRuleFinding, Report};

/// 7ルールの修正の箇所。空白を消す箇所の位置は、消す空白の並びの先頭である
pub fn find(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<FixRuleFinding> {
    let _ = (src, texts, builtin);
    todo!("skeleton: not implemented")
}

/// 7ルールの指摘
pub fn report(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<Report> {
    let _ = (src, texts, builtin);
    todo!("skeleton: not implemented")
}

/// `text` の `start` (空白の並びの先頭) が行頭の字下げか。本文の範囲の中の直前の改行から `start` までが、
/// 半角空白とタブだけなら字下げである。本文の範囲が改行を含まなければ字下げは無い
fn is_indentation(text: &str, start: usize) -> bool {
    let _ = (text, start);
    todo!("skeleton: not implemented")
}
