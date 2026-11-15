//! `@textlint-ja/no-synonyms`。文書の全体で、同じ同義語の集まりの別の見出しが2つ以上使われたら指摘する。
//! 同義語辞書は `no_synonyms/sudachi-synonyms.tsv` (sudachi-synonyms-dictionary 20.0.0 から作ったもの) で、
//! 見出しで引く表は `build.rs` が作る

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;

use super::AnalyzedProse;
use crate::rules::builtin::NoSynonyms;
use crate::rules::{Report, RuleId};

/// 1行が1つの見出しで、`見出し\tグループの番号,語彙番号,英字の印;…`。見出しのバイト列の順に並ぶ
static HEADWORDS: &str = include_str!(concat!(env!("OUT_DIR"), "/synonyms-headwords.tsv"));

/// 見出しのFNV-1a (32bit) の下位ビットで位置を決めた開番地法の表。リトルエンディアンの `u32` の並びで、値は
/// `HEADWORDS` の行の先頭のバイト位置に1を足したもの、空きは0である
static TABLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/synonyms-table.bin"));

/// 見出しに当てる、空白をまたがずにつなげたトークンの並びの長さの上限
const MAX_TOKENS: usize = 8;

/// 32bitのFNV-1aの初期値
const FNV_OFFSET_BASIS: u32 = 0x811c_9dc5;

/// 32bitのFNV-1aの乗数
const FNV_PRIME: u32 = 0x0100_0193;

/// 文書の中で使われた見出しの出現。本文の範囲の繰り返しで集め、繰り返しの後に `report` で指摘にする
#[derive(Default)]
pub struct Occurrences {
    /// 見出しと、その値 (`HEADWORDS` の行のタブの後) と、出現の元のソースでの範囲。出現の順に並ぶ
    found: Vec<(&'static str, &'static str, Range<usize>)>,
}

/// 同義語の集まり。グループの番号と、語彙番号ごとに分けるときは語彙番号 (分けないときと、語彙番号の無い項目は空)
type Collection = (&'static str, &'static str);

/// 文書の中で使われた見出し
struct Used {
    headword: &'static str,
    value: &'static str,
    /// 文書の中で最後に出た元のソースでの範囲
    last: Range<usize>,
}

impl Occurrences {
    /// `prose` が引用の中でもリンクの文字列でもなければ、トークンの最長一致で語を区切り、見出しに当たる語の出現を足す
    pub fn collect(&mut self, prose: &AnalyzedProse) {
        if prose.context.blockquote || prose.context.link {
            return;
        }
        let tokens = prose.tokens;
        let mut i = 0;
        while i < tokens.len() {
            let start = tokens[i].range.start;
            let mut end = start;
            let mut hash = FNV_OFFSET_BASIS;
            let mut longest = None;
            for (j, token) in tokens[i..].iter().enumerate().take(MAX_TOKENS) {
                if token.pos_detail_1 == "空白" {
                    break;
                }
                hash = fnv1a(hash, &prose.text.as_bytes()[end..token.range.end]);
                end = token.range.end;
                if let Some((headword, value)) = lookup(&prose.text[start..end], hash) {
                    longest = Some((i + j, headword, value));
                }
            }
            let Some((last, headword, value)) = longest else {
                i += 1;
                continue;
            };
            let range = prose.start + start..prose.start + tokens[last].range.end;
            self.found.push((headword, value, range));
            i = last + 1;
        }
    }

    /// 集めた出現を、`rule` のオプションで同義語の集まりごとにまとめ、指摘を返す
    pub fn report(self, rule: &NoSynonyms) -> Vec<Report> {
        let mut used: Vec<Used> = Vec::new();
        let mut first_seen: HashMap<&str, usize> = HashMap::new();
        for (headword, value, range) in self.found {
            match first_seen.get(headword) {
                Some(&i) => used[i].last = range,
                None => {
                    first_seen.insert(headword, used.len());
                    used.push(Used {
                        headword,
                        value,
                        last: range,
                    });
                }
            }
        }
        // 集まりごとの、使われた項目 (`used` の番号を最初に出た順に)
        let mut collections: BTreeMap<Collection, Vec<usize>> = BTreeMap::new();
        for (i, item) in used.iter().enumerate() {
            if rule.allows.iter().any(|allow| allow == item.headword)
                || (rule.allow_number && is_number(item.headword))
            {
                continue;
            }
            for (collection, alphabet) in collections_of(item.value, rule.allow_lexeme) {
                if rule.allow_alphabet && alphabet {
                    continue;
                }
                let members = collections.entry(collection).or_default();
                // 同じ集まりに同じ見出しの項目が2つあっても、1つに数える
                if members.last() != Some(&i) {
                    members.push(i);
                }
            }
        }
        let headword_collections = |word: &str| -> Vec<Collection> {
            lookup(word, fnv1a(FNV_OFFSET_BASIS, word.as_bytes()))
                .map(|(_, value)| {
                    collections_of(value, rule.allow_lexeme)
                        .map(|(collection, _)| collection)
                        .collect()
                })
                .unwrap_or_default()
        };
        let prefer: Vec<(&str, Vec<Collection>)> = rule
            .prefer_words
            .iter()
            .map(|word| (word.as_str(), headword_collections(word)))
            .collect();
        let allowed: Vec<Collection> = rule
            .allows
            .iter()
            .flat_map(|word| headword_collections(word))
            .collect();
        let mut reports: Vec<Report> = Vec::new();
        let mut seen: HashSet<(Range<usize>, String)> = HashSet::new();
        for (collection, members) in &collections {
            let preferred = prefer
                .iter()
                .find(|(_, collections)| collections.contains(collection))
                .filter(|_| !allowed.contains(collection))
                .map(|(word, _)| *word);
            let found: Vec<(Range<usize>, String)> = match preferred {
                Some(word) => members
                    .iter()
                    .map(|&i| &used[i])
                    .filter(|item| item.headword != word)
                    .map(|item| {
                        let message = format!("「{}」は同義語の「{word}」にする", item.headword);
                        (item.last.clone(), message)
                    })
                    .collect(),
                None if members.len() >= 2 => {
                    let headwords: Vec<&str> = members.iter().map(|&i| used[i].headword).collect();
                    let message = format!("同義語の「{}」が混ざっている", headwords.join("」と「"));
                    members
                        .iter()
                        .map(|&i| (used[i].last.clone(), message.clone()))
                        .collect()
                }
                None => Vec::new(),
            };
            for (range, message) in found {
                // 1つの見出しは複数の集まりに属しうるので、集まりが違っても同じ指摘になりうる
                if !seen.insert((range.clone(), message.clone())) {
                    continue;
                }
                reports.push(Report {
                    range,
                    rule: RuleId::NoSynonyms.name(),
                    message: Cow::Owned(message),
                });
            }
        }
        reports
    }
}

