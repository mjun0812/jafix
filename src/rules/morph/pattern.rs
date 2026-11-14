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
    let _ = (tokens, pattern);
    todo!("skeleton: not implemented")
}
