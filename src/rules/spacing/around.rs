//! インラインのnodeの前後に当てる4ルール。`ja-space-around-code` はインラインコード、`ja-space-around-link` は
//! リンク、`ja-space-around-strong` は太字、`ja-space-around-emphasis` は強調の前後の文字を見る

use crate::prose::InlineNode;

use super::super::builtin::Builtin;
use super::super::{FixRuleFinding, Report};

/// `before`・`after` が `false` の側の修正の箇所。nodeの前か後ろに文字が無い側は判定しない
pub fn find(src: &str, nodes: &[InlineNode], builtin: &Builtin) -> Vec<FixRuleFinding> {
    let _ = (src, nodes, builtin);
    todo!("skeleton: not implemented")
}

/// `before`・`after` が `true` の側の指摘。位置は、nodeの前の文字か後ろの文字である
pub fn report(src: &str, nodes: &[InlineNode], builtin: &Builtin) -> Vec<Report> {
    let _ = (src, nodes, builtin);
    todo!("skeleton: not implemented")
}

/// 太字か強調の `node` の前か後ろの空白 (`space`、元のソース上のバイト範囲) を消しても、消した後の文字列を
/// Markdownとして解析し直して、同じ範囲が同じ種類のnodeのままか。区切りが `_` のときや、区切りの内側の文字が
/// 句読点か記号のときは、消すと強調として読まれなくなる
fn keeps_emphasis(src: &str, node: &InlineNode, space: std::ops::Range<usize>) -> bool {
    let _ = (src, node, space);
    todo!("skeleton: not implemented")
}
