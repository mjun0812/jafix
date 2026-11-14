use crate::rules::{Report, RuleId};

use super::AnalyzedProse;
use super::dict::DOUBLE_NEGATIVES;
use super::pattern;

/// 二重否定の並びに一致した箇所ごとに、一致の最後のトークンに指摘を出す。重なる一致も、それぞれ指摘する
pub(super) fn report(prose: &AnalyzedProse) -> Vec<Report<'static>> {
    let tokens = prose.tokens;
    DOUBLE_NEGATIVES
        .iter()
        .flat_map(|phrase| pattern::find(tokens, phrase.pattern))
        .map(|found| {
            let index = |k: usize| found.tokens[k].expect("二重否定の並びにskippableは無い");
            let first = &tokens[index(0)];
            let last = &tokens[index(found.tokens.len() - 1)];
            prose.report(
                last.range.clone(),
                RuleId::NoDoubleNegativeJa,
                format!(
                    "二重否定「{}」を使っている",
                    &prose.text[first.range.start..last.range.end]
                ),
            )
        })
        .collect()
}
