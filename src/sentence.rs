mod english;

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{Inline, InlineKind, Paragraph};

/// 段落の中の1文。`range` は元のソース上のバイト範囲で、先頭の空白を含まず、末尾の区切り文字を含む
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    pub range: Range<usize>,
    /// 文に入る段落の子。本文の文字列は、区切りで分けた範囲の `Inline` になる。それ以外の子は、丸ごと1つの文に入る
    pub children: Vec<Inline>,
}

impl Sentence {
    fn push(&mut self, child: Inline) {
        self.range.end = child.range.end;
        self.children.push(child);
    }
}

/// ハード改行で文を閉じるか
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardBreak {
    /// 次の行が列挙の記号で始まるときと、ラベル行の末尾で文を閉じる。それ以外は、1文の折り返しとして扱う
    ListItems,
    /// 文を開かず、開いている文があればその文に入れる。括弧と引用符の対は、行をまたいで閉じてよいからである
    Continue,
}

/// リストとラベル行を区切りにして文に分けるルールで、同じ段落の結果を共有する
pub fn list_sentences<'a>(src: &str, paragraph: &'a Paragraph) -> &'a [Sentence] {
    paragraph
        .list_sentences
        .get_or_init(|| split(src, paragraph, HardBreak::ListItems))
}

/// 段落を文に区切る。区切り文字は `.` `．` `。` `?` `!` `？` `！` で、括弧と引用符が開いている間と、
/// 英語の略語の `.` と `!` では区切らない。`.` は、次の文字が空白か段落の終わりのときだけ区切る。
/// ハード改行の扱いは `hard_break` で決める
pub fn split(src: &str, paragraph: &Paragraph, hard_break: HardBreak) -> Vec<Sentence> {
    let mut splitter = Splitter {
        src,
        paragraph: paragraph.range.clone(),
        open_pairs: [false; PAIRS.len()],
        abbreviation_end: 0,
        may_have_abbreviations: src[paragraph.range.clone()]
            .bytes()
            .any(|byte| matches!(byte, b'.' | b'!')),
        current: None,
        sentences: Vec::new(),
    };
    for child in &paragraph.children {
        match (&child.kind, hard_break) {
            (InlineKind::Text, _) => splitter.text(child.range.clone()),
            (InlineKind::Break, HardBreak::ListItems)
                if starts_list_item(&src[child.range.end..paragraph.range.end])
                    || ends_label_line(
                        &src[paragraph.range.start..child.range.start],
                        &src[child.range.end..paragraph.range.end],
                    ) =>
            {
                splitter.close();
            }
            (InlineKind::Break, _) => {
                if let Some(sentence) = &mut splitter.current {
                    sentence.push(child.clone());
                }
            }
            _ => splitter.open(child.range.start).push(child.clone()),
        }
    }
    splitter.close();
    splitter.sentences
}

/// Markdownのリストにならない列挙も含む。記号の無い行は、文の途中か独立した項目か判別できないので継続する。
fn starts_list_item(rest: &str) -> bool {
    static MARKER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:[-+*–—•][ \t]+|・|[0-9０-９]+(?:[.．][^0-9０-９]|[)、])|[（(](?:[0-9０-９]+|[a-zA-Z])[）)]|[A-Za-z_][A-Za-z0-9_-]*[ \t]+(?:->|=>|→)[ \t]+\S+[ \t]*$)").unwrap()
    });
    MARKER.is_match(
        rest.lines()
            .next()
            .unwrap_or_default()
            .trim_start_matches([' ', '\t']),
    )
}

