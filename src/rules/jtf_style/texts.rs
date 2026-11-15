//! 段落の直下の本文の範囲 (`Context::plain`) に当てる24ルール。`1.2.1`・`1.2.2`・`2.1.2`・`2.1.5` (半角カナの部分)・
//! `2.1.8`・`2.1.9`・`2.1.10`・`2.2.2`・`3.1.1`・`3.1.2`・`3.2`・`3.3`・`4.1.1`・`4.1.3`・`4.2.1`・`4.2.2`・`4.2.4`〜`4.2.9`・
//! `4.3.1`・`4.3.2`。前後の文字は同じ本文の範囲の中の文字で、すべての位置で判定する

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::Prose;

use super::super::builtin::JtfStyle;
use super::super::spacing::is_indentation;
use super::super::{Finding, FixRuleFinding, Report, RuleId};
use super::{HANKAKU_KANA, JAPANESE, JAPANESE_WITHOUT_MARKS, JtfRule, contains};

static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new("[ \u{3000}]+").unwrap());

/// `3.3` の括弧
const PARENTHESES: [char; 12] = [
    '(', ')', '[', ']', '（', '）', '［', '］', '「', '」', '『', '』',
];

/// 修正ルールになる7ルールの箇所。空白を消す箇所の位置は、消す空白の並びの先頭である
pub fn find(src: &str, texts: &[Prose], rules: &JtfStyle) -> Vec<FixRuleFinding> {
    let mut findings = Vec::new();
    for Prose { range, context } in texts {
        if !context.plain {
            continue;
        }
        let text = &src[range.clone()];
        let mut push = |local: Range<usize>,
                        replacement: Cow<'static, str>,
                        rule,
                        message: Cow<'static, str>| {
            findings.push(FixRuleFinding {
                finding: Finding {
                    range: range.start + local.start..range.start + local.end,
                    replacement,
                    rule: RuleId::JtfStyle(rule),
                    position: range.start + local.start,
                    message,
                },
                heading: false,
            });
        };
        if rules.enabled(JtfRule::R2_1_5) {
            for run in runs(text, |c| HANKAKU_KANA.contains(&c)) {
                let kana = &text[run.clone()];
                push(
                    run,
                    Cow::Owned(to_zenkaku(kana)),
                    JtfRule::R2_1_5,
                    Cow::Owned(format!("半角カナ「{kana}」を全角にする")),
                );
            }
        }
        for (rule, is_full_width, name) in [
            (
                JtfRule::R2_1_8,
                (|c| matches!(c, '０'..='９')) as fn(char) -> bool,
                "数字",
            ),
            (
                JtfRule::R2_1_9,
                |c| matches!(c, 'Ａ'..='Ｚ' | 'ａ'..='ｚ'),
                "アルファベット",
            ),
        ] {
            if !rules.enabled(rule) {
                continue;
            }
            for run in runs(text, is_full_width) {
                let full_width = &text[run.clone()];
                let half_width = full_width
                    .chars()
                    .map(|c| char::from_u32(c as u32 - 0xFEE0).expect("全角の英数字から引く"))
                    .collect();
                push(
                    run,
                    Cow::Owned(half_width),
                    rule,
                    Cow::Owned(format!("全角の{name}「{full_width}」を半角にする")),
                );
            }
        }
        for (i, _) in text.match_indices(' ') {
            let (Some(prev), Some(next)) =
                (text[..i].chars().next_back(), text[i + 1..].chars().next())
            else {
                continue;
            };
            let alnum = |c: char| c.is_ascii_alphanumeric();
            let japanese = |c| contains(JAPANESE_WITHOUT_MARKS, c);
            if rules.enabled(JtfRule::R3_1_1)
                && (alnum(prev) && japanese(next) || japanese(prev) && alnum(next))
            {
                push(
                    i..i + 1,
                    Cow::Borrowed(""),
                    JtfRule::R3_1_1,
                    Cow::Borrowed("全角文字と半角文字の間の空白を削除する"),
                );
            }
            let katakana = |c| matches!(c, 'ァ'..='ヶ');
            if rules.enabled(JtfRule::R3_1_2)
                && contains(JAPANESE, prev)
                && contains(JAPANESE, next)
                && !(katakana(prev) && katakana(next))
            {
                push(
                    i..i + 1,
                    Cow::Borrowed(""),
                    JtfRule::R3_1_2,
                    Cow::Borrowed("全角文字どうしの間の空白を削除する"),
                );
            }
        }
        if rules.enabled(JtfRule::R3_3) {
            let allow = rules.parentheses.allow_outside_half_parentheses;
            for spaces in SPACES.find_iter(text) {
                let run = spaces.as_str();
                // `allowOutsideHalfParentheses` が許すのは、半角の括弧の外側に接する文字が半角空白の並びである
                let removes_before = text[..spaces.start()].chars().next_back().is_some_and(|c| {
                    PARENTHESES.contains(&c) && !(allow && c == ')' && run.starts_with(' '))
                });
                let removes_after = text[spaces.end()..].chars().next().is_some_and(|c| {
                    PARENTHESES.contains(&c) && !(allow && c == '(' && run.ends_with(' '))
                });
                if (removes_before || removes_after) && !is_indentation(text, spaces.start()) {
                    push(
                        spaces.range(),
                        Cow::Borrowed(""),
                        JtfRule::R3_3,
                        Cow::Borrowed("括弧の外側と内側の空白を削除する"),
                    );
                }
            }
        }
        if rules.enabled(JtfRule::R4_2_4) {
            for (i, dot) in text.match_indices('･') {
                let neighbour = |c: Option<char>| {
                    c.is_some_and(|c| contains(JAPANESE, c) || c.is_ascii_alphabetic())
                };
                if neighbour(text[..i].chars().next_back())
                    && neighbour(text[i + dot.len()..].chars().next())
                {
                    push(
                        i..i + dot.len(),
                        Cow::Borrowed("・"),
                        JtfRule::R4_2_4,
                        Cow::Borrowed("半角の中黒「･」を全角の「・」にする"),
                    );
                }
            }
        }
    }
    findings
}

