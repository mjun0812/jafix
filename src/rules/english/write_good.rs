//! `write-good` (textlint-rule-write-good 2.0.0、write-good 1.0.8)。英語の弱い語・繰り返し・受動態・冗長な句などの
//! 9つの検査を、段落の直下の本文の範囲に当てて指摘する

mod words;

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use crate::js_regex::{self, Regex};
use crate::prose::Prose;
use crate::rules::builtin::{WriteGood, WriteGoodCheck};
use crate::rules::{Report, RuleId, one_line};

use super::word_list::{self, Token, WordList};
use super::{character_references, in_character_reference};

/// `Context::plain` の本文の範囲に有効な検査を当て、`whitelist` の文字列のすべての出現と重なる提案を外し、範囲が同じ提案を
/// 1件にまとめて、文面を検査の順につなぐ。範囲は提案の範囲 (当たった文字列の全体) である
pub fn report(src: &str, texts: &[Prose], rule: &WriteGood) -> Vec<Report> {
    let mut reports = Vec::new();
    for prose in texts.iter().filter(|prose| prose.context.plain) {
        let text = &src[prose.range.clone()];
        let allowed = whitelisted(text, &rule.whitelist);
        let references = character_references(text);
        let mut suggestions: Vec<(Range<usize>, Vec<WriteGoodCheck>)> = Vec::new();
        for &check in &rule.checks {
            for range in suggest(check, text) {
                if allowed.iter().any(|allowed| overlaps(&range, allowed))
                    || in_character_reference(&references, &range)
                {
                    continue;
                }
                match suggestions.iter_mut().find(|(other, _)| *other == range) {
                    Some((_, checks)) => checks.push(check),
                    None => suggestions.push((range, vec![check])),
                }
            }
        }
        reports.extend(suggestions.into_iter().map(|(range, checks)| {
            let explanations: Vec<&str> = checks.iter().map(|&check| explanation(check)).collect();
            let names: Vec<&str> = checks.iter().map(|check| check.name()).collect();
            Report {
                message: Cow::Owned(format!(
                    "「{}」は{}である({})",
                    one_line(&text[range.clone()]),
                    explanations.join("、"),
                    names.join("・")
                )),
                range: prose.range.start + range.start..prose.range.start + range.end,
                rule: RuleId::WriteGood.name(),
            }
        }));
    }
    reports
}

/// `text` の中の `whitelist` の各文字列のすべての出現 (重なるものを含む) の範囲
fn whitelisted(text: &str, whitelist: &[String]) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    for allowed in whitelist {
        let mut from = 0;
        while let Some(index) = text[from..].find(allowed.as_str()) {
            let start = from + index;
            found.push(start..start + allowed.len());
            from = start + text[start..].chars().next().map_or(1, char::len_utf8);
        }
    }
    found
}

/// 提案 `suggestion` が出現 `allowed` を含むか、提案の始まりか終わりが出現の範囲 (端を含む) にあるか
fn overlaps(suggestion: &Range<usize>, allowed: &Range<usize>) -> bool {
    suggestion.start <= allowed.start && allowed.end <= suggestion.end
        || (allowed.start..=allowed.end).contains(&suggestion.start)
        || (allowed.start..=allowed.end).contains(&suggestion.end)
}

/// `check` の提案の、`text` の中のバイト範囲。位置の順に並ぶ
fn suggest(check: WriteGoodCheck, text: &str) -> Vec<Range<usize>> {
    match check {
        WriteGoodCheck::Weasel => WEASEL
            .find_iter(text)
            .filter(|m| {
                let lower = text[m.range()].to_lowercase();
                !((lower == "many" || lower == "few") && preceded_by_too(&text[..m.start()]))
            })
            .map(|m| m.range())
            .collect(),
        WriteGoodCheck::Illusion => {
            let mut last = String::new();
            let mut found = Vec::new();
            for m in WORDS.find_iter(text) {
                let range = m.group(2).expect("the word group always takes part");
                let word = &text[range.clone()];
                let lower = word.to_lowercase();
                if lower == last && word.bytes().any(|b| b.is_ascii_alphanumeric() || b == b'_') {
                    found.push(range);
                }
                last = lower;
            }
            found
        }
        WriteGoodCheck::So => sentence_starts(&SO, text),
        WriteGoodCheck::ThereIs => sentence_starts(&THERE_IS, text),
        WriteGoodCheck::Passive => PASSIVE
            .find_iter(text)
            .filter(|m| {
                let participle = m.group(2).expect("the participle group always takes part");
                text[participle].to_lowercase() != "indeed"
            })
            .map(|m| m.range())
            .collect(),
        WriteGoodCheck::Adverb => ADVERB.find_iter(text),
        WriteGoodCheck::TooWordy => TOO_WORDY.find_iter(text),
        WriteGoodCheck::Cliches => CLICHES.find_iter(text),
        WriteGoodCheck::Eprime => E_PRIME.find_iter(text).map(|m| m.range()).collect(),
    }
}

