use std::borrow::Cow;
use std::path::Path;

use regex::{Regex, RegexBuilder};
use toml::{Table, Value};

use super::RuleId;
use super::morph::pattern::{Expect, OwnedFields};

/// 組み込みのルールの設定。有効・無効と、オプションを持つルールのオプションである。
/// 既定では、`preset-ja-technical-writing` の23ルールと修正ルールの3つが有効で、オプションはtextlintの
/// `preset-ja-technical-writing` と同じである。`@textlint-ja/preset-ai-writing` と `preset-ai-words-ja` の
/// 7ルールは無効である
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
    pub max_ten: Option<MaxTen>,
    pub no_mix_dearu_desumasu: Option<NoMixDearuDesumasu>,
    pub no_doubled_conjunction: bool,
    pub no_doubled_conjunctive_particle_ga: bool,
    pub no_double_negative_ja: bool,
    pub no_doubled_joshi: Option<NoDoubledJoshi>,
    pub no_dropping_the_ra: bool,
    pub ja_no_weak_phrase: bool,
    pub ja_no_successive_word: Option<JaNoSuccessiveWord>,
    pub ja_no_abusage: bool,
    pub ja_no_redundant_expression: Option<JaNoRedundantExpression>,
    pub no_ai_list_formatting: Option<NoAiListFormatting>,
    pub no_ai_hype_expressions: Option<NoAiHypeExpressions>,
    pub no_ai_emphasis_patterns: Option<NoAiEmphasisPatterns>,
    pub no_ai_colon_continuation: Option<NoAiColonContinuation>,
    pub ai_tech_writing_guideline: Option<AiTechWritingGuideline>,
    pub no_ai_words: Option<NoAiWords>,
    pub no_short_topic_comma: Option<NoShortTopicComma>,
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
            max_ten: Some(MaxTen::default()),
            no_mix_dearu_desumasu: Some(NoMixDearuDesumasu::default()),
            no_doubled_conjunction: true,
            no_doubled_conjunctive_particle_ga: true,
            no_double_negative_ja: true,
            no_doubled_joshi: Some(NoDoubledJoshi::default()),
            no_dropping_the_ra: true,
            ja_no_weak_phrase: true,
            ja_no_successive_word: Some(JaNoSuccessiveWord::default()),
            ja_no_abusage: true,
            ja_no_redundant_expression: Some(JaNoRedundantExpression::default()),
            no_ai_list_formatting: None,
            no_ai_hype_expressions: None,
            no_ai_emphasis_patterns: None,
            no_ai_colon_continuation: None,
            ai_tech_writing_guideline: None,
            no_ai_words: None,
            no_short_topic_comma: None,
        }
    }
}

impl Builtin {
    /// 設定の `[builtin]` の表から作る。キーはルールのIDかpresetの名前である。ルールのIDの値は `false` (無効)・
    /// `true` (既定のオプションで有効)・オプションの表 (有効)、presetの名前の値は `true` か `false` である。
    /// `config_dir` は設定ファイルのあるディレクトリで、オプションの相対パスをここから解決する。
    /// 未知のキー、値の型の誤り、未知のオプション、値の誤りは、キーとオプションの名前を含む理由を返す
    pub fn from_table(table: &Table, config_dir: &Path) -> Result<Self, String> {
        let mut builtin = Builtin::default();
        // presetの名前のキーを先に当て、ルールのIDのキーで上書きする。TOMLの表は順を持たないので、書いた順によらない
        for (key, value) in table {
            if let Some(preset) = Preset::from_name(key) {
                builtin
                    .apply_preset(preset, value, config_dir)
                    .map_err(|reason| format!("`{key}`: {reason}"))?;
            }
        }
        let defaults = Table::new();
        for (key, value) in table
            .iter()
            .filter(|(key, _)| Preset::from_name(key).is_none())
        {
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
                .set(id, options, config_dir)
                .map_err(|reason| format!("`{key}`: {reason}"))?;
        }
        Ok(builtin)
    }

    /// `preset` のルールを、`value` が `true` ならpresetの既定の有効・無効とオプションにし、`false` ならすべて無効にする。
    /// `value` が `true` でも `false` でもなければ理由を返す
    fn apply_preset(
        &mut self,
        preset: Preset,
        value: &Value,
        config_dir: &Path,
    ) -> Result<(), String> {
        let _ = (preset, value, config_dir);
        todo!("skeleton: not implemented")
    }

