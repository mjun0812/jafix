//! `@textlint-ja/no-insert-dropping-sa`。さ入れ言葉とさ抜き言葉を、本文の範囲のすべてに当てる。
//! 照合の状態は、本文の範囲をまたいで持ち越さない

use std::sync::LazyLock;

use super::AnalyzedProse;
use super::hiragana::SurfaceIndex;
use super::hiragana_dict::{SA_PATTERNS, SaPattern};
use super::pattern::StaticFields;
use crate::rules::Report;

/// 3つのパターンの索引。プロセスの中で1度だけ作る
static INDEX: LazyLock<SurfaceIndex<'static, StaticFields>> =
    LazyLock::new(|| SurfaceIndex::new(SA_PATTERNS.iter().map(|sa| sa.pattern)));

/// `prose` のさ入れ・さ抜きの指摘を返す。置き換え後の語は、さ入れでは `さ` のトークンを除いた表層、さ抜きでは
/// 2つのトークンの間に `さ` を入れたものである
pub fn report(prose: &AnalyzedProse) -> Vec<Report> {
    let _ = (prose, &INDEX, SurfaceIndex::<StaticFields>::find);
    let _ = |sa: &SaPattern| sa.inserted;
    todo!("skeleton: not implemented")
}
