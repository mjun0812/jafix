use crate::morph::Token;
use crate::rules::builtin::MaxTen;
use crate::rules::{Report, RuleId};

use super::Analyzed;

/// 文の中の読点を数え、`max` を超えたら最後に数えた読点に指摘を出す。`!` などと `kuten` で数を0に戻す
pub(super) fn report(sentence: &Analyzed, rule: &MaxTen) -> Vec<Report<'static>> {
    let tokens = sentence.tokens;
    let mut reports = Vec::new();
    let mut count = 0;
    for (i, token) in tokens.iter().enumerate() {
        if token.surface == rule.touten && (rule.strict || !between_nouns(tokens, i)) {
            count += 1;
            if count > rule.max {
                reports.push(sentence.report(
                    token,
                    RuleId::MaxTen,
                    format!("1文の「{}」が{}個を超えている", rule.touten, rule.max),
                ));
                count = 0;
            }
        }
        if ["?", "!", "？", "！"].contains(&token.surface) || token.surface == rule.kuten {
            count = 0;
        }
    }
    reports
}

/// `i` 番目の読点が、「東京、大阪」のように、括弧を飛ばした前後の名詞に挟まれているか
fn between_nouns(tokens: &[Token], i: usize) -> bool {
    let prev = meaningful(tokens[..i].iter().rev());
    let next = meaningful(tokens[i + 1..].iter());
    prev.is_some_and(|prev| prev.pos == "名詞") && next.is_some_and(|next| next.pos == "名詞")
}

/// 括弧を飛ばして、最初のトークンを返す
fn meaningful<'t, 'a: 't>(
    mut tokens: impl Iterator<Item = &'t Token<'a>>,
) -> Option<&'t Token<'a>> {
    tokens.find(|token| {
        let bracket = (token.pos == "記号" && token.pos_detail_1.starts_with("括弧"))
            || token.surface == "("
            || token.surface == ")";
        !bracket
    })
}
