use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{InlineKind, Paragraph};
use crate::rules::builtin::{self, JaNoRedundantExpression, NodeType};
use crate::rules::{Report, RuleId};

use super::Analyzed;
use super::dict::{REDUNDANTS, Redundant};
use super::pattern;

/// 辞書の項目ごとの既定の `allows`
static DEFAULT_ALLOWS: LazyLock<Vec<Vec<Regex>>> = LazyLock::new(|| {
    REDUNDANTS
        .iter()
        .map(|entry| {
            entry
                .allows
                .iter()
                .map(|allow| builtin::pattern(allow).expect("辞書の既定のパターンは読める"))
                .collect()
        })
        .collect()
});

/// `paragraph` の文が対象か。`allowNodeTypes` の `BlockQuote` と `ListItem` は、引用とリスト項目の中の段落を除く
pub(super) fn targets(rule: &JaNoRedundantExpression, paragraph: &Paragraph) -> bool {
    let allowed = |node_type| rule.allow_node_types.contains(&node_type);
    !(paragraph.blockquote && allowed(NodeType::BlockQuote))
        && !(paragraph.list_item && allowed(NodeType::ListItem))
}

/// 辞書の項目の並びに一致した箇所のうち、`allows` に当たらず、`allowNodeTypes` の要素の中にないものに指摘を出す
pub(super) fn report(sentence: &Analyzed, rule: &JaNoRedundantExpression) -> Vec<Report<'static>> {
    let tokens = &sentence.tokens;
    let mut reports = Vec::new();
    for ((entry, option), default_allows) in REDUNDANTS
        .iter()
        .zip(&rule.dicts)
        .zip(DEFAULT_ALLOWS.iter())
    {
        if option.disabled {
            continue;
        }
        let allows = option.allows.as_ref().unwrap_or(default_allows);
        for found in pattern::find(tokens, entry.pattern) {
            let surfaces: Vec<&str> = found
                .tokens
                .iter()
                .map(|i| i.map_or("", |i| tokens[i].surface))
                .collect();
            let expression = surfaces.concat();
            if allows.iter().any(|allow| allow.is_match(&expression)) {
                continue;
            }
            let first = &tokens[found.tokens[0].expect("辞書の並びの先頭はskippableでない")];
            let start = sentence.source(first).start;
            let in_allowed_node = sentence.inside(start, |kind| {
                rule.allow_node_types
                    .iter()
                    .any(|&node_type| is_node_type(kind, node_type))
            });
            if in_allowed_node {
                continue;
            }
            reports.push(sentence.report(
                first,
                RuleId::JaNoRedundantExpression,
                format!(
                    "冗長な表現「{expression}」({}): {}",
                    entry.id,
                    suggestion(entry, &surfaces)
                ),
            ));
        }
    }
    reports
}

/// 辞書の項目ごとの、直し方の文面。`surfaces` は並びの期待するトークンごとの表層で、飛ばしたものは空である
fn suggestion(entry: &Redundant, surfaces: &[&str]) -> String {
    match entry.id {
        // 「する」「こと」「が」と、続く読点
        "dict1" | "dict2" => format!("「{}」を省ける", surfaces[..4].concat()),
        // 4つ目の期待するトークンが「と」である
        "dict3" | "dict4" => format!("「である」か「{}」を省ける", surfaces[3..].concat()),
        _ => format!("「{}する」などにできる", surfaces[0]),
    }
}

/// インラインの要素の型が、`allowNodeTypes` に書ける型の `node_type` か
fn is_node_type(kind: &InlineKind, node_type: NodeType) -> bool {
    matches!(
        (kind, node_type),
        (InlineKind::Link { .. }, NodeType::Link)
            | (InlineKind::LinkReference, NodeType::LinkReference)
            | (InlineKind::Image, NodeType::Image)
            | (InlineKind::Emphasis, NodeType::Emphasis)
            | (InlineKind::Strong, NodeType::Strong)
            | (InlineKind::Delete, NodeType::Delete)
    )
}
