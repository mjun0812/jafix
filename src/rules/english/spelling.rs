//! `spelling` (textlint-rule-spelling 0.3.0)。本文の範囲と画像のaltから取り出した英単語を、`dictionary-en` 4.0.0 の
//! en_USのhunspellの辞書で `spellbook` に照合し、辞書に無い語を指摘する。textlint-rule-spellingのコードはGPLなので写さず、
//! 判定の条件から書き直した

use std::ops::Range;
use std::sync::LazyLock;

use foldhash::fast::FixedState;
use spellbook::Dictionary;

use crate::prose::Prose;
use crate::rules::Report;
use crate::rules::builtin::Spelling;

/// `dictionary-en` 4.0.0 の `index.aff` と `index.dic` を、中身を変えずに写したもの
static AFF: &str = include_str!("data/en_US.aff");
static DIC: &str = include_str!("data/en_US.dic");

/// 照合する語が最初に来たときに1回だけ読む辞書。`Dictionary::new` はプロセスごとに種の変わるhasherで語の表を作り、
/// `suggest` の候補の並びが実行ごとに変わるので、固定の種の `FixedState` で作る。種を変えると候補の並びが変わりうる
#[allow(dead_code)] // skeleton: not implemented
static DICTIONARY: LazyLock<Dictionary<FixedState>> = LazyLock::new(|| {
    let _ = (AFF, DIC);
    todo!("skeleton: not implemented")
});

/// `texts` と `alts` の元のソースに `wordDefinitionRegexp` を当てて語を取り出し、既定の除外 (ASCIIの数字か `_` を含む語、
/// 2文字目以降に `A`〜`Z` を含む語) と `skipPatterns` の一致と重なる語を除き、辞書に無い語を指摘する。範囲は語の全体である。
/// `suggestCorrections` が真なら、指摘にする語にだけ候補を求め、同じプロセスの中で同じ語の候補を再利用する
pub fn report(src: &str, texts: &[&Prose], alts: &[Range<usize>], rule: &Spelling) -> Vec<Report> {
    let _ = (src, texts, alts, rule);
    todo!("skeleton: not implemented")
}
