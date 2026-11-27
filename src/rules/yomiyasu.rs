//! yomiyasu 3.4.1由来の指摘ルール。太字、執筆時の指示、チャット応答はこのファイルで判定し、
//! ほかのルールは下のファイルで判定する。下のファイルのルールが共に使う前処理は、このファイルに置く

mod phrase;
mod placeholder;
mod shape;
mod symbol;
mod word;

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
    reports.extend(phrase::report(src, document, builtin));
    reports.extend(shape::report(src, document, builtin));
    if builtin.yomiyasu_placeholder_residue {
        reports.extend(placeholder::report(src, document));
    }
    reports.extend(symbol::report(src, document, builtin));
    reports.extend(word::report(src, document, builtin));
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

/// 執筆時の指示とチャット応答が見る文の文字列
fn visible_sentence(src: &str, range: &Range<usize>, document: &Document) -> String {
    let text = visible_text(src, range, document);
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

/// `range` の本文の文字列。強調の記号を除き、インラインコードなどを挟んだ本文の間には空白を1つ入れる
fn visible_text(src: &str, range: &Range<usize>, document: &Document) -> String {
    visible_pieces(src, range, document)
        .map(|(part, piece)| part.map_or(" ", |_| &src[piece]))
        .collect()
}

/// `visible_text` の文字列を作る切れ端の、元のソース上の範囲と、それが置かれた本文の範囲の並び。本文の間に入れる空白は、
/// 次の本文の先頭の空の範囲で、本文の範囲は `None` である
fn visible_pieces<'a>(
    src: &'a str,
    range: &Range<usize>,
    document: &'a Document,
) -> impl Iterator<Item = (Option<&'a Prose>, Range<usize>)> {
    let Range {
        start: from,
        end: to,
    } = *range;
    let mut previous = from;
    prose_in(document, range).iter().flat_map(move |part| {
        let start = part.range.start.max(from);
        let end = part.range.end.min(to);
        // 強調の記号は取り除き、コードなどの前後にある別々の語はつなげない。
        let gap = src[previous..start]
            .bytes()
            .any(|byte| !matches!(byte, b'*' | b'_' | b'~'));
        previous = end;
        gap.then_some((None, start..start))
            .into_iter()
            .chain([(Some(part), start..end)])
    })
}

/// 本文の文字列と、その各文字の元のソース上の位置。`text` から字数を変えずに作った文字列 (かぎ括弧の中などを伏せた
/// 文字列) の位置も、元のソース上の位置に戻せる
struct Visible {
    text: String,
    /// `text` の各文字の、元のソース上のバイト範囲。本文の間に入れた空白は、次の本文の先頭の空の範囲である
    sources: Vec<Range<usize>>,
    /// `text` のうち、リンクの文字列のバイト範囲。yomiyasuはリンクの文字列を角括弧ごと伏せるので、ルールによって伏せる
    links: Vec<Range<usize>>,
}

impl Visible {
    fn new(src: &str, range: &Range<usize>, document: &Document) -> Self {
        let mut visible = Visible {
            text: String::new(),
            sources: Vec::new(),
            links: Vec::new(),
        };
        for (part, piece) in visible_pieces(src, range, document) {
            let Some(part) = part else {
                visible.text.push(' ');
                visible.sources.push(piece);
                continue;
            };
            if part.context.link || part.context.link_reference {
                let start = visible.text.len();
                visible.links.push(start..start + piece.len());
            }
            visible.text.push_str(&src[piece.clone()]);
            visible.sources.extend(
                src[piece.clone()]
                    .char_indices()
                    .map(|(i, c)| piece.start + i..piece.start + i + c.len_utf8()),
            );
        }
        visible
    }

    /// `derived` (`text` から字数を変えずに作った文字列) のバイト範囲 `hit` の、元のソース上のバイト範囲
    fn source(&self, derived: &str, hit: Range<usize>) -> Range<usize> {
        let first = derived[..hit.start].chars().count();
        let last = first + derived[hit].chars().count() - 1;
        self.sources[first].start..self.sources[last].end
    }

    /// リンクの文字列を、1文字ずつ半角空白にした `text`
    fn unlinked(&self) -> String {
        blank(&self.text, &self.links)
    }
}

