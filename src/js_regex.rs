//! 利用者がJavaScriptの正規表現のように書いた文字列を、`regress` の正規表現にする。
//! `prh` と `@textlint-rule/pattern` だけが使い、組み込みのルールのオプションのパターン (`builtin::pattern`) は使わない

// skeleton: not implemented (実装したら、この行と次の行を消す)
#![allow(dead_code)]

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

/// prhの形で読む。`^/(.*)/([gimsuy]*)$` に当たれば、`/` の間を正規表現の元の文字列にし、フラグは `m` と、
/// 書いたものにあれば `i`・`s` にする。当たらなければ、`escape` した文字列そのものと `m` だけにする。
/// 元の文字列を返すのは、prhが配列の要素を `(?:…|…)` につなぎ、`wordBoundary` で `\b` を付けてから読むためである
pub fn prh(value: &str) -> (Cow<'_, str>, Flags) {
    let _ = value;
    todo!("skeleton: not implemented")
}

/// `@textlint/regexp-string-matcher` の形で読む。`^/(.+)/([guimysd]*)$` に当たれば、書いた `i`・`m`・`s` を付けた
/// 正規表現、当たらなければ文字列そのものにする。`y` のフラグと、読めない正規表現は理由を返す
pub fn pattern(value: &str) -> Result<Regex, String> {
    let _ = value;
    todo!("skeleton: not implemented")
}

/// 文字列そのものに当たる正規表現の元の文字列にする。`\ * + . ? { } ( ) | ^ $ [ ] /` の前に `\` を足す
pub fn escape(value: &str) -> String {
    let _ = value;
    todo!("skeleton: not implemented")
}

/// `source` を `u` と `flags` を付けて読む。読めなければ理由を返す
pub fn compile(source: &str, flags: Flags) -> Result<Regex, String> {
    let _ = (source, flags);
    todo!("skeleton: not implemented")
}