/// コードだけの値を持つラベル行と、URLだけの行への紹介文は、本文の折り返しではない。
fn ends_label_line(before: &str, after: &str) -> bool {
    static LABEL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\s*\*\*[^*]+\*\*[:：]\s*(?:`[^`]+`(?:,\s*)?)+\s*$").unwrap()
    });
    let line = before.rsplit('\n').next().unwrap().trim_end();
    LABEL.is_match(line)
        || (line.ends_with([':', '：'])
            && after.lines().next().is_some_and(|line| {
                let line = line.trim();
                (line.starts_with("https://") || line.starts_with("http://"))
                    && !line.chars().any(char::is_whitespace)
            }))
}

const SEPARATORS: [char; 7] = ['.', '．', '。', '?', '!', '？', '！'];

/// 開いている間は区切らない括弧と引用符の組
const PAIRS: [(char, char); 12] = [
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
    ('《', '》'),
];

/// sentence-splitterの文の組み立てを写したもの。括弧の開いた状態と略語の範囲は、段落の終わりまで残る
struct Splitter<'s> {
    src: &'s str,
    /// 略語の単語と `.` の次の文字は、段落の中だけを読む
    paragraph: Range<usize>,
    open_pairs: [bool; PAIRS.len()],
    /// 直近の略語の終わり。文字を左から読むので、終わるまでは `.` と `!` を区切りにしない
    abbreviation_end: usize,
    may_have_abbreviations: bool,
    current: Option<Sentence>,
    sentences: Vec<Sentence>,
}

impl<'s> Splitter<'s> {
    /// 段落の直下の文字列を1文字ずつ読む。空白と改行は文を開かず、開いている文があればその文に入る
    fn text(&mut self, range: Range<usize>) {
        let src = self.src;
        // 開いている文に入れる、この文字列の中の範囲の始まり
        let mut segment = self.current.is_some().then_some(range.start);
        let mut chars = src[range.clone()]
            .char_indices()
            .map(|(i, c)| (range.start + i, c))
            .peekable();
        while let Some((i, c)) = chars.next() {
            if is_space(c) {
                continue;
            }
            if self.is_separator(i, c) {
                let mut end = i + c.len_utf8();
                while let Some(&(j, next)) = chars.peek()
                    && self.is_separator(j, next)
                {
                    end = j + next.len_utf8();
                    chars.next();
                }
                // 文が開いていないときの区切り文字は、どの文にも入れない
                if let Some(start) = segment.take() {
                    self.push_text(start..end);
                    self.close();
                }
                continue;
            }
            if segment.is_none() {
                self.open(i);
                segment = Some(i);
            }
            if self.may_have_abbreviations {
                self.mark_abbreviation(i, c);
            }
            self.mark_pair(c);
        }
        if let Some(start) = segment {
            self.push_text(start..range.end);
        }
    }

    fn push_text(&mut self, range: Range<usize>) {
        let text = self.src[range.clone()].to_owned();
        self.current
            .as_mut()
            .expect("文字列の範囲は開いている文に入れる")
            .push(Inline {
                kind: InlineKind::Text,
                range,
                text,
                children: Vec::new(),
            });
    }

    /// 開いている文を返す。無ければ `start` から文を開く
    fn open(&mut self, start: usize) -> &mut Sentence {
        self.current.get_or_insert_with(|| Sentence {
            range: start..start,
            children: Vec::new(),
        })
    }

    fn close(&mut self) {
        self.sentences.extend(self.current.take());
    }

    fn is_separator(&self, i: usize, c: char) -> bool {
        SEPARATORS.contains(&c)
            && !self.open_pairs.contains(&true)
            && !self.in_abbreviation(i)
            // `1.5` のような数を区切らないためである
            && (c != '.'
                || self.src[i + 1..self.paragraph.end]
                    .chars()
                    .next()
                    .is_none_or(is_space))
    }

    fn in_abbreviation(&self, i: usize) -> bool {
        i < self.abbreviation_end
    }

    fn mark_pair(&mut self, c: char) {
        let Some(k) = PAIRS
            .iter()
            .position(|&(opening, closing)| c == opening || c == closing)
        else {
            return;
        };
        let (opening, closing) = PAIRS[k];
        // 同じ組の開きが開いている間の同じ開きと、開いていない組の閉じは無視する。`"` は開きと閉じを兼ねる
        self.open_pairs[k] = if self.open_pairs[k] {
            c != closing
        } else {
            c == opening
        };
    }

    /// `i` から始まる単語が英語の略語なら、その範囲を区切らない範囲にする
    fn mark_abbreviation(&mut self, i: usize, c: char) {
        if is_word_boundary(c)
            || self.in_abbreviation(i)
            || self.src[self.paragraph.start..i]
                .chars()
                .next_back()
                .is_some_and(|prev| !is_word_boundary(prev))
        {
            return;
        }
        let word = self.word_at(i);
        // 区切り文字を含まない単語は、区切らない範囲にしても結果が変わらない
        if !word.contains(['.', '!']) {
            return;
        }
        if is_abbreviation(
            word,
            || self.prev_word(i),
            || self.next_word(i + word.len()),
        ) {
            self.abbreviation_end = i + word.len();
        }
    }

    /// `i` から、空白かCJKの文字の手前までの単語
    fn word_at(&self, i: usize) -> &'s str {
        let rest = &self.src[i..self.paragraph.end];
        let end = rest.find(is_word_boundary).unwrap_or(rest.len());
        &rest[..end]
    }

    /// 単語の終わり `end` の次の1文字を飛ばした位置から始まる単語。sentence-splitterは単語の後ろに空白が1つあるとみなす
    fn next_word(&self, end: usize) -> &'s str {
        match self.src[end..self.paragraph.end].chars().next() {
            Some(c) => self.word_at(end + c.len_utf8()),
            None => "",
        }
    }

    /// `i` の前の空白を飛ばし、さらに空白でない文字を飛ばした位置から始まる単語
    fn prev_word(&self, i: usize) -> &'s str {
        let before = self.src[self.paragraph.start..i].trim_end_matches(is_space);
        let start = before
            .char_indices()
            .rev()
            .find(|&(_, c)| is_space(c))
            .map_or(0, |(j, c)| j + c.len_utf8());
        self.word_at(self.paragraph.start + start)
    }
}