/// `text` の中で、`belongs` を満たす文字の続きのバイト範囲
fn runs(text: &str, belongs: impl Fn(char) -> bool) -> impl Iterator<Item = Range<usize>> {
    let mut chars = text.char_indices().peekable();
    std::iter::from_fn(move || {
        let (start, _) = chars.find(|&(_, c)| belongs(c))?;
        let mut end = text.len();
        while let Some(&(i, c)) = chars.peek() {
            if !belongs(c) {
                end = i;
                break;
            }
            chars.next();
        }
        Some(start..end)
    })
}

/// 指摘ルールになる側の指摘。`3.3` は `requireOutsideHalfParentheses = true` の側、`4.2.1`・`4.2.2` は両方の部分である
pub fn report(src: &str, texts: &[Prose], rules: &JtfStyle) -> Vec<Report> {
    let unimplemented = [
        JtfRule::R1_2_1,
        JtfRule::R1_2_2,
        JtfRule::R2_1_2,
        JtfRule::R2_1_10,
        JtfRule::R2_2_2,
        JtfRule::R3_2,
        JtfRule::R4_1_1,
        JtfRule::R4_1_3,
        JtfRule::R4_2_1,
        JtfRule::R4_2_2,
        JtfRule::R4_2_5,
        JtfRule::R4_2_6,
        JtfRule::R4_2_7,
        JtfRule::R4_2_8,
        JtfRule::R4_2_9,
        JtfRule::R4_3_1,
        JtfRule::R4_3_2,
    ];
    if unimplemented.into_iter().any(|rule| rules.enabled(rule)) {
        let _ = is_joyo_kanji;
        todo!("skeleton: not implemented");
    }
    let mut reports = Vec::new();
    for Prose { range, context } in texts {
        if !context.plain {
            continue;
        }
        let text = &src[range.clone()];
        if rules.enabled(JtfRule::R3_3) && rules.parentheses.require_outside_half_parentheses {
            for (i, paren) in text.match_indices(['(', ')']) {
                let neighbour = if paren == "(" {
                    text[..i].chars().next_back()
                } else {
                    text[i + 1..].chars().next()
                };
                if neighbour.is_some_and(|c| contains(JAPANESE, c)) {
                    reports.push(Report {
                        range: range.start + i..range.start + i + 1,
                        rule: JtfRule::R3_3.name(),
                        message: Cow::Borrowed("半角の括弧の外側に半角空白を入れる"),
                    });
                }
            }
        }
    }
    reports
}

/// 半角カナの続き `run` を、`hankaku::HANKAKU_TO_ZENKAKU` で全角にする。表にある濁点・半濁点の付く2文字の組を
/// 先に1文字にし、残りを1文字ずつ変える
fn to_zenkaku(run: &str) -> String {
    let mut zenkaku = String::new();
    let mut rest = run;
    while !rest.is_empty() {
        let (kana, c) = super::hankaku::HANKAKU_TO_ZENKAKU
            .iter()
            .filter(|(kana, _)| rest.starts_with(kana))
            .max_by_key(|(kana, _)| kana.len())
            .expect("半角カナの表はU+FF65〜U+FF9Fのすべての文字を持つ");
        zenkaku.push(*c);
        rest = &rest[kana.len()..];
    }
    zenkaku
}

/// `c` が常用漢字か
fn is_joyo_kanji(c: char) -> bool {
    let _ = (c, super::joyo::JOYO_KANJI);
    todo!("skeleton: not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_half_width_kana_has_a_full_width_form() {
        for c in HANKAKU_KANA {
            let zenkaku = to_zenkaku(&c.to_string());
            assert_eq!(zenkaku.chars().count(), 1, "{c}");
            assert!(
                !contains(&[HANKAKU_KANA], zenkaku.chars().next().unwrap()),
                "{c}"
            );
        }
    }
}
