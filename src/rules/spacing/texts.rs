//! 本文の範囲に当てる7ルール。`ja-space-between-half-and-full-width` (`lintStyledNode` が真ならすべての本文の範囲、
//! 偽なら `Context::plain` の本文の範囲)、`ja-no-space-around-parentheses`、`ja-no-space-around-slash`、
//! `ja-no-space-between-full-width`、`ja-nakaguro-or-halfwidth-space-between-katakana`、
//! `ja-space-after-exclamation`、`ja-space-after-question` (どれも `Context::plain` の本文の範囲)

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::Prose;

use super::super::builtin::{Builtin, JaSpaceBetweenHalfAndFullWidth};
use super::super::{Finding, FixRuleFinding, Report, RuleId};
use super::chars::{self, BETWEEN_FULL_WIDTH, HALF_AND_FULL_WIDTH, KATAKANA, KATAKANA_WORD};

static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new("[ \u{3000}]+").unwrap());

static SLASH: LazyLock<Regex> = LazyLock::new(|| Regex::new("[ \u{3000}]*/[ \u{3000}]*").unwrap());

const PARENTHESES: [char; 10] = ['[', ']', '（', '）', '［', '］', '「', '」', '『', '』'];

/// 7ルールの修正の箇所。空白を消す箇所の位置は、消す空白の並びの先頭である
pub fn find(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<FixRuleFinding> {
    let mut findings = Vec::new();
    for Prose { range, context } in texts {
        let text = &src[range.clone()];
        let at = |spaces: Range<usize>,
                  replacement: &'static str,
                  rule: RuleId,
                  position: usize,
                  message: &'static str| FixRuleFinding {
            finding: Finding {
                range: range.start + spaces.start..range.start + spaces.end,
                replacement: Cow::Borrowed(replacement),
                rule,
                position: range.start + position,
                message: Cow::Borrowed(message),
            },
            heading: context.heading,
        };
        if let Some(rule) = &builtin.ja_space_between_half_and_full_width
            && !rule.space.demands_space()
            && (rule.lint_styled_node || context.plain)
        {
            for (i, space) in text
                .char_indices()
                .filter(|&(_, c)| matches!(c, ' ' | '\u{3000}'))
            {
                let end = i + space.len_utf8();
                let (Some(prev), Some(next)) =
                    (text[..i].chars().next_back(), text[end..].chars().next())
                else {
                    continue;
                };
                let between = (prev.is_ascii_alphanumeric() && is_full_width(rule, next))
                    || (is_full_width(rule, prev) && next.is_ascii_alphanumeric());
                if between && !is_allowed(rule, text, i - prev.len_utf8()) {
                    findings.push(at(
                        i..end,
                        "",
                        RuleId::JaSpaceBetweenHalfAndFullWidth,
                        i,
                        "全角文字と半角文字の間の空白を削除する",
                    ));
                }
            }
        }
        if !context.plain {
            continue;
        }
        if builtin.ja_no_space_around_parentheses {
            for spaces in SPACES.find_iter(text) {
                let prev = text[..spaces.start()].chars().next_back();
                let next = text[spaces.end()..].chars().next();
                let around = [prev, next]
                    .into_iter()
                    .flatten()
                    .any(|c| PARENTHESES.contains(&c));
                if around && !is_indentation(text, spaces.start()) {
                    findings.push(at(
                        spaces.range(),
                        "",
                        RuleId::JaNoSpaceAroundParentheses,
                        spaces.start(),
                        "括弧の外側と内側の空白を削除する",
                    ));
                }
            }
        }
        if builtin.ja_no_space_around_slash {
            for slash in SLASH.find_iter(text) {
                let slash_start =
                    slash.start() + slash.as_str().find('/').expect("一致は `/` を含む");
                let start = if is_indentation(text, slash.start()) {
                    slash_start
                } else {
                    slash.start()
                };
                if start == slash_start && slash.end() == slash_start + 1 {
                    continue;
                }
                let position = if start < slash_start {
                    start
                } else {
                    slash_start + 1
                };
                findings.push(at(
                    start..slash.end(),
                    "/",
                    RuleId::JaNoSpaceAroundSlash,
                    position,
                    "スラッシュの前後の空白を削除する",
                ));
            }
        }
        if builtin.ja_no_space_between_full_width {
            for (i, _) in text.match_indices(' ') {
                let (Some(prev), Some(next)) =
                    (text[..i].chars().next_back(), text[i + 1..].chars().next())
                else {
                    continue;
                };
                let full_width = chars::contains(BETWEEN_FULL_WIDTH, prev)
                    && chars::contains(BETWEEN_FULL_WIDTH, next);
                if full_width && !(KATAKANA.contains(&prev) && KATAKANA.contains(&next)) {
                    findings.push(at(
                        i..i + 1,
                        "",
                        RuleId::JaNoSpaceBetweenFullWidth,
                        i,
                        "全角文字どうしの間の空白を削除する",
                    ));
                }
            }
        }
    }
    findings
}

