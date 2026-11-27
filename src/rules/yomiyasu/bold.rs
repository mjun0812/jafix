//! 太字にならない `**` の組。yomiyasu 3.4.1の `bold_not_rendered` の検査と、その直し方 (`_bold_fix_with_context`)

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{self, Document, Inline, InlineKind, InlineNodeKind, Prose};
use crate::rules::{Finding, FixRuleFinding, Report, RuleId};

use super::{finding, prose_in};

/// 直し方を確かめられた組の修正の箇所。見出しの中の組は `heading` にするので、直さずに指摘になる
pub(super) fn find(src: &str, document: &Document) -> Vec<FixRuleFinding> {
    broken_pairs(src, document)
        .into_iter()
        .filter_map(|pair| {
            let repair = pair.repair?;
            Some(FixRuleFinding {
                finding: Finding {
                    position: pair.range.start,
                    range: pair.range,
                    replacement: Cow::Owned(repair.replacement),
                    rule: RuleId::YomiyasuBoldNotRendered,
                    message: Cow::Borrowed(repair.message),
                },
                heading: pair.heading,
            })
        })
        .collect()
}

/// 直し方を確かめられない組の指摘。直せる組は `find` の箇所になるので、ここでは指摘しない
pub(super) fn report(src: &str, document: &Document) -> Vec<Report> {
    broken_pairs(src, document)
        .into_iter()
        .filter(|pair| pair.repair.is_none())
        .map(|pair| {
            finding(
                pair.range,
                RuleId::YomiyasuBoldNotRendered,
                "太字の記号がそのまま表示される。記号と括弧や空白の位置を確認する",
            )
        })
        .collect()
}

/// 太字にならない `**` の組
struct BrokenPair {
    /// 開きの `**` の先頭から、閉じの `**` の末尾までの元のソース上のバイト範囲
    range: Range<usize>,
    heading: bool,
    repair: Option<Repair>,
}

struct Repair {
    /// `range` を置き換える文字列。外側に入れる空白を含む
    replacement: String,
    message: &'static str,
}

fn broken_pairs(src: &str, document: &Document) -> Vec<BrokenPair> {
    if !src.contains("**") {
        return Vec::new();
    }
    let blocks = document
        .paragraphs
        .iter()
        .map(|paragraph| (&paragraph.range, false))
        .chain(
            document
                .headings
                .iter()
                .map(|heading| (&heading.range, true)),
        )
        .chain(document.table_cells.iter().map(|cell| (cell, false)));
    let mut pairs = Vec::new();
    for (block, heading) in blocks {
        let mut parsed = None;
        for (open, close) in unrendered(src, block, prose_in(document, block)) {
            let repair = repair(src, block, &open, &close, &mut parsed);
            pairs.push(BrokenPair {
                range: open.start..close.end,
                heading,
                repair,
            });
        }
    }
    pairs
}

/// `block` の中で、太字にならない組の開きと閉じの `*` の連なりの範囲
fn unrendered(
    src: &str,
    block: &Range<usize>,
    prose: &[Prose],
) -> Vec<(Range<usize>, Range<usize>)> {
    static STARS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*{2,}").unwrap());
    let mut runs = prose.iter().flat_map(|part| {
        STARS
            .find_iter(&src[part.range.clone()])
            .filter_map(|star| {
                let start = part.range.start + star.start();
                let end = part.range.start + star.end();
                let bytes = src.as_bytes();
                // 本文のTextノードに残った記号だけを見る。既存の強調やエスケープに属する記号の残りは組にしない。
                (!escaped(bytes, start)
                    && !bytes[..start].ends_with(b"*")
                    && !bytes[end..].starts_with(b"*"))
                .then_some(start..end)
            })
    });
    let mut pairs = Vec::new();
    let Some(mut open) = runs.next() else {
        return pairs;
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
            pairs.push((open, close));
            let Some(next) = runs.next() else {
                break;
            };
            open = next;
        } else {
            open = close;
        }
    }
    pairs
}

/// `bytes` の `position` の文字が、奇数個続く `\` の直後にあってエスケープされているか
fn escaped(bytes: &[u8], position: usize) -> bool {
    bytes[..position]
        .iter()
        .rev()
        .take_while(|&&b| b == b'\\')
        .count()
        % 2
        == 1
}

