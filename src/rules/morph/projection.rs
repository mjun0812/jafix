use crate::prose::{Inline, InlineKind, Paragraph};
use crate::sentence::Sentence;

/// 文か段落の文字列と、元のソースへの位置の対応。文字列は、文か段落の子の `Inline` から作る。
/// 段落の直下の文字列は元のソースのまま、インラインコードは中身と同じ文字数の置き換えの文字 (文では `ー`)、画像はalt、
/// リンクや強調は中の文字列で、HTMLのタグ・ハード改行・脚注への参照は入らない
pub struct Projection {
    pub text: String,
    /// `text` の各文字の先頭のバイト位置と、戻す先の元のソースのバイト位置の組。`text` の順に並び、
    /// 末尾に `text` の終わりと文の終わりの組を置く
    positions: Vec<(usize, usize)>,
}

impl Projection {
    /// `src` は、`sentence` を区切った元のソースである。リンクや強調の中の文字列の位置を、文字の境界に寄せるのに使う
    pub fn new(src: &str, sentence: &Sentence) -> Self {
        let mut projection = Self {
            text: String::new(),
            positions: Vec::new(),
        };
        // コードの中身を判定に入れず、後ろの `、` と1つの未知語にまとまらないように、`ー` にする
        projection.push_inlines(src, &sentence.children, 'ー');
        projection
            .positions
            .push((projection.text.len(), sentence.range.end));
        projection
    }

    /// 段落の全体を、文と同じ作り方で1つの文字列にする。インラインコードは、中身と同じ文字数の `code` にする
    pub fn paragraph(src: &str, paragraph: &Paragraph, code: char) -> Self {
        let mut projection = Self {
            text: String::new(),
            positions: Vec::new(),
        };
        projection.push_inlines(src, &paragraph.children, code);
        projection
            .positions
            .push((projection.text.len(), paragraph.range.end));
        projection
    }

    fn push_inlines(&mut self, src: &str, children: &[Inline], code: char) {
        for child in children {
            match child.kind {
                // 段落の直下の文字列は元のソースのままなので、位置は1対1で戻る。リンクや強調の中の文字列は
                // エスケープと文字参照を解決した値なので、元のソースと長さが変わることがある
                InlineKind::Text => {
                    for (offset, c) in child.text.char_indices() {
                        let source = (child.range.start + offset).min(child.range.end);
                        self.push(c, src.floor_char_boundary(source));
                    }
                }
                InlineKind::Code => {
                    for _ in child.text.chars() {
                        self.push(code, child.range.start);
                    }
                }
                InlineKind::Image => {
                    for c in child.text.chars() {
                        self.push(c, child.range.start);
                    }
                }
                InlineKind::Link { .. }
                | InlineKind::LinkReference
                | InlineKind::Emphasis
                | InlineKind::Strong
                | InlineKind::Delete => self.push_inlines(src, &child.children, code),
                InlineKind::Break => self.push(' ', child.range.start),
                InlineKind::Html | InlineKind::FootnoteReference => {}
            }
        }
    }

    fn push(&mut self, c: char, source: usize) {
        self.positions.push((self.text.len(), source));
        self.text.push(c);
    }

    /// `text` の中のバイト位置を、元のソースのバイト位置に戻す。インラインコードと画像の中の位置は、その要素の先頭に戻る
    pub fn source(&self, index: usize) -> usize {
        let i = self.positions.partition_point(|&(start, _)| start <= index);
        self.positions[i - 1].1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prose, sentence};

    /// 1つ目の段落の1文目の文字列と、`text` の中の `needle` の位置を戻した元のソースの位置
    fn project(src: &str, needle: &str) -> (String, usize) {
        let paragraph = &prose::parse(src).paragraphs[0];
        let sentence = &sentence::split(src, paragraph, sentence::HardBreak::ListItems)[0];
        let projection = Projection::new(src, sentence);
        let index = projection.text.find(needle).unwrap();
        let source = projection.source(index);
        (projection.text, source)
    }

    #[test]
    fn keeps_direct_text_as_source_and_maps_positions_one_to_one() {
        let src = "前 a\\*b 後。";
        assert_eq!(
            project(src, "後"),
            (src.to_owned(), src.find("後").unwrap())
        );
    }

    #[test]
    fn replaces_inline_code_with_long_vowel_marks_mapped_to_the_code_start() {
        let src = "前`ab`後。";
        let (text, source) = project(src, "ー");
        assert_eq!(text, "前ーー後。");
        assert_eq!(source, src.find('`').unwrap());
        assert_eq!(project(src, "後").1, src.find("後").unwrap());
    }

    #[test]
    fn uses_image_alt_mapped_to_the_image_start_and_drops_html_and_link_markup() {
        let src = "前![代替](a.png)<b>[文字](u)後。";
        let (text, source) = project(src, "替");
        assert_eq!(text, "前代替文字後。");
        assert_eq!(source, src.find('!').unwrap());
        assert_eq!(project(src, "字").1, src.find("字").unwrap());
    }

    #[test]
    fn snaps_positions_inside_resolved_text_to_a_character_boundary_of_the_source() {
        // リンクの中の `&amp;` は `&` に解決されるので、文字列の中の「語」のバイト位置を
        // リンクの文字列の先頭に足すと、元のソースの「日」の途中になる
        let src = "[&amp;日本語](u)。";
        let (text, source) = project(src, "語");
        assert_eq!(text, "&日本語。");
        assert_eq!(source, src.find("日").unwrap());
    }
}
