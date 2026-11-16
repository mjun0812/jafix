use std::sync::LazyLock;

use regex::Regex;

use crate::rules::builtin::JaNoSuccessiveWord;
use crate::rules::{Report, RuleId};

use super::AnalyzedProse;

/// `allowOnomatopee` で指摘しない、カタカナの語。`ヴ` は含まない
static ONOMATOPEE: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[ァ-ロワヲンー]*$").unwrap());

/// 同じ語と、括弧・踊り字が連続する箇所を指摘する。踊り字はトークンの分かれ方によらず1文字ずつ見る
pub(super) fn report(prose: &AnalyzedProse, rule: &JaNoSuccessiveWord) -> Vec<Report> {
    let mut reports = Vec::new();
    let mut previous = None;
    for (i, c) in prose.text.char_indices() {
        if previous == Some(c) && is_iteration_mark(c) {
            let surface = &prose.text[i..i + c.len_utf8()];
            if !rule.allow.iter().any(|allow| allow.find(surface).is_some()) {
                reports.push(prose.report(
                    i..i + c.len_utf8(),
                    RuleId::JaNoSuccessiveWord,
                    format!("「{c}」が2回続いている"),
                ));
            }
        }
        previous = Some(c);
    }
    reports.extend(prose.tokens.windows(2).filter_map(|pair| {
        let (previous, token) = (&pair[0], &pair[1]);
        if previous.surface != token.surface || token.surface.chars().all(is_iteration_mark) {
            return None;
        }
        // 伏せ字や罫線、空白のトークンの連続は許す。踊り字は文字ごとに検査済みである。
        let symbol = token.pos == "記号"
            && !matches!(token.pos_detail_1, "句点" | "読点" | "括弧開" | "括弧閉");
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
    }));
    reports
}

fn is_iteration_mark(c: char) -> bool {
    matches!(c, '々' | '〻' | 'ゝ' | 'ゞ' | 'ヽ' | 'ヾ')
}
