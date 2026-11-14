use std::ops::Range;

use markdown::mdast::{Node, ReferenceKind};
use markdown::{Constructs, ParseOptions};

/// Markdownから取り出した、ルールを当てる単位
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    /// 本文の範囲。出現順に並ぶ
    pub texts: Vec<Prose>,
    /// 段落。文に区切るルールと、段落の末尾を見るルールが使う。見出しと表のセルは段落ではない
    pub paragraphs: Vec<Paragraph>,
    /// 見出し。段落と同じ形で持つ。`list_item` と `footnote` は使わない。`paragraphs` には入らない
    pub headings: Vec<Paragraph>,
}

/// 本文の1つの範囲
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prose {
    /// 元のソース上のバイト範囲
    pub range: Range<usize>,
    pub context: Context,
}

/// 本文の範囲が置かれた文脈。textlintのルールが祖先のnodeの型で対象を絞るのに合わせて持つ
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Context {
    /// 見出し (ATX形式とsetext形式) の中。見出しの文字を変えるとアンカーが変わるので、修正ルールの多くは指摘にする
    pub heading: bool,
    /// 引用の中
    pub blockquote: bool,
    /// 強調 (`*x*`) の中。太字 (`**x**`) は含めない
    pub emphasis: bool,
    /// リンク (`[x](url)`) の中。参照形式のリンクは含めない
    pub link: bool,
    /// 段落の直下にあり、引用の中でない
    pub plain: bool,
}

/// 段落。`range` は元のソース上のバイト範囲
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paragraph {
    pub range: Range<usize>,
    /// 引用の中
    pub blockquote: bool,
    /// リスト項目の中
    pub list_item: bool,
    /// 脚注の定義の中
    pub footnote: bool,
    pub children: Vec<Inline>,
}

/// 段落の中のインラインの要素。`range` は元のソース上のバイト範囲
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inline {
    pub kind: InlineKind,
    pub range: Range<usize>,
    /// 段落の文字列のうち、この要素の分。文の長さと `,` を数えるのに使う。
    /// 段落の直下の文字列は元のソースのまま、リンクや強調の中の文字列はエスケープと文字参照を解決したもの、
    /// インラインコードは中身、画像はaltで、HTMLのタグ・ハード改行・脚注への参照は空である
    pub text: String,
    /// リンク・強調・太字・取り消し線・参照形式のリンクの中身
    pub children: Vec<Inline>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlineKind {
    Text,
    Code,
    /// 画像 (参照形式を含む)
    Image,
    /// リンク (自動リンクと裸のURLを含む)。裸のURLの範囲は、Markdownの解析結果のままにする
    Link {
        url: String,
    },
    /// 参照形式のリンク
    LinkReference,
    Emphasis,
    Strong,
    Delete,
    /// インラインHTMLのタグ
    Html,
    /// ハード改行
    Break,
    /// 脚注への参照
    FootnoteReference,
}

/// Markdownを解析し、本文の範囲と段落を取り出す
pub fn parse(src: &str) -> Document {
    let options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::gfm()
        },
        ..ParseOptions::gfm()
    };
    let root = markdown::to_mdast(src, &options).expect("MDXを有効にしない解析は失敗しない");
    let mut texts = Vec::new();
    collect(&root, src, Context::default(), &mut texts);
    let mut blocks = Blocks::default();
    collect_paragraphs(&root, src, false, false, false, &mut blocks);
    Document {
        texts,
        paragraphs: blocks.paragraphs,
        headings: blocks.headings,
    }
}

fn node_range(node: &Node) -> Range<usize> {
    let position = node.position().expect("to_mdastのnodeは位置を持つ");
    position.start.offset..position.end.offset
}

// `context` は `node` が置かれた文脈である。
// コード・HTML・front matterのnodeは子を持たず、値をText nodeにしないので、たどるだけで除かれる
fn collect(node: &Node, src: &str, context: Context, ranges: &mut Vec<Prose>) {
    match node {
        Node::Text(_) => ranges.push(Prose {
            range: node_range(node),
            context,
        }),
        // 省略形とcollapsed形の文字列は、定義のラベルを兼ねるので直さない
        Node::LinkReference(reference) if reference.reference_kind != ReferenceKind::Full => {}
        // `<https://...>` の自動リンクと裸のURLもLinkになり、子のTextがURLそのものになる
        Node::Link(_) if src[node_range(node)].starts_with('<') => {}
        // 裸のURLは空白までを取り込み、`）` や日本語の文字も含む。対応の取れない `）` から後ろはURLではない
        Node::Link(_) if !src[node_range(node)].starts_with('[') => {
            let link = node_range(node);
            // URLの後ろの本文は、Linkの外でLinkと並ぶ文字列として扱う
            ranges.extend(
                after_unmatched_closing_paren(&src[link.clone()], link.start)
                    .map(|range| Prose { range, context }),
            );
        }
        _ => {
            let blockquote = context.blockquote || matches!(node, Node::Blockquote(_));
            let children_context = Context {
                heading: context.heading || matches!(node, Node::Heading(_)),
                blockquote,
                emphasis: context.emphasis || matches!(node, Node::Emphasis(_)),
                link: context.link || matches!(node, Node::Link(_)),
                plain: matches!(node, Node::Paragraph(_)) && !blockquote,
            };
            for child in node.children().into_iter().flatten() {
                collect(child, src, children_context, ranges);
            }
        }
    }
}