/// `range` を行ごとに分けた範囲。yomiyasuは記号と語の検査を行ごとに当てる
fn lines(src: &str, range: &Range<usize>) -> impl Iterator<Item = Range<usize>> {
    let mut start = range.start;
    src[range.clone()].split('\n').map(move |line| {
        let line = start..start + line.len();
        start = line.end + 1;
        line
    })
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

fn finding(range: Range<usize>, rule: RuleId, message: impl Into<Cow<'static, str>>) -> Report {
    Report {
        range,
        rule: rule.name(),
        message: message.into(),
    }
}

/// yomiyasuの文の単位。`。` `！` `？` `!` `?` の直後で区切り、前後の空白を除く。jafixの文は括弧の中で区切らないので、
/// 括弧の中で終わる文 (`「いかがでしたでしょうか。」と結ぶ。` の `「いかがでしたでしょうか。`) もここで分ける
fn clauses(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive(['。', '！', '？', '!', '?'])
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
}

/// 文の並びに、文末の飾りを除いた文を足す。飾りの無い文は足さない
fn with_undecorated<'a>(
    clauses: impl IntoIterator<Item = &'a str>,
    closers: &str,
) -> Vec<Cow<'a, str>> {
    let mut all = Vec::new();
    let mut undecorated = Vec::new();
    for clause in clauses {
        all.push(Cow::Borrowed(clause));
        undecorated.extend(without_end_decoration(clause, closers).map(Cow::Owned));
    }
    all.extend(undecorated);
    all
}

const END_MARKS: [char; 6] = ['。', '．', '！', '？', '!', '?'];
const END_BRACKETS: [(char, char); 7] = [
    ('（', '）'),
    ('(', ')'),
    ('【', '】'),
    ('〔', '〕'),
    ('〈', '〉'),
    ('《', '》'),
    ('［', '］'),
];
const END_QUOTES: [(char, char); 6] = [
    ('「', '」'),
    ('『', '』'),
    ('“', '”'),
    ('‘', '’'),
    ('"', '"'),
    ('\'', '\''),
];

/// 文の終わりの後ろの飾りを除いた文。飾りが無ければ `None` である。句点・感嘆符・疑問符は除かず、飾りの前へ詰める
/// (`見ていきましょう🚀！` は `見ていきましょう！`)。飾りは、空白、絵文字と記号、キーキャップと、`closers` の字である。
/// `closers` の閉じかっこは、対の開きかっこが文頭か、句点か述語の直後にあるときだけ除く
fn without_end_decoration(text: &str, closers: &str) -> Option<String> {
    let mut end = text.len();
    let mut marks = Vec::new();
    let mut removed = false;
    while let Some(c) = text[..end].chars().next_back() {
        let start = end - c.len_utf8();
        if END_MARKS.contains(&c) {
            marks.push(c);
        } else if c == '\u{20E3}' {
            let keycap = text[..start]
                .strip_suffix('\u{FE0F}')
                .unwrap_or(&text[..start]);
            end = keycap
                .strip_suffix(|c: char| c.is_ascii_digit() || c == '#' || c == '*')
                .unwrap_or(keycap)
                .len();
            removed = true;
            continue;
        } else if c.is_whitespace()
            || end_decoration(c)
            || (closers.contains(c)
                && END_BRACKETS
                    .iter()
                    .find(|&&(_, close)| close == c)
                    .is_none_or(|&(open, _)| bracket_opens_sentence(&text[..start], open, c)))
        {
            removed = true;
        } else {
            break;
        }
        end = start;
    }
    removed.then(|| text[..end].chars().chain(marks.into_iter().rev()).collect())
}

/// 文末の後ろに置かれる絵文字と記号と、絵文字の並びをつなぐ字 (異体字セレクタ、ZWJ、タグ) の文字クラス
const END_DECORATION: &str = concat!(
    r"[[\p{Emoji}\u{2600}-\u{27BF}\u{2B05}-\u{2B07}\u{2B1B}\u{2B1C}\u{2B50}\u{2B55}\u{231A}\u{231B}",
    r"\u{23E9}-\u{23FA}\u{2934}\u{2935}\u{3297}\u{3299}\u{1F000}-\u{1FAFF}\u{FE0E}\u{FE0F}\u{200D}",
    r"\u{E0020}-\u{E007F}]--[0-9#*]]"
);

fn end_decoration(c: char) -> bool {
    static DECORATION: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(&format!("^{END_DECORATION}$")).unwrap());
    DECORATION.is_match(c.encode_utf8(&mut [0; 4]))
}

