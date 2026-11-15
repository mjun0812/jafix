//! `@textlint-ja/no-synonyms`。文書の全体で、同じ同義語の集まりの別の見出しが2つ以上使われたら指摘する。
//! 同義語辞書は `no_synonyms/sudachi-synonyms.tsv` (sudachi-synonyms-dictionary 20.0.0 から作ったもの) で、
//! 見出しで引く表は `build.rs` が作る

use std::ops::Range;

use super::AnalyzedProse;
use crate::rules::Report;
use crate::rules::builtin::NoSynonyms;

/// 1行が1つの見出しで、`見出し\tグループの番号,語彙番号,英字の印;…`。見出しのバイト列の順に並ぶ
static HEADWORDS: &str = include_str!(concat!(env!("OUT_DIR"), "/synonyms-headwords.tsv"));

/// 見出しのFNV-1a (32bit) の下位ビットで位置を決めた開番地法の表。リトルエンディアンの `u32` の並びで、値は
/// `HEADWORDS` の行の先頭のバイト位置に1を足したもの、空きは0である
static TABLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/synonyms-table.bin"));

/// 見出しに当てる、空白をまたがずにつなげたトークンの並びの長さの上限
const MAX_TOKENS: usize = 8;

/// 文書の中で使われた見出しの出現。本文の範囲の繰り返しで集め、繰り返しの後に `report` で指摘にする
#[derive(Default)]
pub struct Occurrences {
    /// 見出しと、その値 (`HEADWORDS` の行のタブの後) と、出現の元のソースでの範囲。出現の順に並ぶ
    #[allow(dead_code)] // skeleton: not implemented
    found: Vec<(&'static str, &'static str, Range<usize>)>,
}

impl Occurrences {
    /// `prose` が引用の中でもリンクの文字列でもなければ、トークンの最長一致で語を区切り、見出しに当たる語の出現を足す
    pub fn collect(&mut self, prose: &AnalyzedProse) {
        let _ = (prose, lookup, MAX_TOKENS);
        todo!("skeleton: not implemented")
    }

    /// 集めた出現を、`rule` のオプションで同義語の集まりごとにまとめ、指摘を返す
    pub fn report(self, rule: &NoSynonyms) -> Vec<Report> {
        let _ = rule;
        todo!("skeleton: not implemented")
    }
}

/// 見出し `key` (FNV-1aが `hash`) の行の値 (`グループの番号,語彙番号,英字の印;…`) を引く。文字列を確保しない
fn lookup(key: &str, hash: u32) -> Option<&'static str> {
    let _ = (key, hash, HEADWORDS, TABLE);
    todo!("skeleton: not implemented")
}
