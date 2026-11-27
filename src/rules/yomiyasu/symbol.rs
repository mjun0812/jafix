//! 記号。yomiyasu 3.4.1の `dash_decoration`、`dash_insertion`、`dash_list_ending`、`emoji_prohibited` の検査

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::Document;
use crate::rules::builtin::Builtin;
use crate::rules::{Report, RuleId};

use super::{BRACKETS, END_DECORATION, Visible, blank, finding, lines, paired_ranges};

pub(super) fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    let dashes = [
        RuleId::YomiyasuDashDecoration,
        RuleId::YomiyasuDashInsertion,
        RuleId::YomiyasuDashListEnding,
    ];
    if dashes.iter().any(|&rule| builtin.is_enabled(rule)) && src.contains(DASHES) {
        // 見出しも見る。表のセルは「該当なし」の意味でダッシュを置くことが多いので見ない。引用も見ない
        let blocks = document
            .paragraphs
            .iter()
            .chain(&document.headings)
            .filter(|block| !block.blockquote);
        for block in blocks {
            for line in lines(src, &block.range).filter(|line| src[line.clone()].contains(DASHES)) {
                for (rule, range, message) in dashes_in(&Visible::new(src, &line, document)) {
                    if builtin.is_enabled(rule) {
                        reports.push(finding(range, rule, message));
                    }
                }
            }
        }
    }
    if builtin.yomiyasu_emoji_prohibited {
        report_emoji(src, document, &mut reports);
    }
    reports
}

/// ダッシュの記号。en dash (`–`)、長音 (`ー`)、波ダッシュ (`〜`) は含めない
const DASHES: [char; 4] = ['—', '―', '─', '━'];

static DASH_RUN: LazyLock<Regex> = LazyLock::new(|| Regex::new("[—―─━]+").unwrap());

/// 1行のダッシュの指摘。yomiyasuと同じく、行を名詞の並びの終わり、挿入句、それ以外の飾りの順に1つのルールへ振り分け、
/// そのルールで当たった箇所ごとに1件を返す
fn dashes_in(visible: &Visible) -> Vec<(RuleId, Range<usize>, String)> {
    // 木構造や図に使う罫線
    static DIAGRAM: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[│┃├┣└┗┌┏┐┓┘┛┬┳┴┻┼╋╭╮╰╯]").unwrap());
    static NOT_PROSE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[\p{P}\p{S}\p{Z}—―─━]*$").unwrap());
    // 文頭から、短い語を読点で3つ以上並べ、ダッシュで閉じて文を終える形 (「速さ、軽さ、静けさ──。」)
    static LIST_ENDING: LazyLock<Regex> = LazyLock::new(|| {
        let item = r"[^、。！？!?\s—―─━]{1,12}";
        Regex::new(&format!(
            r"^\s*{item}(?:、[ \t　]*{item}){{2,}}[ \t　]*([—―─━]+)(?:[0-9#*]\x{{FE0F}}?\x{{20E3}}|\s|{END_DECORATION})*(?:[。．.]|$)"
        ))
        .unwrap()
    });
    // ダッシュで補足や言い換えを挟む形 (「今日——正確には昨日——連絡しました。」)
    static INSERTION: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[—―─━]{1,2}[^—―─━。\n]{1,40}[—―─━]{1,2}").unwrap());
    if !visible.text.contains(DASHES) {
        return Vec::new();
    }
    let text = masked_dashes(visible);
    let trimmed = text.trim();
    let rest = trimmed.trim_matches(|c| DASHES.contains(&c) || c == ' ' || c == '　');
    if !trimmed.contains(DASHES) || DIAGRAM.is_match(trimmed) || NOT_PROSE.is_match(rest) {
        return Vec::new();
    }
    let mut hits = Vec::new();
    let mut start = 0;
    for sentence in text.split_inclusive(['。', '！', '？', '!', '?']) {
        if let Some(dash) = LIST_ENDING.captures(sentence).and_then(|hit| hit.get(1)) {
            hits.push((
                RuleId::YomiyasuDashListEnding,
                start + dash.start()..start + dash.end(),
                "名詞の並びをダッシュで終えている".to_owned(),
            ));
        }
        start += sentence.len();
    }
    if hits.is_empty() {
        hits.extend(INSERTION.find_iter(&text).map(|insertion| {
            (
                RuleId::YomiyasuDashInsertion,
                insertion.range(),
                "ダッシュで挟んだ挿入句を使っている".to_owned(),
            )
        }));
    }
    if hits.is_empty() {
        hits.extend(DASH_RUN.find_iter(&text).map(|run| {
            (
                RuleId::YomiyasuDashDecoration,
                run.range(),
                format!("ダッシュ「{}」を使っている", run.as_str()),
            )
        }));
    }
    hits.into_iter()
        .map(|(rule, range, message)| (rule, visible.source(&text, range), message))
        .collect()
}