/// `before` の直後の閉じかっこ `close` と対の開きかっこ `open` が、文頭か、句点か述語の直後にあるか。
/// 目次の副題 (`実践編（〜）`) の閉じかっこは飾りにしない。見るのは閉じかっこの前の256字までである
fn bracket_opens_sentence(before: &str, open: char, close: char) -> bool {
    static PREDICATE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?:です|ます|でした|ました|ません|ない|だ|た|る)$").unwrap());
    let floor = before.char_indices().rev().nth(255).map_or(0, |(i, _)| i);
    let mut depth = 0;
    for (i, c) in before[floor..].char_indices().rev() {
        if c == close {
            depth += 1;
        } else if c == open {
            if depth > 0 {
                depth -= 1;
                continue;
            }
            let head = before[floor..floor + i]
                .trim_end_matches(|c: char| c.is_whitespace() || end_decoration(c));
            let head = &before[..floor + head.len()];
            if head.is_empty() {
                return true;
            }
            return head.len() > floor && (head.ends_with(END_MARKS) || PREDICATE.is_match(head));
        }
    }
    false
}

/// 段落の文の末尾で飾りとして除く字。閉じかっこ・矢印・波線と、段落の初めから文の終わる行までに対の開きが無い
/// 閉じかぎと閉じ引用符である。対になった引用 (語への言及や会話) の閉じは除かない
fn end_closers(
    src: &str,
    document: &Document,
    paragraph: &Range<usize>,
    sentence: &Range<usize>,
) -> String {
    let mut closers: String = END_BRACKETS
        .iter()
        .map(|&(_, close)| close)
        .chain(['←', '↑', '→', '↓', '〜', '～'])
        .collect();
    let line_start = src[paragraph.start..sentence.end]
        .rfind('\n')
        .map_or(paragraph.start, |i| paragraph.start + i + 1);
    let line_end = src[sentence.end..paragraph.end]
        .find('\n')
        .map_or(paragraph.end, |i| sentence.end + i);
    let line = visible_text(src, &(line_start..line_end), document);
    if !line.contains(|c| END_QUOTES.iter().any(|&(_, close)| close == c)) {
        return closers;
    }
    let before = visible_text(src, &(paragraph.start..line_end), document);
    for (open, close) in END_QUOTES {
        if line.contains(close) && before.matches(open).count() == usize::from(open == close) {
            closers.push(close);
        }
    }
    closers
}

/// 語の規則が見る文字列。かぎ括弧・角括弧・引用符の中と、英数字の識別子 (`snake_case`、`camelCase`、`.` や `/` を含む語)
/// を、1文字ずつ半角空白にする
fn masked_words(text: &str) -> String {
    static QUOTES: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("“[^”\n]*”|\"[^\"\n]*\"|‘[^’\n]*’").unwrap());
    static IDENTIFIER: LazyLock<regress::Regex> = LazyLock::new(|| {
        regress::Regex::new(concat!(
            r"(?<![A-Za-z0-9])[A-Za-z0-9]+(?:[_./\\][A-Za-z0-9]+)+[_./]?",
            r"|(?<![a-z])[a-z]+[A-Z][A-Za-z0-9]*|[_./\\][A-Za-z0-9_]+"
        ))
        .unwrap()
    });
    let text = blank(text, &paired_ranges(text.char_indices(), &BRACKETS));
    let quotes: Vec<_> = QUOTES.find_iter(&text).map(|hit| hit.range()).collect();
    let text = blank(&text, &quotes);
    let identifiers: Vec<_> = IDENTIFIER.find_iter(&text).map(|hit| hit.range()).collect();
    blank(&text, &identifiers)
}

/// 引用として中を伏せる括弧の組。yomiyasuは `[` `]` も引用として伏せる
const BRACKETS: [(char, char); 3] = [('「', '」'), ('『', '』'), ('[', ']')];

/// 対になった括弧の、開きから閉じまでの範囲。括弧の種類ごとに対を取り、入れ子の内側の対の範囲も返す
fn paired_ranges(
    chars: impl IntoIterator<Item = (usize, char)>,
    pairs: &[(char, char)],
) -> Vec<Range<usize>> {
    let mut opens = vec![Vec::new(); pairs.len()];
    let mut ranges = Vec::new();
    for (i, c) in chars {
        if let Some(kind) = pairs.iter().position(|&(open, _)| open == c) {
            opens[kind].push(i);
        } else if let Some(kind) = pairs.iter().position(|&(_, close)| close == c)
            && let Some(start) = opens[kind].pop()
        {
            ranges.push(start..i + c.len_utf8());
        }
    }
    ranges
}

/// `ranges` の中の文字を、1文字ずつ半角空白にする。字数を変えないので、字数を数える正規表現の結果は変わらない
fn blank(text: &str, ranges: &[Range<usize>]) -> String {
    text.char_indices()
        .map(|(i, c)| {
            if ranges.iter().any(|range| range.contains(&i)) {
                ' '
            } else {
                c
            }
        })
        .collect()
}
