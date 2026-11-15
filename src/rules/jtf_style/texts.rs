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
use super::super::text::NUMBER_PATTERNS;
use super::super::{Finding, FixRuleFinding, Report, RuleId};
use super::{
    HANKAKU_KANA, JAPANESE, JAPANESE_WITHOUT_MARKS, JtfRule, KANJI, KATAKANA_WORD, contains,
};

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

/// 指摘ルールになる側の指摘。`3.3` は `requireOutsideHalfParentheses = true` の側、`4.2.1`・`4.2.2` は両方の部分である。
/// `4.3.1` の括弧は複数の条件に当たることがあり、同じ括弧の指摘は呼び出し側が1件にまとめる
pub fn report(src: &str, texts: &[Prose], rules: &JtfStyle) -> Vec<Report> {
    let mut reports = Vec::new();
    for Prose { range, context } in texts {
        if !context.plain {
            continue;
        }
        let text = &src[range.clone()];
        let mut push = |local: Range<usize>, rule: JtfRule, message: Cow<'static, str>| {
            reports.push(Report {
                range: range.start + local.start..range.start + local.end,
                rule: rule.name(),
                message,
            });
        };
        let japanese = |c: Option<char>| c.is_some_and(|c| contains(JAPANESE, c));
        let japanese_without_marks =
            |c: Option<char>| c.is_some_and(|c| contains(JAPANESE_WITHOUT_MARKS, c));
        let katakana_word = |c: Option<char>| c.is_some_and(|c| contains(KATAKANA_WORD, c));
        let digit = |c: Option<char>| c.is_some_and(|c| c.is_ascii_digit());
        for (i, c) in text.char_indices() {
            let here = i..i + c.len_utf8();
            let prev = text[..i].chars().next_back();
            let next = text[here.end..].chars().next();
            match c {
                ',' | '.' => {
                    if rules.enabled(JtfRule::R1_2_1) && (japanese(prev) || japanese(next)) {
                        let japanese_mark = if c == ',' { '、' } else { '。' };
                        push(
                            here.clone(),
                            JtfRule::R1_2_1,
                            Cow::Owned(format!(
                                "和文の句読点に「{c}」を使わず「{japanese_mark}」にする"
                            )),
                        );
                    }
                    if c == '.' && rules.enabled(JtfRule::R4_1_3) && japanese_without_marks(prev) {
                        push(
                            here,
                            JtfRule::R4_1_3,
                            Cow::Borrowed("和文の句点に「.」を使わず「。」にする"),
                        );
                    }
                }
                '，' | '．' if rules.enabled(JtfRule::R1_2_2) => {
                    let half_width = if c == '，' { ',' } else { '.' };
                    push(
                        here,
                        JtfRule::R1_2_2,
                        Cow::Owned(format!("全角の「{c}」を使わず半角の「{half_width}」にする")),
                    );
                }
                '\u{3000}'
                    if rules.enabled(JtfRule::R3_2)
                        && katakana_word(prev)
                        && katakana_word(next) =>
                {
                    push(
                        here,
                        JtfRule::R3_2,
                        Cow::Borrowed("カタカナ語の間は、全角空白ではなく中黒か半角空白で区切る"),
                    );
                }
                '。' if rules.enabled(JtfRule::R4_1_1)
                    && matches!(next, Some('」' | '）' | ')')) =>
                {
                    push(
                        here,
                        JtfRule::R4_1_1,
                        Cow::Borrowed("閉じ括弧の前に「。」を付けない"),
                    );
                }
                '!' if rules.enabled(JtfRule::R4_2_1) && japanese_without_marks(prev) => {
                    push(
                        here,
                        JtfRule::R4_2_1,
                        Cow::Borrowed("感嘆符は全角の「！」にする"),
                    );
                }
                '?' if rules.enabled(JtfRule::R4_2_2) && japanese(prev) => {
                    push(
                        here,
                        JtfRule::R4_2_2,
                        Cow::Borrowed("疑問符は全角の「？」にする"),
                    );
                }
                ' ' if next.is_some_and(|c| c != '\n') => {
                    let (rule, mark) = match prev {
                        Some('！') => (JtfRule::R4_2_1, '！'),
                        Some('？') => (JtfRule::R4_2_2, '？'),
                        _ => continue,
                    };
                    if rules.enabled(rule) {
                        push(
                            here,
                            rule,
                            Cow::Owned(format!(
                                "文末の「{mark}」の後に別の文が続くので、直後の半角空白を全角空白にする"
                            )),
                        );
                    }
                }
                '~' if rules.enabled(JtfRule::R4_2_5) && digit(prev) && digit(next) => {
                    push(
                        here,
                        JtfRule::R4_2_5,
                        Cow::Borrowed("数値の範囲は「~」ではなく全角の「〜」にする"),
                    );
                }
                '-' if rules.enabled(JtfRule::R4_2_6)
                    && (japanese(prev) || japanese(next))
                    && !digit(next) =>
                {
                    push(
                        here,
                        JtfRule::R4_2_6,
                        Cow::Borrowed(
                            "和文にハイフン「-」を使わない。住所や電話番号の区切りは例外",
                        ),
                    );
                }
                ':' if rules.enabled(JtfRule::R4_2_7) && japanese(prev) => {
                    push(
                        here,
                        JtfRule::R4_2_7,
                        Cow::Borrowed("コロンは全角の「：」にする"),
                    );
                }
                ';' if rules.enabled(JtfRule::R4_2_8) && japanese(prev) => {
                    push(
                        here,
                        JtfRule::R4_2_8,
                        Cow::Borrowed("和文にセミコロン「;」を使わない"),
                    );
                }
                '\u{2012}'..='\u{2015}' if rules.enabled(JtfRule::R4_2_9) && japanese(prev) => {
                    push(
                        here,
                        JtfRule::R4_2_9,
                        Cow::Owned(format!("和文にダッシュ「{c}」を使わない")),
                    );
                }
                '(' | ')' if rules.enabled(JtfRule::R4_3_1) => {
                    let at_the_edge = if c == '(' {
                        i == 0 && japanese(next)
                    } else {
                        next.is_none() && japanese(prev)
                    };
                    if at_the_edge || japanese(prev) && japanese(next) {
                        push(here, JtfRule::R4_3_1, round_bracket_message(c));
                    }
                }
                '[' | ']' if rules.enabled(JtfRule::R4_3_2) && japanese(prev) => {
                    let full_width = if c == '[' { '［' } else { '］' };
                    push(
                        here,
                        JtfRule::R4_3_2,
                        Cow::Owned(format!("半角の大括弧「{c}」を全角の「{full_width}」にする")),
                    );
                }
                _ if rules.enabled(JtfRule::R2_1_2) && contains(KANJI, c) && !is_joyo_kanji(c) => {
                    push(
                        here,
                        JtfRule::R2_1_2,
                        Cow::Owned(format!("「{c}」は常用漢字表にない漢字")),
                    );
                }
                _ => {}
            }
        }
        if rules.enabled(JtfRule::R4_3_1) {
            for captures in PARENTHESES_AROUND_JAPANESE.captures_iter(text) {
                for paren in [captures.get(1), captures.get(2)] {
                    let paren = paren.expect("2つの括弧は必ず捕まえる");
                    let c = paren.as_str().chars().next().expect("括弧は1文字である");
                    push(paren.range(), JtfRule::R4_3_1, round_bracket_message(c));
                }
            }
        }
        if rules.enabled(JtfRule::R2_1_10) {
            for run in runs(text, |c| c.is_ascii_digit() || c == ',') {
                let Some((zeros, digits)) = text[run.clone()].split_once(',') else {
                    continue;
                };
                if !zeros.is_empty()
                    && zeros.bytes().all(|b| b == b'0')
                    && !digits.is_empty()
                    && digits.bytes().all(|b| b.is_ascii_digit())
                {
                    let comma = run.start + zeros.len();
                    push(
                        comma..comma + 1,
                        JtfRule::R2_1_10,
                        Cow::Borrowed("小数点に「,」を使わず「.」にする"),
                    );
                }
            }
        }
        if rules.enabled(JtfRule::R2_2_2) {
            for pattern in NUMBER_PATTERNS.iter() {
                let ignored: Vec<Range<usize>> = pattern
                    .ignore
                    .iter()
                    .flat_map(|ignore| ignore.find_iter(text).map(|m| m.range()))
                    .collect();
                for m in pattern.regex.find_iter(text) {
                    let followed_by_excluded = pattern
                        .not_followed_by
                        .is_some_and(|c| text[m.end()..].starts_with(c));
                    let overlaps_ignored = ignored
                        .iter()
                        .any(|ignore| ignore.start < m.end() && m.start() < ignore.end);
                    if !followed_by_excluded && !overlaps_ignored {
                        push(
                            m.range(),
                            JtfRule::R2_2_2,
                            Cow::Owned(format!("{}: {}", pattern.message, m.as_str())),
                        );
                    }
                }
            }
        }
        if rules.enabled(JtfRule::R3_3) && rules.parentheses.require_outside_half_parentheses {
            for (i, paren) in text.match_indices(['(', ')']) {
                let neighbour = if paren == "(" {
                    text[..i].chars().next_back()
                } else {
                    text[i + 1..].chars().next()
                };
                if japanese(neighbour) {
                    push(
                        i..i + 1,
                        JtfRule::R3_3,
                        Cow::Borrowed("半角の括弧の外側に半角空白を入れる"),
                    );
                }
            }
        }
    }
    reports
}

/// `4.3.1` の1つ目の条件。`.` は改行に当たらず、最長に当たるので、行の中で最初の括弧と、日本語を挟んだ最後の括弧を捕まえる
static PARENTHESES_AROUND_JAPANESE: LazyLock<Regex> = LazyLock::new(|| {
    let japanese: String = JAPANESE
        .iter()
        .map(|range| {
            format!(
                r"\x{{{:X}}}-\x{{{:X}}}",
                u32::from(*range.start()),
                u32::from(*range.end())
            )
        })
        .collect();
    Regex::new(&format!(r"([()])(?:.*[{japanese}].*)([()])")).unwrap()
});

fn round_bracket_message(c: char) -> Cow<'static, str> {
    let full_width = if c == '(' { '（' } else { '）' };
    Cow::Owned(format!("半角の丸括弧「{c}」を全角の「{full_width}」にする"))
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

fn is_joyo_kanji(c: char) -> bool {
    super::joyo::JOYO_KANJI.binary_search(&c).is_ok()
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
