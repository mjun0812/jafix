use std::borrow::Cow;

use crate::morph;
use crate::prose::{BlockKind, Paragraph};
use crate::rules::builtin::NoAiColonContinuation;
use crate::rules::morph::projection::Projection;
use crate::rules::{Report, RuleId};

use super::allowed;

/// 文書の直下の段落のうち、述語とコロンで終わり、次にコードブロック・リスト・引用・表が続くものの、そのコロンを指摘する
pub fn report(
    src: &str,
    paragraphs: &[Paragraph],
    rule: &NoAiColonContinuation,
) -> Vec<Report<'static>> {
    let mut reports = Vec::new();
    for paragraph in paragraphs {
        let Some(next) = paragraph.next_block else {
            continue;
        };
        let (disabled, name) = match next {
            BlockKind::CodeBlock => (rule.disable_code_block, "コードブロック"),
            BlockKind::List => (rule.disable_list, "リスト"),
            BlockKind::Blockquote => (rule.disable_quote, "引用"),
            BlockKind::Table => (rule.disable_table, "表"),
        };
        let source = &src[paragraph.range.clone()];
        if disabled || allowed(&rule.allows, source) || !source.trim().ends_with([':', '：']) {
            continue;
        }
        let projection = Projection::paragraph(src, paragraph, 'ー');
        let plain = projection.text.trim();
        let Some(colon) = plain
            .chars()
            .next_back()
            .filter(|c| matches!(c, ':' | '：'))
        else {
            continue;
        };
        let before = plain[..plain.len() - colon.len_utf8()].trim();
        if !ends_with_predicate(before) {
            continue;
        }
        let colon_start = paragraph.range.start
            + source
                .rfind(colon)
                .expect("段落の元の文字列はコロンで終わる");
        reports.push(Report {
            range: colon_start..colon_start + colon.len_utf8(),
            rule: RuleId::NoAiColonContinuation.name(),
            message: Cow::Owned(format!("述語の後のコロンで{name}を続けている")),
        });
    }
    reports
}

/// コロンの前の文字列 `before` (前後の空白を除いたもの) が述語で終わるか。英語の文字列、2文字以下の文字列、
/// 名詞で終わる文字列は、コロンで続けても自然なので述語としない
fn ends_with_predicate(before: &str) -> bool {
    let english = before
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c.is_whitespace() || matches!(c, '-' | '_' | '.'))
        && before.chars().any(|c| c.is_ascii_alphabetic());
    if english || before.chars().count() <= 2 {
        return false;
    }
    morph::tokenize(before)
        .last()
        .is_some_and(|token| token.pos != "名詞")
}
