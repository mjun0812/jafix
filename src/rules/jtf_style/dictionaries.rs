//! 辞書の4ルール (`2.1.5`・`2.1.6`・`2.2.1`・`2.2.3`)。textlint-rule-preset-jtf-style 3.0.3 の `dict/*.yml` を元の
//! ファイルのまま埋め込み、`prh` の辞書の読み込みと照合で当てる。箇所はすべて指摘にする

use crate::prose::Prose;

use super::super::Report;
use super::super::builtin::JtfStyle;
use super::super::prh::dictionary::Dictionary;
use super::JtfRule;

/// 辞書を持つルールと、その辞書のYAML
const DICTIONARIES: [(JtfRule, &str); 4] = [
    (JtfRule::R2_1_5, include_str!("dict/2.1.5.yml")),
    (JtfRule::R2_1_6, include_str!("dict/2.1.6.yml")),
    (JtfRule::R2_2_1, include_str!("dict/2.2.1.yml")),
    (JtfRule::R2_2_3, include_str!("dict/2.2.3.yml")),
];

/// 有効な辞書のルールを、`texts` のうち `Checks::default().covers` が真の本文の範囲に当てた指摘。
/// 文面は `prh` と同じ `「<一致>」を「<置き換え後>」にする` である。見出しの中の箇所も、文面を変えずに指摘にする
pub fn report(src: &str, texts: &[&Prose], rules: &JtfStyle) -> Vec<Report> {
    let _ = (src, texts, rules, dictionary);
    todo!("skeleton: not implemented")
}

/// `rule` の辞書。最初に呼ばれたときに1回だけ読む。埋め込んだ辞書は読めることがテストで決まっているので、
/// 読めなければ不具合として止める
fn dictionary(rule: JtfRule) -> &'static Dictionary {
    let _ = (rule, DICTIONARIES);
    todo!("skeleton: not implemented")
}
