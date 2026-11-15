//! 文体の3ルール (`1.1.1`・`1.1.3`・`1.1.5`) と見出しの `1.1.2`。敬体と常体の判定は、`analyze-desumasu-dearu` 2.1.5 の
//! 4つの正規表現を写す。`1.1.1` は文書の全体の本文、`1.1.3` はリストごとの直下の項目、`1.1.5` は文書のすべての
//! インラインの画像のaltで数える

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{Document, InlineNodeKind, Paragraph};

use super::super::Report;
use super::super::builtin::JtfStyle;
use super::{JtfRule, texts_in};

/// 4ルールの指摘
pub fn report(src: &str, document: &Document, rules: &JtfStyle) -> Vec<Report> {
    let rules_here = [
        JtfRule::R1_1_1,
        JtfRule::R1_1_2,
        JtfRule::R1_1_3,
        JtfRule::R1_1_5,
    ];
    if !rules_here.into_iter().any(|rule| rules.enabled(rule)) {
        return Vec::new();
    }
    let mut reports = Vec::new();
    if rules.enabled(JtfRule::R1_1_1) {
        let matches: Vec<StyleMatch> = document
            .texts
            .iter()
            .filter(|prose| {
                let context = prose.context;
                !(context.link || context.blockquote || context.emphasis || context.list_item)
            })
            .flat_map(|prose| style_matches(src, prose.range.clone()))
            .collect();
        reports.extend(style_reports(src, &matches, JtfRule::R1_1_1, "本文"));
    }
    if rules.enabled(JtfRule::R1_1_2) {
        for heading in document
            .headings
            .iter()
            .filter(|heading| !heading.blockquote)
        {
            let source = src[heading.range.clone()].trim_end();
            if source.ends_with('。') {
                let end = heading.range.start + source.len();
                reports.push(Report {
                    range: end - '。'.len_utf8()..end,
                    rule: JtfRule::R1_1_2.name(),
                    message: Cow::Borrowed("見出しの文末に「。」を付けない"),
                });
            }
        }
    }
    if rules.enabled(JtfRule::R1_1_3) {
        reports.extend(list_reports(src, document));
    }
    if rules.enabled(JtfRule::R1_1_5) {
        let matches: Vec<StyleMatch> = document
            .inline_nodes
            .iter()
            .filter_map(|node| match &node.kind {
                InlineNodeKind::Image { alt } => Some(style_matches(src, alt.clone())),
                _ => None,
            })
            .flatten()
            .collect();
        reports.extend(style_reports(
            src,
            &matches,
            JtfRule::R1_1_5,
            "図表のキャプション",
        ));
    }
    reports
}

/// `1.1.3` の指摘。リストごとに、直下の項目の直下の段落で、句点の有無と文体の多数決をとる
fn list_reports(src: &str, document: &Document) -> Vec<Report> {
    // 項目の番号ごとの、その項目を最も内側の項目とする段落
    let mut item_paragraphs: BTreeMap<usize, Vec<&Paragraph>> = BTreeMap::new();
    for paragraph in &document.paragraphs {
        if let Some(index) = paragraph.list_item_index {
            item_paragraphs.entry(index).or_default().push(paragraph);
        }
    }
    // リストの番号ごとの、直下の項目の先頭の位置と段落
    let mut lists: BTreeMap<usize, Vec<(usize, &[&Paragraph])>> = BTreeMap::new();
    for (index, paragraphs) in &item_paragraphs {
        let item = &document.list_items[*index];
        lists
            .entry(item.list)
            .or_default()
            .push((item.range.start, paragraphs));
    }
    let mut reports = Vec::new();
    for items in lists.values() {
        // 項目ごとの、最後の段落が `。` で終わるか
        let periods: Vec<bool> = items
            .iter()
            .map(|(_, paragraphs)| {
                let last = paragraphs.last().expect("段落を持つ項目だけを集めた");
                src[last.range.clone()].trim_end().ends_with('。')
            })
            .collect();
        let with_period = periods.iter().filter(|&&period| period).count();
        let without_period = periods.len() - with_period;
        if with_period > 0 && without_period > 0 {
            let flags_period = with_period < without_period;
            let message = if flags_period {
                "箇条書きの文末の「。」の有無を揃える: 「。」を外す"
            } else {
                "箇条書きの文末の「。」の有無を揃える: 「。」を付ける"
            };
            for ((start, _), &period) in items.iter().zip(&periods) {
                if period == flags_period {
                    // 項目はリストの記号 (`-` `*` `+` か数字) で始まり、どれも1バイトのASCIIである
                    reports.push(Report {
                        range: *start..start + 1,
                        rule: JtfRule::R1_1_3.name(),
                        message: Cow::Borrowed(message),
                    });
                }
            }
        }
        let matches: Vec<StyleMatch> = items
            .iter()
            .flat_map(|(_, paragraphs)| paragraphs.iter())
            .flat_map(|paragraph| texts_in(&document.texts, &paragraph.range))
            .flat_map(|prose| style_matches(src, prose.range.clone()))
            .collect();
        reports.extend(style_reports(src, &matches, JtfRule::R1_1_3, "箇条書き"));
    }
    reports
}

