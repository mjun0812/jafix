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
    /// 見出し。段落と同じ形で持つ。`footnote`・`list_item_index`・`next_block` は使わない。
    /// `paragraphs` には入らない
    pub headings: Vec<Paragraph>,
    /// リスト項目。出現順に並び、入れ子の項目は親の項目の後に来る
    pub list_items: Vec<ListItem>,
    /// 文書のどこにあるかによらない、すべてのインラインコード・リンク・太字・強調・画像。出現順に並び、
    /// 入れ子のnodeは外側のnodeの後に来る。段落・見出し・表のセル・リンクの文字列・太字の中を含む
    pub inline_nodes: Vec<InlineNode>,
    /// 文書のどこにあるかによらない、すべてのHTMLのnode (ブロックとインライン) の中のコメント。出現順に並ぶ。
    /// 引用・リスト項目・表のセル・見出しの中を含み、コードブロックとインラインコードの中はHTMLのnodeにならないので入らない
    pub comments: Vec<HtmlComment>,
    /// 文書のどこにあるかによらない、インラインのリンク・自動リンク・裸のURL・参照リンクの定義の宛先。出現順に並ぶ。
    /// 画像の宛先と、インラインコード・コードブロック・HTMLの中のURLは入らない
    pub links: Vec<LinkDestination>,
}

/// HTMLのコメント (`<!--` から次の `-->` まで)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlComment {
    /// `<!--` の元のソース上の位置
    pub start: usize,
    /// `<!--` と `-->` の間の中身の、元のソース上のバイト範囲
    pub content: Range<usize>,
}

/// リンクの宛先
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkDestination {
    /// 宛先の文字列。インラインのリンク・自動リンク・定義と、`http://`・`https://` で始まらない裸のURL
    /// (メールアドレス) では、markdown-rsの `url` (エスケープと文字参照を解いた文字列) である。`http://`・`https://`
    /// の裸のURLでは、本文の範囲と同じく対にならない最初の `）` の手前で切った元のソースで、`www.` で始まれば
    /// 先頭に `http://` を足す
    pub url: String,
    /// 宛先の元のソース上のバイト範囲。インラインのリンクでは `](` の後 (`<` で囲めばその内側)、自動リンクでは
    /// `<` と `>` の内側、裸のURLでは切った元のソース、定義では `]:` の後の宛先の文字列である
    pub range: Range<usize>,
    /// 引用の中
    pub blockquote: bool,
}

/// インラインのnodeの前後を見るルールが当てるnode
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineNode {
    pub kind: InlineNodeKind,
    /// 元のソース上のバイト範囲。区切りの記号 (`` ` ``・`**`・`[`・`<` など) を含む
    pub range: Range<usize>,
    /// 見出しの中
    pub heading: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlineNodeKind {
    Code,
    /// インラインのリンク (`[x](u)`) と自動リンク (`<…>`)。裸のURLと参照形式のリンクは含めない
    Link,
    Strong,
    Emphasis,
    /// インラインの画像 (`![alt](url)`)。参照形式の画像は含めない
    Image {
        /// altの元のソース上のバイト範囲。`![` の直後から、対応する `]` の直前までである
        alt: Range<usize>,
    },
}

/// リスト項目
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// 元のソース上のバイト範囲。リストの記号から、入れ子の項目と継続行を含む項目の終わりまでである
    pub range: Range<usize>,
    /// 項目が属するリストの、文書の中での出現順の番号。同じリストの項目は同じ番号を持ち、入れ子のリストは
    /// 親のリストと別の番号を持つ
    pub list: usize,
    /// GFMのタスクの記号。未完了 (`[ ]`) なら `Some(false)`、完了 (`[x]`) なら `Some(true)`、無ければ `None` である
    pub checked: Option<bool>,
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
    /// 段落の中。リスト項目・引用・脚注の定義の中の段落を含み、見出しと表のセルは段落ではない
    pub paragraph: bool,
    /// リスト項目の中
    pub list_item: bool,
    /// 表の中
    pub table: bool,
    /// 太字 (`**x**`) の中
    pub strong: bool,
    /// 取り消し線の中
    pub delete: bool,
    /// full形の参照リンク (`[x][id]`) の中
    pub link_reference: bool,
}

/// 段落。`range` は元のソース上のバイト範囲
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paragraph {
    pub range: Range<usize>,
    /// 引用の中
    pub blockquote: bool,
    /// 脚注の定義の中
    pub footnote: bool,
    /// 段落を含むリスト項目のうち、最も内側のものの `Document::list_items` での番号
    pub list_item_index: Option<usize>,
    /// 文書の直下の段落で、次のブロックが `BlockKind` のどれかなら、その種類
    pub next_block: Option<BlockKind>,
    pub children: Vec<Inline>,
}

