use crate::rules::{Report, RuleId};

use super::AnalyzedProse;
use super::dict::WEAK_PHRASES;
use super::pattern;

/// 弱い表現の並びに一致した箇所ごとに、一致の最初のトークンに指摘を出す
pub(super) fn report(prose: &AnalyzedProse) -> Vec<Report> {
    let tokens = prose.tokens;
    WEAK_PHRASES
        .iter()
        .flat_map(|phrase| {
            pattern::find(tokens, phrase.pattern)
                .into_iter()
                .map(move |found| (phrase, found))
        })
        .map(|(phrase, found)| {
            let first = &tokens[found.tokens[0].expect("弱い表現の並びにskippableは無い")];
            prose.report(
                first.range.clone(),
                RuleId::JaNoWeakPhrase,
                format!("弱い表現「{}」を使っている", phrase.name),
            )
        })
        .collect()
}
