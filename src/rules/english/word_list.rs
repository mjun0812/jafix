//! 語や句の一覧のすべてを選択にした正規表現 `\b(?:<一覧>)\b` (`i`・`u`) と同じ一致を、正規表現を組み立てずに求める。
//! 数百〜数千の選択の正規表現は、組み立てに数十〜数百msかかり、当てるのも遅い。hookは1回ごとに別のプロセスなので、
//! 一覧を先頭の文字で引く表にし、語の境界の位置ごとに、その文字で始まる選択を一覧の順に試す

use std::collections::HashMap;
use std::ops::Range;

/// 選択の1文字分の条件
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    /// `fold` した文字が等しい1文字
    Char(char),
    /// `?` の付いた1文字。正規表現と同じく、当てて残りを試し、駄目なら飛ばして残りを試す
    Optional(char),
    /// `[\b\s‌]*` (バックスペース・空白・ゼロ幅非接合子 U+200C の0文字以上)。長いほうから試す
    Gap,
}

/// 一覧の選択を、`fold` した先頭の文字ごとに一覧の順に並べた表
pub struct WordList {
    by_first: HashMap<char, Vec<Vec<Token>>>,
    /// 選択の前に `(?<!-)` があるか
    not_after_hyphen: bool,
}

impl WordList {
    /// `alternatives` は一覧の順の選択で、どれも `Token::Char` で始まる。`not_after_hyphen` なら、`-` の直後からは当てない
    pub fn new(alternatives: impl IntoIterator<Item = Vec<Token>>, not_after_hyphen: bool) -> Self {
        let mut by_first: HashMap<char, Vec<Vec<Token>>> = HashMap::new();
        for tokens in alternatives {
            let Some(&Token::Char(first)) = tokens.first() else {
                panic!("an alternative of a word list starts with a character");
            };
            by_first.entry(first).or_default().push(tokens);
        }
        Self {
            by_first,
            not_after_hyphen,
        }
    }

    /// `text` の中の重ならない一致のバイト範囲を、位置の順に返す。正規表現と同じく、左の位置から、語の境界の位置で一覧の順に
    /// 選択を試し、選択の後も語の境界である最初の選択を一致にして、一致の終わりから続ける
    pub fn find_iter(&self, text: &str) -> Vec<Range<usize>> {
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let byte = |index: usize| chars.get(index).map_or(text.len(), |&(byte, _)| byte);
        let mut found = Vec::new();
        let mut index = 0;
        while index < chars.len() {
            let end = (boundary(&chars, index)
                && !(self.not_after_hyphen && index > 0 && chars[index - 1].1 == '-'))
                .then(|| self.by_first.get(&fold(chars[index].1)))
                .flatten()
                .and_then(|alternatives| {
                    alternatives
                        .iter()
                        .find_map(|tokens| match_tokens(tokens, &chars, index))
                });
            match end {
                Some(end) => {
                    found.push(byte(index)..byte(end));
                    index = end;
                }
                None => index += 1,
            }
        }
        found
    }
}

/// 一覧の語を、1文字ずつ `Token::Char` にする
pub fn literal(word: &str) -> Vec<Token> {
    word.chars().map(|c| Token::Char(fold(c))).collect()
}

/// `too-wordy` と `no-cliches` の句を、正規表現の元の文字列として読む。空白は `Token::Gap`、`?` は直前の文字を
/// `Token::Optional` にする。一覧の句に、ほかの正規表現の記号は無い
pub fn phrase(phrase: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for c in phrase.chars() {
        match c {
            ' ' => tokens.push(Token::Gap),
            '?' => {
                let Some(Token::Char(optional)) = tokens.pop() else {
                    panic!("`?` in a phrase follows a character");
                };
                tokens.push(Token::Optional(optional));
            }
            _ => tokens.push(Token::Char(fold(c))),
        }
    }
    tokens
}

/// `tokens` を `chars` の `index` 文字目から当て、続く位置が語の境界になる一致の終わり (文字の位置) を返す
fn match_tokens(tokens: &[Token], chars: &[(usize, char)], index: usize) -> Option<usize> {
    let at = |index: usize| chars.get(index).map(|&(_, c)| fold(c));
    match tokens.split_first() {
        None => boundary(chars, index).then_some(index),
        Some((&Token::Char(c), rest)) => (at(index) == Some(c))
            .then(|| match_tokens(rest, chars, index + 1))
            .flatten(),
        Some((&Token::Optional(c), rest)) => (at(index) == Some(c))
            .then(|| match_tokens(rest, chars, index + 1))
            .flatten()
            .or_else(|| match_tokens(rest, chars, index)),
        Some((Token::Gap, rest)) => {
            let run = chars[index..].iter().take_while(|&&(_, c)| gap(c)).count();
            (0..=run)
                .rev()
                .find_map(|length| match_tokens(rest, chars, index + length))
        }
    }
}

