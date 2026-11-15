use crate::rules::{Report, RuleId};

use super::Analyzed;

/// 文を区切り文字でさらに区切り、区切りごとに逆接の「が」が2つ以上あれば、最初の「が」に指摘を出す
pub(super) fn report(sentence: &Analyzed) -> Vec<Report> {
    sentence
        .tokens
        .split(|token| [".", "．", "。", "?", "!", "？", "！"].contains(&token.surface))
        .filter_map(|part| {
            let mut particles = part
                .iter()
                .filter(|token| token.pos_detail_1 == "接続助詞" && token.surface == "が");
            let first = particles.next()?;
            particles.next()?;
            Some(sentence.report(
                first,
                RuleId::NoDoubledConjunctiveParticleGa,
                "1文に逆接の「が」が2回以上ある".to_owned(),
            ))
        })
        .collect()
}
