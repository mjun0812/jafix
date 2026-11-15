//! 利用者がJavaScriptの正規表現のように書いた文字列を、`regress` の正規表現にする。
//! `prh` と `@textlint-rule/pattern` だけが使い、組み込みのルールのオプションのパターン (`builtin::pattern`) は使わない

use std::borrow::Cow;

use regress::Regex;

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
