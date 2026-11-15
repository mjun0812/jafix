//! `preset-ja-spacing` の11ルール。本文の範囲に当てる7ルール (`texts`) と、インラインのnodeの前後に当てる
//! 4ルール (`around`) に分ける。日本語の文字の範囲は、jafix独自の修正ルールの範囲ではなく、ルールごとの
//! textlintの範囲 (`chars`) である

#![allow(dead_code)] // skeleton: not implemented

mod around;
mod chars;
mod texts;

use crate::prose::Document;

use super::builtin::Builtin;
use super::{FixRuleFinding, Report};

/// 11ルールのうち修正ルールになる側の箇所を、見出しの中のものも含めて返す。重なりは解かない
pub fn find(src: &str, document: &Document, builtin: &Builtin) -> Vec<FixRuleFinding> {
    let _ = (src, document, builtin);
    todo!("skeleton: not implemented")
}

/// 11ルールのうち指摘ルールになる側 (空白を求める `ja-space-between-half-and-full-width`、
/// 4つの `ja-space-around-*` の `true` の側、`ja-space-after-exclamation`、`ja-space-after-question`、
/// `ja-nakaguro-or-halfwidth-space-between-katakana`) の指摘を返す。見出しの中でも、アンカーの補足を付けない
pub fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let _ = (src, document, builtin);
    todo!("skeleton: not implemented")
}
