//! `spelling` (textlint-rule-spelling 0.3.0)。本文の範囲と画像のaltから取り出した英単語を、`dictionary-en` 4.0.0 の
//! en_USのhunspellの辞書で `spellbook` に照合し、辞書に無い語を指摘する。textlint-rule-spellingのコードはGPLなので写さず、
//! 判定の条件から書き直した

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::{LazyLock, Mutex};

use foldhash::fast::FixedState;
use spellbook::Dictionary;

use crate::prose::Prose;
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

/// 語から、`、` でつないだ候補 (無ければ空)。`suggest` は1語に数十msかかるので、同じプロセスの中で同じ語には1回だけ求める
static SUGGESTIONS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(Mutex::default);

/// `texts` と `alts` の元のソースに `wordDefinitionRegexp` を当てて語を取り出し、既定の除外 (ASCIIの数字か `_` を含む語、
/// 2文字目以降に `A`〜`Z` を含む語)、`skipPatterns` の一致と重なる語、文字参照の名前を除き、辞書に無い語を指摘する。範囲は語の全体である。
/// `suggestCorrections` が真なら、指摘にする語にだけ候補を求め、同じプロセスの中で同じ語の候補を再利用する
pub fn report(src: &str, texts: &[&Prose], alts: &[Range<usize>], rule: &Spelling) -> Vec<Report> {
    let mut reports = Vec::new();
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
                || is_identifier(word)
                || in_character_reference(&references, &m.range())
                || skipped
                    .iter()
                    .any(|skip| m.start() <= skip.end && skip.start <= m.end())
                || DICTIONARY.check(word)
            {
                continue;
            }
            let found = one_line(word);
            let message = if rule.suggest_corrections {
                match suggestions(word).as_str() {
                    "" => format!("英語の辞書に無い語「{found}」を使っている。候補は無い"),
                    candidates => {
                        format!("英語の辞書に無い語「{found}」を使っている。候補: {candidates}")
                    }
                }
            } else {
                format!("英語の辞書に無い語「{found}」を使っている")
            };
            reports.push(Report {
                range: range.start + m.start()..range.start + m.end(),
                rule: RuleId::Spelling.name(),
                message: Cow::Owned(message),
            });
        }
    }
    reports
}

/// 綴りの誤りではなく、識別子・略語・固有名詞として書かれた語 (ASCIIの数字か `_` を含む語、2文字目以降に `A`〜`Z` を
/// 含む語) か
fn is_identifier(word: &str) -> bool {
    word.bytes().any(|b| b.is_ascii_digit() || b == b'_')
        || word.chars().skip(1).any(|c| c.is_ascii_uppercase())
}

fn suggestions(word: &str) -> String {
    let mut cache = SUGGESTIONS.lock().unwrap();
    if let Some(candidates) = cache.get(word) {
        return candidates.clone();
    }
    let mut candidates = Vec::new();
    DICTIONARY.suggest(word, &mut candidates);
    let joined = candidates.join("、");
    cache.insert(word.to_owned(), joined.clone());
    joined
}
