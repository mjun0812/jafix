use crate::morph::Token;
use crate::rules::builtin::NoDoubledJoshi;
use crate::rules::{Report, RuleId};

use super::Analyzed;

/// 同じ助詞が、数えるトークンの列で `min_interval` 以下の間隔で続いたら、後ろの助詞に指摘を出す
pub(super) fn report(sentence: &Analyzed, rule: &NoDoubledJoshi) -> Vec<Report<'static>> {
    let merged = merge(&sentence.tokens);
    // `merged` の中の番号。並びの位置の差が、助詞の間隔になる
    let countable: Vec<usize> = (0..merged.len())
        .filter(|&i| is_countable(&merged[i], rule))
        .collect();
    // 同じ助詞の、`countable` の中の位置。助詞が最初に出た順に並ぶ
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (position, &i) in countable.iter().enumerate() {
        let particle = &merged[i];
        if !is_particle(particle.token) {
            continue;
        }
        let key = format!(
            "{}:{}.{}.{}.{}",
            particle.surface,
            particle.token.pos,
            particle.pos_detail_1,
            particle.token.pos_detail_2,
            particle.token.pos_detail_3
        );
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, positions)) => positions.push(position),
            None => groups.push((key, vec![position])),
        }
    }
    let mut reports = Vec::new();
    for (_, positions) in &groups {
        let indices: Vec<usize> = positions.iter().map(|&p| countable[p]).collect();
        if rule.allow.contains(&merged[indices[0]].surface) {
            continue;
        }
        if !rule.strict && is_exception(&merged, &indices) {
            continue;
        }
        for (pair, &i) in positions.windows(2).zip(&indices[1..]) {
            if pair[1] - pair[0] <= rule.min_interval {
                let particle = &merged[i];
                reports.push(sentence.report(
                    particle.token,
                    RuleId::NoDoubledJoshi,
                    format!("同じ助詞「{}」が続いている", particle.surface),
                ));
            }
        }
    }
    reports
}

/// 続く助詞を1つにまとめたトークン。位置と、表層・品詞細分類1以外の項目は、最初のトークンのものを使う
struct Merged<'t, 'a> {
    token: &'t Token<'a>,
    surface: String,
    /// 最初のトークンの品詞細分類1に、続くトークンの表層をつなげたもの
    pos_detail_1: String,
}

fn merge<'t, 'a>(tokens: &'t [Token<'a>]) -> Vec<Merged<'t, 'a>> {
    let mut merged: Vec<Merged> = Vec::new();
    for token in tokens {
        if is_particle(token)
            && let Some(last) = merged.last_mut()
            && is_particle(last.token)
        {
            last.surface.push_str(token.surface);
            last.pos_detail_1.push_str(token.surface);
            continue;
        }
        merged.push(Merged {
            token,
            surface: token.surface.to_owned(),
            pos_detail_1: token.pos_detail_1.to_owned(),
        });
    }
    merged
}

fn is_particle(token: &Token) -> bool {
    token.pos.starts_with("助詞")
}

fn is_countable(merged: &Merged, rule: &NoDoubledJoshi) -> bool {
    let token = merged.token;
    if rule.strict {
        return is_particle(token);
    }
    let bracket = token.pos == "記号" && ["括弧開", "括弧閉"].contains(&token.pos_detail_1);
    let comma =
        rule.comma_characters.contains(&merged.surface) && ["名詞", "記号"].contains(&token.pos);
    bracket || rule.separator_characters.contains(&merged.surface) || comma || is_particle(token)
}

/// 同じ助詞が続いても自然な使い方か。`indices` は、同じ助詞の出現の全部の `merged` の中の番号である
fn is_exception(merged: &[Merged], indices: &[usize]) -> bool {
    let first = &merged[indices[0]];
    match (first.pos_detail_1.as_str(), first.surface.as_str()) {
        // 「の」の連続 (私の本の表紙)、「を」の連続 (本を読み絵を見る)、「て」の連続 (走って食べて)
        ("連体化", _) | ("格助詞", "を") | ("接続助詞", "て") => return true,
        _ => {}
    }
    let &[first, second] = indices else {
        return false;
    };
    // 「犬や猫や」のような並立
    if merged[first].pos_detail_1 == "並立助詞" && merged[second].pos_detail_1 == "並立助詞"
    {
        return true;
    }
    // 「〜かどうか」
    merged[first].surface == "か"
        && merged[second].surface == "か"
        && merged[second - 1].surface == "どう"
}