/// `i` と `u` の照合の文字の同一視 (Unicodeの単純な大文字と小文字の畳み込み) を、一覧と `terminology` の項目の接頭辞に
/// 使う文字 (ASCIIと `ı`) の範囲で行う。ほかの文字に畳み込まれてASCIIの文字になるのは、`ſ` (U+017F) と `K` (U+212A)
/// だけである
pub fn fold(c: char) -> char {
    match c {
        '\u{17f}' => 's',
        '\u{212a}' => 'k',
        _ => c.to_ascii_lowercase(),
    }
}

/// `chars` の `index` 文字目の前が `\b` (`i`・`u`) か。ASCIIの英数字と `_` に、畳み込むとそれになる `ſ`・`K` を語の文字とする
fn boundary(chars: &[(usize, char)], index: usize) -> bool {
    let word = |index: Option<usize>| {
        index
            .and_then(|index| chars.get(index))
            .is_some_and(|&(_, c)| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | '\u{17f}' | '\u{212a}')
            })
    };
    word(index.checked_sub(1)) != word(Some(index))
}

/// `[\b\s‌]` の文字。`\s` はECMAScriptのWhiteSpaceとLineTerminatorである
fn gap(c: char) -> bool {
    matches!(c, '\u{2000}'..='\u{200a}')
        || matches!(
            c,
            '\u{8}'
                | '\u{200c}'
                | '\t'
                | '\n'
                | '\u{b}'
                | '\u{c}'
                | '\r'
                | ' '
                | '\u{a0}'
                | '\u{1680}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js_regex;

    /// `regex` と `list` の一致が、`inputs` のどれでも同じであることを確かめる
    fn assert_same_as_regex(regex: &str, list: &WordList, inputs: &[&str]) {
        let regex = js_regex::compile(
            regex,
            js_regex::Flags {
                ignore_case: true,
                ..js_regex::Flags::default()
            },
        )
        .unwrap();
        for input in inputs {
            let expected: Vec<Range<usize>> = regex.find_iter(input).map(|m| m.range()).collect();
            assert_eq!(list.find_iter(input), expected, "{input:?}");
        }
    }

    #[test]
    fn takes_the_first_alternative_in_list_order_that_ends_at_a_word_boundary() {
        let list = WordList::new(["ab", "abc", "ab-c"].map(literal), false);
        assert_eq!(list.find_iter("abc ab-c ab abd"), [0..3, 4..6, 9..11]);
        assert_same_as_regex(
            r"\b(?:ab|abc|ab-c)\b",
            &list,
            &["abc ab-c ab abd", "xab ab_ ab1 ab-ab", "ABC Ab aB-C"],
        );
    }

    #[test]
    fn folds_case_like_the_unicode_ignore_case_regex() {
        // `ſ` (U+017F) は `s` に、`K` (U+212A) は `k` に畳み込まれ、語の文字でもある。`ı` と `İ` は `i` に畳み込まれない
        let list = WordList::new(["sk", "\u{131}x", "ix"].map(literal), false);
        assert_same_as_regex(
            r"\b(?:sk|\u0131x|ix)\b",
            &list,
            &[
                "SK sK \u{17f}\u{212a} a\u{17f}k sk\u{17f}",
                "\u{131}x \u{130}x IX \u{131}X ix\u{131}",
                "\u{e9}sk sk\u{e9} \u{ff53}k",
            ],
        );
    }

    #[test]
    fn reads_spaces_and_question_marks_of_phrases_like_the_regex() {
        let list = WordList::new(["a lot", "will cease?", "be ok"].map(phrase), true);
        assert_same_as_regex(
            "\\b(?<!-)(a[\\b\\s\u{200c}]*lot|will[\\b\\s\u{200c}]*ceas(e)?|be[\\b\\s\u{200c}]*ok)\\b",
            &list,
            &[
                "a lot alot a  \n lot a\u{200c}lot a\u{8}lot a\u{3000}lot a\u{a0}\u{2009}lot a-lot",
                "will cease will ceas will ceasex will ceasee -a lot x-be ok",
                "A LOT be\tok beok be\u{200b}ok",
            ],
        );
    }
}
