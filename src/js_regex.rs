//! 利用者がJavaScriptの正規表現のように書いた文字列を、`regress` の正規表現にする。
//! `prh`・`@textlint-rule/pattern`・組み込みのルールのオプションと既定のパターン・`allowlist` が、設定に書かれたパターンを読むのに使う

use std::borrow::Cow;

pub use regress::{Match, Regex};

/// 正規表現に付けるフラグ。`u` は常に付ける
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    /// `i`
    pub ignore_case: bool,
    /// `m`
    pub multiline: bool,
    /// `s`
    pub dot_all: bool,
}

/// `/source/flags` の形なら、`source` と `flags` を返す。`flags_allowed` は `flags` に書ける文字で、
/// `non_empty` なら `source` が空の形を当てない。`.*`・`.+` は最長に当たるので、最後の `/` で区切る
fn slash_form<'v>(
    value: &'v str,
    flags_allowed: &str,
    non_empty: bool,
) -> Option<(&'v str, &'v str)> {
    let (source, flags) = value.strip_prefix('/')?.rsplit_once('/')?;
    (flags.chars().all(|c| flags_allowed.contains(c)) && !(non_empty && source.is_empty()))
        .then_some((source, flags))
}

/// prhの形で読む。`^/(.*)/([gimsuy]*)$` に当たれば、`/` の間を正規表現の元の文字列にし、フラグは `m` と、
/// 書いたものにあれば `i`・`s` にする。当たらなければ、`escape` した文字列そのものと `m` だけにする。
/// 元の文字列を返すのは、prhが配列の要素を `(?:…|…)` につなぎ、`wordBoundary` で `\b` を付けてから読むためである
pub fn prh(value: &str) -> (Cow<'_, str>, Flags) {
    match slash_form(value, "gimsuy", false) {
        Some((source, flags)) => (
            Cow::Borrowed(source),
            Flags {
                ignore_case: flags.contains('i'),
                multiline: true,
                dot_all: flags.contains('s'),
            },
        ),
        None => (
            Cow::Owned(escape(value)),
            Flags {
                multiline: true,
                ..Flags::default()
            },
        ),
    }
}

/// `@textlint/regexp-string-matcher` の形で読む。`^/(.+)/([guimysd]*)$` に当たれば、書いた `i`・`m`・`s` を付けた
/// 正規表現、当たらなければ文字列そのものにする。`y` のフラグと、読めない正規表現は理由を返す
pub fn pattern(value: &str) -> Result<Regex, String> {
    let Some((source, flags)) = slash_form(value, "guimysd", true) else {
        return compile(&escape(value), Flags::default())
            .map_err(|reason| format!("`{value}` を{reason}"));
    };
    // textlintでは `matchAll` と組み合わさり、先頭から続く一致だけになるので、書いた意図どおりに働かない
    if flags.contains('y') {
        return Err(format!("`{value}` のフラグ `y` は使えない"));
    }
    compile(
        source,
        Flags {
            ignore_case: flags.contains('i'),
            multiline: flags.contains('m'),
            dot_all: flags.contains('s'),
        },
    )
    .map_err(|reason| format!("`{value}` を{reason}"))
}

/// `/source/flags` の形の文字列だけを正規表現として読む (`spelling` の `wordDefinitionRegexp`)。`i`・`m`・`s` は書いたとおりに
/// 付け、`g` は書いても書かなくても同じである。`/source/flags` の形でないもの、`y` のフラグ、読めない正規表現は理由を返す
pub fn slash_regex(value: &str) -> Result<Regex, String> {
    let _ = value;
    todo!("skeleton: not implemented")
}

/// JavaScriptの `String.prototype.replace` の置き換えの記法 (ECMAScriptのGetSubstitution: `$1`〜`$99`・`$&`・`` $` ``・
/// `$'`・`$$`・`$<name>`) を、`text` の中の一致 `m` で展開する。参照できない記法は書いたまま残す
pub fn substitute(replacement: &str, m: &Match, text: &str) -> String {
    let named = m.named_groups().next().is_some();
    let mut out = String::with_capacity(replacement.len());
    for piece in substitution_pieces(replacement, m.captures.len(), named) {
        let range = match piece {
            Piece::Text(text) => {
                out.push_str(text);
                continue;
            }
            Piece::Match => Some(m.range()),
            Piece::Before => Some(0..m.start()),
            Piece::After => Some(m.end()..text.len()),
            Piece::Group(index) => m.group(index),
            Piece::Named(name) => m.named_group(name),
        };
        // 参加しなかったグループと、無い名前のグループは、JavaScriptと同じく空の文字列になる
        if let Some(range) = range {
            out.push_str(&text[range]);
        }
    }
    out
}