/// 文書の直下の段落の次に来るブロックのうち、ルールが見る種類
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// コードブロック (フェンスとインデントの両方)
    CodeBlock,
    List,
    Blockquote,
    Table,
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

/// Markdownを解析し、本文の範囲・段落・見出し・リスト項目・インラインのnodeを取り出す
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
    collect_paragraphs(&root, src, Ancestors::default(), None, &mut blocks);
    let mut inline_nodes = Vec::new();
    collect_inline_nodes(&root, src, false, &mut inline_nodes);
    Document {
        texts,
        paragraphs: blocks.paragraphs,
        headings: blocks.headings,
        list_items: blocks.list_items,
        inline_nodes,
        // skeleton: not implemented (`collect_comments` と `collect_links` で集める)
        comments: Vec::new(),
        links: Vec::new(),
    }
}

/// `node` の中のHTMLのnodeから、コメントを出現順に集める
#[allow(dead_code)] // skeleton: not implemented
fn collect_comments(node: &Node, src: &str, comments: &mut Vec<HtmlComment>) {
    let _ = (node, src, comments);
    todo!("skeleton: not implemented")
}

/// `node` の中のリンクの宛先を出現順に集める。`blockquote` は、`node` が引用の中にあるか
#[allow(dead_code)] // skeleton: not implemented
fn collect_links(node: &Node, src: &str, blockquote: bool, links: &mut Vec<LinkDestination>) {
    let _ = (node, src, blockquote, links);
    todo!("skeleton: not implemented")
}

/// `node` の中のインラインコード・リンク・太字・強調・画像を `nodes` に集める。`heading` は、`node` が見出しの中にあるか
fn collect_inline_nodes(node: &Node, src: &str, heading: bool, nodes: &mut Vec<InlineNode>) {
    let kind = match node {
        Node::InlineCode(_) => Some(InlineNodeKind::Code),
        // 裸のURLもLinkになるが、`[` か `<` で始まらない
        Node::Link(_) if matches!(src.as_bytes()[node_range(node).start], b'[' | b'<') => {
            Some(InlineNodeKind::Link)
        }
        Node::Strong(_) => Some(InlineNodeKind::Strong),
        Node::Emphasis(_) => Some(InlineNodeKind::Emphasis),
        Node::Image(_) => {
            let image = node_range(node);
            let alt_start = image.start + "![".len();
            Some(InlineNodeKind::Image {
                alt: alt_start..alt_end(&src[..image.end], alt_start),
            })
        }
        _ => None,
    };
    if let Some(kind) = kind {
        nodes.push(InlineNode {
            kind,
            range: node_range(node),
            heading,
        });
    }
    let heading = heading || matches!(node, Node::Heading(_));
    for child in node.children().into_iter().flatten() {
        collect_inline_nodes(child, src, heading, nodes);
    }
}

/// `start` から始まる画像のaltの終わり (対応する `]` の位置) を返す。`image` は、元のソースの先頭から画像の
/// 終わりまでである。mdastの画像はaltの位置を持たないので、元のソースの括弧を数える。CommonMarkと同じく、
/// altの中の対になった `[` `]`、エスケープした `\]`、コードスパンの中の `[` `]` では終わらない
fn alt_end(image: &str, start: usize) -> usize {
    let bytes = image.as_bytes();
    let mut depth = 0;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'`' => {
                let run = backtick_run(bytes, i);
                // 閉じの並びが無いバッククォートは、コードスパンを始めず文字のまま残る
                i = code_span_end(bytes, i + run, run).unwrap_or(i + run);
                continue;
            }
            b'[' => depth += 1,
            b']' if depth == 0 => return i,
            b']' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    // 自動リンクやHTMLのタグの中の `]` など、上で数えない書き方のaltでは、画像の最後の `](` を終わりとみなす
    image[start..]
        .rfind("](")
        .map_or(bytes.len(), |end| start + end)
}

/// `i` から続くバッククォートの数
fn backtick_run(bytes: &[u8], i: usize) -> usize {
    bytes[i..].iter().take_while(|&&b| b == b'`').count()
}

/// `from` より後ろで、長さがちょうど `run` のバッククォートの並びを探し、その並びの直後の位置を返す
fn code_span_end(bytes: &[u8], from: usize, run: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let len = backtick_run(bytes, i);
        if len == run {
            return Some(i + len);
        }
        i += len;
    }
    None
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
                paragraph: context.paragraph || matches!(node, Node::Paragraph(_)),
                list_item: context.list_item || matches!(node, Node::ListItem(_)),
                table: context.table || matches!(node, Node::Table(_)),
                strong: context.strong || matches!(node, Node::Strong(_)),
                delete: context.delete || matches!(node, Node::Delete(_)),
                link_reference: context.link_reference || matches!(node, Node::LinkReference(_)),
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
    list_items: Vec<ListItem>,
    /// これまでに見たリストの数
    lists: usize,
}

