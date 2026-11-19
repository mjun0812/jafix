//! yomiyasu 3.2.0の `bold_not_rendered`、`instruction_residue`、`meta_filler` のチャット応答の検査。

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{Document, Prose};
use crate::sentence::{self, HardBreak};

use super::builtin::Builtin;
use super::{Report, RuleId};

pub fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    if builtin.yomiyasu_bold_not_rendered && src.contains("**") {
        let blocks = document
            .paragraphs
            .iter()
            .chain(&document.headings)
            .map(|paragraph| &paragraph.range)
            .chain(&document.table_cells);
        for block in blocks {
            report_bold(src, block, prose_in(document, block), &mut reports);
        }
    }
    if builtin.yomiyasu_instruction_residue || builtin.yomiyasu_chat_response {
        for paragraph in document.paragraphs.iter().filter(|p| !p.blockquote) {
            let mut sentences = sentence::split(src, paragraph, HardBreak::ListItems)
                .into_iter()
                .map(|sentence| {
                    let text = visible_sentence(src, &sentence.range, document);
                    (sentence.range, text)
                })
                .peekable();
            while let Some((range, text)) = sentences.next() {
                let text = without_trailing_decoration(text.trim());
                let following = sentences.peek().map_or("", |(_, next)| next.as_str());
                if builtin.yomiyasu_instruction_residue && instruction_residue(text, following) {
                    reports.push(finding(
                        range.clone(),
                        RuleId::YomiyasuInstructionResidue,
                        "執筆時の指示が本文に残っていないか確認する",
                    ));
                }
                if builtin.yomiyasu_chat_response && chat_response(text) {
                    reports.push(finding(
                        range,
                        RuleId::YomiyasuChatResponse,
                        "チャット応答の定型句が本文に残っていないか確認する",
                    ));
                }
            }
        }
    }
    reports
}

fn report_bold(src: &str, block: &Range<usize>, prose: &[Prose], reports: &mut Vec<Report>) {
    static STARS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*{2,}").unwrap());
    let mut runs = prose.iter().flat_map(|part| {
        STARS
            .find_iter(&src[part.range.clone()])
            .filter_map(|star| {
                let start = part.range.start + star.start();
                let end = part.range.start + star.end();
                let bytes = src.as_bytes();
                let escaped = bytes[..start]
                    .iter()
                    .rev()
                    .take_while(|&&b| b == b'\\')
                    .count()
                    % 2
                    == 1;
                // 本文のTextノードに残った記号だけを見る。既存の強調やエスケープに属する記号の残りは組にしない。
                (!escaped && !bytes[..start].ends_with(b"*") && !bytes[end..].starts_with(b"*"))
                    .then_some(start..end)
            })
    });
    let Some(mut open) = runs.next() else {
        return;
    };
    while let Some(close) = runs.next() {
        let middle = &src[open.end..close.start];
        let can_open = can_open_emphasis(
            src[block.start..open.start].chars().next_back(),
            middle.chars().next(),
        );
        let can_close = can_open_emphasis(
            src[close.end..block.end].chars().next(),
            middle.chars().next_back(),
        );
        if !middle.trim().is_empty() && (!can_open || !can_close) {
            reports.push(finding(
                open.start..close.end,
                RuleId::YomiyasuBoldNotRendered,
                "太字の記号がそのまま表示される。記号と括弧や空白の位置を確認する",
            ));
            let Some(next) = runs.next() else {
                break;
            };
            open = next;
        } else {
            open = close;
        }
    }
}

fn can_open_emphasis(before: Option<char>, after: Option<char>) -> bool {
    static PUNCTUATION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{P}\p{S}]").unwrap());
    let punctuation = |c: char| PUNCTUATION.is_match(c.encode_utf8(&mut [0; 4]));
    after.is_some_and(|after| {
        !after.is_whitespace()
            && (!punctuation(after) || before.is_none_or(|c| c.is_whitespace() || punctuation(c)))
    })
}

fn visible_sentence(src: &str, range: &Range<usize>, document: &Document) -> String {
    let mut text = String::new();
    let mut previous = range.start;
    for part in prose_in(document, range) {
        let start = part.range.start.max(range.start);
        let end = part.range.end.min(range.end);
        // 強調の記号は取り除き、コードなどの前後にある別々の語はつなげない。
        if src[previous..start]
            .bytes()
            .any(|byte| !matches!(byte, b'*' | b'_' | b'~'))
        {
            text.push(' ');
        }
        text.push_str(&src[start..end]);
        previous = end;
    }
    // 引用として括った文言には当てない。入れ子では外側の引用をまとめて空白にする。
    let mut quotes = Vec::new();
    let mut visible = String::new();
    let mut cursor = 0;
    for (i, c) in text.char_indices() {
        if let Some(close) = match c {
            '「' => Some('」'),
            '『' => Some('』'),
            _ => None,
        } {
            quotes.push((i, close));
        } else if quotes.last().is_some_and(|&(_, close)| c == close) {
            let (start, _) = quotes.pop().unwrap();
            if quotes.is_empty() {
                visible.push_str(&text[cursor..start]);
                visible.push(' ');
                cursor = i + c.len_utf8();
            }
        }
    }
    if cursor == 0 {
        return text;
    }
    visible.push_str(&text[cursor..]);
    visible
}

