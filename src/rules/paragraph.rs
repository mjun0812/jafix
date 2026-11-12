use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{Inline, InlineKind, Paragraph};
use crate::sentence::{self, Sentence, is_space};

use super::builtin::{Builtin, CountBy, JaNoMixedPeriod, MaxComma, SentenceLength};
use super::{Report, RuleId};

/// 段落ごとに当てる指摘ルール (`sentence-length`、`max-comma`、`ja-no-mixed-period`、`no-unmatched-pair`) の指摘を返す
pub fn report(src: &str, paragraphs: &[Paragraph], builtin: &Builtin) -> Vec<Report<'static>> {
    let mut reports = Vec::new();
    for paragraph in paragraphs {
        if let Some(rule) = &builtin.ja_no_mixed_period {
            reports.extend(mixed_period(src, paragraph, rule));
        }
        // textlintは、子がリンク1つだけの段落の長さを数えない
        let only_link = matches!(
            paragraph.children.as_slice(),
            [Inline {
                kind: InlineKind::Link { .. },
                ..
            }]
        );
        for sentence in sentence::split(src, paragraph) {
            if let Some(rule) = &builtin.sentence_length
                && !paragraph.blockquote
                && !only_link
            {
                reports.extend(sentence_length(&sentence, rule));
            }
            if let Some(rule) = &builtin.max_comma {
                reports.extend(max_comma(src, &sentence, rule));
            }
            if builtin.no_unmatched_pair {
                reports.extend(unmatched_pairs(src, &sentence));
            }
        }
    }
    reports
}

fn at(range: Range<usize>, rule: RuleId, message: String) -> Report<'static> {
    Report {
        range,
        rule: rule.name(),
        message: Cow::Owned(message),
    }
}

fn sentence_length(sentence: &Sentence, rule: &SentenceLength) -> Option<Report<'static>> {
    let mut text: String = sentence
        .children
        .iter()
        .filter(|child| {
            !(rule.skip_url_string_link
                && matches!(&child.kind, InlineKind::Link { url } if *url == child.text))
        })
        .map(|child| child.text.as_str())
        .collect();
    for pattern in &rule.skip_patterns {
        text = pattern.replace_all(&text, "").into_owned();
    }
    let len = match rule.count_by {
        CountBy::CodeUnits => text.encode_utf16().count(),
        CountBy::CodePoints => text.chars().count(),
    };
    (len > rule.max).then(|| {
        at(
            sentence.range.clone(),
            RuleId::SentenceLength,
            format!("1文が{}文字を超えている ({len}文字)", rule.max),
        )
    })
}

fn max_comma(src: &str, sentence: &Sentence, rule: &MaxComma) -> Option<Report<'static>> {
    let mut count = 0;
    let mut last = None;
    count_commas(src, &sentence.children, &mut count, &mut last);
    let last = last.filter(|_| count > rule.max)?;
    Some(at(
        last..last + ','.len_utf8(),
        RuleId::MaxComma,
        format!("1文の「,」が{}個を超えている ({count}個)", rule.max),
    ))
}

/// インラインコードの外にある `,` を数え、最後の `,` の元のソースでの位置を `last` に入れる。
/// 数えるのは、段落の文字列に入る文字列 (`Inline` の `text`) の `,` である
fn count_commas(src: &str, children: &[Inline], count: &mut usize, last: &mut Option<usize>) {
    for child in children {
        match child.kind {
            InlineKind::Code => {}
            InlineKind::Text | InlineKind::Image => {
                let commas = child.text.matches(',').count();
                if commas > 0 {
                    *count += commas;
                    let raw = &src[child.range.clone()];
                    *last = Some(child.range.start + raw.rfind(',').unwrap_or(0));
                }
            }
            _ => count_commas(src, &child.children, count, last),
        }
    }
}

/// 対になる括弧と引用符。文の区切りの組から `《 》` を除き、`« »` と `‹ ›` を足したもの
const PAIRS: [(char, char); 13] = [
    ('"', '"'),
    ('[', ']'),
    ('(', ')'),
    ('{', '}'),
    ('「', '」'),
    ('（', '）'),
    ('『', '』'),
    ('｛', '｝'),
    ('［', '］'),
    ('〚', '〛'),
    ('【', '】'),
    ('«', '»'),
    ('‹', '›'),
];