/// ダッシュを探す行の文字列。出典を示す行頭のダッシュ、リンクの後ろで著者を添えるダッシュ、区間を表すダッシュ
/// (`東京―大阪間`) と、リンクの文字列、かぎ括弧と角括弧の中を、1文字ずつ半角空白にする
fn masked_dashes(visible: &Visible) -> String {
    let text = &visible.text;
    let mut credits = visible.links.clone();
    if let Some(dash) = DASH_RUN.find(text)
        && text[..dash.start()].trim().is_empty()
        && is_credit(
            text[dash.start()..]
                .trim_start_matches(|c| DASHES.contains(&c) || matches!(c, ' ' | '\t' | '　')),
        )
    {
        credits.push(dash.range());
    }
    credits.extend(
        DASH_RUN
            .find_iter(text)
            .filter(|run| {
                let before = text[..run.start()].trim_end_matches([' ', '\t']);
                before.ends_with(']') || visible.links.iter().any(|link| link.end == before.len())
            })
            .map(|run| run.range()),
    );
    let text = blank(Cow::Borrowed(text.as_str()), &credits);
    let brackets = paired_ranges(text.char_indices(), &BRACKETS);
    let text = blank(text, &brackets);
    let routes = route_dashes(&text);
    blank(text, &routes).into_owned()
}

/// 出典のラベル、著者と書名、英語の著者名と書誌情報
fn is_credit(text: &str) -> bool {
    static LABEL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)^(?:出典|引用元|著者|Source|Author)\s*[:：]").unwrap());
    static BOOK: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[一-龥々ァ-ヴーA-Za-z .・'’-]+『[^』]+』(?:[ \t　]*(?:（[^）]*）|\([^)]*\)))*[。.]?$")
            .unwrap()
    });
    static ENGLISH: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[A-Z][A-Za-z.'’−-]*(?:[ \t]+[A-Z][A-Za-z.'’−-]*)+,[ \t]+\S").unwrap()
    });
    LABEL.is_match(text) || BOOK.is_match(text) || ENGLISH.is_match(text)
}

/// 区間や路線を表すダッシュ (`東京―大阪間`、`〜線`、`〜便`) の範囲。1つのダッシュが漢字・カタカナの語の間にあり、右の語が
/// 「間」「線」「便」で終わるものである。「人間」「目線」のような普通名詞で終わる語は、区間とみない
fn route_dashes(text: &str) -> Vec<Range<usize>> {
    const NOT_ROUTE: [&str; 16] = [
        "人間",
        "時間",
        "期間",
        "空間",
        "世間",
        "仲間",
        "手間",
        "目線",
        "視線",
        "動線",
        "光線",
        "伏線",
        "生命線",
        "方便",
        "利便",
        "不便",
    ];
    let noun =
        |c: char| ('一'..='龥').contains(&c) || ('ァ'..='ヴ').contains(&c) || "ー々".contains(c);
    DASH_RUN
        .find_iter(text)
        .filter(|run| {
            if run.as_str().chars().count() != 1
                || !text[..run.start()].chars().next_back().is_some_and(noun)
            {
                return false;
            }
            let right = &text[run.end()..];
            let destination = &right[..right.find(|c| !noun(c)).unwrap_or(right.len())];
            destination.char_indices().any(|(i, c)| {
                let head = &destination[..i + c.len_utf8()];
                let suffix = if head.ends_with("区間") { 2 } else { 1 };
                "間線便".contains(c)
                    && head.chars().count() > suffix
                    && !NOT_ROUTE.iter().any(|word| head.ends_with(word))
            })
        })
        .map(|run| run.range())
        .collect()
}

/// 本文の絵文字を、間に何も挟まずに続く絵文字 (ZWJでつないだものを含む) ごとに指摘する。見出し・段落・リスト・表のセル・
/// 引用のすべてを見る
fn report_emoji(src: &str, document: &Document, reports: &mut Vec<Report>) {
    // Unicodeの絵文字の性質を持つ字から、数字・`#`・`*`・地域指示記号を除く。文字として表示するのが標準の36字は、
    // 異体字セレクタ U+FE0F が続くときだけ絵文字とみる。U+FE0E が続く字は文字として表示するので当てない
    // (先に当てて読み飛ばす)。キーキャップと、地域指示記号2つの組 (旗) も絵文字とみる
    static EMOJI: LazyLock<Regex> = LazyLock::new(|| {
        let text_default = "©®‼⁉™ℹ↔↕↖↗↘↙↩↪Ⓜ▪▫▶◀◻◼⤴⤵⬅⬆⬇〰〽㊗㊙🅰🅱🅾🅿🈂🈷";
        Regex::new(&format!(
            concat!(
                r"(?:[\p{{Emoji}}--[0-9#*\p{{Regional_Indicator}}]]\x{{FE0E}}|[0-9#*]\x{{FE0F}}\x{{20E3}}",
                r"|\p{{Regional_Indicator}}{{2}}|[{0}]\x{{FE0F}}|[\p{{Emoji}}--[0-9#*\p{{Regional_Indicator}}{0}]]\x{{FE0F}}?)",
                r"[\x{{E0020}}-\x{{E007F}}]*"
            ),
            text_default
        ))
        .unwrap()
    });
    for part in &document.texts {
        let text = &src[part.range.clone()];
        let mut runs: Vec<Range<usize>> = Vec::new();
        for emoji in EMOJI.find_iter(text) {
            if emoji.as_str().contains('\u{FE0E}') {
                continue;
            }
            match runs.last_mut() {
                Some(run)
                    if text[run.end..emoji.start()]
                        .chars()
                        .all(|c| c == '\u{200D}') =>
                {
                    run.end = emoji.end();
                }
                _ => runs.push(emoji.range()),
            }
        }
        for run in runs {
            let range = part.range.start + run.start..part.range.start + run.end;
            let message = format!("絵文字「{}」を使っている", &src[range.clone()]);
            reports.push(finding(range, RuleId::YomiyasuEmojiProhibited, message));
        }
    }
}
