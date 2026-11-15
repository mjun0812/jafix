use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{Document, Paragraph};
use crate::rules::builtin::NoAiEmphasisPatterns;
use crate::rules::{Report, RuleId};

use super::table::{EMPHASIS_EMOJIS, INFO_PREFIXES};
use super::{allowed, overlaps_code_or_html};

static EMOJI_BOLD: LazyLock<Regex> = LazyLock::new(|| {
    let emojis: Vec<String> = EMPHASIS_EMOJIS
        .iter()
        .map(|emoji| regex::escape(emoji))
        .collect();
    Regex::new(&format!(r"(?:{})\s*\*\*[^*]+\*\*", emojis.join("|"))).unwrap()
});

static INFO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"\*\*({})(?:[：:].*?)?\*\*",
        INFO_PREFIXES.join("|")
    ))
    .unwrap()
});

/// textlintの `(\*\*|__)(.*?)\1` を、後方参照を使えない `regex` crateで2つに分けたもの
static HEADING_BOLDS: LazyLock<[Regex; 2]> = LazyLock::new(|| {
    [
        Regex::new(r"\*\*(.*?)\*\*").unwrap(),
        Regex::new(r"__(.*?)__").unwrap(),
    ]
});

/// 段落の絵文字と太字・太字の情報プレフィックスと、見出しの中の太字を指摘する
pub fn report(src: &str, document: &Document, rule: &NoAiEmphasisPatterns) -> Vec<Report> {
    let mut reports = Vec::new();
    for paragraph in &document.paragraphs {
        // textlintはリスト項目の段落を、項目の元の文字列 (入れ子の項目を含む) で判定する
        let allow_target = match paragraph.list_item_index {
            Some(index) => &src[document.list_items[index].range.clone()],
            None => &src[paragraph.range.clone()],
        };
        if allowed(&rule.allows, allow_target) {
            continue;
        }
        reports.extend(paragraph_reports(src, paragraph, rule));
    }
    if rule.disable_heading_emphasis_patterns {
        return reports;
    }
    for heading in &document.headings {
        let text = &src[heading.range.clone()];
        if allowed(&rule.allows, text) {
            continue;
        }
        reports.extend(
            heading_bolds(text)
                .map(|bold| heading.range.start + bold.start..heading.range.start + bold.end)
                .filter(|range| !overlaps_code_or_html(heading, range.clone()))
                .map(|range| Report {
                    range,
                    rule: RuleId::NoAiEmphasisPatterns.name(),
                    message: Cow::Borrowed("見出しの中で太字を使っている"),
                }),
        );
    }
    reports
}

fn paragraph_reports(src: &str, paragraph: &Paragraph, rule: &NoAiEmphasisPatterns) -> Vec<Report> {
    let text = &src[paragraph.range.clone()];
    let start = paragraph.range.start;
    let list = if paragraph.list_item_index.is_some() {
        "リスト項目で"
    } else {
        ""
    };
    let emoji_bolds: Vec<Range<usize>> = if rule.disable_emoji_emphasis_patterns {
        Vec::new()
    } else {
        EMOJI_BOLD
            .find_iter(text)
            .map(|m| start + m.start()..start + m.end())
            .filter(|range| !overlaps_code_or_html(paragraph, range.clone()))
            .collect()
    };
    let mut reports: Vec<Report> = emoji_bolds
        .iter()
        .map(|range| Report {
            range: range.clone(),
            rule: RuleId::NoAiEmphasisPatterns.name(),
            message: Cow::Owned(format!("{list}絵文字と太字を組み合わせている")),
        })
        .collect();
    if rule.disable_info_patterns {
        return reports;
    }
    for captures in INFO.captures_iter(text) {
        let whole = captures.get(0).expect("一致の全体はある");
        let range = start + whole.start()..start + whole.end();
        if overlaps_code_or_html(paragraph, range.clone())
            || emoji_bolds
                .iter()
                .any(|emoji| range.start < emoji.end && emoji.start < range.end)
        {
            continue;
        }
        reports.push(Report {
            range,
            rule: RuleId::NoAiEmphasisPatterns.name(),
            message: Cow::Owned(format!(
                "{list}太字の情報プレフィックス「**{}**」を使っている",
                &captures[1]
            )),
        });
    }
    reports
}

/// 見出しの元の文字列 `text` の中の太字の範囲。textlintの正規表現と同じく、左から順に、`**` と `__` のうち
/// 先に始まる一致 (同じ位置なら `**`) を採り、その一致の後ろから探し直す
fn heading_bolds(text: &str) -> impl Iterator<Item = Range<usize>> {
    let mut from = 0;
    std::iter::from_fn(move || {
        let bold = HEADING_BOLDS
            .iter()
            .filter_map(|regex| regex.find_at(text, from))
            .min_by_key(|m| m.start())?;
        from = bold.end();
        Some(bold.range())
    })
}