fn prose_in<'a>(document: &'a Document, range: &Range<usize>) -> &'a [Prose] {
    let first = document
        .texts
        .partition_point(|part| part.range.end <= range.start);
    let last = document
        .texts
        .partition_point(|part| part.range.start < range.end);
    &document.texts[first..last]
}

fn without_trailing_decoration(text: &str) -> &str {
    static DECORATION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[\s\p{Extended_Pictographic}\p{Regional_Indicator}\p{Emoji_Modifier}\u{FE0F}\u{200D}]+$").unwrap()
    });
    DECORATION
        .find(text)
        .map_or(text, |hit| &text[..hit.start()])
}

fn instruction_residue(text: &str, following: &str) -> bool {
    // yomiyasu 3.2.0の執筆予告、範囲の除外、参照先の例外に使うパターン。
    static ACTION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
        r"(?:書(?:きます|く|いてみます|いていきます)|(?:説明|紹介|解説)し(?:ます|ていきます)|",
        r"まとめ(?:ます|る|ていきます)|お伝えし(?:ます|ていきます)|述べ(?:ます|る)|",
        r"振り返(?:ります|る|っていきます)|お話し?し(?:ます|ていきます)|お届けします|話していきます)[。！？!?]?$"
    )).unwrap()
    });
    const OPENER: &str = r"^(?:この記事|本記事|本稿|このブログ|ここ|以下|今回)(?:では|は)";
    static ARTICLE_OPENER: LazyLock<Regex> = LazyLock::new(|| Regex::new(OPENER).unwrap());
    static STYLE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
        r"(?:トーン|文体|口調)で|(?:専門用語|箇条書き)を\s*(?:できるだけ|なるべく)?",
        r"(?:使わず|用いず|避け)|(?:AI|ＡＩ)(?:っぽさ|らしさ)(?:が出ないよう|を(?:抑え|なく|避け))|",
        r"飾らない言葉で|語り口で|トーンを意識|[0-9０-９]+文?字(?:程度|以内|前後)?で"
    )).unwrap()
    });
    static LIMIT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"に(?:絞って|限定して|焦点を(?:当てて|絞って))|触れず|扱わず|立ち入らず|[はを]抜きにして").unwrap()
    });
    static EXCLUSION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(&format!(
        r"{OPENER}[^。！？!?]*(?:扱いません|扱わない|触れません|触れない|省きます|省く|割愛します|省略します)[。！？!?]?$"
    )).unwrap()
    });
    static READER_SCOPE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:この|本)?(?:マニュアル|手順書|ガイド|論文|研究)(?:では|は)").unwrap()
    });
    static REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
        r"参照|ご覧|別途|付録|後述|後ほど|別の?[^。！？!?]{0,8}(?:記事|ページ|章|節|報告書?|資料|文書|ガイド)|",
        r"(?:後|次|前)の(?:章|節)|第[0-9０-９一二三四五六七八九十]+章"
    )).unwrap()
    });

    let action = ACTION.find(text);
    if action.is_some_and(|hit| hit.as_str().contains("ます") || ARTICLE_OPENER.is_match(text))
        && STYLE.is_match(text)
    {
        return true;
    }
    (EXCLUSION.is_match(text) || (action.is_some() && LIMIT.is_match(text)))
        && !READER_SCOPE.is_match(text)
        && !REFERENCE.is_match(text)
        && !REFERENCE.is_match(following)
}

fn chat_response(text: &str) -> bool {
    static OFFER: LazyLock<regress::Regex> = LazyLock::new(|| {
        regress::Regex::new(concat!(
        r"^(必要なら|ご希望があれば|よろしければ)、?(次に|続けて|この後(?!の))[^。！？!?]*",
        r"(?:ます|ましょう)(?:か|よ|ね|よね)?(?:（[^。！？!?（）]*）|\([^。！？!?()]*\))?[。！？!?]?$"
    )).unwrap()
    });
    text.starts_with("ご質問ありがとうございます") || OFFER.find(text).is_some()
}

fn finding(range: Range<usize>, rule: RuleId, message: &'static str) -> Report {
    Report {
        range,
        rule: rule.name(),
        message: Cow::Borrowed(message),
    }
}