/// 文の元のソースのうち、インラインコード・リンク・強調・太字の外で、文の終わりまで閉じない開きごとに指摘を返す。
/// 位置は開きの文字の直後である
fn unmatched_pairs(src: &str, sentence: &Sentence) -> Vec<Report<'static>> {
    let mut ignored = Vec::new();
    ignored_ranges(&sentence.children, &mut ignored);
    // 開いている組の、開きの文字の直後の位置
    let mut open: [Option<usize>; PAIRS.len()] = [None; PAIRS.len()];
    for (i, c) in src[sentence.range.clone()].char_indices() {
        let i = sentence.range.start + i;
        if ignored
            .iter()
            .any(|range: &Range<usize>| range.contains(&i))
        {
            continue;
        }
        let Some(k) = PAIRS
            .iter()
            .position(|&(opening, closing)| c == opening || c == closing)
        else {
            continue;
        };
        let (opening, closing) = PAIRS[k];
        // 同じ組の開きが開いている間の同じ開きと、開いていない組の閉じは無視する。`"` は開きと閉じを兼ねる
        match open[k] {
            Some(_) if c == closing => open[k] = None,
            None if c == opening => open[k] = Some(i + c.len_utf8()),
            _ => {}
        }
    }
    PAIRS
        .iter()
        .zip(open)
        .filter_map(|(&(opening, closing), after)| {
            let after = after?;
            Some(at(
                after..after,
                RuleId::NoUnmatchedPair,
                format!("{opening} に対応する {closing} が無い"),
            ))
        })
        .collect()
}

/// textlintが括弧を数えない要素の範囲。取り消し線・参照形式のリンク・画像・HTMLのタグの中は数える
fn ignored_ranges(children: &[Inline], ranges: &mut Vec<Range<usize>>) {
    for child in children {
        match child.kind {
            InlineKind::Code
            | InlineKind::Link { .. }
            | InlineKind::Emphasis
            | InlineKind::Strong => ranges.push(child.range.clone()),
            _ => ignored_ranges(&child.children, ranges),
        }
    }
}

// textlintの正規表現の範囲をそのまま写す。U+20000からU+2FFFFは、textlintがサロゲートペアで書いている範囲である
static JAPANESE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[々〇〻\x{3400}-\x{4DBF}\x{4E00}-\x{9FFF}\x{F900}-\x{FAFF}\x{20000}-\x{2FFFF}ぁ-んァ-ヶ]",
    )
    .unwrap()
});

static EMOJI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\p{Extended_Pictographic}$").unwrap());

/// 段落の最後の子が日本語を含む文字列のときだけ、その末尾が句点で終わるかを見る
fn mixed_period(
    src: &str,
    paragraph: &Paragraph,
    rule: &JaNoMixedPeriod,
) -> Option<Report<'static>> {
    if paragraph.list_item || paragraph.blockquote || paragraph.footnote {
        return None;
    }
    let last = paragraph
        .children
        .last()
        .filter(|child| child.kind == InlineKind::Text)?;
    let text = &src[last.range.clone()];
    if !JAPANESE.is_match(text) {
        return None;
    }
    let end = text.chars().next_back()?;
    let period = &rule.period_mark;
    if is_space(end) {
        return Some(at(
            last.range.start + text.trim_end_matches(is_space).len()..last.range.end,
            RuleId::JaNoMixedPeriod,
            format!("文末が「{period}」で終わっていない: 末尾の空白を削除する"),
        ));
    }
    if "!?！？)）」』".contains(end)
        || (rule.allow_emoji_at_end && EMOJI.is_match(end.encode_utf8(&mut [0; 4])))
        || std::iter::once(period)
            .chain(&rule.allow_period_marks)
            .any(|mark| text.ends_with(mark.as_str()))
    {
        return None;
    }
    let message = if matches!(end, '。' | '.') {
        format!("文末が「{period}」で終わっていない: 「{end}」を「{period}」にする")
    } else {
        format!("文末が「{period}」で終わっていない")
    };
    Some(at(
        last.range.end - end.len_utf8()..last.range.end,
        RuleId::JaNoMixedPeriod,
        message,
    ))
}
