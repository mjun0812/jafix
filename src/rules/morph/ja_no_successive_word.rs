use std::sync::LazyLock;

use regex::Regex;

use crate::rules::builtin::JaNoSuccessiveWord;
use crate::rules::{Report, RuleId};

use super::AnalyzedProse;

/// `allowOnomatopee` で指摘しない、カタカナの語。`ヴ` は含まない
static ONOMATOPEE: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[ァ-ロワヲンー]*$").unwrap());

/// 表層が同じトークンが隣り合ったら、2つ目のトークンに指摘を出す
pub(super) fn report(prose: &AnalyzedProse, rule: &JaNoSuccessiveWord) -> Vec<Report> {
    prose
        .tokens
        .windows(2)
        .filter_map(|pair| {
            let (previous, token) = (&pair[0], &pair[1]);
            if previous.surface != token.surface {
                return None;
            }
            // linderaは改行とタブで区切るので、続く改行やタブが別々の空白のトークンになる
            let space = previous.pos_detail_1 == "空白" || token.pos_detail_1 == "空白";
            let allowed = rule.allow.iter().any(|allow| allow.is_match(token.surface));
            let number = [previous, token]
                .iter()
                .all(|token| token.pos == "名詞" && token.pos_detail_1 == "数");
            let onomatopee = rule.allow_onomatopee && ONOMATOPEE.is_match(token.surface);
            if space || allowed || number || onomatopee {
                return None;
            }
            Some(prose.report(
                token.range.clone(),
                RuleId::JaNoSuccessiveWord,
                format!("「{}」が2回続いている", token.surface),
            ))
        })
        .collect()
}