#[derive(Default)]
struct Blocks {
    paragraphs: Vec<Paragraph>,
    headings: Vec<Paragraph>,
}

/// `node` の中の段落と見出しを集める。3つのflagは、`node` の祖先に引用・リスト項目・脚注の定義があるかである
fn collect_paragraphs(
    node: &Node,
    src: &str,
    blockquote: bool,
    list_item: bool,
    footnote: bool,
    blocks: &mut Blocks,
) {
    let target = match node {
        Node::Paragraph(_) => &mut blocks.paragraphs,
        Node::Heading(_) => &mut blocks.headings,
        _ => {
            for child in node.children().into_iter().flatten() {
                collect_paragraphs(
                    child,
                    src,
                    blockquote || matches!(node, Node::Blockquote(_)),
                    list_item || matches!(node, Node::ListItem(_)),
                    footnote || matches!(node, Node::FootnoteDefinition(_)),
                    blocks,
                );
            }
            return;
        }
    };
    target.push(Paragraph {
        range: node_range(node),
        blockquote,
        list_item,
        footnote,
        children: node
            .children()
            .into_iter()
            .flatten()
            .filter_map(|child| inline(child, src, true))
            .collect(),
    });
}

/// 段落の中のnodeを `Inline` にする。`direct` は、`node` が段落の直下にあるか。
/// textlintと同じく、段落の直下の文字列は元のソースのまま、リンクや強調の中の文字列はnodeの値で数える
fn inline(node: &Node, src: &str, direct: bool) -> Option<Inline> {
    let range = node_range(node);
    let leaf = |kind, text: &str| Inline {
        kind,
        range: range.clone(),
        text: text.to_owned(),
        children: Vec::new(),
    };
    let kind = match node {
        Node::Text(_) if direct => return Some(leaf(InlineKind::Text, &src[range.clone()])),
        Node::Text(text) => return Some(leaf(InlineKind::Text, &text.value)),
        Node::InlineCode(code) => return Some(leaf(InlineKind::Code, &code.value)),
        Node::Image(image) => return Some(leaf(InlineKind::Image, &image.alt)),
        Node::ImageReference(image) => return Some(leaf(InlineKind::Image, &image.alt)),
        Node::Html(_) => return Some(leaf(InlineKind::Html, "")),
        Node::Break(_) => return Some(leaf(InlineKind::Break, "")),
        Node::FootnoteReference(_) => return Some(leaf(InlineKind::FootnoteReference, "")),
        Node::Link(link) => InlineKind::Link {
            url: link.url.clone(),
        },
        Node::LinkReference(_) => InlineKind::LinkReference,
        Node::Emphasis(_) => InlineKind::Emphasis,
        Node::Strong(_) => InlineKind::Strong,
        Node::Delete(_) => InlineKind::Delete,
        _ => return None,
    };
    let children: Vec<Inline> = node
        .children()
        .into_iter()
        .flatten()
        .filter_map(|child| inline(child, src, false))
        .collect();
    Some(Inline {
        kind,
        text: children.iter().map(|child| child.text.as_str()).collect(),
        range,
        children,
    })
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
        parse(src)
            .texts
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

    fn in_heading(src: &str) -> Vec<(&str, bool)> {
        parse(src)
            .texts
            .into_iter()
            .map(|p| (&src[p.range], p.context.heading))
            .collect()
    }

    #[test]
    fn marks_text_inside_atx_and_setext_headings() {
        assert_eq!(
            in_heading("# 見出し **強調**\n\n段落\n\n設定 A\n===\n"),
            vec![
                ("見出し ", true),
                ("強調", true),
                ("段落", false),
                ("設定 A", true)
            ]
        );
    }

    fn contexts(src: &str) -> Vec<(&str, Context)> {
        parse(src)
            .texts
            .into_iter()
            .map(|p| (&src[p.range], p.context))
            .collect()
    }

    #[test]
    fn marks_the_context_of_each_text_by_its_ancestors() {
        let plain = Context {
            plain: true,
            ..Context::default()
        };
        assert_eq!(
            contexts(
                "段 *強 [リ](u)* **太** [参][r]\n\n- 項\n\n> 引\n\n| 表 |\n| - |\n\n# 見\n\n[r]: u\n"
            ),
            vec![
                ("段 ", plain),
                (
                    "強 ",
                    Context {
                        emphasis: true,
                        ..Context::default()
                    }
                ),
                (
                    "リ",
                    Context {
                        emphasis: true,
                        link: true,
                        ..Context::default()
                    }
                ),
                (" ", plain),
                ("太", Context::default()),
                (" ", plain),
                ("参", Context::default()),
                ("項", plain),
                (
                    "引",
                    Context {
                        blockquote: true,
                        ..Context::default()
                    }
                ),
                ("表", Context::default()),
                (
                    "見",
                    Context {
                        heading: true,
                        ..Context::default()
                    }
                ),
            ]
        );
    }

    #[test]
    fn treats_text_after_a_bare_url_as_outside_the_link() {
        assert_eq!(
            contexts("公式（https://example.com）を参照"),
            vec![
                (
                    "公式（",
                    Context {
                        plain: true,
                        ..Context::default()
                    }
                ),
                (
                    "）を参照",
                    Context {
                        plain: true,
                        ..Context::default()
                    }
                ),
            ]
        );
    }

    #[test]
    fn marks_text_after_a_bare_url_inside_a_heading() {
        assert_eq!(
            in_heading(
                "# 公式（https://example.com）の説明\n\n本文（https://example.com）の説明\n"
            ),
            vec![
                ("公式（", true),
                ("）の説明", true),
                ("本文（", false),
                ("）の説明", false)
            ]
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

    fn paragraphs(src: &str) -> Vec<(&str, bool, bool, bool)> {
        parse(src)
            .paragraphs
            .into_iter()
            .map(|p| (&src[p.range], p.blockquote, p.list_item, p.footnote))
            .collect()
    }

    #[test]
    fn collects_paragraphs_outside_headings_and_table_cells_with_their_containers() {
        assert_eq!(
            paragraphs("段\n\n- 項\n\n> 引\n\n# 見\n\n| 表 |\n| - |\n\n文[^1]\n\n[^1]: 脚\n"),
            vec![
                ("段", false, false, false),
                ("項", false, true, false),
                ("引", true, false, false),
                ("文[^1]", false, false, false),
                ("脚", false, false, true),
            ]
        );
    }

    #[test]
    fn collects_atx_and_setext_headings_with_inline_children_and_blockquote_flag() {
        let src = "# 見 **太**\n\n段\n\n設定\n===\n\n> ## 引\n";
        let headings: Vec<(&str, bool, Vec<String>)> = parse(src)
            .headings
            .into_iter()
            .map(|h| {
                let texts = h.children.iter().map(|c| c.text.clone()).collect();
                (&src[h.range], h.blockquote, texts)
            })
            .collect();
        assert_eq!(
            headings,
            vec![
                (
                    "# 見 **太**",
                    false,
                    vec!["見 ".to_owned(), "太".to_owned()]
                ),
                ("設定\n===", false, vec!["設定".to_owned()]),
                ("## 引", true, vec!["引".to_owned()]),
            ]
        );
    }

    fn inline_texts(src: &str) -> Vec<(&str, String)> {
        parse(src).paragraphs[0]
            .children
            .iter()
            .map(|inline| (&src[inline.range.clone()], inline.text.clone()))
            .collect()
    }

    #[test]
    fn keeps_direct_text_as_source_and_resolves_text_inside_elements() {
        assert_eq!(
            inline_texts("a\\*b&amp;*c\\*d&amp;*`e` ![f](g)<br>h  \ni[^1]\n\n[^1]: x\n"),
            vec![
                ("a\\*b&amp;", "a\\*b&amp;".to_owned()),
                ("*c\\*d&amp;*", "c*d&".to_owned()),
                ("`e`", "e".to_owned()),
                (" ", " ".to_owned()),
                ("![f](g)", "f".to_owned()),
                ("<br>", String::new()),
                ("h", "h".to_owned()),
                ("  \n", String::new()),
                ("i", "i".to_owned()),
                ("[^1]", String::new()),
            ]
        );
    }

    #[test]
    fn keeps_a_bare_url_as_one_link_including_the_text_after_it() {
        let src = "公式（https://example.com）を参照。";
        let paragraph = &parse(src).paragraphs[0];
        let link = &paragraph.children[1];
        assert_eq!(&src[link.range.clone()], "https://example.com）を参照。");
        assert!(matches!(link.kind, InlineKind::Link { .. }));
    }
}
