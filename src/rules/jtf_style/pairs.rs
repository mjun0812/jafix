//! 段落ごとに対を数える5ルール (`4.3.3`〜`4.3.7`)。引用の中にない段落の中の本文の範囲 (リンクの文字列・太字・
//! 強調・取り消し線の中を含み、インラインコードの中身と画像のaltを含まない) を順に見る

use crate::prose::Document;

use super::super::Report;
use super::super::builtin::JtfStyle;

/// 5ルールの指摘。開きと閉じが違う記号は、開きを積み、閉じで直前の開きと対にして、対にならないものを指摘する。
/// `4.3.5` の `"` は、段落の中の数が奇数なら最後の `"` を指摘する
pub fn report(src: &str, document: &Document, rules: &JtfStyle) -> Vec<Report> {
    let _ = (src, document, rules);
    todo!("skeleton: not implemented")
}
