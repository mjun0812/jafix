//! 段落ごとに対を数える5ルール (`4.3.3`〜`4.3.7`)。引用の中にない段落の中の本文の範囲 (リンクの文字列・太字・
//! 強調・取り消し線の中を含み、インラインコードの中身と画像のaltを含まない) を順に見る

use crate::prose::Document;

use super::super::Report;
use super::super::builtin::JtfStyle;
use super::JtfRule;

/// 5ルールの指摘。開きと閉じが違う記号は、開きを積み、閉じで直前の開きと対にして、対にならないものを指摘する。
/// `4.3.5` の `"` は、段落の中の数が奇数なら最後の `"` を指摘する
pub fn report(src: &str, document: &Document, rules: &JtfStyle) -> Vec<Report> {
    let rules_here = [
        JtfRule::R4_3_3,
        JtfRule::R4_3_4,
        JtfRule::R4_3_5,
        JtfRule::R4_3_6,
        JtfRule::R4_3_7,
    ];
    if !rules_here.into_iter().any(|rule| rules.enabled(rule)) {
        return Vec::new();
    }
    let _ = (src, document);
    todo!("skeleton: not implemented")
}
