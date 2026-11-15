use crate::morph::Token;
use crate::rules::{Report, RuleId};

use super::AnalyzedProse;

/// 「見れる」「来れる」と、一段動詞の未然形に続く「れる」に指摘を出す
pub(super) fn report(prose: &AnalyzedProse) -> Vec<Report> {
    if prose.in_quote_emphasis_or_link() {
        return Vec::new();
    }
    let tokens = prose.tokens;
    tokens
        .iter()
        .enumerate()
        .filter_map(|(i, token)| {
            if token.pos == "動詞" && ["来れる", "見れる"].contains(&token.basic_form) {
                // 「見れる」「来れる」は1つのトークンなので、2文字目の「れ」を指す
                let first = token.surface.chars().next().unwrap().len_utf8();
                return Some(token.range.start + first..token.range.end);
            }
            let after_ichidan_mizen = i > 0 && is_ichidan_mizen(&tokens[i - 1]);
            let reru =
                token.pos == "動詞" && token.pos_detail_1 == "接尾" && token.basic_form == "れる";
            (after_ichidan_mizen && reru).then(|| token.range.clone())
        })
        .map(|range| {
            prose.report(
                range,
                RuleId::NoDroppingTheRa,
                "ら抜き言葉を使っている".to_owned(),
            )
        })
        .collect()
}

fn is_ichidan_mizen(token: &Token) -> bool {
    token.pos == "動詞"
        && token.pos_detail_1 == "自立"
        && token.conjugated_type == "一段"
        && token.conjugated_form == "未然形"
}