/// sentence-splitterの英語の略語の規則。前後の単語は、要るときだけ求める
fn is_abbreviation<'w>(
    word: &str,
    prev_word: impl FnOnce() -> &'w str,
    next_word: impl FnOnce() -> &'w str,
) -> bool {
    let lowercase = word.to_lowercase();
    let same_ascii = |abbreviation: &&str| abbreviation.eq_ignore_ascii_case(&lowercase);
    static EXCLAMATIONS: LazyLock<Vec<String>> =
        LazyLock::new(|| english::EXCLAMATION_WORDS.map(str::to_lowercase).into());
    // `U.S.A.` のような、1文字のアルファベットと `.` が3回以上続く単語
    let initials = word.len() >= 6
        && word
            .as_bytes()
            .chunks(2)
            .all(|pair| matches!(pair, [letter, b'.'] if letter.is_ascii_alphabetic()));
    // `1.` のような番号
    let line_index = word
        .strip_suffix('.')
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()));
    if initials
        || line_index
        || EXCLAMATIONS.contains(&lowercase)
        || english::PREPOSITIVE_ABBREVIATIONS.iter().any(same_ascii)
    {
        return true;
    }
    let next_word = next_word();
    // `Albert I. Jones` の `I.` のように、大文字で始まる単語に挟まれた1文字の大文字と `.`
    let mut chars = word.chars();
    if chars.next().is_some_and(is_uppercase_letter)
        && chars.next() == Some('.')
        && starts_with_uppercase_letter(prev_word())
        && starts_with_uppercase_letter(next_word)
    {
        return true;
    }
    english::ABBREVIATIONS.iter().any(same_ascii) && !starts_with_uppercase_letter(next_word)
}

fn starts_with_uppercase_letter(word: &str) -> bool {
    word.chars().next().is_some_and(is_uppercase_letter)
}

static UPPERCASE_LETTER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{Lu}$").unwrap());

static CJK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\p{sc=Han}\p{sc=Hiragana}\p{sc=Katakana}\p{sc=Hangul}]$").unwrap()
});

fn is_uppercase_letter(c: char) -> bool {
    UPPERCASE_LETTER.is_match(c.encode_utf8(&mut [0; 4]))
}

/// 日本語などの文は単語を空白で区切らないので、CJKの文字も単語の境目にする
fn is_word_boundary(c: char) -> bool {
    is_space(c)
        // 本文に頻出するUnicode Scriptの範囲は、1文字ずつ正規表現に渡さず判定する。
        || matches!(c, 'ぁ'..='ゖ' | 'ァ'..='ヺ' | '\u{4E00}'..='\u{9FAF}')
        || (!c.is_ascii() && CJK.is_match(c.encode_utf8(&mut [0; 4])))
}