    /// `id` のルールを、`options` が無ければ無効にし、あればそのオプションで有効にする
    fn set(
        &mut self,
        id: RuleId,
        options: Option<&Table>,
        config_dir: &Path,
    ) -> Result<(), String> {
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
            RuleId::MaxTen => self.max_ten = options.map(MaxTen::from_options).transpose()?,
            RuleId::NoMixDearuDesumasu => {
                self.no_mix_dearu_desumasu =
                    options.map(NoMixDearuDesumasu::from_options).transpose()?;
            }
            RuleId::NoDoubledConjunction => self.no_doubled_conjunction = enabled(options)?,
            RuleId::NoDoubledConjunctiveParticleGa => {
                self.no_doubled_conjunctive_particle_ga = enabled(options)?;
            }
            RuleId::NoDoubleNegativeJa => self.no_double_negative_ja = enabled(options)?,
            RuleId::NoDoubledJoshi => {
                self.no_doubled_joshi = options.map(NoDoubledJoshi::from_options).transpose()?;
            }
            RuleId::NoDroppingTheRa => self.no_dropping_the_ra = enabled(options)?,
            RuleId::JaNoWeakPhrase => self.ja_no_weak_phrase = enabled(options)?,
            RuleId::JaNoSuccessiveWord => {
                self.ja_no_successive_word =
                    options.map(JaNoSuccessiveWord::from_options).transpose()?;
            }
            RuleId::JaNoAbusage => self.ja_no_abusage = enabled(options)?,
            RuleId::JaNoRedundantExpression => {
                self.ja_no_redundant_expression = options
                    .map(JaNoRedundantExpression::from_options)
                    .transpose()?;
            }
            RuleId::NoAiListFormatting => {
                self.no_ai_list_formatting =
                    options.map(NoAiListFormatting::from_options).transpose()?;
            }
            RuleId::NoAiHypeExpressions => {
                self.no_ai_hype_expressions =
                    options.map(NoAiHypeExpressions::from_options).transpose()?;
            }
            RuleId::NoAiEmphasisPatterns => {
                self.no_ai_emphasis_patterns = options
                    .map(NoAiEmphasisPatterns::from_options)
                    .transpose()?;
            }
            RuleId::NoAiColonContinuation => {
                self.no_ai_colon_continuation = options
                    .map(NoAiColonContinuation::from_options)
                    .transpose()?;
            }
            RuleId::AiTechWritingGuideline => {
                self.ai_tech_writing_guideline = options
                    .map(AiTechWritingGuideline::from_options)
                    .transpose()?;
            }
            RuleId::NoAiWords => {
                self.no_ai_words = options
                    .map(|options| NoAiWords::from_options(options, config_dir))
                    .transpose()?;
            }
            RuleId::NoShortTopicComma => {
                self.no_short_topic_comma =
                    options.map(NoShortTopicComma::from_options).transpose()?;
            }
        }
        Ok(())
    }
}

/// `[builtin]` に名前で書けるpreset。名前はtextlintの設定に書く名前と同じである
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    JaTechnicalWriting,
    AiWriting,
    AiWordsJa,
}

impl Preset {
    pub const ALL: [Preset; 3] = [
        Preset::JaTechnicalWriting,
        Preset::AiWriting,
        Preset::AiWordsJa,
    ];

    /// `[builtin]` のキーに使う名前
    pub fn name(self) -> &'static str {
        match self {
            Preset::JaTechnicalWriting => "preset-ja-technical-writing",
            Preset::AiWriting => "@textlint-ja/preset-ai-writing",
            Preset::AiWordsJa => "preset-ai-words-ja",
        }
    }

    fn from_name(name: &str) -> Option<Preset> {
        Preset::ALL.into_iter().find(|preset| preset.name() == name)
    }

    /// presetが束ねるルールと、presetの既定で有効か。`preset-ja-technical-writing` は、jafix独自の修正ルール3つを含まない
    #[allow(dead_code)] // skeleton: not implemented
    fn rules(self) -> &'static [(RuleId, bool)] {
        match self {
            Preset::JaTechnicalWriting => &[
                (RuleId::SentenceLength, true),
                (RuleId::MaxComma, true),
                (RuleId::MaxKanjiContinuousLen, true),
                (RuleId::ArabicKanjiNumbers, true),
                (RuleId::JaNoMixedPeriod, true),
                (RuleId::NoNfd, true),
                (RuleId::NoInvalidControlCharacter, true),
                (RuleId::NoZeroWidthSpaces, true),
                (RuleId::NoExclamationQuestionMark, true),
                (RuleId::NoHankakuKana, true),
                (RuleId::JaUnnaturalAlphabet, true),
                (RuleId::NoUnmatchedPair, true),
                (RuleId::MaxTen, true),
                (RuleId::NoMixDearuDesumasu, true),
                (RuleId::NoDoubledConjunction, true),
                (RuleId::NoDoubledConjunctiveParticleGa, true),
                (RuleId::NoDoubleNegativeJa, true),
                (RuleId::NoDoubledJoshi, true),
                (RuleId::NoDroppingTheRa, true),
                (RuleId::JaNoWeakPhrase, true),
                (RuleId::JaNoSuccessiveWord, true),
                (RuleId::JaNoAbusage, true),
                (RuleId::JaNoRedundantExpression, true),
            ],
            Preset::AiWriting => &[
                (RuleId::NoAiListFormatting, true),
                (RuleId::NoAiHypeExpressions, true),
                (RuleId::NoAiEmphasisPatterns, true),
                (RuleId::NoAiColonContinuation, true),
                (RuleId::AiTechWritingGuideline, true),
            ],
            Preset::AiWordsJa => &[
                (RuleId::NoAiWords, true),
                (RuleId::NoShortTopicComma, false),
            ],
        }
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

