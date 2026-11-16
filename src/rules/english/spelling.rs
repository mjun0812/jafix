//! `spelling` (textlint-rule-spelling 0.3.0)。本文の範囲と画像のaltから取り出した英単語を、`dictionary-en` 4.0.0 の
//! en_USのhunspellの辞書で `spellbook` に照合し、辞書に無い語を指摘する。textlint-rule-spellingのコードはGPLなので写さず、
//! 判定の条件から書き直した

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::num::NonZero;
use std::ops::Range;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::thread;

use foldhash::fast::FixedState;
use spellbook::Dictionary;

use crate::prose::{LinkDestination, Prose};
use crate::rules::builtin::Spelling;
use crate::rules::{Report, RuleId, one_line};

use super::{character_references, in_character_reference};

/// `dictionary-en` 4.0.0 の `index.aff` と `index.dic` を、中身を変えずに写したもの
static AFF: &str = include_str!("data/en_US.aff");
static DIC: &str = include_str!("data/en_US.dic");

/// 辞書の語の表のhasherの種。変えると `suggest` の候補の並びが変わりうる
const SEED: u64 = 0;

/// 照合する語が最初に来たときに1回だけ読む辞書。`Dictionary::new` はプロセスごとに種の変わるhasherで語の表を作り、
/// `suggest` の候補の並びが実行ごとに変わるので、固定の種の `FixedState` で作る
static DICTIONARY: LazyLock<Dictionary<FixedState>> = LazyLock::new(|| {
    Dictionary::new_with_hasher(AFF, DIC, FixedState::with_seed(SEED))
        .expect("the embedded en_US dictionary parses")
});

/// 語から、`、` でつないだ候補 (無ければ空)。同じプロセスの中で同じ語には1回だけ求める
static SUGGESTIONS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(Mutex::default);

/// `texts` と `alts` の元のソースに `wordDefinitionRegexp` を当てて語を取り出し、既定の除外 (`is_identifier`)、
/// `skipPatterns` の一致と重なる語、文字参照の名前を除き、辞書に無い語を指摘する。範囲は語の全体である。
/// `suggestCorrections` が真なら、指摘にする語にだけ候補を求め、同じプロセスの中で同じ語の候補を再利用する
pub fn report(
    src: &str,
    texts: &[&Prose],
    alts: &[Range<usize>],
    links: &[LinkDestination],
    rule: &Spelling,
) -> Vec<Report> {
    let repositories: HashSet<&str> = links
        .iter()
        .filter_map(|link| {
            let path = link.url.strip_prefix("https://github.com/")?;
            let owner_end = path.find('/')?;
            let end = path[owner_end + 1..]
                .find(['/', '?', '#'])
                .map_or(path.len(), |i| owner_end + 1 + i);
            Some(path[..end].strip_suffix(".git").unwrap_or(&path[..end]))
        })
        .collect();
    let mut misspelled = Vec::new();
    let mut known = HashMap::new();
    for range in texts.iter().map(|prose| &prose.range).chain(alts) {
        let text = &src[range.clone()];
        let skipped: Vec<Range<usize>> = rule
            .skip_patterns
            .iter()
            .flat_map(|pattern| pattern.find_iter(text).map(|m| m.range()))
            .collect();
        let references = character_references(text);
        for m in rule.word_definition.find_iter(text) {
            let word = &text[m.range()];
            if word.is_empty()
                || is_identifier(text, m.range())
                || in_character_reference(&references, &m.range())
                || skipped
                    .iter()
                    .any(|skip| m.start() <= skip.end && skip.start <= m.end())
                || in_name(text, m.range(), &repositories, &mut known)
                || *known.entry(word).or_insert_with(|| DICTIONARY.check(word))
            {
                continue;
            }
            misspelled.push((range.start + m.start()..range.start + m.end(), word));
        }
    }
    let suggestions = rule
        .suggest_corrections
        .then(|| suggestions(misspelled.iter().map(|&(_, word)| word)));
    misspelled
        .into_iter()
        .map(|(range, word)| {
            let found = one_line(word);
            let message = match suggestions
                .as_ref()
                .map(|suggestions| suggestions[word].as_str())
            {
                None => format!("英語の辞書に無い語「{found}」を使っている"),
                Some("") => format!("英語の辞書に無い語「{found}」を使っている。候補は無い"),
                Some(candidates) => {
                    format!("英語の辞書に無い語「{found}」を使っている。候補: {candidates}")
                }
            };
            Report {
                range,
                rule: RuleId::Spelling.name(),
                message: Cow::Owned(message),
            }
        })
        .collect()
}