fn can_open_emphasis(before: Option<char>, after: Option<char>) -> bool {
    static PUNCTUATION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{P}\p{S}]").unwrap());
    let punctuation = |c: char| PUNCTUATION.is_match(c.encode_utf8(&mut [0; 4]));
    after.is_some_and(|after| {
        !after.is_whitespace()
            && (!punctuation(after) || before.is_none_or(|c| c.is_whitespace() || punctuation(c)))
    })
}

/// 組の直し方。かっこの内側だけを太字にする、句読点を太字の外に出す、空白を直す、の順に試し、確かめられた最初の
/// 直し方を返す。`parsed` は、`block` を解析した結果を、同じブロックの組で使い回すための入れ物である
fn repair(
    src: &str,
    block: &Range<usize>,
    open: &Range<usize>,
    close: &Range<usize>,
    parsed: &mut Option<Standalone>,
) -> Option<Repair> {
    let inner = &src[open.end..close.start];
    // `*` が3つ以上続く記号と内側の `*` は、どの記号を組にするつもりかが決まらない。改行を含む組を直すと、hookで
    // 組の始まりの行だけが対象行のときに、対象行でない行まで書き換わる
    if open.len() != 2 || close.len() != 2 || inner.contains(['*', '\n']) {
        return None;
    }
    let base = parsed.get_or_insert_with(|| Standalone::new(&src[block.clone()]));
    // `*` の連なりが3つ以上あるブロックでは、空白だけが原因の組は、どの記号を組にするつもりかが決まらない
    if base.star_runs > 2 && (inner.starts_with(is_space) || inner.ends_with(is_space)) {
        return None;
    }
    let base = &*base;
    candidates(src, block, open, close)
        .into_iter()
        .find(|candidate| renders(src, block, open, close, candidate, base))
        .map(|candidate| Repair {
            replacement: candidate.replacement,
            message: candidate.message,
        })
}

/// 直し方の案。案は、内側の一部 (`body`) をそのまま残し、その前後の記号と空白だけを変える
struct Candidate {
    replacement: String,
    /// 置き換えの中にそのまま残る、元のソース上のバイト範囲
    body: Range<usize>,
    /// `replacement` の中の `body` の始まり
    body_at: usize,
    message: &'static str,
}

const BRACKETS: [(char, char); 13] = [
    ('「', '」'),
    ('『', '』'),
    ('（', '）'),
    ('(', ')'),
    ('【', '】'),
    ('〔', '〕'),
    ('［', '］'),
    ('[', ']'),
    ('〈', '〉'),
    ('《', '》'),
    ('“', '”'),
    ('‘', '’'),
    ('＜', '＞'),
];

fn candidates(
    src: &str,
    block: &Range<usize>,
    open: &Range<usize>,
    close: &Range<usize>,
) -> Vec<Candidate> {
    let inner = &src[open.end..close.start];
    let mut candidates = Vec::new();
    let (Some(first), Some(last)) = (inner.chars().next(), inner.chars().next_back()) else {
        return candidates;
    };
    let length = inner.chars().count();
    if length >= 3
        && let Some(&(_, closing)) = BRACKETS.iter().find(|&&(opening, _)| opening == first)
        && closing_of(inner, first, closing) == Some(inner.len() - last.len_utf8())
    {
        let body = open.end + first.len_utf8()..close.start - last.len_utf8();
        candidates.push(Candidate {
            replacement: format!("{first}**{}**{last}", &src[body.clone()]),
            body,
            body_at: first.len_utf8() + 2,
            message: "太字の記号を括弧の内側に移す",
        });
    }
    if length >= 2 && "。、．，！？!?".contains(last) {
        let body = open.end..close.start - last.len_utf8();
        candidates.push(Candidate {
            replacement: format!("**{}**{last}", &src[body.clone()]),
            body,
            body_at: 2,
            message: "句読点を太字の外に出す",
        });
    }
    let start = open.end + inner.len() - inner.trim_start_matches(is_space).len();
    let body = start..start + inner.trim_matches(is_space).len();
    let text = &src[body.clone()];
    let left_opens = |punctuation: Punctuation| {
        opens(
            src[block.start..open.start].chars().next_back(),
            text.chars().next(),
            punctuation,
        )
    };
    let right_closes = |punctuation: Punctuation| {
        closes(
            text.chars().next_back(),
            src[close.end..block.end].chars().next(),
            punctuation,
        )
    };
    let left = if PUNCTUATIONS.into_iter().all(left_opens) {
        ""
    } else {
        " "
    };
    let right = if PUNCTUATIONS.into_iter().all(right_closes) {
        ""
    } else {
        " "
    };
    candidates.push(Candidate {
        replacement: format!("{left}**{text}**{right}"),
        body,
        body_at: left.len() + 2,
        message: if left.is_empty() && right.is_empty() {
            "太字の内側の空白を削除する"
        } else {
            "太字の外側に空白を入れる"
        },
    });
    candidates
}