/// 見出しの値 (`グループの番号,語彙番号,英字の印;…`) の項目ごとの、集まりと、英字の表記か
fn collections_of(
    value: &'static str,
    allow_lexeme: bool,
) -> impl Iterator<Item = (Collection, bool)> {
    value.split(';').map(move |item| {
        let mut fields = item.splitn(3, ',');
        let (Some(group), Some(lexeme), Some(alphabet)) =
            (fields.next(), fields.next(), fields.next())
        else {
            unreachable!("build.rs が値を `グループの番号,語彙番号,英字の印` の形で書く: {item}");
        };
        let lexeme = if allow_lexeme { lexeme } else { "" };
        ((group, lexeme), alphabet == "1")
    })
}

/// `allowNumber` が数えない見出し。ASCIIの数字を含むか、漢数字1字である
fn is_number(headword: &str) -> bool {
    headword.bytes().any(|byte| byte.is_ascii_digit())
        || matches!(
            headword,
            "〇" | "一" | "二" | "三" | "四" | "五" | "六" | "七" | "八" | "九"
        )
}

/// 見出し `key` (FNV-1aが `hash`) の行の見出しと値 (`グループの番号,語彙番号,英字の印;…`) を引く。文字列を確保しない
fn lookup(key: &str, hash: u32) -> Option<(&'static str, &'static str)> {
    let mask = TABLE.len() / 4 - 1;
    let mut slot = hash as usize & mask;
    loop {
        let bytes = TABLE[slot * 4..slot * 4 + 4]
            .try_into()
            .expect("4バイトである");
        let line_start = u32::from_le_bytes(bytes) as usize;
        if line_start == 0 {
            return None;
        }
        let line = &HEADWORDS[line_start - 1..];
        if let Some(rest) = line.strip_prefix(key)
            && let Some(rest) = rest.strip_prefix('\t')
        {
            let value = rest.split('\n').next().expect("splitは1つ以上を返す");
            return Some((&line[..key.len()], value));
        }
        slot = (slot + 1) & mask;
    }
}

/// `hash` に `bytes` を続けた32bitのFNV-1a。`build.rs` の表と同じ値になる
fn fnv1a(hash: u32, bytes: &[u8]) -> u32 {
    bytes.iter().fold(hash, |hash, &byte| {
        (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(headword: &str) -> Option<(&'static str, &'static str)> {
        lookup(headword, fnv1a(FNV_OFFSET_BASIS, headword.as_bytes()))
    }

    fn occurrences(headwords: &[&str]) -> Occurrences {
        let found = headwords
            .iter()
            .enumerate()
            .map(|(i, headword)| {
                let (headword, value) = find(headword).unwrap();
                (headword, value, i * 10..i * 10 + 1)
            })
            .collect();
        Occurrences { found }
    }

    #[test]
    fn every_headword_in_the_table_is_found() {
        let mut count = 0;
        for line in HEADWORDS.lines() {
            let (headword, value) = line.split_once('\t').unwrap();
            assert_eq!(find(headword), Some((headword, value)), "{headword}");
            count += 1;
        }
        assert_eq!(count, 68_701);
    }

    #[test]
    fn a_word_that_is_not_a_headword_is_not_found() {
        assert_eq!(find("サー"), None);
        assert_eq!(find(""), None);
    }

    #[test]
    fn server_spellings_fall_in_the_same_collection() {
        let reports = occurrences(&["サーバ", "サーバー"]).report(&NoSynonyms::default());
        assert_eq!(
            reports
                .iter()
                .map(|report| (report.range.start, report.message.as_ref()))
                .collect::<Vec<_>>(),
            [
                (0, "同義語の「サーバ」と「サーバー」が混ざっている"),
                (10, "同義語の「サーバ」と「サーバー」が混ざっている"),
            ]
        );
    }

    #[test]
    fn items_without_a_lexeme_number_form_one_collection_in_the_group() {
        // どちらも `インターネットゲーム` のグループの、語彙番号の無い項目
        let reports = occurrences(&["ネットゲー", "ネトゲ"]).report(&NoSynonyms::default());
        assert_eq!(reports.len(), 2);
        assert_eq!(
            reports[0].message,
            "同義語の「ネットゲー」と「ネトゲ」が混ざっている"
        );
        // 語彙番号のある項目とは、語彙番号ごとに分けると別の集まりになる
        let separate = occurrences(&["ネトゲ", "ネットゲーム"]).report(&NoSynonyms::default());
        assert_eq!(separate, []);
    }
}