/// `text` の `range` の語が、綴りの誤りではなく、識別子・略語・固有名詞として書かれた語か。ASCIIの数字か `_` を
/// 含む語、2文字目以降に `A`〜`Z` を含む語、`\_` に接する語である
fn is_identifier(text: &str, range: Range<usize>) -> bool {
    let word = &text[range.clone()];
    word.bytes().any(|b| b.is_ascii_digit() || b == b'_')
        || word.chars().skip(1).any(|c| c.is_ascii_uppercase())
        // 描画では `\_` が `_` になるので、`mytoken\_value` の `mytoken` も `_` を含む語の一部である
        || text[..range.start].ends_with("\\_")
        || text[range.end..].starts_with("\\_")
}

/// ファイル名・ドメイン名と、パスや名前として書かれたスラッシュの並びを外す。
/// 普通の語がほかの要素にあれば列挙とみなすが、パスの明示や既知のディレクトリ・名前空間を優先する。
fn in_name<'t>(
    text: &'t str,
    range: Range<usize>,
    repositories: &HashSet<&str>,
    known: &mut HashMap<&'t str, bool>,
) -> bool {
    let bytes = text.as_bytes();
    let name_byte = |b: &u8| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/');
    let start = bytes[..range.start]
        .iter()
        .rposition(|b| !name_byte(b))
        .map_or(0, |i| i + 1);
    let end = bytes[range.end..]
        .iter()
        .position(|b| !name_byte(b))
        .map_or(text.len(), |i| range.end + i);
    let run = &text[start..end];
    if run.as_bytes().windows(2).enumerate().any(|(i, pair)| {
        pair[0] == b'.'
            && pair[1].is_ascii_alphanumeric()
            && (i == 0 || run.as_bytes()[i - 1] != b'.')
    }) {
        return true;
    }
    if !run.contains('/') {
        return false;
    }
    if run.starts_with(['/', '.'])
        || run.ends_with('/')
        || repositories.contains(run.trim_end_matches('.'))
    {
        return true;
    }
    let first = run.split('/').next().unwrap();
    if matches!(
        first,
        "owner"
            | "src"
            | "lib"
            | "bin"
            | "etc"
            | "usr"
            | "var"
            | "tmp"
            | "home"
            | "docs"
            | "topics"
    ) {
        return true;
    }
    let parts: Vec<&str> = run.trim_end_matches('.').split('/').collect();
    if parts.iter().any(|part| {
        if is_identifier(part, 0..part.len())
            || part.ends_with("fs")
            || matches!(*part, "stdin" | "stdout" | "stderr")
        {
            return true;
        }
        let valid = *known.entry(part).or_insert_with(|| DICTIONARY.check(part));
        !valid
            && (part.contains('-')
                || part.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                || (part.len() <= 3 && !super::common_misspellings::is_known_misspelling(part)))
    }) {
        return true;
    }
    // 先頭が既知の語や誤綴りでない2要素はowner/repoとして扱う。3要素以上は、ほかに辞書の語があれば見る。
    // 辞書はreport内で語ごとに1回だけ照合し、候補はこの判定に使わない。
    if *known
        .entry(first)
        .or_insert_with(|| DICTIONARY.check(first))
        || super::common_misspellings::is_known_misspelling(first)
    {
        return false;
    }
    parts.len() == 2
        || !parts
            .iter()
            .any(|part| *known.entry(part).or_insert_with(|| DICTIONARY.check(part)))
}

/// `words` のすべての語の候補を入れた `SUGGESTIONS` を返す。`suggest` は1語に数ms〜十数msかかるので、まだ求めていない語は
/// 語ごとに複数のthreadで並列に求める
fn suggestions<'w>(
    words: impl Iterator<Item = &'w str>,
) -> MutexGuard<'static, HashMap<String, String>> {
    let mut cache = SUGGESTIONS.lock().unwrap();
    let mut missing: Vec<&str> = words.filter(|word| !cache.contains_key(*word)).collect();
    missing.sort_unstable();
    missing.dedup();
    let threads = thread::available_parallelism()
        .map_or(1, NonZero::get)
        .min(missing.len());
    let queue = Mutex::new(missing.into_iter());
    thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut found = Vec::new();
                    let mut candidates = Vec::new();
                    loop {
                        // `while let` の条件に書くと、取り出した語の候補を求める間もlockを持ち続ける
                        let next = queue.lock().unwrap().next();
                        let Some(word) = next else {
                            return found;
                        };
                        DICTIONARY.suggest(word, &mut candidates);
                        found.push((word.to_owned(), candidates.join("、")));
                    }
                })
            })
            .collect();
        for worker in workers {
            cache.extend(worker.join().unwrap());
        }
    });
    cache
}
