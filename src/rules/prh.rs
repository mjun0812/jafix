//! `prh` (textlint-rule-prh 6.1.1)。利用者の辞書を本文の範囲に当て、置き換えの箇所を利用者の置き換えにする

pub mod dictionary;

use std::borrow::Cow;

use crate::prose::{Context, Prose};

use self::dictionary::Replacement;
use super::builtin::Prh;
use super::{Finding, RuleId, UserReplacement, one_line};

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
    pub fn covers(&self, context: &Context) -> bool {
        !(context.link && !self.link
            || context.blockquote && !self.block_quote
            || context.emphasis && !self.emphasis
            || context.heading && !self.header
            || context.paragraph && !self.paragraph)
    }
}

/// 対象行と重なる本文の範囲のうち `rule.checks` が見るものに辞書を当て、置き換えの箇所を返す。
/// 本文の範囲の中の重なりは辞書が解き、`@textlint-rule/pattern` の箇所との重なりは `rules::user_replacements` が解く
pub fn replacements(src: &str, texts: &[&Prose], rule: &Prh) -> Vec<UserReplacement> {
    let mut replacements = Vec::new();
    for prose in texts {
        if !rule.checks.covers(&prose.context) {
            continue;
        }
        let text = &src[prose.range.clone()];
        for Replacement {
            range,
            text: replaced,
            rule: index,
        } in rule.dictionary.replacements(text)
        {
            let dictionary_rule = &rule.dictionary.rules()[index];
            let mut message = format!(
                "「{}」を「{}」にする",
                one_line(&text[range.clone()]),
                one_line(&replaced)
            );
            if let Some(note) = &dictionary_rule.note {
                message.push_str(": ");
                message.push_str(note);
            }
            let start = prose.range.start + range.start;
            replacements.push(UserReplacement {
                finding: Finding {
                    range: start..prose.range.start + range.end,
                    replacement: Cow::Owned(replaced),
                    rule: RuleId::Prh,
                    position: start,
                    message: Cow::Owned(message),
                },
                heading: prose.context.heading,
            });
        }
    }
    replacements
}
