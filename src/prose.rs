use std::ops::Range;

use markdown::mdast::{Node, ReferenceKind};
use markdown::{Constructs, ParseOptions};

/// 本文の1つの範囲
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prose {
    /// 元のソース上のバイト範囲
    pub range: Range<usize>,
    /// 見出し (ATX形式とsetext形式) の中にあるか。見出しの空白を直すとアンカーが変わるので、空白のルールは指摘にする
    pub in_heading: bool,
}

/// Markdownの本文に当たるText nodeの範囲を出現順に返す
pub fn prose_ranges(src: &str) -> Vec<Prose> {
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
fn collect(node: &Node, src: &str, ranges: &mut Vec<Prose>) {
    let range = |node: &Node| {
        let position = node.position().expect("to_mdastのnodeは位置を持つ");
        position.start.offset..position.end.offset
    };
    match node {
        Node::Text(_) => ranges.push(Prose {
            range: range(node),
            in_heading: false,
        }),
        // 省略形とcollapsed形の文字列は、定義のラベルを兼ねるので直さない
        Node::LinkReference(reference) if reference.reference_kind != ReferenceKind::Full => {}
        // `<https://...>` の自動リンクと裸のURLもLinkになり、子のTextがURLそのものになる
        Node::Link(_) if src[range(node)].starts_with('<') => {}
        // 裸のURLは空白までを取り込み、`）` や日本語の文字も含む。対応の取れない `）` から後ろはURLではない
        Node::Link(_) if !src[range(node)].starts_with('[') => {
            let link = range(node);
            ranges.extend(
                after_unmatched_closing_paren(&src[link.clone()], link.start).map(|range| Prose {
                    range,
                    in_heading: false,
                }),
            );
        }
        _ => {
            for child in node.children().into_iter().flatten() {
                collect(child, src, ranges);
            }
        }
    }
}

/// `text` のうち、`（` と対にならない最初の `）` から終わりまでの範囲を返す。`start` は `text` のソース上の開始位置
fn after_unmatched_closing_paren(text: &str, start: usize) -> Option<Range<usize>> {
    let mut depth = 0;
    for (offset, c) in text.char_indices() {
        match c {
            '（' => depth += 1,
            '）' if depth == 0 => return Some(start + offset..start + text.len()),
            '）' => depth -= 1,
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prose(src: &str) -> Vec<&str> {
        prose_ranges(src)
            .into_iter()
            .map(|p| &src[p.range])
            .collect()
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

    #[test]
    fn excludes_shortcut_and_collapsed_reference_link_text_but_keeps_full_reference_link_text() {
        assert_eq!(
            prose(
                "詳細は[設定 API]を参照。\n\n別に[設定 API][]と[文字][設定 API]も。\n\n[設定 API]: https://example.com/api\n"
            ),
            vec!["詳細は", "を参照。", "別に", "と", "文字", "も。"]
        );
    }

    #[test]
    fn treats_reference_link_text_without_definition_as_prose() {
        assert_eq!(
            prose("[設定 API] と [設定 API][]\n"),
            vec!["[設定 API] と [設定 API][]"]
        );
    }

    #[test]
    fn ends_bare_urls_at_the_first_unmatched_fullwidth_closing_paren() {
        assert_eq!(
            prose("公式サイト（https://example.com）を参照する。"),
            vec!["公式サイト（", "）を参照する。"]
        );
    }

    #[test]
    fn keeps_balanced_fullwidth_parens_inside_bare_urls() {
        assert_eq!(
            prose("公式サイト（https://ja.wikipedia.org/wiki/東京（駅））を参照する。"),
            vec!["公式サイト（", "）を参照する。"]
        );
        assert_eq!(
            prose("https://ja.wikipedia.org/wiki/東京（駅） の記事"),
            vec![" の記事"]
        );
    }

    #[test]
    fn keeps_fullwidth_parens_inside_autolinks() {
        assert_eq!(prose("<https://example.com/a）b> 外"), vec![" 外"]);
    }
}
