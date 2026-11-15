//! `write-good` (textlint-rule-write-good 2.0.0、write-good 1.0.8)。英語の弱い語・繰り返し・受動態・冗長な句などの
//! 9つの検査を、段落の直下の本文の範囲に当てて指摘する

#[allow(dead_code)] // skeleton: not implemented
mod words;

use crate::prose::Prose;
use crate::rules::Report;
use crate::rules::builtin::{WriteGood, WriteGoodCheck};

/// `Context::plain` の本文の範囲に有効な検査を当て、`whitelist` の文字列のすべての出現と重なる提案を外し、範囲が同じ提案を
/// 1件にまとめて、文面を検査の順につなぐ。範囲は提案の範囲 (当たった文字列の全体) である
pub fn report(src: &str, texts: &[Prose], rule: &WriteGood) -> Vec<Report> {
    let _ = (src, texts, rule, explanation);
    todo!("skeleton: not implemented")
}

/// 文面の `「<一致>」は<説明>である(<検査名>)` の説明
fn explanation(check: WriteGoodCheck) -> &'static str {
    match check {
        WriteGoodCheck::Weasel => "曖昧な語",
        WriteGoodCheck::Illusion => "直前の語の繰り返し",
        WriteGoodCheck::So => "意味を足さない文頭の語",
        WriteGoodCheck::ThereIs => "冗長な文頭の句",
        WriteGoodCheck::Passive => "受動態かもしれない句",
        WriteGoodCheck::Adverb => "意味を弱める語",
        WriteGoodCheck::TooWordy => "冗長か不要な句",
        WriteGoodCheck::Cliches => "決まり文句",
        WriteGoodCheck::Eprime => "be動詞の形",
    }
}
