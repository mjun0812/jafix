//! `prh` (textlint-rule-prh 6.1.1)。利用者の辞書を本文の範囲に当て、置き換えの箇所を利用者の置き換えにする

pub mod dictionary;

use crate::apply::TargetLines;
use crate::prose::{Context, Document};

use super::UserReplacement;
use super::builtin::Prh;

/// `check*` のオプション。本文の範囲のうち、辞書を当てるものを決める。既定はtextlint-rule-prhと同じである
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checks {
    /// `checkLink`
    pub link: bool,
    /// `checkBlockQuote`
    pub block_quote: bool,
    /// `checkEmphasis`
    pub emphasis: bool,
    /// `checkHeader`
    pub header: bool,
    /// `checkParagraph`
    pub paragraph: bool,
}

impl Default for Checks {
    fn default() -> Self {
        Self {
            link: false,
            block_quote: false,
            emphasis: false,
            header: true,
            paragraph: true,
        }
    }
}

impl Checks {
    /// `context` の本文の範囲に辞書を当てるか
    #[allow(dead_code)] // skeleton: not implemented
    pub fn covers(&self, context: &Context) -> bool {
        let _ = context;
        todo!("skeleton: not implemented")
    }
}

/// 対象行と重なる本文の範囲のうち `rule.checks` が見るものに辞書を当て、置き換えの箇所を返す。
/// 本文の範囲の中の重なりは辞書が解き、`@textlint-rule/pattern` の箇所との重なりは `rules::replacements` が解く
pub fn replacements(
    src: &str,
    document: &Document,
    rule: &Prh,
    targets: &TargetLines,
) -> Vec<UserReplacement> {
    let _ = (src, document, rule, targets);
    todo!("skeleton: not implemented")
}
