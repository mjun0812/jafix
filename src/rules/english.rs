//! 英語の4つのルール (`terminology`・`spelling`・`common-misspellings`・`write-good`)。`terminology` の利用者の
//! `terms` の項目の箇所は利用者の置き換えで、ほかはすべて指摘である

mod common_misspellings;
mod spelling;
mod terminology;
pub mod terms;
mod word_list;
mod write_good;

use std::ops::Range;
use std::sync::LazyLock;
use std::thread;

use markdown::ParseOptions;
use regex::Regex;

use crate::prose::{Document, Prose};

use super::builtin::{Builtin, Terminology};
use super::{Report, UserReplacement};

/// 有効な4つのルールの指摘を返す。`texts` と `alts` は、対象行と重なる本文の範囲と、インラインの画像のaltの元のソース上の
/// バイト範囲である。`terminology` は `texts` に当たる組み込みの辞書の項目の箇所を、`spelling` は `texts` と `alts` の語を、
/// `common-misspellings` と `write-good` は文書の本文の範囲を見る。利用者の `terms` の項目の直さない箇所は、
/// `rules::report` が利用者の置き換えとして指摘にする。指摘は、`terminology`・`spelling`・`common-misspellings`・
/// `write-good` の順に並ぶ
pub fn report(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    texts: &[&Prose],
    alts: &[Range<usize>],
) -> Vec<Report> {
    // hookの時間の大半は `spelling` の辞書の読み込み (約5ms) と候補の計算 (1語に数〜十数ms) なので、threadを3か所で立てる。
    // `rules::report` が4つのルールを1つのthreadでほかのルールと並行に当て、ここで `spelling` を1つのthreadでほかの3つと
    // 並行に当て、`spelling::suggestions` がまだ求めていない語の候補を、語の数とCPUの数の小さいほうのthreadで求める
    thread::scope(|scope| {
        let spelling = builtin
            .spelling
            .as_ref()
            .map(|rule| scope.spawn(|| spelling::report(src, texts, alts, rule)));
        let terminology = builtin
            .terminology
            .as_ref()
            .map(|rule| terminology::report(src, texts, rule));
        let common_misspellings = builtin
            .common_misspellings
            .as_ref()
            .map(|rule| common_misspellings::report(src, &document.texts, rule));
        let write_good = builtin
            .write_good
            .as_ref()
            .map(|rule| write_good::report(src, &document.texts, rule));
        let spelling = spelling.map(|worker| worker.join().unwrap());
        [terminology, spelling, common_misspellings, write_good]
            .into_iter()
            .flatten()
            .flatten()
            .collect()
    })
}

/// 4つのルールのどれかが有効か
pub fn enabled(builtin: &Builtin) -> bool {
    builtin.terminology.is_some()
        || builtin.spelling.is_some()
        || builtin.common_misspellings.is_some()
        || builtin.write_good.is_some()
}

/// 辞書で語を別の書き方にするよう求める2つ (`terminology` の組み込みの辞書と `common-misspellings`) のうち、有効なものの
/// `prose` への指摘
pub fn dictionary_report(src: &str, prose: &Prose, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    if let Some(rule) = &builtin.terminology {
        reports.extend(terminology::report(src, &[prose], rule));
    }
    if let Some(rule) = &builtin.common_misspellings {
        reports.extend(common_misspellings::report(
            src,
            std::slice::from_ref(prose),
            rule,
        ));
    }
    reports
}

/// `terminology` の利用者の `terms` の項目の箇所を返す。重なりは `rules::replacements` が解く
pub fn replacements(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<UserReplacement> {
    terminology::replacements(src, texts, rule)
}

/// 文字参照になりうる `&` から `;` までの並び。名前付きの文字参照は実在の名前だけが文字参照になるので、この並びは候補である
static REFERENCE_CANDIDATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("&#?[0-9A-Za-z]+;").unwrap());

/// 本文の範囲の元のソース `text` の中の文字参照 (`&name;`・`&#数字;`・`&#x16進;`) の範囲。textlintは文字参照を解いた文字列を
/// 見るので、4つのルールは参照の名前 (`&nbsp;` の `nbsp`) を語として扱わない。CommonMarkでは実在しない名前の `&foo;` や
/// 桁の多すぎる数値は文字のまま残るので、候補をmarkdown-rsに解析させ、別の文字に解けたものだけを文字参照とする。
/// 直前に奇数個の `\` がある `&` はエスケープした文字なので、`\&nbsp;` は文字参照ではない
fn character_references(text: &str) -> Vec<Range<usize>> {
    REFERENCE_CANDIDATE
        .find_iter(text)
        .filter(|candidate| {
            text[..candidate.start()]
                .bytes()
                .rev()
                .take_while(|&b| b == b'\\')
                .count()
                % 2
                == 0
        })
        .filter(|candidate| {
            markdown::to_mdast(candidate.as_str(), &ParseOptions::default())
                .expect("MDXを有効にしない解析は失敗しない")
                .to_string()
                != candidate.as_str()
        })
        .map(|candidate| candidate.range())
        .collect()
}

/// `range` が `references` の文字参照のどれかと重なるか。端が接するだけ (`word&nbsp;`) なら重ならない
fn in_character_reference(references: &[Range<usize>], range: &Range<usize>) -> bool {
    references
        .iter()
        .any(|reference| range.start < reference.end && reference.start < range.end)
}
