//! `common-misspellings` (textlint-rule-common-misspellings 1.0.1)。Wikipediaの誤綴りの一覧 (CC BY-SA 3.0) の誤綴りを
//! 本文の範囲に当て、正しい語を示して指摘する。textlint-rule-common-misspellingsと `misspellings` のコードと一覧はGPLなので
//! 写さず、判定の条件から書き直し、一覧はWikipediaのページの固定したrevisionから作った

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::js_regex::{self, Regex};
use crate::prose::Prose;
use crate::rules::builtin::CommonMisspellings;
use crate::rules::{Report, RuleId};

use super::{character_references, in_character_reference};

/// 先頭の `#` の行が出典とライセンスで、後は1行1項目の `<小文字の誤綴り>\t<正しい語>,<正しい語>…` である
static LIST: &str = include_str!("data/common-misspellings.txt");

/// 一覧を読んだもの。4,310件の選択の正規表現は読むのに時間がかかるので、ルールが有効で本文の範囲があるときに1回だけ読む
struct Misspellings {
    /// 一覧の誤綴りのすべての選択を語の境界で包んだ正規表現 (`i`・`u`)。長い誤綴りを先に並べ、同じ位置では最も長い一致を取る
    regex: Regex,
    /// 小文字の誤綴りから、カンマで区切った正しい語
    corrects: HashMap<&'static str, &'static str>,
}

static MISSPELLINGS: LazyLock<Misspellings> = LazyLock::new(|| {
    let entries: Vec<(&str, &str)> = LIST
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            line.split_once('\t')
                .expect("each line of the misspelling list has a tab")
        })
        .collect();
    let mut misspellings: Vec<&str> = entries
        .iter()
        .map(|&(misspelling, _)| misspelling)
        .collect();
    // 選択は前から試すので、長い順に並べると、同じ位置で語の境界で終わる最も長い一致になる。大文字と小文字を区別しない
    // 照合でも、一致の文字の数は誤綴りの文字の数と同じである
    misspellings.sort_by_key(|misspelling| Reverse(misspelling.chars().count()));
    let alternatives: Vec<String> = misspellings.iter().map(|m| js_regex::escape(m)).collect();
    let regex = js_regex::compile(
        &format!(r"\b(?:{})\b", alternatives.join("|")),
        js_regex::Flags {
            ignore_case: true,
            ..js_regex::Flags::default()
        },
    )
    .expect("the misspelling list compiles");
    Misspellings {
        regex,
        corrects: entries.into_iter().collect(),
    }
});

/// 引用の中にない本文の範囲に、一覧の誤綴りのすべてを選択にした `\b(?:…)\b` を大文字と小文字を区別せずに当てる。
/// 同じ位置では最も長い一致を取り、一致を小文字にして一覧を引き、`ignore` に無ければ、正しい語のすべてに一致の大文字を
/// 写して文面に含めた指摘にする。範囲は一致の全体である
pub fn report(src: &str, texts: &[Prose], rule: &CommonMisspellings) -> Vec<Report> {
    let mut reports = Vec::new();
    for prose in texts.iter().filter(|prose| !prose.context.blockquote) {
        let text = &src[prose.range.clone()];
        let references = character_references(text);
        for m in MISSPELLINGS.regex.find_iter(text) {
            if in_character_reference(&references, &m.range()) {
                continue;
            }
            let matched = &text[m.range()];
            let lower = matched.to_lowercase();
            // `i` と `u` の照合は `ſ` (U+017F) を `s` に当てるので、小文字にしても一覧に無い一致がありうる
            let Some(corrects) = MISSPELLINGS.corrects.get(lower.as_str()) else {
                continue;
            };
            if rule.ignore.contains(&lower) {
                continue;
            }
            let corrects: Vec<String> = corrects
                .split(',')
                .map(|correct| format!("「{}」", copy_case(matched, correct)))
                .collect();
            reports.push(Report {
                range: prose.range.start + m.start()..prose.range.start + m.end(),
                rule: RuleId::CommonMisspellings.name(),
                message: Cow::Owned(format!(
                    "よくある綴りの誤り「{matched}」を使っている。正しくは{}",
                    corrects.join("か")
                )),
            });
        }
    }
    reports
}

/// 正しい語の各位置の文字を、一致の同じ位置の文字が `A`〜`Z` で、正しい語の文字が `a`〜`z` なら大文字にする
fn copy_case(matched: &str, correct: &str) -> String {
    let mut matched = matched.chars();
    correct
        .chars()
        .map(|c| match matched.next() {
            Some(m) if m.is_ascii_uppercase() && c.is_ascii_lowercase() => c.to_ascii_uppercase(),
            _ => c,
        })
        .collect()
}
