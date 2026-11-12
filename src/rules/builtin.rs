use std::borrow::Cow;

use regex::{Regex, RegexBuilder};
use toml::{Table, Value};

use super::RuleId;

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
    pub fn from_table(table: &Table) -> Result<Self, String> {
        let mut builtin = Builtin::default();
        let defaults = Table::new();
        for (key, value) in table {
            let id = RuleId::ALL
                .into_iter()
                .find(|id| id.name() == key)
                .ok_or_else(|| format!("`{key}` は組み込みのルールのIDではない"))?;
            let options = match value {
                Value::Boolean(false) => None,
                Value::Boolean(true) => Some(&defaults),
                Value::Table(options) => Some(options),
                _ => {
                    return Err(format!(
                        "`{key}`: 値は `true`・`false`・オプションの表のどれかで書く"
                    ));
                }
            };
            builtin
                .set(id, options)
                .map_err(|reason| format!("`{key}`: {reason}"))?;
        }
        Ok(builtin)
    }

    /// `id` のルールを、`options` が無ければ無効にし、あればそのオプションで有効にする
    fn set(&mut self, id: RuleId, options: Option<&Table>) -> Result<(), String> {
        match id {
            RuleId::FullwidthParen => self.fullwidth_paren = enabled(options)?,
            RuleId::SpaceBetweenJaAndAlnum => self.space_between_ja_and_alnum = enabled(options)?,
            RuleId::SpaceBetweenJaAndParen => self.space_between_ja_and_paren = enabled(options)?,
            RuleId::SentenceLength => {
                self.sentence_length = options.map(SentenceLength::from_options).transpose()?;
            }
            RuleId::MaxComma => self.max_comma = options.map(MaxComma::from_options).transpose()?,
            RuleId::MaxKanjiContinuousLen => {
                self.max_kanji_continuous_len = options
                    .map(MaxKanjiContinuousLen::from_options)
                    .transpose()?;
            }
            RuleId::ArabicKanjiNumbers => self.arabic_kanji_numbers = enabled(options)?,
            RuleId::JaNoMixedPeriod => {
                self.ja_no_mixed_period = options.map(JaNoMixedPeriod::from_options).transpose()?;
            }
            RuleId::NoNfd => self.no_nfd = enabled(options)?,
            RuleId::NoInvalidControlCharacter => {
                self.no_invalid_control_character = options
                    .map(NoInvalidControlCharacter::from_options)
                    .transpose()?;
            }
            RuleId::NoZeroWidthSpaces => self.no_zero_width_spaces = enabled(options)?,
            RuleId::NoExclamationQuestionMark => {
                self.no_exclamation_question_mark = options
                    .map(NoExclamationQuestionMark::from_options)
                    .transpose()?;
            }
            RuleId::NoHankakuKana => self.no_hankaku_kana = enabled(options)?,
            RuleId::JaUnnaturalAlphabet => {
                self.ja_unnatural_alphabet =
                    options.map(JaUnnaturalAlphabet::from_options).transpose()?;
            }
            RuleId::NoUnmatchedPair => self.no_unmatched_pair = enabled(options)?,
        }
        Ok(())
    }
}

/// オプションを持たないルールが有効か。表にキーがあれば、どのキーも使えないオプションである
fn enabled(options: Option<&Table>) -> Result<bool, String> {
    match options.and_then(|options| options.keys().next()) {
        Some(key) => Err(unknown_option(key)),
        None => Ok(options.is_some()),
    }
}

fn unknown_option(key: &str) -> String {
    format!("`{key}` は使えないオプション")
}

fn integer(key: &str, value: &Value, min: usize) -> Result<usize, String> {
    value
        .as_integer()
        .and_then(|n| usize::try_from(n).ok())
        .filter(|&n| n >= min)
        .ok_or_else(|| format!("`{key}` は{min}以上の整数で書く"))
}

fn boolean(key: &str, value: &Value) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| format!("`{key}` は `true` か `false` で書く"))
}

fn string<'v>(key: &str, value: &'v Value) -> Result<&'v str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("`{key}` は文字列で書く"))
}

fn strings<'v>(key: &str, value: &'v Value) -> Result<Vec<&'v str>, String> {
    value
        .as_array()
        .and_then(|values| values.iter().map(Value::as_str).collect())
        .ok_or_else(|| format!("`{key}` は文字列の配列で書く"))
}

fn patterns(key: &str, value: &Value) -> Result<Vec<Regex>, String> {
    strings(key, value)?
        .into_iter()
        .map(|value| pattern(value).map_err(|reason| format!("`{key}`: {reason}")))
        .collect()
}

