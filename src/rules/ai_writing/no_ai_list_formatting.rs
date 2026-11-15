use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{Document, ListItem, Paragraph};
use crate::rules::builtin::NoAiListFormatting;
use crate::rules::{Report, RuleId};

use super::table::{BOLD_LIST_ITEM, LIST_EMOJIS};
use super::{allowed, overlaps_code_or_html};

static BOLD: LazyLock<Regex> = LazyLock::new(|| Regex::new(BOLD_LIST_ITEM).unwrap());

static EMOJI: LazyLock<Regex> = LazyLock::new(|| {
    let alternation: Vec<String> = LIST_EMOJIS
        .iter()
        .map(|emoji| regex::escape(emoji))
        .collect();
    Regex::new(&alternation.join("|")).unwrap()
});

/// リスト項目ごとに、先頭の「**太字**: 説明」型と、項目に属する段落の絵文字を、それぞれ最初の1件だけ指摘する
pub fn report(src: &str, document: &Document, rule: &NoAiListFormatting) -> Vec<Report<'static>> {
    let mut reports = Vec::new();
    for item in &document.list_items {
        let text = &src[item.range.clone()];
        if allowed(&rule.allows, text) {
            continue;
        }
        let paragraphs = paragraphs_in(&document.paragraphs, item);
        if !rule.disable_bold_list_items
            && let Some(report) = bold(text, item.range.start, paragraphs)
        {
            reports.push(report);
        }
        if rule.disable_emoji_list_items {
            continue;
        }
        let Some((range, emoji)) = paragraphs.iter().find_map(|paragraph| {
            EMOJI
                .find_iter(&src[paragraph.range.clone()])
                .map(|m| {
                    let start = paragraph.range.start + m.start();
                    (start..start + m.len(), m.as_str())
                })
                .find(|(range, _)| !overlaps_code_or_html(paragraph, range.clone()))
        }) else {
            continue;
        };
        // 親の項目の範囲には子の項目の段落も入るので、子に絵文字があれば、親と子が同じ箇所を指摘する
        if reports.iter().any(|report| report.range == range) {
            continue;
        }
        reports.push(Report {
            range,
            rule: RuleId::NoAiListFormatting.name(),
            message: Cow::Owned(format!("リスト項目で絵文字「{emoji}」を使っている")),
        });
    }
    reports
}

/// `item` の範囲に入る段落 (入れ子の項目の段落を含む)。`paragraphs` はソースの順に並ぶ
fn paragraphs_in<'p>(paragraphs: &'p [Paragraph], item: &ListItem) -> &'p [Paragraph] {
    let start = paragraphs.partition_point(|paragraph| paragraph.range.start < item.range.start);
    let len =
        paragraphs[start..].partition_point(|paragraph| paragraph.range.start < item.range.end);
    &paragraphs[start..start + len]
}

/// リスト項目の元の文字列 `text` (ソースでの開始位置が `start`) の先頭の「**太字**: 説明」型の指摘
fn bold(text: &str, start: usize, paragraphs: &[Paragraph]) -> Option<Report<'static>> {
    let captures = BOLD.captures(text)?;
    let whole = captures.get(0)?;
    // textlintの末尾の先読み `(?=\s)`: ハイフン・ダッシュの直後が空白でなければ当たらない
    if captures.get(3).is_some() && !text[whole.end()..].starts_with(char::is_whitespace) {
        return None;
    }
    let separator = captures.get(2).or(captures.get(3))?.as_str();
    let range = start + whole.start()..start + whole.end();
    let end = range.end - 1;
    if paragraphs
        .iter()
        .find(|paragraph| paragraph.range.contains(&end))
        .is_some_and(|paragraph| overlaps_code_or_html(paragraph, range.clone()))
    {
        return None;
    }
    let name = match separator {
        ":" | "：" => "コロン",
        "-" => "ハイフン",
        _ => "ダッシュ",
    };
    Some(Report {
        range,
        rule: RuleId::NoAiListFormatting.name(),
        message: Cow::Owned(format!("リスト項目の先頭で太字と{name}を組み合わせている")),
    })
}
