use std::sync::LazyLock;

use regex::Regex;

use crate::rules::{Report, RuleId};

use super::AnalyzedProse;
use super::dict::{ABUSAGE_WORDS, ABUSAGES};
use super::pattern;

static WORDS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    ABUSAGE_WORDS
        .iter()
        .map(|word| Regex::new(word.pattern).expect("辞書の正規表現は読める"))
        .collect()
});

/// 誤用の文字列とトークンの並びに一致した箇所に指摘を出す
pub(super) fn report(prose: &AnalyzedProse) -> Vec<Report> {
    let mut reports = phrase_reports(prose);
    if !prose.in_quote_emphasis_or_link() {
        reports.extend(word_reports(prose));
    }
    reports
}

/// トークンの並びに一致した箇所ごとに、一致の最初のトークンに指摘を出す
fn phrase_reports(prose: &AnalyzedProse) -> Vec<Report> {
    let tokens = prose.tokens;
    ABUSAGES
        .iter()
        .flat_map(|phrase| {
            pattern::find(tokens, phrase.pattern)
                .into_iter()
                .map(move |found| (phrase, found))
        })
        .map(|(phrase, found)| {
            let first = &tokens[found.tokens[0].expect("誤用の並びにskippableは無い")];
            let message = match phrase.name {
                "を適応" => "誤用の可能性がある: 「適応」は「適用」と書く".to_owned(),
                "可変する" => "「可変する」という使い方は適切でない".to_owned(),
                "ずらい" => format!(
                    "誤用の可能性がある: 「{verb}ずらい」は「{verb}づらい」と書く",
                    verb = first.surface
                ),
                name => unreachable!("誤用の並びの名前ではない: {name}"),
            };
            prose.report(first.range.clone(), RuleId::JaNoAbusage, message)
        })
        .collect()
}

/// 73件の文字列の一致を合わせ、重なる一致は先頭に近いものだけを残して指摘を出す
fn word_reports(prose: &AnalyzedProse) -> Vec<Report> {
    let mut found: Vec<_> = WORDS
        .iter()
        .zip(&ABUSAGE_WORDS)
        .flat_map(|(regex, word)| {
            regex
                .captures_iter(prose.text)
                .map(move |captures| (captures, word))
        })
        .collect();
    // 先頭が同じ一致は、辞書の順に残す
    found.sort_by_key(|(captures, _)| captures.get_match().start());
    let mut reports = Vec::new();
    let mut end = 0;
    for (captures, word) in found {
        let whole = captures.get_match();
        if whole.start() < end {
            continue;
        }
        end = whole.end();
        // 一致しなかったグループは空の文字列になる
        let mut expected = String::new();
        captures.expand(word.expected, &mut expected);
        reports.push(prose.report(
            whole.range(),
            RuleId::JaNoAbusage,
            format!(
                "誤用の可能性がある: 「{}」は「{expected}」と書く",
                whole.as_str()
            ),
        ));
    }
    reports
}