/// 許可や除外に書く文字列を正規表現にする。`/source/flags` の形 (`flags` が `g`・`i`・`m`・`s`・`u`・`y` だけ) は
/// 正規表現 (`regex` crateの構文。フラグは `i`・`m`・`s` を使え、`g`・`u` は無視し、`y` は不正)、
/// それ以外は文字列そのものに当たる
pub fn pattern(value: &str) -> Result<Regex, String> {
    if value.is_empty() {
        return Err("空の文字列はパターンにできない".to_owned());
    }
    let (source, flags) = match value
        .strip_prefix('/')
        .and_then(|rest| rest.rsplit_once('/'))
    {
        // textlintと同じく、後ろがフラグの文字だけのときに正規表現の形とみなす。`/usr/bin` は文字列そのものである
        Some((source, flags)) if flags.chars().all(|c| "gimsuy".contains(c)) => {
            (Cow::Borrowed(source), flags)
        }
        _ => (Cow::Owned(regex::escape(value)), ""),
    };
    let mut builder = RegexBuilder::new(&source);
    for flag in flags.chars() {
        match flag {
            'i' => builder.case_insensitive(true),
            'm' => builder.multi_line(true),
            's' => builder.dot_matches_new_line(true),
            'g' | 'u' => &mut builder,
            _ => {
                return Err(format!(
                    "`{value}` のフラグ `{flag}` は使えない (`i`・`m`・`s`・`g`・`u` を使える)"
                ));
            }
        };
    }
    builder
        .build()
        .map_err(|e| format!("`{value}` を正規表現として読めない: {e}"))
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

impl SentenceLength {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "max" => rule.max = integer(key, value, 0)?,
                "skipPatterns" => rule.skip_patterns = patterns(key, value)?,
                "skipUrlStringLink" => rule.skip_url_string_link = boolean(key, value)?,
                "countBy" => {
                    rule.count_by = match string(key, value)? {
                        "codeunits" => CountBy::CodeUnits,
                        "codepoints" => CountBy::CodePoints,
                        _ => {
                            return Err(format!(
                                "`{key}` は \"codeunits\" か \"codepoints\" で書く"
                            ));
                        }
                    }
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
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

impl MaxComma {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "max" => rule.max = integer(key, value, 1)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
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

impl MaxKanjiContinuousLen {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "max" => rule.max = integer(key, value, 1)?,
                "allow" => {
                    rule.allow = strings(key, value)?
                        .into_iter()
                        .map(str::to_owned)
                        .collect();
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
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

impl JaNoMixedPeriod {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "periodMark" => rule.period_mark = string(key, value)?.to_owned(),
                "allowPeriodMarks" => {
                    rule.allow_period_marks = strings(key, value)?
                        .into_iter()
                        .map(str::to_owned)
                        .collect();
                }
                "allowEmojiAtEnd" => rule.allow_emoji_at_end = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug, Default)]
pub struct NoInvalidControlCharacter {
    pub allow: Vec<char>,
}

impl NoInvalidControlCharacter {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allow" => {
                    rule.allow = strings(key, value)?
                        .into_iter()
                        .map(|allowed| {
                            let mut chars = allowed.chars();
                            match (chars.next(), chars.next()) {
                                (Some(c), None) => Ok(c),
                                _ => Err(format!("`{key}` の要素は1文字の文字列で書く")),
                            }
                        })
                        .collect::<Result<_, _>>()?;
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
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
        Self::from_options(&Table::new()).expect("既定のオプションは読める")
    }
}

impl NoExclamationQuestionMark {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self {
            allow: Vec::new(),
            allow_half_width_exclamation: false,
            allow_full_width_exclamation: false,
            allow_half_width_question: false,
            allow_full_width_question: false,
        };
        for (key, value) in options {
            match key.as_str() {
                "allow" => rule.allow = patterns(key, value)?,
                "allowHalfWidthExclamation" => {
                    rule.allow_half_width_exclamation = boolean(key, value)?;
                }
                "allowFullWidthExclamation" => {
                    rule.allow_full_width_exclamation = boolean(key, value)?;
                }
                "allowHalfWidthQuestion" => rule.allow_half_width_question = boolean(key, value)?,
                "allowFullWidthQuestion" => rule.allow_full_width_question = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        rule.allow
            .push(pattern("Yahoo!").expect("文字列のパターンは読める"));
        Ok(rule)
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
        Self::from_options(&Table::new()).expect("既定のオプションは読める")
    }
}

impl JaUnnaturalAlphabet {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut allow = None;
        let mut allow_common_case = true;
        for (key, value) in options {
            match key.as_str() {
                "allow" => allow = Some(patterns(key, value)?),
                "allowCommonCase" => allow_common_case = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        let mut allow = match allow {
            Some(allow) => allow,
            None => ["a", "i", "u", "e", "o", "n", "/[A-Z]/"]
                .into_iter()
                .map(|source| pattern(source).expect("既定のパターンは読める"))
                .collect(),
        };
        if allow_common_case {
            allow.extend(
                [
                    "/[a-zA-Zａ-ｚＡ-Ｚ]言語/",
                    "/[x-zX-Z]座標/",
                    "/[x-zX-Z]軸/",
                    "/Eメール/i",
                ]
                .into_iter()
                .map(|source| pattern(source).expect("決まった言い回しのパターンは読める")),
            );
        }
        Ok(Self { allow })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_matches_a_slash_string_with_non_flag_letters_literally() {
        let regex = pattern("/usr/bin").unwrap();
        assert!(regex.is_match("見る /usr/bin を"));
        assert!(!regex.is_match("usr"));
    }

    #[test]
    fn pattern_rejects_the_sticky_flag() {
        assert!(pattern("/a/y").is_err());
    }

    #[test]
    fn pattern_reads_a_slash_string_with_flags_as_a_regex() {
        let regex = pattern("/a.c/gi").unwrap();
        assert!(regex.is_match("xAbCx"));
        assert!(!regex.is_match("a.d"));
    }
}
