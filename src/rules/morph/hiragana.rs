//! ひらがな系の5ルール (`ja-hiragana-hojodoushi`・`ja-hiragana-keishikimeishi`・`ja-keishikimeishi`・
//! `ja-hiragana-fukushi`・`ja-hiragana-daimeishi`)。本文の範囲のすべてに、辞書のトークンの並びを当てる。
//! 照合の状態は、本文の範囲をまたいで持ち越さない

use std::collections::HashMap;

use super::AnalyzedProse;
use super::hiragana_dict::{DAIMEISHI, Entry, FUKUSHI, HOJODOUSHI, KEISHIKIMEISHI};
use super::pattern::{Expect, Fields, Match};
use crate::morph::Token;
use crate::rules::Report;
use crate::rules::builtin::Builtin;

/// 有効なひらがな系のルールの辞書 (組み込みの辞書か `rulePath` の辞書) と、その索引。`morph::report` の1回ごとに
/// 1つ作り、本文の範囲ごとに当てる
pub struct Matcher<'a> {
    #[allow(dead_code)] // skeleton: not implemented
    builtin: &'a Builtin,
}

impl<'a> Matcher<'a> {
    /// ひらがな系のルールが1つも有効でなければ、呼ばない
    pub fn new(builtin: &'a Builtin) -> Self {
        let _ = (builtin, &HOJODOUSHI, &FUKUSHI, &DAIMEISHI, &KEISHIKIMEISHI);
        let _ = |entry: &Entry| (entry.expected, entry.offset, entry.pattern);
        todo!("skeleton: not implemented")
    }

    /// `prose` に当たる、有効なひらがな系のルールの指摘を返す。同じルールの、範囲と文面が同じ指摘は1件にする
    pub fn report(&self, prose: &AnalyzedProse) -> Vec<Report> {
        let _ = prose;
        todo!("skeleton: not implemented")
    }
}

/// 表層を書いた期待するトークンで、並びの照合の開始位置を引く索引。並びのうち、前に飛ばせる期待するトークンの無い、
/// 表層を書いた最初のトークンの表層で引き、本文のトークンの位置からその並びの中の位置を引いた開始位置で当てる。
/// 表層を書いたトークンを持たない並びは、すべての開始位置で当てる。結果は、並びをすべての開始位置から当てたもの
/// (`pattern::find`) と同じである
pub struct SurfaceIndex<'d, F> {
    #[allow(dead_code)] // skeleton: not implemented
    patterns: Vec<&'d [Expect<F>]>,
    /// 表層から、(並びの番号, 表層を書いたトークンの並びの中の位置) の一覧
    #[allow(dead_code)] // skeleton: not implemented
    by_surface: HashMap<&'d str, Vec<(usize, usize)>>,
    /// 表層を書いたトークンを持たない並びの番号
    #[allow(dead_code)] // skeleton: not implemented
    unindexed: Vec<usize>,
}

impl<'d, F: Fields> SurfaceIndex<'d, F> {
    pub fn new(patterns: impl IntoIterator<Item = &'d [Expect<F>]>) -> Self {
        let _ = (patterns.into_iter(), F::surfaces);
        todo!("skeleton: not implemented")
    }

    /// `tokens` に当たる一致を、(`new` に渡した並びの番号, 一致) で返す
    pub fn find(&self, tokens: &[Token]) -> Vec<(usize, Match)> {
        let _ = tokens;
        todo!("skeleton: not implemented")
    }
}
