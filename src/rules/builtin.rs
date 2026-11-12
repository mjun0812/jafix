#![expect(dead_code, unused_variables, reason = "skeleton: not implemented")]

use regex::Regex;

/// 組み込みのルールの設定。有効・無効と、オプションを持つルールのオプションである。
/// 既定では、どのルールも有効で、オプションはtextlintの `preset-ja-technical-writing` と同じである
#[derive(Debug)]
pub struct Builtin {
    pub fullwidth_paren: bool,
    pub space_between_ja_and_alnum: bool,
    pub space_between_ja_and_paren: bool,
    pub sentence_length: Option<SentenceLength>,
    pub max_comma: Option<MaxComma>,
    pub max_kanji_continuous_len: Option<MaxKanjiContinuousLen>,
    pub arabic_kanji_numbers: bool,
    pub ja_no_mixed_period: Option<JaNoMixedPeriod>,
    pub no_nfd: bool,
    pub no_invalid_control_character: Option<NoInvalidControlCharacter>,
    pub no_zero_width_spaces: bool,
    pub no_exclamation_question_mark: Option<NoExclamationQuestionMark>,
    pub no_hankaku_kana: bool,
    pub ja_unnatural_alphabet: Option<JaUnnaturalAlphabet>,
    pub no_unmatched_pair: bool,
}

impl Default for Builtin {
    fn default() -> Self {
        Self {
            fullwidth_paren: true,
            space_between_ja_and_alnum: true,
            space_between_ja_and_paren: true,
            sentence_length: Some(SentenceLength::default()),
            max_comma: Some(MaxComma::default()),
            max_kanji_continuous_len: Some(MaxKanjiContinuousLen::default()),
            arabic_kanji_numbers: true,
            ja_no_mixed_period: Some(JaNoMixedPeriod::default()),
            no_nfd: true,
            no_invalid_control_character: Some(NoInvalidControlCharacter::default()),
            no_zero_width_spaces: true,
            no_exclamation_question_mark: Some(NoExclamationQuestionMark::default()),
            no_hankaku_kana: true,
            ja_unnatural_alphabet: Some(JaUnnaturalAlphabet::default()),
            no_unmatched_pair: true,
        }
    }
}

impl Builtin {
    /// 設定の `[builtin]` の表から作る。キーはルールのID、値は `false` (無効)・`true` (既定のオプションで有効)・
    /// オプションの表 (有効) である。未知のID、boolでも表でもない値、未知のオプション、値の誤りは、
    /// IDとオプションの名前を含む理由を返す
    pub fn from_table(table: &toml::Table) -> Result<Self, String> {
        todo!("skeleton: not implemented")
    }
}

/// 許可や除外に書く文字列を正規表現にする。`/source/flags` の形は正規表現 (`regex` crateの構文。
/// フラグは `i`・`m`・`s` を使え、`g`・`u` は無視する)、それ以外は文字列そのものに当たる
pub fn pattern(value: &str) -> Result<Regex, String> {
    todo!("skeleton: not implemented")
}

#[derive(Debug)]
pub struct SentenceLength {
    pub max: usize,
    /// 長さを数える前に、文の文字列から取り除く
    pub skip_patterns: Vec<Regex>,
    /// 文字列が宛先と同じリンクを数えない
    pub skip_url_string_link: bool,
    pub count_by: CountBy,
}

impl Default for SentenceLength {
    fn default() -> Self {
        Self {
            max: 100,
            skip_patterns: Vec::new(),
            skip_url_string_link: true,
            count_by: CountBy::CodeUnits,
        }
    }
}

/// 長さの数え方。textlintの既定はUTF-16のコードユニットである
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountBy {
    CodeUnits,
    CodePoints,
}

#[derive(Debug)]
pub struct MaxComma {
    pub max: usize,
}

impl Default for MaxComma {
    fn default() -> Self {
        Self { max: 3 }
    }
}

#[derive(Debug)]
pub struct MaxKanjiContinuousLen {
    pub max: usize,
    /// 漢字の連続の全体と一致したら指摘しない語
    pub allow: Vec<String>,
}

impl Default for MaxKanjiContinuousLen {
    fn default() -> Self {
        Self {
            max: 6,
            allow: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct JaNoMixedPeriod {
    pub period_mark: String,
    pub allow_period_marks: Vec<String>,
    pub allow_emoji_at_end: bool,
}

impl Default for JaNoMixedPeriod {
    fn default() -> Self {
        Self {
            period_mark: "。".to_owned(),
            allow_period_marks: Vec::new(),
            allow_emoji_at_end: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct NoInvalidControlCharacter {
    pub allow: Vec<char>,
}

#[derive(Debug)]
pub struct NoExclamationQuestionMark {
    /// 一致した範囲 (直後の1文字を含む) にある記号を指摘しない。`Yahoo!` は常に入る
    pub allow: Vec<Regex>,
    pub allow_half_width_exclamation: bool,
    pub allow_full_width_exclamation: bool,
    pub allow_half_width_question: bool,
    pub allow_full_width_question: bool,
}

impl Default for NoExclamationQuestionMark {
    fn default() -> Self {
        Self {
            allow: vec![Regex::new("Yahoo!").unwrap()],
            allow_half_width_exclamation: false,
            allow_full_width_exclamation: false,
            allow_half_width_question: false,
            allow_full_width_question: false,
        }
    }
}

#[derive(Debug)]
pub struct JaUnnaturalAlphabet {
    /// 一致した範囲 (直後の1文字を含む) にあるアルファベットを指摘しない。
    /// `allow` と、`allowCommonCase` が有効なときの決まった言い回しを合わせたもの
    pub allow: Vec<Regex>,
}

impl Default for JaUnnaturalAlphabet {
    fn default() -> Self {
        let allow = [
            "a",
            "i",
            "u",
            "e",
            "o",
            "n",
            "[A-Z]",
            "[a-zA-Zａ-ｚＡ-Ｚ]言語",
            "[x-zX-Z]座標",
            "[x-zX-Z]軸",
            "(?i)Eメール",
        ];
        Self {
            allow: allow
                .into_iter()
                .map(|source| Regex::new(source).unwrap())
                .collect(),
        }
    }
}
