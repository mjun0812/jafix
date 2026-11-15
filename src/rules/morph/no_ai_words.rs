use super::AnalyzedProse;
use super::dict::AI_WORDS;
use crate::rules::Report;
use crate::rules::builtin::NoAiWords;

/// 組み込みの辞書 (`dictionaryMode` が `"override"` なら使わない) と利用者の辞書の並びを、本文の範囲のトークンの
/// すべての位置から当てる。照合の状態は、本文の範囲をまたいで持ち越さない
pub fn report(prose: &AnalyzedProse, rule: &NoAiWords) -> Vec<Report<'static>> {
    let _ = (prose, rule, &AI_WORDS);
    todo!("skeleton: not implemented")
}