/// `ja-space-between-half-and-full-width` の全角文字か。`、` `。` は `punctuation` が真のときだけ数える
fn is_full_width(rule: &JaSpaceBetweenHalfAndFullWidth, c: char) -> bool {
    chars::contains(HALF_AND_FULL_WIDTH, c) && (rule.space.punctuation || !matches!(c, '、' | '。'))
}

/// 判定に使った並びの先頭 `start` が、`allows` のどれかの一致の範囲の中 (終わりの位置を含まない) にあるか
fn is_allowed(rule: &JaSpaceBetweenHalfAndFullWidth, text: &str, start: usize) -> bool {
    rule.allows
        .iter()
        .any(|regex| regex.find_iter(text).any(|m| m.range().contains(&start)))
}

/// 7ルールの指摘
pub fn report(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    for Prose { range, context } in texts {
        let text = &src[range.clone()];
        let at = |i: usize, c: char, rule: RuleId, message: &'static str| Report {
            range: range.start + i..range.start + i + c.len_utf8(),
            rule: rule.name(),
            message: Cow::Borrowed(message),
        };
        if let Some(rule) = &builtin.ja_space_between_half_and_full_width
            && rule.space.demands_space()
            && (rule.lint_styled_node || context.plain)
        {
            let half_width = |c: char| {
                (rule.space.alphabets && c.is_ascii_alphabetic())
                    || (rule.space.numbers && c.is_ascii_digit())
            };
            let mut chars = text.char_indices().peekable();
            while let Some((i, c)) = chars.next() {
                let Some(&(_, next)) = chars.peek() else {
                    break;
                };
                let pair = (half_width(c) && is_full_width(rule, next))
                    || (is_full_width(rule, c) && half_width(next));
                if pair && !is_allowed(rule, text, i) {
                    reports.push(at(
                        i,
                        c,
                        RuleId::JaSpaceBetweenHalfAndFullWidth,
                        "全角文字と半角文字の間に半角空白を入れる",
                    ));
                }
            }
        }
        if !context.plain {
            continue;
        }
        if builtin.ja_nakaguro_or_halfwidth_space_between_katakana {
            for (i, space) in text.match_indices('\u{3000}') {
                let prev = text[..i].chars().next_back();
                let next = text[i + space.len()..].chars().next();
                if [prev, next]
                    .into_iter()
                    .all(|c| c.is_some_and(|c| chars::contains(KATAKANA_WORD, c)))
                {
                    reports.push(at(
                        i,
                        '\u{3000}',
                        RuleId::JaNakaguroOrHalfwidthSpaceBetweenKatakana,
                        "カタカナ語の間は、全角空白ではなく中黒か半角空白で区切る",
                    ));
                }
            }
        }
        for (enabled, mark, rule, message) in [
            (
                builtin.ja_space_after_exclamation,
                '！',
                RuleId::JaSpaceAfterExclamation,
                "文末の「！」の後に別の文が続くので、直後の半角空白を全角空白にする",
            ),
            (
                builtin.ja_space_after_question,
                '？',
                RuleId::JaSpaceAfterQuestion,
                "文末の「？」の後に別の文が続くので、直後の半角空白を全角空白にする",
            ),
        ] {
            if !enabled {
                continue;
            }
            for (i, _) in text.match_indices(mark) {
                let space = i + mark.len_utf8();
                let mut after = text[space..].chars();
                if after.next() == Some(' ') && after.next().is_some_and(|c| c != '\n') {
                    reports.push(at(space, ' ', rule, message));
                }
            }
        }
    }
    reports
}

/// `text` の `start` (空白の並びの先頭) が行頭の字下げか。本文の範囲の中の直前の改行から `start` までが、
/// 半角空白とタブだけなら字下げである。本文の範囲が改行を含まなければ字下げは無い
pub fn is_indentation(text: &str, start: usize) -> bool {
    text[..start].rfind('\n').is_some_and(|newline| {
        text[newline + 1..start]
            .bytes()
            .all(|b| matches!(b, b' ' | b'\t'))
    })
}
