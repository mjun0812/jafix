//! `no-todo`。本文の範囲の `todo:` と、リスト項目の未完了のタスクの記号を指摘する

use crate::prose::Document;
use crate::rules::Report;

/// リンクの文字列と引用の中を除く本文の範囲の `todo:` (大文字と小文字を区別しない) のすべての一致と、リスト項目ごとの
/// 最初の未完了のタスクの記号 (項目の先頭のGFMのタスクの記号か、項目の直下の段落の直下の文字列の `[ ]`) を返す
pub fn report(src: &str, document: &Document) -> Vec<Report> {
    let _ = (src, document);
    todo!("skeleton: not implemented")
}