/// `node` の祖先にある、段落の性質を決めるブロック
#[derive(Clone, Copy, Default)]
struct Ancestors {
    blockquote: bool,
    /// 最も内側のリスト項目の `Blocks::list_items` での番号
    list_item: Option<usize>,
    /// 最も内側のリストの、文書の中での出現順の番号。リストの外では使わない
    list: usize,
    footnote: bool,
}

/// `node` の中の段落・見出し・リスト項目を集める。`next_block` は、`node` が文書の直下の段落のときの、次の兄弟の種類である
fn collect_paragraphs(
    node: &Node,
    src: &str,
    ancestors: Ancestors,
    next_block: Option<BlockKind>,
    blocks: &mut Blocks,
) {
    let target = match node {
        Node::Paragraph(_) => &mut blocks.paragraphs,
        Node::Heading(_) => &mut blocks.headings,
        _ => {
            let mut children_ancestors = Ancestors {
                blockquote: ancestors.blockquote || matches!(node, Node::Blockquote(_)),
                footnote: ancestors.footnote || matches!(node, Node::FootnoteDefinition(_)),
                ..ancestors
            };
            let children = node.children().map(Vec::as_slice).unwrap_or_default();
            if matches!(node, Node::List(_)) {
                children_ancestors.list = blocks.lists;
                blocks.lists += 1;
            }
            if let Node::ListItem(item) = node {
                children_ancestors.list_item = Some(blocks.list_items.len());
                // markdown-rsは、空行が続く項目の範囲に、続く改行 (引用の中では次の行の `>` まで) を含めるので、
                // textlintと同じく最後の子のブロックの終わりで項目を終える
                let range = node_range(node);
                let end = children
                    .last()
                    .map_or(range.end, |last| node_range(last).end);
                blocks.list_items.push(ListItem {
                    range: range.start..end,
                    list: ancestors.list,
                    checked: item.checked,
                });
            }
            for (i, child) in children.iter().enumerate() {
                let next_block = match (node, child) {
                    (Node::Root(_), Node::Paragraph(_)) => children.get(i + 1).and_then(block_kind),
                    _ => None,
                };
                collect_paragraphs(child, src, children_ancestors, next_block, blocks);
            }
            return;
        }
    };
    target.push(Paragraph {
        range: node_range(node),
        blockquote: ancestors.blockquote,
        footnote: ancestors.footnote,
        list_item_index: ancestors.list_item,
        next_block,
        children: node
            .children()
            .into_iter()
            .flatten()
            .filter_map(|child| inline(child, src, true))
            .collect(),
    });
}

