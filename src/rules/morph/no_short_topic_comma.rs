use super::AnalyzedProse;
use crate::morph::Token;
use crate::rules::builtin::NoShortTopicComma;
use crate::rules::{Report, RuleId};

/// 係助詞の `は` の直後の読点のうち、文頭から読点の手前までの文字数が `maxLength` 以下のものを指摘する
pub fn report(prose: &AnalyzedProse, rule: &NoShortTopicComma) -> Vec<Report> {
    let tokens = prose.tokens;
    let mut reports = Vec::new();
    // 文頭の、`text` の中のバイト位置
    let mut start = 0;
    for (i, token) in tokens.iter().enumerate() {
        if ends_sentence(token) {
            start = token.range.end;
            continue;
        }
        let Some(comma) = tokens.get(i + 1) else {
            break;
        };
        if !is_topic_wa(token) || !is_touten(comma) {
            continue;
        }
        let head = &prose.text[start..comma.range.start];
        // 本文の範囲の中の改行 (段落の中の改行) の直後も文頭である
        let head = head.rfind('\n').map_or(head, |i| &head[i + 1..]);
        let length = head.chars().count();
        if length <= rule.max_length {
            reports.push(prose.report(
                comma.range.clone(),
                RuleId::NoShortTopicComma,
                format!("主題を{length}文字示しただけで読点を打っている"),
            ));
        }
    }
    reports
}

fn ends_sentence(token: &Token) -> bool {
    token.pos == "記号"
        && matches!(token.pos_detail_1, "句点" | "一般")
        && matches!(token.surface, "。" | "！" | "？" | "!" | "?")
}

fn is_topic_wa(token: &Token) -> bool {
    token.pos == "助詞" && token.pos_detail_1 == "係助詞" && token.surface == "は"
}

fn is_touten(token: &Token) -> bool {
    token.pos == "記号" && token.pos_detail_1 == "読点"
}
