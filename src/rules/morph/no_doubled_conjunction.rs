use crate::morph::Token;
use crate::rules::{Report, RuleId};

use super::Analyzed;

/// 段落の文ごとの最初の接続詞を、接続詞を持つ直前の文の最初の接続詞と比べ、同じなら後ろの文の接続詞に指摘を出す
pub(super) fn report(sentences: &[Analyzed]) -> Vec<Report<'static>> {
    let mut reports = Vec::new();
    let mut previous: Option<&str> = None;
    for sentence in sentences {
        let Some(conjunction) = first_conjunction(&sentence.tokens) else {
            continue;
        };
        if previous == Some(conjunction.surface) {
            reports.push(sentence.report(
                conjunction,
                RuleId::NoDoubledConjunction,
                format!("同じ接続詞「{}」が続いている", conjunction.surface),
            ));
        }
        previous = Some(conjunction.surface);
    }
    reports
}

/// 空白の直後にないものの中で、最初の接続詞
fn first_conjunction<'t, 'a>(tokens: &'t [Token<'a>]) -> Option<&'t Token<'a>> {
    tokens.iter().enumerate().find_map(|(i, token)| {
        let after_space = i > 0 && tokens[i - 1].pos_detail_1 == "空白";
        (token.pos == "接続詞" && !after_space).then_some(token)
    })
}