/// `matches` の多数決で、少ないほうの文体の一致を指摘する。`target` は文面の「<target>を…に揃える」に入れる対象の名前
fn style_reports(src: &str, matches: &[StyleMatch], rule: JtfRule, target: &str) -> Vec<Report> {
    let Some(flagged) = minority(matches) else {
        return Vec::new();
    };
    let (expected, found) = match flagged {
        Style::Dearu => ("敬体(ですます調)", "常体"),
        Style::Desumasu => ("常体(である調)", "敬体"),
    };
    matches
        .iter()
        .filter(|found_match| found_match.style == flagged)
        .map(|found_match| Report {
            range: found_match.range.clone(),
            rule: rule.name(),
            message: Cow::Owned(format!(
                "{target}を{expected}に揃える: 「{}」は{found}",
                &src[found_match.range.clone()]
            )),
        })
        .collect()
}

/// 文体
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    /// 敬体 (ですます調)
    Desumasu,
    /// 常体 (である調)
    Dearu,
}

/// 文体の1つの一致。`range` は元のソース上のバイト範囲
struct StyleMatch {
    range: Range<usize>,
    style: Style,
}

/// 文体ごとの、接続の形と文末の形。常体の文末の形の `ではない` と `ないか` は、`analyze-desumasu-dearu` では全角の
/// 縦線で区切った1つの形 `ではない｜ないか` で、どの文にも当たらない。選択の書き誤りなので2つの形にする
static PATTERNS: LazyLock<[(Regex, Style); 4]> = LazyLock::new(|| {
    [
        (
            Regex::new("でしたが|でしたので|ですので|ですが").unwrap(),
            Style::Desumasu,
        ),
        (
            Regex::new("(?:です|ます|ました|ません|ですね|でしょうか|ください|ませ)。").unwrap(),
            Style::Desumasu,
        ),
        (
            Regex::new("のだが|ないかと|であるから").unwrap(),
            Style::Dearu,
        ),
        (
            Regex::new(
                "(?:だ|である|った|ではない|ないか|しろ|しなさい|いただきたい|いただく|ならない|あろう|られる)。",
            )
            .unwrap(),
            Style::Dearu,
        ),
    ]
});

/// 元のソースの `range` の文字列の中の、接続の形と文末の形の一致を、それぞれ重ねずにすべて返す
fn style_matches(src: &str, range: Range<usize>) -> Vec<StyleMatch> {
    let text = &src[range.clone()];
    PATTERNS
        .iter()
        .flat_map(|(pattern, style)| {
            pattern.find_iter(text).map(|found| StyleMatch {
                range: range.start + found.start()..range.start + found.end(),
                style: *style,
            })
        })
        .collect()
}

/// 多数決で指摘する側の文体。敬体と常体の両方が1件以上あるときだけ返し、多いほうの逆を、同数なら常体を返す
fn minority(matches: &[StyleMatch]) -> Option<Style> {
    let desumasu = matches
        .iter()
        .filter(|found| found.style == Style::Desumasu)
        .count();
    let dearu = matches.len() - desumasu;
    if desumasu == 0 || dearu == 0 {
        return None;
    }
    if dearu > desumasu {
        return Some(Style::Desumasu);
    }
    Some(Style::Dearu)
}
