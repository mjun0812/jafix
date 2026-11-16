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
            // 伏せ字の `○○` や2倍のダッシュ `──` は正しい書き方なので、句読点のほかの記号の連続は指摘しない。
            // linderaが改行やタブを1文字ずつに分けた空白のトークンも、記号である
            let symbol = token.pos == "記号" && !matches!(token.pos_detail_1, "句点" | "読点");
            let allowed = rule
                .allow
                .iter()
                .any(|allow| allow.find(token.surface).is_some());
            let number = [previous, token]
                .iter()
                .all(|token| token.pos == "名詞" && token.pos_detail_1 == "数");
            let onomatopee = rule.allow_onomatopee && ONOMATOPEE.is_match(token.surface);
            if symbol || allowed || number || onomatopee {
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
