use std::ops::Range;

use markdown::mdast::Node;
use markdown::{Constructs, ParseOptions};

/// Markdownの本文に当たるText nodeの、元のソース上のバイト範囲を出現順に返す
pub fn prose_ranges(src: &str) -> Vec<Range<usize>> {
    let options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::gfm()
        },
        ..ParseOptions::gfm()
    };
    let root = markdown::to_mdast(src, &options).expect("MDXを有効にしない解析は失敗しない");
    let mut ranges = Vec::new();
    collect(&root, src, &mut ranges);
    ranges
}

// コード・HTML・front matterのnodeは子を持たず、値をText nodeにしないので、たどるだけで除かれる
fn collect(node: &Node, src: &str, ranges: &mut Vec<Range<usize>>) {
    let range = |node: &Node| {
        let position = node.position().expect("to_mdastのnodeは位置を持つ");
        position.start.offset..position.end.offset
    };
    match node {
        Node::Text(_) => ranges.push(range(node)),
        // `<https://...>` の自動リンクと裸のURLもLinkになり、子のTextがURLそのものになる
        Node::Link(_) if !src[range(node)].starts_with('[') => {}
        _ => {
            for child in node.children().into_iter().flatten() {
                collect(child, src, ranges);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prose(src: &str) -> Vec<&str> {
        prose_ranges(src).into_iter().map(|r| &src[r]).collect()
    }

    #[test]
    fn collects_text_from_prose_blocks() {
        assert_eq!(
            prose("# 見出し\n\n段落\n\n- リスト\n\n> 引用\n"),
            vec!["見出し", "段落", "リスト", "引用"]
        );
    }

    #[test]
    fn collects_text_inside_inline_elements() {
        assert_eq!(
            prose("a **強調** ~~消し~~ [文字](https://example.com) <span>中</span> b"),
            vec!["a ", "強調", " ", "消し", " ", "文字", " ", "中", " b"]
        );
    }

    #[test]
    fn collects_table_cells_and_footnote_bodies() {
        assert_eq!(
            prose("| 列 |\n| - |\n| セル |\n\n脚注[^1]\n\n[^1]: 本文\n"),
            vec!["列", "セル", "脚注", "本文"]
        );
    }

    #[test]
    fn excludes_code_html_and_front_matter() {
        assert_eq!(
            prose("---\na: 語 A\n---\n\n```\n語 A\n```\n\n<div>\n語 A\n</div>\n\n`語 A` 外\n"),
            vec![" 外"]
        );
        assert_eq!(prose("+++\na = \"語 A\"\n+++\n\n外\n"), vec!["外"]);
    }

    #[test]
    fn excludes_autolinks_and_bare_urls() {
        assert_eq!(
            prose("<https://example.com> と https://example.com と www.example.com と"),
            vec![" と ", " と ", " と"]
        );
    }

    #[test]
    fn keeps_escapes_and_character_references_in_the_original_form() {
        assert_eq!(prose("a\\*b &amp; c"), vec!["a\\*b &amp; c"]);
    }

    #[test]
    fn excludes_image_alt_text_and_link_destinations() {
        assert_eq!(
            prose("![代替 A](a.png) [文字](<語 A> \"題 A\") 外"),
            vec![" ", "文字", " 外"]
        );
    }
}
