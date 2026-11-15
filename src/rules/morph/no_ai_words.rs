use super::AnalyzedProse;
use super::dict::AI_WORDS;
use super::pattern::{self, Match};
use crate::rules::builtin::{DictionaryMode, NoAiWords};
use crate::rules::{Report, RuleId};

/// 組み込みの辞書 (`dictionaryMode` が `"override"` なら使わない) と利用者の辞書の並びを、本文の範囲のトークンの
/// すべての位置から当てる。照合の状態は、本文の範囲をまたいで持ち越さない
pub fn report(prose: &AnalyzedProse, rule: &NoAiWords) -> Vec<Report> {
    let tokens = prose.tokens;
    let builtin: &[_] = match rule.dictionary_mode {
        DictionaryMode::Append => &AI_WORDS,
        DictionaryMode::Override => &[],
    };
    let builtin = builtin.iter().flat_map(|phrase| {
        pattern::find(tokens, phrase.pattern)
            .into_iter()
            .map(|found| (found, None))
    });
    let user = rule.user_words.iter().flat_map(|word| {
        pattern::find(tokens, &word.pattern)
            .into_iter()
            .map(|found| (found, Some(&word.message)))
    });
    builtin
        .chain(user)
        .filter_map(|(found, message)| {
            let range = matched_range(prose, &found)?;
            let text = &prose.text[range.clone()];
            if rule.allows.iter().any(|allow| allow.find(text).is_some()) {
                return None;
            }
            let message = match message {
                Some(message) => message.clone(),
                None => format!("AIが多用する語「{text}」を使っている"),
            };
            Some(prose.report(range, RuleId::NoAiWords, message))
        })
        .collect()
}

/// 一致の最初のトークンの先頭から最後のトークンの末尾まで。利用者の辞書の並びがすべて飛ばされたときは無い
fn matched_range(prose: &AnalyzedProse, found: &Match) -> Option<std::ops::Range<usize>> {
    let mut matched = found.tokens.iter().flatten().map(|&i| &prose.tokens[i]);
    let first = matched.next()?;
    let end = matched.next_back().unwrap_or(first).range.end;
    Some(first.range.start..end)
}