/// `text` の先頭のかっこ `opening` と対になる閉じかっこ `closing` の、`text` の中の位置。入れ子の対を数える
fn closing_of(text: &str, opening: char, closing: char) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        if c == opening {
            depth += 1;
        } else if c == closing {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// 案を当てた後のブロックで、2つの `**` がGFMとCommonMarkの両方の規則で太字の始まりと終わりになれ、ブロックを解析し
/// 直すと、対象の組だけが太字に増え、ほかのインラインのnodeとリンクの宛先が変わらないか
fn renders(
    src: &str,
    block: &Range<usize>,
    open: &Range<usize>,
    close: &Range<usize>,
    candidate: &Candidate,
    base: &Standalone,
) -> bool {
    let replacement = &candidate.replacement;
    let text = format!(
        "{}{replacement}{}",
        &src[block.start..open.start],
        &src[close.end..block.end]
    );
    let start = open.start - block.start;
    let first = start + replacement.find("**").unwrap();
    let second = start + replacement.rfind("**").unwrap();
    let before = |i: usize| text[..i].chars().next_back();
    let after = |i: usize| text[i..].chars().next();
    if !PUNCTUATIONS.into_iter().all(|punctuation| {
        opens(before(first), after(first + 2), punctuation)
            && closes(before(second), after(second + 2), punctuation)
    }) {
        return false;
    }
    let end = close.end - block.start;
    let body = candidate.body.start - block.start..candidate.body.end - block.start;
    // 元のブロックの位置を、案を当てた後の位置にする。動かした記号と空白の位置は対応しないので `None` である
    let moved = |position: usize| {
        if position <= start {
            Some(position)
        } else if position >= end {
            Some(position - end + start + replacement.len())
        } else if (body.start..=body.end).contains(&position) {
            Some(position - body.start + start + candidate.body_at)
        } else {
            None
        }
    };
    let Some(mut expected) = base
        .marks
        .iter()
        .map(|(mark, range)| Some((*mark, moved(range.start)?..moved(range.end)?)))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    expected.push((Mark::Strong, first..second + 2));
    expected.sort_by_key(|(mark, range)| (range.start, std::cmp::Reverse(range.end), *mark));
    let parsed = Standalone::new(&text);
    parsed.shape == base.shape && parsed.marks == expected
}

/// 区切りの記号の前後の文字を句読点や記号とみなす規則
type Punctuation = fn(char) -> bool;

/// GFM (ASCIIの記号とUnicodeのP) と、CommonMark (UnicodeのPとS) の規則
const PUNCTUATIONS: [Punctuation; 2] = [
    |c| c.is_ascii_punctuation() || PUNCTUATION_GFM.is_match(c.encode_utf8(&mut [0; 4])),
    |c| PUNCTUATION_COMMONMARK.is_match(c.encode_utf8(&mut [0; 4])),
];

static PUNCTUATION_GFM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{P}$").unwrap());
static PUNCTUATION_COMMONMARK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{P}\p{S}]$").unwrap());

/// `before` と `after` の間の `**` が太字を始められるか。ブロックの端 (`None`) は空白とみなす
fn opens(before: Option<char>, after: Option<char>, punctuation: Punctuation) -> bool {
    let space = |c: Option<char>| c.is_none_or(is_space);
    let symbol = |c: Option<char>| c.is_some_and(punctuation);
    !space(after) && (!symbol(after) || space(before) || symbol(before))
}