/// `replacement` を、`substitute` が展開する記法で区切った固定の部分。`$$` は `$` として部分に含める。
/// 利用者の置き換えの逆の置き換えの判定に渡す
pub fn substitution_fixed_parts(replacement: &str) -> Vec<String> {
    // 当てる正規表現のグループの数が分からないので、2桁までの番号と名前の参照をすべて参照とみなし、固定の部分を短く取る
    let mut parts = vec![String::new()];
    for piece in substitution_pieces(replacement, 99, true) {
        match piece {
            Piece::Text(text) => parts.last_mut().unwrap().push_str(text),
            _ => parts.push(String::new()),
        }
    }
    parts.retain(|part| !part.is_empty());
    parts
}

/// 置き換えの記法で区切った、置き換えの文字列の1片
enum Piece<'r> {
    /// そのまま入れる文字列。`$$` は `$` で、参照できない記法は書いたままである
    Text(&'r str),
    /// `$&`
    Match,
    /// `` $` ``
    Before,
    /// `$'`
    After,
    /// `$1`〜`$99`
    Group(usize),
    /// `$<name>`
    Named(&'r str),
}

/// ECMAScriptのGetSubstitutionと同じ規則で、`replacement` を片に分ける。`groups` は捕捉グループの数、`named` は
/// 名前の付いたグループがあるかである。2桁の番号がグループの数を超えれば1桁の番号と数字として読み、`$0`・`$00` と
/// グループの数を超える番号は書いたまま残す。名前の付いたグループが無ければ `$<` を書いたまま残す
fn substitution_pieces(replacement: &str, groups: usize, named: bool) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let mut rest = replacement;
    while let Some(dollar) = rest.find('$') {
        if dollar > 0 {
            pieces.push(Piece::Text(&rest[..dollar]));
        }
        rest = &rest[dollar..];
        let after = &rest.as_bytes()[1..];
        let (piece, len) = match after.first() {
            Some(b'$') => (Piece::Text("$"), 2),
            Some(b'&') => (Piece::Match, 2),
            Some(b'`') => (Piece::Before, 2),
            Some(b'\'') => (Piece::After, 2),
            Some(b'0'..=b'9') => {
                let value = |digits: &[u8]| {
                    digits
                        .iter()
                        .fold(0, |n, digit| n * 10 + usize::from(digit - b'0'))
                };
                let mut digits = if after.get(1).is_some_and(u8::is_ascii_digit) {
                    2
                } else {
                    1
                };
                if digits == 2 && value(&after[..2]) > groups {
                    digits = 1;
                }
                if (1..=groups).contains(&value(&after[..digits])) {
                    (Piece::Group(value(&after[..digits])), 1 + digits)
                } else {
                    (Piece::Text(&rest[..1 + digits]), 1 + digits)
                }
            }
            Some(b'<') if named => match rest[2..].find('>') {
                Some(end) => (Piece::Named(&rest[2..2 + end]), 2 + end + 1),
                None => (Piece::Text("$<"), 2),
            },
            _ => (Piece::Text("$"), 1),
        };
        pieces.push(piece);
        rest = &rest[len..];
    }
    if !rest.is_empty() {
        pieces.push(Piece::Text(rest));
    }
    pieces
}

/// 文字列そのものに当たる正規表現の元の文字列にする。`\ * + . ? { } ( ) | ^ $ [ ] /` の前に `\` を足す
pub fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        if matches!(
            c,
            '\\' | '*'
                | '+'
                | '.'
                | '?'
                | '{'
                | '}'
                | '('
                | ')'
                | '|'
                | '^'
                | '$'
                | '['
                | ']'
                | '/'
        ) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

/// `text` の中の `regex` の一致を、左から重ねずにすべて取り除いた文字列を返す。長さ0の一致は取り除くものが無いので、
/// 次の文字へ進む
pub fn remove_all<'t>(regex: &Regex, text: &'t str) -> Cow<'t, str> {
    let mut matches = regex.find_iter(text).filter(|m| !m.range().is_empty());
    let Some(first) = matches.next() else {
        return Cow::Borrowed(text);
    };
    let mut removed = String::with_capacity(text.len());
    let mut kept_from = 0;
    for m in std::iter::once(first).chain(matches) {
        removed.push_str(&text[kept_from..m.start()]);
        kept_from = m.end();
    }
    removed.push_str(&text[kept_from..]);
    Cow::Owned(removed)
}