/// JavaScriptの正規表現の `\s` に当たる文字。改行を含む
pub fn is_space(c: char) -> bool {
    c == '\u{FEFF}' || (c.is_whitespace() && c != '\u{85}')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prose::parse;

    fn sentences(src: &str) -> Vec<&str> {
        let document = parse(src);
        document
            .paragraphs
            .iter()
            .flat_map(|paragraph| split(src, paragraph, HardBreak::ListItems))
            .map(|sentence| &src[sentence.range])
            .collect()
    }

    #[test]
    fn splits_at_separators_and_keeps_a_run_of_them_in_one_sentence() {
        assert_eq!(sentences("文。。！？次"), vec!["文。。！？", "次"]);
        assert_eq!(
            sentences("一文目。二文目！三文目？\u{3000}四文目.五 1.5です. 次。"),
            vec![
                "一文目。",
                "二文目！",
                "三文目？",
                "四文目.五 1.5です.",
                "次。"
            ]
        );
    }

    #[test]
    fn leaves_a_separator_outside_of_any_sentence_at_the_start() {
        assert_eq!(sentences("。先頭の句点"), vec!["先頭の句点"]);
    }

    #[test]
    fn does_not_split_while_a_pair_is_open() {
        assert_eq!(
            sentences("「括弧の中。」の後。(半角。)後。"),
            vec!["「括弧の中。」の後。", "(半角。)後。"]
        );
        assert_eq!(sentences("「閉じない。まだ。"), vec!["「閉じない。まだ。"]);
    }

    #[test]
    fn does_not_split_at_english_abbreviations() {
        assert_eq!(
            sentences("Mr. Smith is here. Yahoo! Japan. e.g. this. 1. あ"),
            vec!["Mr. Smith is here.", "Yahoo! Japan.", "e.g. this.", "1. あ"]
        );
        assert_eq!(sentences("U.S.A. is big."), vec!["U.S.A. is big."]);
    }

    #[test]
    fn splits_after_an_abbreviation_followed_by_a_capitalized_word() {
        assert_eq!(
            sentences("It is in Calif. The next."),
            vec!["It is in Calif.", "The next."]
        );
        assert_eq!(
            sentences("you and I. Did you see Albert I. Jones"),
            vec!["you and I.", "Did you see Albert I. Jones"]
        );
    }

    #[test]
    fn does_not_split_inside_inline_elements() {
        assert_eq!(sentences("前`a。b`後。次。"), vec!["前`a。b`後。", "次。"]);
        assert_eq!(sentences("前**a。b**後。"), vec!["前**a。b**後。"]);
    }

    #[test]
    fn does_not_split_at_newlines() {
        assert_eq!(sentences("一行目\n二行目。"), vec!["一行目\n二行目。"]);
    }

    #[test]
    fn keeps_wrapped_sentences_open_at_hard_breaks() {
        assert_eq!(sentences("一行目  \n二行目。"), vec!["一行目  \n二行目。"]);
        assert_eq!(sentences("一行目\\\n二行目。"), vec!["一行目\\\n二行目。"]);
        assert_eq!(
            sentences("「一行目  \n二行目」。"),
            vec!["「一行目  \n二行目」。"]
        );
    }

    #[test]
    fn keeps_a_sentence_open_across_hard_breaks_when_asked() {
        let src = "「一行目  \n二行目」。三行目\\\n四行目。";
        let document = parse(src);
        let sentences: Vec<&str> = split(src, &document.paragraphs[0], HardBreak::Continue)
            .into_iter()
            .map(|sentence| &src[sentence.range])
            .collect();
        assert_eq!(
            sentences,
            vec!["「一行目  \n二行目」。", "三行目\\\n四行目。"]
        );
    }

    #[test]
    fn keeps_text_children_as_the_split_ranges_and_other_children_whole() {
        let src = "前`コード`後。次。";
        let document = parse(src);
        let sentences = split(src, &document.paragraphs[0], HardBreak::ListItems);
        let children: Vec<Vec<(&str, &str)>> = sentences
            .iter()
            .map(|sentence| {
                sentence
                    .children
                    .iter()
                    .map(|child| (&src[child.range.clone()], child.text.as_str()))
                    .collect()
            })
            .collect();
        assert_eq!(
            children,
            vec![
                vec![("前", "前"), ("`コード`", "コード"), ("後。", "後。")],
                vec![("次。", "次。")]
            ]
        );
    }
}