/// 一致の直前の4文字 (本文の範囲の先頭から4文字に足りなければ、ある分) が `too ` か
fn preceded_by_too(before: &str) -> bool {
    let start = before
        .char_indices()
        .rev()
        .nth(3)
        .map_or(0, |(index, _)| index);
    &before[start..] == "too "
}

/// 文ごとに、文の先頭に `regex` を当て、2つ目のグループ (文頭の空白を飛ばした語) の範囲を返す。文は、`[^\n.;!?]+` と
/// その後ろに続く `[.;!?]+` で、終わりの記号の無い最後の `[^\n.;!?]+` も1文とする
fn sentence_starts(regex: &Regex, text: &str) -> Vec<Range<usize>> {
    SENTENCES
        .find_iter(text)
        .filter_map(|sentence| {
            let body = sentence
                .group(1)
                .expect("the sentence body always takes part");
            let m = regex.find(&text[body.clone()])?;
            let word = m.group(2).expect("the word group always takes part");
            Some(body.start + word.start..body.start + word.end)
        })
        .collect()
}

/// write-goodの各packageの正規表現と同じ形で、`i` を付けて読む
fn compile(source: &str) -> Regex {
    js_regex::compile(
        source,
        js_regex::Flags {
            ignore_case: true,
            ..js_regex::Flags::default()
        },
    )
    .expect("the write-good patterns compile")
}

static WEASEL: LazyLock<Regex> =
    LazyLock::new(|| compile(&format!(r"\b({})\b", words::WEASELS.join("|"))));

static WORDS: LazyLock<Regex> = LazyLock::new(|| compile(r"(\s*)([^\s]+)"));

static SENTENCES: LazyLock<Regex> = LazyLock::new(|| compile(r"([^\n.;!?]+)(?:[.;!?]+|$)"));

static SO: LazyLock<Regex> = LazyLock::new(|| compile(r"^(\s*)(so)\b[\s\S]"));

static THERE_IS: LazyLock<Regex> = LazyLock::new(|| compile(r"^(\s*)(there\b\s(?:is|are)\b)"));

static PASSIVE: LazyLock<Regex> = LazyLock::new(|| {
    compile(&format!(
        r"\b(am|are|were|being|is|been|was|be)\b\s*([\w]+ed|{})\b",
        words::PASSIVE_IRREGULARS.join("|")
    ))
});

/// adverb-whereの `\b((<adverbs>)(y)|(<weakens>))\b`
static ADVERB: LazyLock<WordList> = LazyLock::new(|| {
    let adverbs = words::ADVERBS.iter().map(|adverb| {
        let mut tokens = word_list::literal(adverb);
        tokens.push(Token::Char('y'));
        tokens
    });
    WordList::new(
        adverbs.chain(
            words::WEAKENS
                .iter()
                .map(|weaken| word_list::literal(weaken)),
        ),
        false,
    )
});

/// too-wordyの `\b(?<!-)(<句>)\b`。句の中の空白は `[\b\s‌]*` である (文字クラスの中の `\b` はバックスペース)
static TOO_WORDY: LazyLock<WordList> = LazyLock::new(|| {
    WordList::new(
        words::TOO_WORDY
            .iter()
            .map(|phrase| word_list::phrase(phrase)),
        true,
    )
});

/// no-clichesの `\b(<句>)\b`。句の中の空白は `TOO_WORDY` と同じである
static CLICHES: LazyLock<WordList> = LazyLock::new(|| {
    WordList::new(
        words::CLICHES
            .iter()
            .map(|phrase| word_list::phrase(phrase)),
        false,
    )
});

/// `‘` と `’` も `'` として当てる
static E_PRIME: LazyLock<Regex> = LazyLock::new(|| {
    compile(&format!(
        r"\b({})\b",
        words::E_PRIME.join("|").replace('\'', "['‘’]")
    ))
});

/// 文面の `「<一致>」は<説明>である(<検査名>)` の説明
fn explanation(check: WriteGoodCheck) -> &'static str {
    match check {
        WriteGoodCheck::Weasel => "曖昧な語",
        WriteGoodCheck::Illusion => "直前の語の繰り返し",
        WriteGoodCheck::So => "意味を足さない文頭の語",
        WriteGoodCheck::ThereIs => "冗長な文頭の句",
        WriteGoodCheck::Passive => "受動態かもしれない句",
        WriteGoodCheck::Adverb => "意味を弱める語",
        WriteGoodCheck::TooWordy => "冗長か不要な句",
        WriteGoodCheck::Cliches => "決まり文句",
        WriteGoodCheck::Eprime => "be動詞の形",
    }
}