fn block_kind(node: &Node) -> Option<BlockKind> {
    match node {
        Node::Code(_) => Some(BlockKind::CodeBlock),
        Node::List(_) => Some(BlockKind::List),
        Node::Blockquote(_) => Some(BlockKind::Blockquote),
        Node::Table(_) => Some(BlockKind::Table),
        _ => None,
    }
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
        let paragraph = Context {
            paragraph: true,
            ..Context::default()
        };
        let plain = Context {
            plain: true,
            ..paragraph
        };
        assert_eq!(
            contexts(
                "段 *強 [リ](u)* **太** [参][r] ~~消~~\n\n- 項\n\n> 引\n\n| 表 |\n| - |\n\n# 見\n\n[r]: u\n"
            ),
            vec![
                ("段 ", plain),
                (
                    "強 ",
                    Context {
                        emphasis: true,
                        ..paragraph
                    }
                ),
                (
                    "リ",
                    Context {
                        emphasis: true,
                        link: true,
                        ..paragraph
                    }
                ),
                (" ", plain),
                (
                    "太",
                    Context {
                        strong: true,
                        ..paragraph
                    }
                ),
                (" ", plain),
                (
                    "参",
                    Context {
                        link_reference: true,
                        ..paragraph
                    }
                ),
                (" ", plain),
                (
                    "消",
                    Context {
                        delete: true,
                        ..paragraph
                    }
                ),
                (
                    "項",
                    Context {
                        list_item: true,
                        ..plain
                    }
                ),
                (
                    "引",
                    Context {
                        blockquote: true,
                        ..paragraph
                    }
                ),
                (
                    "表",
                    Context {
                        table: true,
                        ..Context::default()
                    }
                ),
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
                        paragraph: true,
                        ..Context::default()
                    }
                ),
                (
                    "）を参照",
                    Context {
                        plain: true,
                        paragraph: true,
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
            .map(|p| {
                (
                    &src[p.range],
                    p.blockquote,
                    p.list_item_index.is_some(),
                    p.footnote,
                )
            })
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

    #[test]
    fn collects_list_items_in_order_with_the_innermost_item_of_each_paragraph() {
        let src = "- 親\n  - 子\n\n  続き\n- 次\n\n段\n";
        let document = parse(src);
        let items: Vec<&str> = document
            .list_items
            .iter()
            .map(|item| &src[item.range.clone()])
            .collect();
        assert_eq!(items, vec!["- 親\n  - 子\n\n  続き", "- 子", "- 次"]);
        let paragraphs: Vec<(&str, Option<usize>)> = document
            .paragraphs
            .iter()
            .map(|p| (&src[p.range.clone()], p.list_item_index))
            .collect();
        assert_eq!(
            paragraphs,
            vec![
                ("親", Some(0)),
                ("子", Some(1)),
                ("続き", Some(0)),
                ("次", Some(2)),
                ("段", None)
            ]
        );
    }

    #[test]
    fn numbers_each_list_and_gives_nested_lists_their_own_number() {
        let src = "- a\n  - b\n  - c\n- d\n\n段\n\n* e\n";
        let lists: Vec<(&str, usize)> = parse(src)
            .list_items
            .iter()
            .map(|item| (&src[item.range.start + 2..item.range.start + 3], item.list))
            .collect();
        assert_eq!(
            lists,
            vec![("a", 0), ("b", 1), ("c", 1), ("d", 0), ("e", 2)]
        );
    }

    #[test]
    fn collects_inline_images_with_the_source_range_of_their_alt() {
        let src =
            "![図 \\] [a] 図](x.png \"t\") ![](y.png) ![参照][r]\n\n# ![見](z.png)\n\n[r]: u\n";
        let alts: Vec<(&str, bool)> = parse(src)
            .inline_nodes
            .into_iter()
            .filter_map(|node| match node.kind {
                InlineNodeKind::Image { alt } => Some((&src[alt], node.heading)),
                _ => None,
            })
            .collect();
        assert_eq!(
            alts,
            vec![("図 \\] [a] 図", false), ("", false), ("見", true)]
        );
    }

    fn image_alts(src: &str) -> Vec<&str> {
        parse(src)
            .inline_nodes
            .into_iter()
            .filter_map(|node| match node.kind {
                InlineNodeKind::Image { alt } => Some(&src[alt]),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn does_not_end_an_image_alt_at_brackets_inside_a_code_span() {
        assert_eq!(image_alts("![a `[` b](x.png)\n"), vec!["a `[` b"]);
        assert_eq!(image_alts("![a `]` b](x.png)\n"), vec!["a `]` b"]);
        assert_eq!(image_alts("![a `[` b](x.png) c]\n"), vec!["a `[` b"]);
        assert_eq!(image_alts("![a ``x`]`` b](x.png)\n"), vec!["a ``x`]`` b"]);
    }

    #[test]
    fn ends_a_list_item_at_its_last_block_before_a_blank_line_inside_a_blockquote() {
        let src = "> - a\n>\n> - b\n";
        let items: Vec<&str> = parse(src)
            .list_items
            .iter()
            .map(|item| &src[item.range.clone()])
            .collect();
        assert_eq!(items, vec!["- a", "- b"]);
    }

    #[test]
    fn marks_the_next_block_only_for_paragraphs_directly_under_the_document() {
        let src = "a\n\n```\nx\n```\n\nb\n- c\n\nd\n\n> e\n\nf\n\n| g |\n| - |\n\nh\n\n# i\n\n> j\n>\n> - k\n";
        let next: Vec<(&str, Option<BlockKind>)> = parse(src)
            .paragraphs
            .iter()
            .map(|p| (&src[p.range.clone()], p.next_block))
            .collect();
        assert_eq!(
            next,
            vec![
                ("a", Some(BlockKind::CodeBlock)),
                ("b", Some(BlockKind::List)),
                ("c", None),
                ("d", Some(BlockKind::Blockquote)),
                ("e", None),
                ("f", Some(BlockKind::Table)),
                ("h", None),
                ("j", None),
                ("k", None),
            ]
        );
    }

    #[test]
    fn collects_inline_nodes_anywhere_except_bare_urls_and_reference_links() {
        let src = "# 見 `a`\n\n[**b** *c*](u) <https://e.com> https://e.com [d][r]\n\n| `e` |\n| - |\n\n[r]: u\n";
        let nodes: Vec<(&str, InlineNodeKind, bool)> = parse(src)
            .inline_nodes
            .into_iter()
            .map(|node| (&src[node.range], node.kind, node.heading))
            .collect();
        assert_eq!(
            nodes,
            vec![
                ("`a`", InlineNodeKind::Code, true),
                ("[**b** *c*](u)", InlineNodeKind::Link, false),
                ("**b**", InlineNodeKind::Strong, false),
                ("*c*", InlineNodeKind::Emphasis, false),
                ("<https://e.com>", InlineNodeKind::Link, false),
                ("`e`", InlineNodeKind::Code, false),
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
