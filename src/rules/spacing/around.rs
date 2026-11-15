//! インラインのnodeの前後に当てる4ルール。`ja-space-around-code` はインラインコード、`ja-space-around-link` は
//! リンク、`ja-space-around-strong` は太字、`ja-space-around-emphasis` は強調の前後の文字を見る

use std::borrow::Cow;
use std::ops::Range;

use crate::prose::{self, InlineNode, InlineNodeKind};

use super::super::builtin::{Builtin, JaSpaceAround};
use super::super::{Finding, FixRuleFinding, Report, RuleId};
use super::chars::{self, AROUND_NODE};

/// `node` に当てるルールと、そのオプションと、文面に使うnodeの名前。ルールが無効なら `None`
fn rule_of<'a>(
    node: &InlineNode,
    builtin: &'a Builtin,
) -> Option<(RuleId, &'a JaSpaceAround, &'static str)> {
    let (rule, options, name) = match node.kind {
        InlineNodeKind::Code => (
            RuleId::JaSpaceAroundCode,
            &builtin.ja_space_around_code,
            "インラインコード",
        ),
        InlineNodeKind::Link => (
            RuleId::JaSpaceAroundLink,
            &builtin.ja_space_around_link,
            "リンク",
        ),
        InlineNodeKind::Strong => (
            RuleId::JaSpaceAroundStrong,
            &builtin.ja_space_around_strong,
            "太字",
        ),
        InlineNodeKind::Emphasis => (
            RuleId::JaSpaceAroundEmphasis,
            &builtin.ja_space_around_emphasis,
            "強調",
        ),
    };
    Some((rule, options.as_ref()?, name))
}

fn is_japanese(c: char) -> bool {
    chars::contains(AROUND_NODE, c)
}

/// `before`・`after` が `false` の側の修正の箇所。nodeの前か後ろに文字が無い側は判定しない
pub fn find(src: &str, nodes: &[InlineNode], builtin: &Builtin) -> Vec<FixRuleFinding> {
    let mut findings = Vec::new();
    for node in nodes {
        let Some((rule, options, name)) = rule_of(node, builtin) else {
            continue;
        };
        let mut before = src[..node.range.start].chars().rev();
        let mut after = src[node.range.end..].chars();
        let spaces = [
            (!options.before
                && before.next() == Some(' ')
                && before.next().is_some_and(is_japanese))
            .then(|| (node.range.start - 1..node.range.start, "前")),
            (!options.after && after.next() == Some(' ') && after.next().is_some_and(is_japanese))
                .then(|| (node.range.end..node.range.end + 1, "後")),
        ];
        for (space, side) in spaces.into_iter().flatten() {
            if matches!(node.kind, InlineNodeKind::Strong | InlineNodeKind::Emphasis)
                && !keeps_emphasis(src, node, space.clone())
            {
                continue;
            }
            findings.push(FixRuleFinding {
                finding: Finding {
                    position: space.start,
                    range: space,
                    replacement: Cow::Borrowed(""),
                    rule,
                    message: Cow::Owned(format!("{name}の{side}の空白を削除する")),
                },
                heading: node.heading,
            });
        }
    }
    findings
}

/// `before`・`after` が `true` の側の指摘。位置は、nodeの前の文字か後ろの文字である
pub fn report(src: &str, nodes: &[InlineNode], builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    for node in nodes {
        let Some((rule, options, name)) = rule_of(node, builtin) else {
            continue;
        };
        let before = src[..node.range.start].chars().next_back();
        let after = src[node.range.end..].chars().next();
        let neighbors = [
            (
                options.before,
                before,
                node.range.start - before.map_or(0, char::len_utf8),
                "前",
            ),
            (options.after, after, node.range.end, "後"),
        ];
        for (demanded, neighbor, start, side) in neighbors {
            let Some(c) = neighbor.filter(|&c| demanded && is_japanese(c)) else {
                continue;
            };
            reports.push(Report {
                range: start..start + c.len_utf8(),
                rule: rule.name(),
                message: Cow::Owned(format!("{name}の{side}に半角空白を入れる")),
            });
        }
    }
    reports
}

/// 太字か強調の `node` の前か後ろの空白 (`space`、元のソース上のバイト範囲) を消しても、消した後の文字列を
/// Markdownとして解析し直して、同じ範囲が同じ種類のnodeのままか。区切りが `_` のときや、区切りの内側の文字が
/// 句読点か記号のときは、消すと強調として読まれなくなる
fn keeps_emphasis(src: &str, node: &InlineNode, space: Range<usize>) -> bool {
    let (left_end, right_start) = if space.end == node.range.start {
        (space.start, node.range.end)
    } else {
        (node.range.start, space.end)
    };
    let left = src[..left_end]
        .chars()
        .next_back()
        .map(String::from)
        .unwrap_or_default();
    let right = src[right_start..]
        .chars()
        .next()
        .map(String::from)
        .unwrap_or_default();
    let removed = format!("{left}{}{right}", &src[node.range.clone()]);
    let range = left.len()..left.len() + node.range.len();
    prose::parse(&removed)
        .inline_nodes
        .iter()
        .any(|parsed| parsed.kind == node.kind && parsed.range == range)
}
