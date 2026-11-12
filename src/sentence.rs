#![expect(dead_code, unused_variables, reason = "skeleton: not implemented")]

mod english;

use std::ops::Range;

use crate::prose::{Inline, Paragraph};

/// 段落の中の1文。`range` は元のソース上のバイト範囲で、先頭の空白を含まず、末尾の区切り文字を含む
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    pub range: Range<usize>,
    /// 文に入る段落の子。本文の文字列は、区切りで分けた範囲の `Inline` になる。それ以外の子は、丸ごと1つの文に入る
    pub children: Vec<Inline>,
}

/// 段落を文に区切る。区切り文字は `.` `．` `。` `?` `!` `？` `！` で、括弧と引用符が開いている間と、
/// 英語の略語の `.` と `!` では区切らない。`.` は、次の文字が空白か段落の終わりのときだけ区切る
pub fn split(src: &str, paragraph: &Paragraph) -> Vec<Sentence> {
    todo!("skeleton: not implemented")
}
