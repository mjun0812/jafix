use std::borrow::Cow;

use crate::prose::Prose;
use crate::rules::builtin::NoAiHypeExpressions;
use crate::rules::{Report, RuleId};

use super::allowed;
use super::table::{HYPE_WORDS, HypeCategory};

/// 本文の範囲ごとに、誇張した表現の語の重ならない一致をすべて指摘する
pub fn report(src: &str, texts: &[Prose], rule: &NoAiHypeExpressions) -> Vec<Report> {
    let words: Vec<&str> = HYPE_WORDS
        .iter()
        .filter(|(category, _)| match category {
            HypeCategory::Absoluteness => !rule.disable_absoluteness_patterns,
            HypeCategory::Abstract => !rule.disable_abstract_patterns,
            HypeCategory::Predictive => !rule.disabled_predictive_patterns,
        })
        .map(|&(_, word)| word)
        .collect();
    let mut reports = Vec::new();
    for Prose { range, .. } in texts {
        let text = &src[range.clone()];
        if allowed(&rule.allows, text) {
            continue;
        }
        for word in &words {
            for (i, _) in text.match_indices(word) {
                let start = range.start + i;
                reports.push(Report {
                    range: start..start + word.len(),
                    rule: RuleId::NoAiHypeExpressions.name(),
                    message: Cow::Owned(format!("誇張した表現「{word}」を使っている")),
                });
            }
        }
    }
    reports
}
