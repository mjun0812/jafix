#![expect(dead_code, unused_variables, reason = "skeleton: not implemented")]

use crate::prose::Prose;

use super::builtin::Builtin;
use super::{Finding, Report};

/// 本文の範囲ごとに当てる修正ルール (`no-nfd`、`no-hankaku-kana`、`no-zero-width-spaces`、
/// `no-invalid-control-character`) の箇所を返す。見出しの中の箇所も返し、指摘にするかは呼び出し側が決める。
/// `no-nfd` は、前の文字と合成して1文字になる組み合わせだけを返す
pub fn find(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<Finding> {
    todo!("skeleton: not implemented")
}

/// 本文の範囲ごとに当てる指摘ルール (`max-kanji-continuous-len`、`arabic-kanji-numbers`、
/// `no-exclamation-question-mark`、`ja-unnatural-alphabet`) の指摘と、`no-nfd` のうち合成できない組み合わせの指摘を返す
pub fn report(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<Report<'static>> {
    todo!("skeleton: not implemented")
}
