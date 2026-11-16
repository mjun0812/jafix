//! `terminology` (textlint-rule-terminology 5.2.16)。利用者の `terms` の項目と組み込みの辞書の項目を同じ照合で当て、
//! 利用者の項目の箇所を利用者の置き換えに、組み込みの辞書の項目の箇所を指摘にする

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use crate::js_regex::{self, Regex};
use crate::prose::Prose;
use crate::rules::builtin::{PairTerm, Terminology};
use crate::rules::textlint_rule_pattern::contains;
use crate::rules::{Finding, Report, RuleId, UserReplacement, one_line};

use super::word_list;
use super::{character_references, in_character_reference};

/// 項目が当たり、置き換え後の語が一致と違う1箇所
struct TermMatch {
    /// 元のソース上のバイト範囲。一致の全体である
    range: Range<usize>,
    /// 置き換え後の語
    replacement: String,
    /// 置き換え後の語を決めた項目が、利用者の `terms` の項目か
    user: bool,
    /// 見出しの中の本文の範囲の箇所
    heading: bool,
}

/// 本文の範囲のうち `skip` の型の中にないものに、文字列の項目をまとめた正規表現、配列の項目の正規表現の順に、
/// 1つずつ重ならない一致をすべて探し、置き換え後の語が一致と違う箇所を返す。本文の範囲ごとに、正規表現の順、位置の順に並ぶ。
/// 置き換え後の語を求められない文字列の項目の一致と、文字参照と重なる一致は捨てる。範囲と置き換え後の語が同じ一致は、最初の1件にまとめ、
/// どれかが利用者の項目の一致なら利用者の一致にする
fn matches(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<TermMatch> {
    let mut found = Vec::new();
    for prose in texts {
        if rule
            .skip
            .iter()
            .any(|&node_type| contains(node_type, &prose.context))
        {
            continue;
        }
        let text = &src[prose.range.clone()];
        let references = character_references(text);
        let first = found.len();
        let words = rule
            .words
            .iter()
            .flat_map(|regex| regex.find_iter(text))
            .filter_map(|m| {
                let lower = text[m.range()].to_lowercase();
                let term = rule
                    .word_terms
                    .iter()
                    .find(|term| term.term.to_lowercase() == lower)?;
                Some((m.range(), term.term.to_string(), term.user))
            });
        // 一致の先頭の文字列が本文の範囲に無い項目は、正規表現を読まずに飛ばす
        let folded: String = text.chars().map(word_list::fold).collect();
        let pairs = rule
            .pair_terms
            .iter()
            .filter(|term| {
                term.prefix
                    .as_ref()
                    .is_none_or(|prefix| folded.contains(prefix.as_str()))
            })
            .flat_map(|term| {
                term.pattern().find_iter(text).map(move |m| {
                    (
                        m.range(),
                        pair_replacement(term, &text[m.range()]),
                        term.user,
                    )
                })
            });
        for (range, replacement, user) in words.chain(pairs) {
            // 文字参照の名前を書き換えると、参照が別の文字列になる
            if in_character_reference(&references, &range) {
                continue;
            }
            let matched = &text[range.clone()];
            let replacement =
                if sentence_start(&text[..range.start]) && upper_first(matched) == matched {
                    upper_first(&replacement)
                } else {
                    replacement
                };
            if replacement == matched {
                continue;
            }
            let range = prose.range.start + range.start..prose.range.start + range.end;
            // 利用者が組み込みの辞書と同じ表記を書くと、同じ箇所が修正と指摘の両方に出るので、修正の1件にする
            if let Some(same) = found[first..]
                .iter_mut()
                .find(|m: &&mut TermMatch| m.range == range && m.replacement == replacement)
            {
                same.user |= user;
                continue;
            }
            found.push(TermMatch {
                range,
                replacement,
                user,
                heading: prose.context.heading,
            });
        }
    }
    found
}

/// 配列の項目の置き換え後の語。`xyz <一致> xyz` に包まない1つ目の要素を1回だけ当てて2つ目の要素で置き換え、
/// 前後の `xyz ` を除く。先読みと後読みが一致の外を見るので、一致だけでなく前後に語を足した文字列に当てる
fn pair_replacement(term: &PairTerm, matched: &str) -> String {
    let text = format!("xyz {matched} xyz");
    let replaced = match term.source().find(&text) {
        Some(m) => format!(
            "{}{}{}",
            &text[..m.start()],
            js_regex::substitute(&term.replacement, &m, &text),
            &text[m.end()..]
        ),
        None => text,
    };
    let replaced = replaced.strip_prefix("xyz ").unwrap_or(&replaced);
    replaced.strip_suffix(" xyz").unwrap_or(replaced).to_owned()
}

/// 一致の手前の文字列 `before` から、一致が文頭にあるか。本文の範囲の先頭か、`\w+[!.?]\)? $` で終わるときである
fn sentence_start(before: &str) -> bool {
    static SENTENCE_END: LazyLock<Regex> = LazyLock::new(|| {
        js_regex::compile(r"\w+[!.?]\)? $", js_regex::Flags::default())
            .expect("the sentence end pattern compiles")
    });
    before.is_empty() || SENTENCE_END.find(before).is_some()
}

/// 先頭の文字を大文字にした文字列
fn upper_first(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// 利用者の `terms` の項目の箇所を、利用者の置き換えとして返す
pub fn replacements(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<UserReplacement> {
    matches(src, texts, rule)
        .into_iter()
        .filter(|m| m.user)
        .map(|m| UserReplacement {
            finding: Finding {
                range: m.range.clone(),
                message: Cow::Owned(message(&src[m.range.clone()], &m.replacement)),
                replacement: Cow::Owned(m.replacement),
                rule: RuleId::Terminology,
                position: m.range.start,
            },
            heading: m.heading,
        })
        .collect()
}

/// 組み込みの辞書の項目の箇所を、指摘として返す
pub fn report(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<Report> {
    matches(src, texts, rule)
        .into_iter()
        .filter(|m| !m.user)
        .map(|m| Report {
            message: Cow::Owned(message(&src[m.range.clone()], &m.replacement)),
            range: m.range,
            rule: RuleId::Terminology.name(),
        })
        .collect()
}

/// `prh` と同じ `「<一致した文字列>」を「<置き換え後>」にする` の文面
fn message(matched: &str, replacement: &str) -> String {
    format!(
        "「{}」を「{}」にする",
        one_line(matched),
        one_line(replacement)
    )
}