/// `source` を `u` と `flags` を付けて読む。読めなければ理由を返す
pub fn compile(source: &str, flags: Flags) -> Result<Regex, String> {
    Regex::with_flags(
        source,
        regress::Flags {
            icase: flags.ignore_case,
            multiline: flags.multiline,
            dot_all: flags.dot_all,
            unicode: true,
            ..regress::Flags::default()
        },
    )
    .map_err(|e| format!("正規表現として読めない: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_match(regex: &Regex, text: &str) -> bool {
        regex.find(text).is_some()
    }

    #[test]
    fn prh_reads_the_slash_form_and_always_adds_multiline() {
        assert_eq!(
            prh("/a.b/gis"),
            (
                Cow::Borrowed("a.b"),
                Flags {
                    ignore_case: true,
                    multiline: true,
                    dot_all: true
                }
            )
        );
    }

    #[test]
    fn prh_reads_a_string_with_non_flag_letters_after_the_slash_literally() {
        let (source, flags) = prh("/usr/bin");
        let regex = compile(&source, flags).unwrap();
        assert!(is_match(&regex, "見る /usr/bin を"));
        assert!(!is_match(&regex, "usr"));
    }

    #[test]
    fn pattern_does_not_read_an_empty_slash_form_as_a_regex() {
        let regex = pattern("//").unwrap();
        assert!(is_match(&regex, "a//b"));
        assert!(!is_match(&regex, "a/b"));
    }

    #[test]
    fn pattern_reads_a_string_with_non_flag_letters_after_the_slash_literally() {
        let regex = pattern("/usr/bin").unwrap();
        assert!(is_match(&regex, "見る /usr/bin を"));
        assert!(!is_match(&regex, "usr"));
    }

    #[test]
    fn pattern_reads_the_slash_form_with_flags_as_a_regex() {
        let regex = pattern("/a.c/gi").unwrap();
        assert!(is_match(&regex, "xAbCx"));
        assert!(!is_match(&regex, "a.d"));
        assert!(is_match(&pattern("/a/d").unwrap(), "xa"));
    }

    #[test]
    fn remove_all_removes_non_overlapping_matches_from_the_left() {
        let regex = pattern("/aa|b/").unwrap();
        assert_eq!(remove_all(&regex, "aaabxb"), "ax");
    }

    #[test]
    fn remove_all_keeps_the_text_when_nothing_matches_or_only_empty_matches() {
        assert_eq!(remove_all(&pattern("/z/").unwrap(), "abc"), "abc");
        assert!(matches!(
            remove_all(&pattern("/x*/").unwrap(), "abc"),
            Cow::Borrowed("abc")
        ));
    }

    #[test]
    fn remove_all_removes_matches_around_empty_ones() {
        assert_eq!(remove_all(&pattern("/b*/").unwrap(), "abbc"), "ac");
    }

    #[test]
    fn pattern_reads_flags_and_rejects_the_sticky_flag() {
        assert!(is_match(&pattern("/todo/i").unwrap(), "TODO"));
        assert!(!is_match(&pattern("/^b/").unwrap(), "a\nb"));
        assert!(is_match(&pattern("/^b/m").unwrap(), "a\nb"));
        assert!(pattern("/a/y").unwrap_err().contains('y'));
    }

    #[test]
    fn escaped_strings_match_themselves() {
        let source = escape(r"a\*+.?{}()|^$[]/b");
        let regex = compile(&source, Flags::default()).unwrap();
        assert!(is_match(&regex, r"xa\*+.?{}()|^$[]/by"));
        assert!(!is_match(&regex, "ab"));
    }

    fn substitute_first(source: &str, text: &str, replacement: &str) -> String {
        let regex = compile(source, Flags::default()).unwrap();
        substitute(replacement, &regex.find(text).unwrap(), text)
    }

    #[test]
    fn substitute_expands_the_javascript_replacement_patterns() {
        assert_eq!(
            substitute_first("b(c)", "abcd", "[$1|$&|$`|$'|$$]"),
            "[c|bc|a|d|$]"
        );
        assert_eq!(substitute_first("(?<x>b)", "abc", "$<x>$<y>"), "b");
        assert_eq!(substitute_first("b(x)?", "abc", "[$1]"), "[]");
    }

    #[test]
    fn substitute_reads_two_digits_only_when_the_group_exists() {
        let source = "(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)";
        assert_eq!(substitute_first(source, "abcdefghijk", "$11|$01"), "k|a");
        assert_eq!(
            substitute_first("(a)", "a", "$10|$2|$0|$00"),
            "a0|$2|$0|$00"
        );
    }

    #[test]
    fn substitute_keeps_unknown_patterns_and_trailing_dollars() {
        assert_eq!(substitute_first("a", "a", "$x$<x>$"), "$x$<x>$");
        assert_eq!(substitute_first("(?<x>a)", "a", "$<x"), "$<x");
        assert_eq!(substitute_first("é", "日本é語", "[$`$']"), "[日本語]");
    }

    #[test]
    fn substitution_fixed_parts_split_at_references() {
        assert_eq!(substitution_fixed_parts("frontend$1"), ["frontend"]);
        assert_eq!(
            substitution_fixed_parts("a$&b$<n>c$12d$$e$0"),
            ["a", "b", "c", "d$e$0"]
        );
        assert!(substitution_fixed_parts("$1$'").is_empty());
    }

    #[test]
    fn compile_uses_javascript_semantics() {
        let regex = compile(r"(?<=a)\bx\d", Flags::default()).unwrap();
        assert!(!is_match(&regex, "ax1"));
        assert!(is_match(
            &compile(r"\bid\b", Flags::default()).unwrap(),
            "ユーザーid"
        ));
        assert!(!is_match(&compile(r"\d", Flags::default()).unwrap(), "１"));
        assert!(compile(r"\-", Flags::default()).is_err());
    }
}