fn non_empty_string<'v>(key: &str, value: &'v Value) -> Result<&'v str, String> {
    Some(string(key, value)?)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("`{key}` は空でない文字列で書く"))
}

fn non_empty_strings(key: &str, value: &Value) -> Result<Vec<String>, String> {
    let values = strings(key, value)?;
    if values.iter().any(|value| value.is_empty()) {
        return Err(format!("`{key}` の要素は空でない文字列で書く"));
    }
    Ok(values.into_iter().map(str::to_owned).collect())
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

#[derive(Debug)]
pub struct MaxTen {
    pub max: usize,
    /// 名詞に挟まれた読点も数える
    pub strict: bool,
    pub touten: String,
    pub kuten: String,
}

impl Default for MaxTen {
    fn default() -> Self {
        Self {
            max: 3,
            strict: false,
            touten: "、".to_owned(),
            kuten: "。".to_owned(),
        }
    }
}

impl MaxTen {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "max" => rule.max = integer(key, value, 0)?,
                "strict" => rule.strict = boolean(key, value)?,
                "touten" => rule.touten = non_empty_string(key, value)?.to_owned(),
                "kuten" => rule.kuten = non_empty_string(key, value)?.to_owned(),
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug)]
pub struct NoDoubledJoshi {
    /// 同じ助詞の間にある、数えるトークンの数がこれ以下なら指摘する
    pub min_interval: usize,
    /// 例外の規則を使わず、助詞だけを数える
    pub strict: bool,
    /// 指摘しない助詞の表層
    pub allow: Vec<String>,
    pub separator_characters: Vec<String>,
    pub comma_characters: Vec<String>,
}

impl Default for NoDoubledJoshi {
    fn default() -> Self {
        let strings = |values: &[&str]| values.iter().map(|&value| value.to_owned()).collect();
        Self {
            min_interval: 1,
            strict: false,
            allow: Vec::new(),
            separator_characters: strings(&[".", "．", "。", "?", "!", "？", "！"]),
            comma_characters: strings(&["、", "，"]),
        }
    }
}

