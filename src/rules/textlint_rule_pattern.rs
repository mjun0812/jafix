//! `@textlint-rule/pattern` (`@textlint-rule/textlint-rule-pattern` 2.0.2)。`replace` のある要素の箇所を利用者の置き換えに、
//! 無い要素の箇所を指摘にする

use std::borrow::Cow;

use crate::prose::{Context, Prose};

use super::builtin::{PatternNodeType, TextlintRulePattern, UserPattern};
use super::{Finding, Report, RuleId, UserReplacement, one_line};

/// `context` の本文の範囲が、`node_type` の型のnodeの中にあるか。`terminology` の `skip` も使う
pub(super) fn contains(node_type: PatternNodeType, context: &Context) -> bool {
    match node_type {
        PatternNodeType::Paragraph => context.paragraph,
        PatternNodeType::Header => context.heading,
        PatternNodeType::BlockQuote => context.blockquote,
        PatternNodeType::ListItem => context.list_item,
        PatternNodeType::Table => context.table,
        PatternNodeType::Emphasis => context.emphasis,
        PatternNodeType::Strong => context.strong,
        PatternNodeType::Delete => context.delete,
        PatternNodeType::Link => context.link,
        PatternNodeType::LinkReference => context.link_reference,
    }
}

/// 対象行と重なる本文の範囲に `replace` のある要素を当て、置き換えの箇所を返す。重なりは `rules::replacements` が解く
pub fn replacements(
    src: &str,
    texts: &[&Prose],
    rule: &TextlintRulePattern,
) -> Vec<UserReplacement> {
    let mut replacements = Vec::new();
    for (prose, pattern, m) in matches(src, texts, rule, true) {
        let Some(replace) = &pattern.replace else {
            continue;
        };
        let text = &src[prose.range.clone()];
        let matched = &text[m.range()];
        let replaced = expand(replace, &m, text);
        if replaced == matched {
            continue;
        }
        let start = prose.range.start + m.start();
        replacements.push(UserReplacement {
            finding: Finding {
                range: start..prose.range.start + m.end(),
                replacement: Cow::Owned(replaced),
                rule: RuleId::TextlintRulePattern,
                position: start,
                message: Cow::Owned(message(matched, pattern)),
            },
            heading: prose.context.heading,
        });
    }
    replacements
}

/// 対象行と重なる本文の範囲に `replace` の無い要素を当て、指摘を返す。重なってもすべて出す
pub fn report(src: &str, texts: &[&Prose], rule: &TextlintRulePattern) -> Vec<Report> {
    matches(src, texts, rule, false)
        .map(|(prose, pattern, m)| Report {
            range: prose.range.start + m.start()..prose.range.start + m.end(),
            rule: RuleId::TextlintRulePattern.name(),
            message: Cow::Owned(message(&src[prose.range.clone()][m.range()], pattern)),
        })
        .collect()
}

fn message(matched: &str, pattern: &UserPattern) -> String {
    format!("「{}」: {}", one_line(matched), one_line(&pattern.message))
}

/// 対象行と重なる本文の範囲への、`replace` の有無が `replace` と同じ要素の一致。長さ0の一致と、`allows` か
/// `allowNodeTypes` で除く一致は含めない。本文の範囲ごとに、要素の順、位置の順に並ぶ
fn matches<'a>(
    src: &'a str,
    texts: &'a [&'a Prose],
    rule: &'a TextlintRulePattern,
    replace: bool,
) -> impl Iterator<Item = (&'a Prose, &'a UserPattern, regress::Match)> {
    texts.iter().copied().flat_map(move |prose| {
        let text = &src[prose.range.clone()];
        rule.patterns
            .iter()
            .filter(move |pattern| {
                pattern.replace.is_some() == replace
                    && !pattern
                        .allow_node_types
                        .iter()
                        .any(|&node_type| contains(node_type, &prose.context))
            })
            .flat_map(move |pattern| {
                pattern
                    .pattern
                    .find_iter(text)
                    .filter(move |m| {
                        !m.range.is_empty()
                            && !pattern
                                .allows
                                .iter()
                                .any(|allow| allow.find(&text[m.range()]).is_some())
                    })
                    .map(move |m| (prose, pattern, m))
            })
    })
}

/// `replace` の `$1`〜`$9` を、一致の捕捉グループに展開する。参加しなかったグループ、グループの数を超える番号、`$0` は書いたまま残す
fn expand(replace: &str, m: &regress::Match, text: &str) -> String {
    let mut out = String::with_capacity(replace.len());
    let mut rest = replace;
    while let Some(dollar) = rest.find('$') {
        out.push_str(&rest[..dollar]);
        let after = &rest[dollar + 1..];
        let group = after
            .bytes()
            .next()
            .filter(|b| matches!(b, b'1'..=b'9'))
            .and_then(|b| m.group(usize::from(b - b'0')));
        match group {
            Some(range) => {
                out.push_str(&text[range]);
                rest = &after[1..];
            }
            None => {
                out.push('$');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}
