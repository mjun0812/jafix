//! `@textlint-ja/no-insert-dropping-sa`。さ入れ言葉とさ抜き言葉を、本文の範囲のすべてに当てる。
//! 照合の状態は、本文の範囲をまたいで持ち越さない

use std::sync::LazyLock;

use super::AnalyzedProse;
use super::hiragana::{SurfaceIndex, matched_range};
use super::hiragana_dict::SA_PATTERNS;
use super::pattern::StaticFields;
use crate::rules::{Report, RuleId};

/// 3つのパターンの索引。プロセスの中で1度だけ作る
static INDEX: LazyLock<SurfaceIndex<'static, StaticFields>> =
    LazyLock::new(|| SurfaceIndex::new(SA_PATTERNS.iter().map(|sa| sa.pattern)));

/// `prose` のさ入れ・さ抜きの指摘を返す。置き換え後の語は、さ入れでは `さ` のトークンを除いた表層、さ抜きでは
/// 2つのトークンの間に `さ` を入れたものである
pub fn report(prose: &AnalyzedProse) -> Vec<Report> {
    INDEX
        .find(prose.tokens)
        .into_iter()
        .filter_map(|(i, found)| {
            let inserted = SA_PATTERNS[i].inserted;
            let whole = matched_range(prose, &found, 0)?;
            let range = matched_range(prose, &found, inserted.unwrap_or(0))?;
            let surfaces = found
                .tokens
                .iter()
                .enumerate()
                .filter_map(|(k, token)| Some((k, prose.tokens[(*token)?].surface)));
            let message = match inserted {
                Some(sa) => format!(
                    "さ入れ言葉「{}」は「{}」にする",
                    &prose.text[whole],
                    surfaces
                        .filter(|&(k, _)| k != sa)
                        .map(|(_, surface)| surface)
                        .collect::<String>()
                ),
                None => format!(
                    "さ抜き言葉「{}」は「{}」にする",
                    &prose.text[whole],
                    surfaces
                        .map(|(_, surface)| surface)
                        .collect::<Vec<_>>()
                        .join("さ")
                ),
            };
            Some(prose.report(range, RuleId::NoInsertDroppingSa, message))
        })
        .collect()
}
