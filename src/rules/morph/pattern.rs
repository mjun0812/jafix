use crate::morph::Token;

/// トークンの項目
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Surface,
    Pos,
    PosDetail1,
    PosDetail2,
    PosDetail3,
    ConjugatedType,
    ConjugatedForm,
    BasicForm,
    Reading,
    Pronunciation,
}

/// 期待するトークン。`fields` のすべての項目が、値のどれかと完全に一致するトークンに一致する。
/// `fields` が空なら、どのトークンにも一致する
#[derive(Debug)]
pub struct Expect {
    pub fields: &'static [(Field, &'static [&'static str])],
    /// 一致しなければ、トークンを進めずに飛ばしてよい
    pub skippable: bool,
}

/// 期待するトークンの並びに一致した箇所
#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    /// 並びのk番目の期待するトークンに一致した、トークンの番号。飛ばしたものは `None`
    pub tokens: Vec<Option<usize>>,
}

/// `pattern` を、`tokens` のすべての位置を開始位置として当て、一致を開始位置の順に返す
pub fn find(tokens: &[Token], pattern: &[Expect]) -> Vec<Match> {
    (0..tokens.len())
        .filter_map(|start| match_at(tokens, pattern, start))
        .collect()
}

/// `start` のトークンから `pattern` を当てる。一致の途中で外れても、ほかの開始位置の結果には影響しない
fn match_at(tokens: &[Token], pattern: &[Expect], start: usize) -> Option<Match> {
    let mut next = start;
    let mut matched = Vec::with_capacity(pattern.len());
    for expect in pattern {
        match tokens.get(next) {
            Some(token) if expect.matches(token) => {
                matched.push(Some(next));
                next += 1;
            }
            // 並びの末尾のskippableは、トークンが残っていなくても飛ばせる
            _ if expect.skippable => matched.push(None),
            _ => return None,
        }
    }
    Some(Match { tokens: matched })
}

impl Expect {
    fn matches(&self, token: &Token) -> bool {
        self.fields
            .iter()
            .all(|&(field, values)| values.contains(&value(token, field)))
    }
}

fn value<'a>(token: &Token<'a>, field: Field) -> &'a str {
    match field {
        Field::Surface => token.surface,
        Field::Pos => token.pos,
        Field::PosDetail1 => token.pos_detail_1,
        Field::PosDetail2 => token.pos_detail_2,
        Field::PosDetail3 => token.pos_detail_3,
        Field::ConjugatedType => token.conjugated_type,
        Field::ConjugatedForm => token.conjugated_form,
        Field::BasicForm => token.basic_form,
        Field::Reading => token.reading,
        Field::Pronunciation => token.pronunciation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(surface: &'static str, pos: &'static str) -> Token<'static> {
        Token {
            range: 0..surface.len(),
            surface,
            pos,
            pos_detail_1: "*",
            pos_detail_2: "*",
            pos_detail_3: "*",
            conjugated_type: "*",
            conjugated_form: "*",
            basic_form: "*",
            reading: "*",
            pronunciation: "*",
        }
    }

    fn matches(tokens: &[Token], pattern: &[Expect]) -> Vec<Vec<Option<usize>>> {
        find(tokens, pattern)
            .into_iter()
            .map(|m| m.tokens)
            .collect()
    }

    #[test]
    fn requires_every_written_field_to_equal_one_of_its_values() {
        let pattern = [Expect {
            fields: &[(Field::Surface, &["a", "b"]), (Field::Pos, &["名詞"])],
            skippable: false,
        }];
        let tokens = [token("a", "名詞"), token("b", "動詞"), token("b", "名詞")];
        assert_eq!(
            matches(&tokens, &pattern),
            vec![vec![Some(0)], vec![Some(2)]]
        );
    }

    #[test]
    fn retries_from_a_token_where_a_previous_attempt_failed() {
        let pattern = [
            Expect {
                fields: &[(Field::Surface, &["x"])],
                skippable: false,
            },
            Expect {
                fields: &[(Field::Surface, &["y"])],
                skippable: false,
            },
        ];
        let tokens = [token("x", "*"), token("x", "*"), token("y", "*")];
        assert_eq!(matches(&tokens, &pattern), vec![vec![Some(1), Some(2)]]);
    }

    #[test]
    fn skips_a_skippable_expect_without_consuming_the_token() {
        let pattern = [
            Expect {
                fields: &[(Field::Surface, &["x"])],
                skippable: false,
            },
            Expect {
                fields: &[(Field::Surface, &["、"])],
                skippable: true,
            },
            Expect {
                fields: &[(Field::Surface, &["y"])],
                skippable: false,
            },
        ];
        let tokens = [
            token("x", "*"),
            token("y", "*"),
            token("x", "*"),
            token("、", "*"),
            token("y", "*"),
        ];
        assert_eq!(
            matches(&tokens, &pattern),
            vec![
                vec![Some(0), None, Some(1)],
                vec![Some(2), Some(3), Some(4)]
            ]
        );
    }

    #[test]
    fn drops_a_trailing_skippable_expect_but_not_a_required_one_at_the_end() {
        let pattern = [
            Expect {
                fields: &[(Field::Surface, &["x"])],
                skippable: false,
            },
            Expect {
                fields: &[(Field::Surface, &["y"])],
                skippable: true,
            },
        ];
        assert_eq!(
            matches(&[token("x", "*")], &pattern),
            vec![vec![Some(0), None]]
        );
        let required = [
            Expect {
                fields: &[(Field::Surface, &["x"])],
                skippable: false,
            },
            Expect {
                fields: &[(Field::Surface, &["y"])],
                skippable: false,
            },
        ];
        assert_eq!(
            matches(&[token("x", "*")], &required),
            Vec::<Vec<Option<usize>>>::new()
        );
    }
}
