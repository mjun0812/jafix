//! `@textlint-rule/pattern` (`@textlint-rule/textlint-rule-pattern` 2.0.2)。`replace` のある要素の箇所を利用者の置き換えに、
//! 無い要素の箇所を指摘にする

use crate::apply::TargetLines;
use crate::prose::{Context, Document};

use super::builtin::{PatternNodeType, TextlintRulePattern};
use super::{Report, UserReplacement};

/// `context` の本文の範囲が、`node_type` の型のnodeの中にあるか
#[allow(dead_code)] // skeleton: not implemented
fn contains(node_type: PatternNodeType, context: &Context) -> bool {
    let _ = (node_type, context);
    todo!("skeleton: not implemented")
}

/// 対象行と重なる本文の範囲に `replace` のある要素を当て、置き換えの箇所を返す。重なりは `rules::replacements` が解く
pub fn replacements(
    src: &str,
    document: &Document,
    rule: &TextlintRulePattern,
    targets: &TargetLines,
) -> Vec<UserReplacement> {
    let _ = (src, document, rule, targets);
    todo!("skeleton: not implemented")
}

/// 対象行と重なる本文の範囲に `replace` の無い要素を当て、指摘を返す。重なってもすべて出す
pub fn report(
    src: &str,
    document: &Document,
    rule: &TextlintRulePattern,
    targets: &TargetLines,
) -> Vec<Report<'static>> {
    let _ = (src, document, rule, targets);
    todo!("skeleton: not implemented")
}