impl NoDoubledJoshi {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "min_interval" => rule.min_interval = integer(key, value, 1)?,
                "strict" => rule.strict = boolean(key, value)?,
                "allow" => rule.allow = non_empty_strings(key, value)?,
                "separatorCharacters" => rule.separator_characters = non_empty_strings(key, value)?,
                "commaCharacters" => rule.comma_characters = non_empty_strings(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// 文体
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Dearu,
    Desumasu,
}

/// `prefer_*` は、その種類で優先する文体。`None` は、多いほうを優先する
#[derive(Debug)]
pub struct NoMixDearuDesumasu {
    pub prefer_in_header: Option<Style>,
    pub prefer_in_body: Option<Style>,
    pub prefer_in_list: Option<Style>,
    /// 文末でない「である」「です」「ます」も数える
    pub strict: bool,
}

impl Default for NoMixDearuDesumasu {
    fn default() -> Self {
        Self {
            prefer_in_header: None,
            prefer_in_body: Some(Style::Desumasu),
            prefer_in_list: Some(Style::Dearu),
            strict: false,
        }
    }
}

impl NoMixDearuDesumasu {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "preferInHeader" => rule.prefer_in_header = prefer(key, value)?,
                "preferInBody" => rule.prefer_in_body = prefer(key, value)?,
                "preferInList" => rule.prefer_in_list = prefer(key, value)?,
                "strict" => rule.strict = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

fn prefer(key: &str, value: &Value) -> Result<Option<Style>, String> {
    match string(key, value)? {
        "" => Ok(None),
        "ですます" => Ok(Some(Style::Desumasu)),
        "である" => Ok(Some(Style::Dearu)),
        _ => Err(format!(
            "`{key}` は \"\"・\"ですます\"・\"である\" のどれかで書く"
        )),
    }
}

#[derive(Debug)]
pub struct JaNoSuccessiveWord {
    /// カタカナの語の繰り返しを指摘しない
    pub allow_onomatopee: bool,
    pub allow: Vec<Regex>,
}

impl Default for JaNoSuccessiveWord {
    fn default() -> Self {
        Self {
            allow_onomatopee: true,
            allow: Vec::new(),
        }
    }
}

impl JaNoSuccessiveWord {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allowOnomatopee" => rule.allow_onomatopee = boolean(key, value)?,
                "allow" => rule.allow = patterns(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `allowNodeTypes` に書ける要素の型。名前はtextlintのnodeの型と同じである
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    BlockQuote,
    ListItem,
    Link,
    LinkReference,
    Image,
    Emphasis,
    Strong,
    Delete,
    /// 書けるが、何も変えない (インラインコードの中身は文の文字列に入らない)
    Code,
    /// 書けるが、何も変えない (Markdownに無い型)
    ReferenceDef,
}

/// 辞書の1項目への設定
#[derive(Debug, Default)]
pub struct DictOption {
    pub disabled: bool,
    /// 一致したトークンの表層をつなげた文字列に当たれば、指摘しない。`None` は辞書の既定値
    pub allows: Option<Vec<Regex>>,
}

#[derive(Debug)]
pub struct JaNoRedundantExpression {
    pub allow_node_types: Vec<NodeType>,
    /// `dict1` から `dict6` の順
    pub dicts: [DictOption; 6],
}

impl Default for JaNoRedundantExpression {
    fn default() -> Self {
        Self {
            allow_node_types: vec![
                NodeType::BlockQuote,
                NodeType::Link,
                NodeType::ReferenceDef,
                NodeType::Code,
            ],
            dicts: Default::default(),
        }
    }
}

impl JaNoRedundantExpression {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allowNodeTypes" => {
                    rule.allow_node_types = strings(key, value)?
                        .into_iter()
                        .map(|name| {
                            node_type(name)
                                .ok_or_else(|| format!("`{key}` の `{name}` は使えない型"))
                        })
                        .collect::<Result<_, _>>()?;
                }
                "dictOptions" => {
                    let dicts = value
                        .as_table()
                        .ok_or_else(|| format!("`{key}` は表で書く"))?;
                    for (id, option) in dicts {
                        let dict = id
                            .strip_prefix("dict")
                            .and_then(|n| n.parse::<usize>().ok())
                            .and_then(|n| rule.dicts.get_mut(n.checked_sub(1)?))
                            .ok_or_else(|| format!("`{key}` の `{id}` は辞書の項目ではない (`dict1`〜`dict6` を使える)"))?;
                        *dict = DictOption::from_options(option)
                            .map_err(|reason| format!("`{key}` の `{id}`: {reason}"))?;
                    }
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

fn node_type(name: &str) -> Option<NodeType> {
    Some(match name {
        "BlockQuote" => NodeType::BlockQuote,
        "ListItem" => NodeType::ListItem,
        "Link" => NodeType::Link,
        "LinkReference" => NodeType::LinkReference,
        "Image" => NodeType::Image,
        "Emphasis" => NodeType::Emphasis,
        "Strong" => NodeType::Strong,
        "Delete" => NodeType::Delete,
        "Code" => NodeType::Code,
        "ReferenceDef" => NodeType::ReferenceDef,
        _ => return None,
    })
}

impl DictOption {
    fn from_options(options: &Value) -> Result<Self, String> {
        let options = options.as_table().ok_or("値は表で書く")?;
        let mut option = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "disabled" => option.disabled = boolean(key, value)?,
                "allows" => option.allows = Some(patterns(key, value)?),
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(option)
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)] // skeleton: not implemented
pub struct NoAiListFormatting {
    /// リスト項目の元の文字列のどこかに当たれば、その項目を指摘しない
    pub allows: Vec<Regex>,
    pub disable_bold_list_items: bool,
    pub disable_emoji_list_items: bool,
}

impl NoAiListFormatting {
    fn from_options(options: &Table) -> Result<Self, String> {
        let _ = options;
        todo!("skeleton: not implemented")
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)] // skeleton: not implemented
pub struct NoAiHypeExpressions {
    /// 本文の範囲の元の文字列のどこかに当たれば、その範囲を指摘しない
    pub allows: Vec<Regex>,
    pub disable_absoluteness_patterns: bool,
    pub disable_abstract_patterns: bool,
    /// textlintのオプションの名前は、他の2つと違い `disabled` で始まる
    pub disabled_predictive_patterns: bool,
}

impl NoAiHypeExpressions {
    fn from_options(options: &Table) -> Result<Self, String> {
        let _ = options;
        todo!("skeleton: not implemented")
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)] // skeleton: not implemented
pub struct NoAiEmphasisPatterns {
    /// 段落・見出しの元の文字列のどこかに当たれば、その段落・見出しを指摘しない
    pub allows: Vec<Regex>,
    pub disable_emoji_emphasis_patterns: bool,
    pub disable_info_patterns: bool,
    pub disable_heading_emphasis_patterns: bool,
}

impl NoAiEmphasisPatterns {
    fn from_options(options: &Table) -> Result<Self, String> {
        let _ = options;
        todo!("skeleton: not implemented")
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)] // skeleton: not implemented
pub struct NoAiColonContinuation {
    /// 段落の元の文字列のどこかに当たれば、その段落を指摘しない
    pub allows: Vec<Regex>,
    pub disable_code_block: bool,
    pub disable_list: bool,
    pub disable_quote: bool,
    pub disable_table: bool,
}

impl NoAiColonContinuation {
    fn from_options(options: &Table) -> Result<Self, String> {
        let _ = options;
        todo!("skeleton: not implemented")
    }
}

/// `disable_*` は、カテゴリごとにパターンを止める。名前はtextlintのオプションと同じで、
/// `disable_voice_guidance` は明確性、`disable_clarity_guidance` は具体性を止める
#[derive(Debug, Default)]
#[allow(dead_code)] // skeleton: not implemented
pub struct AiTechWritingGuideline {
    /// 段落の文字列のどこかに当たれば、その段落を指摘しない
    pub allows: Vec<Regex>,
    pub disable_redundancy_guidance: bool,
    pub disable_voice_guidance: bool,
    pub disable_clarity_guidance: bool,
    pub disable_consistency_guidance: bool,
    pub disable_structure_guidance: bool,
    /// 文書の先頭に、件数とカテゴリ別の内訳の指摘を足す。textlintと違い、既定は `false` である
    pub enable_document_analysis: bool,
}

impl AiTechWritingGuideline {
    fn from_options(options: &Table) -> Result<Self, String> {
        let _ = options;
        todo!("skeleton: not implemented")
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)] // skeleton: not implemented
pub struct NoAiWords {
    /// 一致した範囲の元の文字列に当たれば、指摘しない
    pub allows: Vec<Regex>,
    /// `dictionaryPath` のファイルから読んだ項目
    pub user_words: Vec<UserWord>,
    pub dictionary_mode: DictionaryMode,
}

impl NoAiWords {
    /// `config_dir` は、`dictionaryPath` の相対パスを解決するディレクトリである
    fn from_options(options: &Table, config_dir: &Path) -> Result<Self, String> {
        let _ = (options, config_dir);
        todo!("skeleton: not implemented")
    }
}

/// 組み込みの辞書と利用者の辞書の使い方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)] // skeleton: not implemented
pub enum DictionaryMode {
    /// 両方を使う
    #[default]
    Append,
    /// 利用者の辞書だけを使う
    Override,
}

/// 利用者の辞書の1項目
#[derive(Debug)]
#[allow(dead_code)] // skeleton: not implemented
pub struct UserWord {
    /// 指摘の文面
    pub message: String,
    pub pattern: Vec<Expect<OwnedFields>>,
}

/// `path` の利用者の辞書 (`{ "entries": [{ "message": …, "tokens": [...] }] }` のJSON) を読む。
/// 読めない、JSONとして読めない、形の誤りは、理由を返す
#[allow(dead_code)] // skeleton: not implemented
fn user_dictionary(path: &Path) -> Result<Vec<UserWord>, String> {
    let _ = path;
    todo!("skeleton: not implemented")
}

#[derive(Debug)]
#[allow(dead_code)] // skeleton: not implemented
pub struct NoShortTopicComma {
    /// 文頭から読点の手前までの文字数がこれ以下なら指摘する
    pub max_length: usize,
}

impl Default for NoShortTopicComma {
    fn default() -> Self {
        Self { max_length: 5 }
    }
}

impl NoShortTopicComma {
    fn from_options(options: &Table) -> Result<Self, String> {
        let _ = options;
        todo!("skeleton: not implemented")
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