/// `before` と `after` の間の `**` が太字を終えられるか
fn closes(before: Option<char>, after: Option<char>, punctuation: Punctuation) -> bool {
    let space = |c: Option<char>| c.is_none_or(is_space);
    let symbol = |c: Option<char>| c.is_some_and(punctuation);
    !space(before) && (!symbol(before) || space(after) || symbol(after))
}

/// yomiyasuが空白とみなす文字。Unicodeの空白 (Zs) と、タブ・改行・改ページ・復帰
fn is_space(c: char) -> bool {
    static SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[\t\n\x0C\r\p{Zs}]$").unwrap());
    SPACE.is_match(c.encode_utf8(&mut [0; 4]))
}

/// ブロックの元のソースだけを解析した結果。位置は、ブロックの先頭からのバイト数である
struct Standalone {
    /// インラインのnodeとリンクの宛先の種類と範囲。位置の順 (同じ位置なら長いものが先) に並ぶ
    marks: Vec<(Mark, Range<usize>)>,
    /// 段落・見出し・表のセル・リスト項目の数
    shape: [usize; 4],
    /// 本文に見える `*` の連なりの数。コード・画像・HTML・リンクの宛先の中と、エスケープした `*` は数えない
    star_runs: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Mark {
    Code,
    Link,
    Strong,
    Emphasis,
    Image,
    Destination,
}

impl Standalone {
    fn new(block: &str) -> Self {
        // 先頭に空白の扱いになる文字を置き、表のセルや見出しの中身の `#` や `-` を、見出しやリストの記号として
        // 読まないようにする。続きの行の先頭にある引用の記号と字下げは、ブロックの外の記号なので空白にする
        const LEAD: char = '\u{A0}';
        let mut source = String::from(LEAD);
        for (i, line) in block.split('\n').enumerate() {
            let content = if i == 0 {
                line
            } else {
                source.push('\n');
                line.trim_start_matches([' ', '\t', '>'])
            };
            source.extend(std::iter::repeat_n(' ', line.len() - content.len()));
            source.push_str(content);
        }
        let document = prose::parse(&source);
        let lead = LEAD.len_utf8();
        let mut marks: Vec<(Mark, Range<usize>)> = document
            .inline_nodes
            .iter()
            .map(|node| {
                let mark = match node.kind {
                    InlineNodeKind::Code => Mark::Code,
                    InlineNodeKind::Link => Mark::Link,
                    InlineNodeKind::Strong => Mark::Strong,
                    InlineNodeKind::Emphasis => Mark::Emphasis,
                    InlineNodeKind::Image { .. } => Mark::Image,
                };
                (mark, node.range.clone())
            })
            .chain(
                document
                    .links
                    .iter()
                    .map(|link| (Mark::Destination, link.range.clone())),
            )
            .collect();
        let mut hidden: Vec<Range<usize>> = marks
            .iter()
            .filter(|(mark, _)| matches!(mark, Mark::Code | Mark::Image | Mark::Destination))
            .map(|(_, range)| range.clone())
            .collect();
        for paragraph in document.paragraphs.iter().chain(&document.headings) {
            collect_html(&paragraph.children, &mut hidden);
        }
        let bytes = source.as_bytes();
        let visible_star = |i: usize| {
            bytes[i] == b'*' && !escaped(bytes, i) && !hidden.iter().any(|range| range.contains(&i))
        };
        let star_runs = (0..bytes.len())
            .filter(|&i| visible_star(i) && (i == 0 || !visible_star(i - 1)))
            .count();
        for (_, range) in &mut marks {
            *range = range.start - lead..range.end - lead;
        }
        marks.sort_by_key(|(mark, range)| (range.start, std::cmp::Reverse(range.end), *mark));
        Standalone {
            marks,
            shape: [
                document.paragraphs.len(),
                document.headings.len(),
                document.table_cells.len(),
                document.list_items.len(),
            ],
            star_runs,
        }
    }
}

fn collect_html(inlines: &[Inline], hidden: &mut Vec<Range<usize>>) {
    for inline in inlines {
        if matches!(inline.kind, InlineKind::Html) {
            hidden.push(inline.range.clone());
        }
        collect_html(&inline.children, hidden);
    }
}
