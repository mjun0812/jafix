use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use regress::Regex;
use serde::Deserialize;
use toml::{Table, Value};
use url::{Origin, Url};

use crate::js_regex::{self, FixedPart};

use super::RuleId;
use super::english::terms::{TERMS, Term};
use super::jtf_style::JtfRule;
use super::morph::pattern::{Expect, Field, OwnedFields};
use super::prh::Checks;
use super::prh::dictionary::{Dictionary, Source, parse_yaml};
use super::{dictionary_rules_reporting, fix_rules_hitting};

/// 組み込みのルールの設定。有効・無効と、オプションを持つルールのオプションである。
/// 既定では、`preset-ja-technical-writing` の23ルールと修正ルールの3つが有効で、オプションはtextlintの
/// `preset-ja-technical-writing` と同じである。`@textlint-ja/preset-ai-writing` と `preset-ai-words-ja` の
/// 7ルール、利用者の置き換えの2ルール (`prh`・`@textlint-rule/pattern`)、`preset-ja-spacing` の11ルール、
/// `no-mixed-zenkaku-and-hankaku-alphabet`・`no-kangxi-radicals`、`preset-jtf-style` の37ルール、ひらがな系の5ルール、
/// `@textlint-ja/no-insert-dropping-sa`・`@textlint-ja/no-synonyms`、`no-todo`・`date-weekday-mismatch`・`no-dead-link`、
/// `terminology`・`spelling`・`common-misspellings`・`write-good` は無効である
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
    pub prh: Option<Prh>,
    pub textlint_rule_pattern: Option<TextlintRulePattern>,
    pub ja_nakaguro_or_halfwidth_space_between_katakana: bool,
    pub ja_no_space_around_parentheses: bool,
    pub ja_no_space_around_slash: bool,
    pub ja_no_space_between_full_width: bool,
    pub ja_space_between_half_and_full_width: Option<JaSpaceBetweenHalfAndFullWidth>,
    pub ja_space_after_exclamation: bool,
    pub ja_space_after_question: bool,
    pub ja_space_around_code: Option<JaSpaceAround>,
    pub ja_space_around_emphasis: Option<JaSpaceAround>,
    pub ja_space_around_link: Option<JaSpaceAround>,
    pub ja_space_around_strong: Option<JaSpaceAround>,
    pub no_mixed_zenkaku_and_hankaku_alphabet: Option<NoMixedZenkakuAndHankakuAlphabet>,
    pub no_kangxi_radicals: bool,
    pub jtf_style: JtfStyle,
    pub ja_hiragana_hojodoushi: Option<JaHiragana>,
    pub ja_hiragana_keishikimeishi: bool,
    pub ja_keishikimeishi: Option<JaKeishikimeishi>,
    pub ja_hiragana_fukushi: Option<JaHiragana>,
    pub ja_hiragana_daimeishi: bool,
    pub no_insert_dropping_sa: bool,
    pub no_synonyms: Option<NoSynonyms>,
    pub no_todo: bool,
    pub date_weekday_mismatch: Option<DateWeekdayMismatch>,
    pub no_dead_link: Option<NoDeadLink>,
    pub terminology: Option<Terminology>,
    pub spelling: Option<Spelling>,
    pub common_misspellings: Option<CommonMisspellings>,
    pub write_good: Option<WriteGood>,
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
            prh: None,
            textlint_rule_pattern: None,
            ja_nakaguro_or_halfwidth_space_between_katakana: false,
            ja_no_space_around_parentheses: false,
            ja_no_space_around_slash: false,
            ja_no_space_between_full_width: false,
            ja_space_between_half_and_full_width: None,
            ja_space_after_exclamation: false,
            ja_space_after_question: false,
            ja_space_around_code: None,
            ja_space_around_emphasis: None,
            ja_space_around_link: None,
            ja_space_around_strong: None,
            no_mixed_zenkaku_and_hankaku_alphabet: None,
            no_kangxi_radicals: false,
            jtf_style: JtfStyle::default(),
            ja_hiragana_hojodoushi: None,
            ja_hiragana_keishikimeishi: false,
            ja_keishikimeishi: None,
            ja_hiragana_fukushi: None,
            ja_hiragana_daimeishi: false,
            no_insert_dropping_sa: false,
            no_synonyms: None,
            no_todo: false,
            date_weekday_mismatch: None,
            no_dead_link: None,
            terminology: None,
            spelling: None,
            common_misspellings: None,
            write_good: None,
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
                .ok_or_else(|| format!("`{key}` は組み込みのルールのIDでもpresetの名前でもない"))?;
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
        // 利用者の辞書を直しても組み込みのルールどうしの不正は残るので、先に出す
        builtin.reject_opposite_rules()?;
        builtin.reject_opposite_replacements()?;
        Ok(builtin)
    }

    /// `OPPOSITE_RULES` の組のうち、両方のルールが有効なものがあれば、最初の組の理由を返す。
    /// すべてのキーを当てた後の有効・無効とオプションで判定する
    fn reject_opposite_rules(&self) -> Result<(), String> {
        for pair in &OPPOSITE_RULES {
            let Some(demand) = (pair.demand)(self) else {
                continue;
            };
            if (pair.fix_rule_enabled)(self) {
                return Err(format!(
                    "`{}`: {demand}、`{}` は{}。`{} = false` か `{} = false` にする",
                    pair.rule.name(),
                    pair.fix_rule.name(),
                    pair.fixes,
                    toml_key(pair.fix_rule.name()),
                    toml_key(pair.rule.name())
                ));
            }
        }
        Ok(())
    }

    /// 利用者の置き換えの固定の部分に、有効な組み込みの修正ルールが直す箇所か、辞書で別の書き方にするよう求める有効な
    /// 組み込みの指摘ルール (`DICTIONARY_REPORT_RULES`) が指摘する箇所があれば、最初の置き換えの理由を返す。
    /// すべてのキーを当てた後の有効・無効で判定する
    fn reject_opposite_replacements(&self) -> Result<(), String> {
        let mut replacements = Vec::new();
        if let Some(rule) = &self.prh {
            replacements.extend(rule.checked_replacements());
        }
        if let Some(rule) = &self.textlint_rule_pattern {
            replacements.extend(rule.checked_replacements());
        }
        if let Some(rule) = &self.terminology {
            replacements.extend(rule.checked_replacements());
        }
        let parts: Vec<&FixedPart> = replacements
            .iter()
            .flat_map(|replacement| &replacement.parts)
            .collect();
        let mut reported = dictionary_rules_reporting(&parts, self).into_iter();
        for replacement in &replacements {
            let fixes: Vec<RuleId> = replacement
                .parts
                .iter()
                .flat_map(|part| fix_rules_hitting(part, self))
                .collect();
            let reports: Vec<RuleId> = reported
                .by_ref()
                .take(replacement.parts.len())
                .flatten()
                .collect();
            if let Some(reason) = opposite_reason(&fixes, &reports) {
                return Err(format!(
                    "`{}`: {}は {reason}、{}",
                    replacement.rule.name(),
                    replacement.shown,
                    replacement.remedy
                ));
            }
        }
        Ok(())
    }

    /// `preset` のルールを、`value` が `true` ならpresetの既定の有効・無効とオプションにし、`false` ならすべて無効にする。
    /// `value` が `true` でも `false` でもなければ理由を返す
    fn apply_preset(
        &mut self,
        preset: Preset,
        value: &Value,
        config_dir: &Path,
    ) -> Result<(), String> {
        let enabled = value.as_bool().ok_or("値は `true` か `false` で書く")?;
        let defaults = Table::new();
        for &(id, default_enabled) in preset.rules() {
            let options = (enabled && default_enabled).then_some(&defaults);
            self.set(id, options, config_dir)?;
        }
        Ok(())
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
            RuleId::Prh => {
                self.prh = options
                    .map(|options| Prh::from_options(options, config_dir))
                    .transpose()?;
            }
            RuleId::TextlintRulePattern => {
                self.textlint_rule_pattern =
                    options.map(TextlintRulePattern::from_options).transpose()?;
            }
            RuleId::JaNakaguroOrHalfwidthSpaceBetweenKatakana => {
                self.ja_nakaguro_or_halfwidth_space_between_katakana = enabled(options)?;
            }
            RuleId::JaNoSpaceAroundParentheses => {
                self.ja_no_space_around_parentheses = enabled(options)?;
            }
            RuleId::JaNoSpaceAroundSlash => self.ja_no_space_around_slash = enabled(options)?,
            RuleId::JaNoSpaceBetweenFullWidth => {
                self.ja_no_space_between_full_width = enabled(options)?;
            }
            RuleId::JaSpaceBetweenHalfAndFullWidth => {
                self.ja_space_between_half_and_full_width = options
                    .map(JaSpaceBetweenHalfAndFullWidth::from_options)
                    .transpose()?;
            }
            RuleId::JaSpaceAfterExclamation => self.ja_space_after_exclamation = enabled(options)?,
            RuleId::JaSpaceAfterQuestion => self.ja_space_after_question = enabled(options)?,
            RuleId::JaSpaceAroundCode => {
                self.ja_space_around_code = options.map(JaSpaceAround::from_options).transpose()?;
            }
            RuleId::JaSpaceAroundEmphasis => {
                self.ja_space_around_emphasis =
                    options.map(JaSpaceAround::from_options).transpose()?;
            }
            RuleId::JaSpaceAroundLink => {
                self.ja_space_around_link = options.map(JaSpaceAround::from_options).transpose()?;
            }
            RuleId::JaSpaceAroundStrong => {
                self.ja_space_around_strong =
                    options.map(JaSpaceAround::from_options).transpose()?;
            }
            RuleId::NoMixedZenkakuAndHankakuAlphabet => {
                self.no_mixed_zenkaku_and_hankaku_alphabet = options
                    .map(NoMixedZenkakuAndHankakuAlphabet::from_options)
                    .transpose()?;
            }
            RuleId::NoKangxiRadicals => self.no_kangxi_radicals = enabled(options)?,
            RuleId::JtfStyle(rule) => self.jtf_style.set(rule, options)?,
            RuleId::JaHiraganaHojodoushi => {
                self.ja_hiragana_hojodoushi = options
                    .map(|options| JaHiragana::from_options(options, config_dir))
                    .transpose()?;
            }
            RuleId::JaHiraganaKeishikimeishi => self.ja_hiragana_keishikimeishi = enabled(options)?,
            RuleId::JaKeishikimeishi => {
                self.ja_keishikimeishi = options.map(JaKeishikimeishi::from_options).transpose()?;
            }
            RuleId::JaHiraganaFukushi => {
                self.ja_hiragana_fukushi = options
                    .map(|options| JaHiragana::from_options(options, config_dir))
                    .transpose()?;
            }
            RuleId::JaHiraganaDaimeishi => self.ja_hiragana_daimeishi = enabled(options)?,
            RuleId::NoInsertDroppingSa => self.no_insert_dropping_sa = enabled(options)?,
            RuleId::NoSynonyms => {
                self.no_synonyms = options.map(NoSynonyms::from_options).transpose()?;
            }
            RuleId::NoTodo => self.no_todo = enabled(options)?,
            RuleId::DateWeekdayMismatch => {
                self.date_weekday_mismatch =
                    options.map(DateWeekdayMismatch::from_options).transpose()?;
            }
            RuleId::NoDeadLink => {
                self.no_dead_link = options
                    .map(|options| NoDeadLink::from_options(options, config_dir))
                    .transpose()?;
            }
            RuleId::Terminology => {
                self.terminology = options
                    .map(|options| Terminology::from_options(options, config_dir))
                    .transpose()?;
            }
            RuleId::Spelling => self.spelling = options.map(Spelling::from_options).transpose()?,
            RuleId::CommonMisspellings => {
                self.common_misspellings =
                    options.map(CommonMisspellings::from_options).transpose()?;
            }
            RuleId::WriteGood => {
                self.write_good = options.map(WriteGood::from_options).transpose()?;
            }
        }
        Ok(())
    }
}

/// 組み込みのルールどうしで、修正ルールと逆の書き方を求める組。両方が有効なら、hookが直した箇所を
/// もう一方のルールが指摘し、agentが元に戻す往復になるので、設定の不正にする
struct OppositeRules {
    /// 設定によって、`fix_rule` と逆の書き方を求めるルール
    rule: RuleId,
    /// `rule` が有効で、その設定が逆の書き方を求めるなら、設定と求める書き方の文面を返す
    /// (`` `space` の "always" は日本語と英数字の間に空白を求め ``)
    demand: fn(&Builtin) -> Option<String>,
    fix_rule: RuleId,
    fix_rule_enabled: fn(&Builtin) -> bool,
    /// `fix_rule` が直すことの文面 (`その空白を削除する`)
    fixes: &'static str,
}

/// 組み込みのルールどうしの逆の書き方の組。判定はこの順に行い、最初に当たった組の理由を出す
const OPPOSITE_RULES: [OppositeRules; 5] = [
    OppositeRules {
        rule: RuleId::JaSpaceBetweenHalfAndFullWidth,
        demand: half_and_full_width_demands_space,
        fix_rule: RuleId::SpaceBetweenJaAndAlnum,
        fix_rule_enabled: |builtin| builtin.space_between_ja_and_alnum,
        fixes: "その空白を削除する",
    },
    OppositeRules {
        rule: RuleId::JtfStyle(JtfRule::R4_3_1),
        demand: |builtin| {
            builtin
                .jtf_style
                .enabled(JtfRule::R4_3_1)
                .then(|| "日本語の隣の丸括弧を全角にすることを求め".to_owned())
        },
        fix_rule: RuleId::FullwidthParen,
        fix_rule_enabled: |builtin| builtin.fullwidth_paren,
        fixes: "全角の丸括弧を半角にする",
    },
    OppositeRules {
        rule: RuleId::JtfStyle(JtfRule::R3_3),
        demand: |builtin| {
            (builtin.jtf_style.enabled(JtfRule::R3_3)
                && builtin.jtf_style.parentheses.require_outside_half_parentheses)
                .then(|| {
                    "`requireOutsideHalfParentheses = true` は日本語と半角の括弧の間に半角空白を求め"
                        .to_owned()
                })
        },
        fix_rule: RuleId::SpaceBetweenJaAndParen,
        fix_rule_enabled: |builtin| builtin.space_between_ja_and_paren,
        fixes: "その空白を削除する",
    },
    OppositeRules {
        rule: RuleId::JaSpaceBetweenHalfAndFullWidth,
        demand: half_and_full_width_demands_space,
        fix_rule: RuleId::JtfStyle(JtfRule::R3_1_1),
        fix_rule_enabled: |builtin| builtin.jtf_style.enabled(JtfRule::R3_1_1),
        fixes: "その空白を削除する",
    },
    OppositeRules {
        rule: RuleId::NoMixedZenkakuAndHankakuAlphabet,
        demand: |builtin| {
            builtin
                .no_mixed_zenkaku_and_hankaku_alphabet
                .as_ref()
                .filter(|rule| rule.prefer == AlphabetWidth::Full)
                .map(|_| "`prefer` の \"全角\" は英字を全角にすることを求め".to_owned())
        },
        fix_rule: RuleId::JtfStyle(JtfRule::R2_1_9),
        fix_rule_enabled: |builtin| builtin.jtf_style.enabled(JtfRule::R2_1_9),
        fixes: "全角の英字を半角にする",
    },
];

/// 逆の書き方の理由の最後の文に書く、ルールのIDの設定のキー。TOMLの裸のキー (英数字・`-`・`_` だけ) に書けない
/// IDは引用符で囲む (`fullwidth-paren`、`"4.3.1.丸かっこ（）"`)
fn toml_key(name: &str) -> Cow<'_, str> {
    if name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(format!("\"{name}\""))
    }
}

/// `ja-space-between-half-and-full-width` が有効で、`space` が英字か数字の隣に空白を求めるなら、その文面を返す
fn half_and_full_width_demands_space(builtin: &Builtin) -> Option<String> {
    let space = &builtin.ja_space_between_half_and_full_width.as_ref()?.space;
    space.demands_space().then(|| {
        format!(
            "`space` の {} は日本語と英数字の間に空白を求め",
            space.written
        )
    })
}

/// `[builtin]` に名前で書けるpreset。名前はtextlintの設定に書く名前と同じである
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    JaTechnicalWriting,
    AiWriting,
    AiWordsJa,
    JaSpacing,
    JtfStyle,
}

impl Preset {
    pub const ALL: [Preset; 5] = [
        Preset::JaTechnicalWriting,
        Preset::AiWriting,
        Preset::AiWordsJa,
        Preset::JaSpacing,
        Preset::JtfStyle,
    ];

    /// `[builtin]` のキーに使う名前
    pub fn name(self) -> &'static str {
        match self {
            Preset::JaTechnicalWriting => "preset-ja-technical-writing",
            Preset::AiWriting => "@textlint-ja/preset-ai-writing",
            Preset::AiWordsJa => "preset-ai-words-ja",
            Preset::JaSpacing => "preset-ja-spacing",
            Preset::JtfStyle => "preset-jtf-style",
        }
    }

    fn from_name(name: &str) -> Option<Preset> {
        Preset::ALL.into_iter().find(|preset| preset.name() == name)
    }

    /// presetが束ねるルールと、presetの既定で有効か。`preset-ja-technical-writing` は、jafix独自の修正ルール3つを含まない
    pub fn rules(self) -> &'static [(RuleId, bool)] {
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
            // presetの既定のオプションは、どれもルールの既定値と同じである
            Preset::JaSpacing => &[
                (RuleId::JaNakaguroOrHalfwidthSpaceBetweenKatakana, true),
                (RuleId::JaNoSpaceAroundParentheses, true),
                (RuleId::JaNoSpaceAroundSlash, true),
                (RuleId::JaNoSpaceBetweenFullWidth, true),
                (RuleId::JaSpaceBetweenHalfAndFullWidth, true),
                (RuleId::JaSpaceAfterExclamation, true),
                (RuleId::JaSpaceAfterQuestion, true),
                (RuleId::JaSpaceAroundCode, true),
                (RuleId::JaSpaceAroundEmphasis, false),
                (RuleId::JaSpaceAroundLink, true),
                (RuleId::JaSpaceAroundStrong, false),
            ],
            // presetの既定のオプションは、どれもルールの既定値と同じである。辞書の3ルールと `2.1.2` は既定で無効である
            Preset::JtfStyle => &[
                (RuleId::JtfStyle(JtfRule::R1_1_1), true),
                (RuleId::JtfStyle(JtfRule::R1_1_2), true),
                (RuleId::JtfStyle(JtfRule::R1_1_3), true),
                (RuleId::JtfStyle(JtfRule::R1_1_5), true),
                (RuleId::JtfStyle(JtfRule::R1_2_1), true),
                (RuleId::JtfStyle(JtfRule::R1_2_2), true),
                (RuleId::JtfStyle(JtfRule::R2_1_2), false),
                (RuleId::JtfStyle(JtfRule::R2_1_5), false),
                (RuleId::JtfStyle(JtfRule::R2_1_6), false),
                (RuleId::JtfStyle(JtfRule::R2_1_8), true),
                (RuleId::JtfStyle(JtfRule::R2_1_9), true),
                (RuleId::JtfStyle(JtfRule::R2_1_10), true),
                (RuleId::JtfStyle(JtfRule::R2_2_1), false),
                (RuleId::JtfStyle(JtfRule::R2_2_2), true),
                (RuleId::JtfStyle(JtfRule::R2_2_3), true),
                (RuleId::JtfStyle(JtfRule::R3_1_1), true),
                (RuleId::JtfStyle(JtfRule::R3_1_2), true),
                (RuleId::JtfStyle(JtfRule::R3_2), true),
                (RuleId::JtfStyle(JtfRule::R3_3), true),
                (RuleId::JtfStyle(JtfRule::R4_1_1), true),
                (RuleId::JtfStyle(JtfRule::R4_1_3), true),
                (RuleId::JtfStyle(JtfRule::R4_2_1), true),
                (RuleId::JtfStyle(JtfRule::R4_2_2), true),
                (RuleId::JtfStyle(JtfRule::R4_2_4), true),
                (RuleId::JtfStyle(JtfRule::R4_2_5), true),
                (RuleId::JtfStyle(JtfRule::R4_2_6), true),
                (RuleId::JtfStyle(JtfRule::R4_2_7), true),
                (RuleId::JtfStyle(JtfRule::R4_2_8), true),
                (RuleId::JtfStyle(JtfRule::R4_2_9), true),
                (RuleId::JtfStyle(JtfRule::R4_3_1), true),
                (RuleId::JtfStyle(JtfRule::R4_3_2), true),
                (RuleId::JtfStyle(JtfRule::R4_3_3), true),
                (RuleId::JtfStyle(JtfRule::R4_3_4), true),
                (RuleId::JtfStyle(JtfRule::R4_3_5), true),
                (RuleId::JtfStyle(JtfRule::R4_3_6), true),
                (RuleId::JtfStyle(JtfRule::R4_3_7), true),
                (RuleId::JtfStyle(JtfRule::R4_3_8), true),
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

pub(crate) fn unknown_option(key: &str) -> String {
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

pub(crate) fn non_empty_string<'v>(key: &str, value: &'v Value) -> Result<&'v str, String> {
    Some(string(key, value)?)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("`{key}` は空でない文字列で書く"))
}

pub(crate) fn non_empty_strings(key: &str, value: &Value) -> Result<Vec<String>, String> {
    let values = strings(key, value)?;
    if values.iter().any(|value| value.is_empty()) {
        return Err(format!("`{key}` の要素は空でない文字列で書く"));
    }
    Ok(values.into_iter().map(str::to_owned).collect())
}

pub(crate) fn patterns(key: &str, value: &Value) -> Result<Vec<Regex>, String> {
    non_empty_strings(key, value)?
        .iter()
        .map(|value| js_regex::pattern(value).map_err(|reason| format!("`{key}`: {reason}")))
        .collect()
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
            .push(js_regex::pattern("Yahoo!").expect("文字列のパターンは読める"));
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
                .map(|source| js_regex::pattern(source).expect("既定のパターンは読める"))
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
                .map(|source| {
                    js_regex::pattern(source).expect("決まった言い回しのパターンは読める")
                }),
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
pub struct NoAiListFormatting {
    /// リスト項目の元の文字列のどこかに当たれば、その項目を指摘しない
    pub allows: Vec<Regex>,
    pub disable_bold_list_items: bool,
    pub disable_emoji_list_items: bool,
}

impl NoAiListFormatting {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allows" => rule.allows = patterns(key, value)?,
                "disableBoldListItems" => rule.disable_bold_list_items = boolean(key, value)?,
                "disableEmojiListItems" => rule.disable_emoji_list_items = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug, Default)]
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
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allows" => rule.allows = patterns(key, value)?,
                "disableAbsolutenessPatterns" => {
                    rule.disable_absoluteness_patterns = boolean(key, value)?;
                }
                "disableAbstractPatterns" => rule.disable_abstract_patterns = boolean(key, value)?,
                "disabledPredictivePatterns" => {
                    rule.disabled_predictive_patterns = boolean(key, value)?;
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug, Default)]
pub struct NoAiEmphasisPatterns {
    /// 段落・見出しの元の文字列のどこかに当たれば、その段落・見出しを指摘しない
    pub allows: Vec<Regex>,
    pub disable_emoji_emphasis_patterns: bool,
    pub disable_info_patterns: bool,
    pub disable_heading_emphasis_patterns: bool,
}

impl NoAiEmphasisPatterns {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allows" => rule.allows = patterns(key, value)?,
                "disableEmojiEmphasisPatterns" => {
                    rule.disable_emoji_emphasis_patterns = boolean(key, value)?;
                }
                "disableInfoPatterns" => rule.disable_info_patterns = boolean(key, value)?,
                "disableHeadingEmphasisPatterns" => {
                    rule.disable_heading_emphasis_patterns = boolean(key, value)?;
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug, Default)]
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
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allows" => rule.allows = patterns(key, value)?,
                "disableCodeBlock" => rule.disable_code_block = boolean(key, value)?,
                "disableList" => rule.disable_list = boolean(key, value)?,
                "disableQuote" => rule.disable_quote = boolean(key, value)?,
                "disableTable" => rule.disable_table = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `disable_*` は、カテゴリごとにパターンを止める。名前はtextlintのオプションと同じで、
/// `disable_voice_guidance` は明確性、`disable_clarity_guidance` は具体性を止める
#[derive(Debug, Default)]
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
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allows" => rule.allows = patterns(key, value)?,
                "disableRedundancyGuidance" => {
                    rule.disable_redundancy_guidance = boolean(key, value)?;
                }
                "disableVoiceGuidance" => rule.disable_voice_guidance = boolean(key, value)?,
                "disableClarityGuidance" => rule.disable_clarity_guidance = boolean(key, value)?,
                "disableConsistencyGuidance" => {
                    rule.disable_consistency_guidance = boolean(key, value)?;
                }
                "disableStructureGuidance" => {
                    rule.disable_structure_guidance = boolean(key, value)?;
                }
                "enableDocumentAnalysis" => rule.enable_document_analysis = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug, Default)]
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
        let mut rule = Self::default();
        let mut dictionary_path = None;
        for (key, value) in options {
            match key.as_str() {
                "allows" => rule.allows = patterns(key, value)?,
                "dictionaryPath" => dictionary_path = Some(non_empty_string(key, value)?),
                "dictionaryMode" => {
                    rule.dictionary_mode = match string(key, value)? {
                        "append" => DictionaryMode::Append,
                        "override" => DictionaryMode::Override,
                        _ => {
                            return Err(format!("`{key}` は \"append\" か \"override\" で書く"));
                        }
                    }
                }
                _ => return Err(unknown_option(key)),
            }
        }
        match dictionary_path {
            Some(path) => {
                let path = resolve_path("dictionaryPath", path, config_dir)?;
                rule.user_words = user_dictionary(&path).map_err(|reason| {
                    format!("`dictionaryPath` の {}: {reason}", path.display())
                })?;
            }
            None if rule.dictionary_mode == DictionaryMode::Override => {
                return Err(
                    "`dictionaryMode` が \"override\" のときは `dictionaryPath` を書く".to_owned(),
                );
            }
            None => {}
        }
        Ok(rule)
    }
}

/// `path` が `~/` で始まればホームディレクトリに展開し、相対パスなら `config_dir` から解決する。`key` は理由に書くオプションの名前
pub(crate) fn resolve_path(key: &str, path: &str, config_dir: &Path) -> Result<PathBuf, String> {
    match path.strip_prefix("~/") {
        Some(rest) => Ok(std::env::home_dir()
            .ok_or_else(|| {
                format!("ホームディレクトリが分からないため `{key}` の `~/` を展開できない")
            })?
            .join(rest)),
        None => Ok(config_dir.join(path)),
    }
}

/// 組み込みの辞書と利用者の辞書の使い方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DictionaryMode {
    /// 両方を使う
    #[default]
    Append,
    /// 利用者の辞書だけを使う
    Override,
}

/// 利用者の辞書の1項目
#[derive(Debug)]
pub struct UserWord {
    /// 指摘の文面
    pub message: String,
    pub pattern: Vec<Expect<OwnedFields>>,
}

#[derive(Deserialize)]
struct RawDictionary {
    entries: Vec<RawEntry>,
}

/// 項目の `message` と `tokens` 以外のキーは、textlintと同じく読まない
#[derive(Deserialize)]
struct RawEntry {
    message: String,
    tokens: Vec<serde_json::Map<String, serde_json::Value>>,
}

/// `path` の利用者の辞書 (`{ "entries": [{ "message": …, "tokens": [...] }] }` のJSON) を読む。
/// 読めない、JSONとして読めない、形の誤りは、理由を返す
fn user_dictionary(path: &Path) -> Result<Vec<UserWord>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("読めない: {e}"))?;
    let raw: RawDictionary = serde_json::from_str(&text).map_err(|e| match e.classify() {
        serde_json::error::Category::Data => format!("辞書の形でない: {e}"),
        _ => format!("JSONとして読めない: {e}"),
    })?;
    raw.entries
        .into_iter()
        .enumerate()
        .map(|(i, entry)| {
            user_word(entry).map_err(|reason| format!("`entries` の{}番目: {reason}", i + 1))
        })
        .collect()
}

fn user_word(entry: RawEntry) -> Result<UserWord, String> {
    if entry.message.is_empty() {
        return Err("`message` は空でない文字列で書く".to_owned());
    }
    if entry.tokens.is_empty() {
        return Err("`tokens` は空でない配列で書く".to_owned());
    }
    let pattern = entry
        .tokens
        .iter()
        .enumerate()
        .map(|(j, token)| {
            expect(token).map_err(|reason| format!("`tokens` の{}番目: {reason}", j + 1))
        })
        .collect::<Result<_, _>>()?;
    Ok(UserWord {
        message: entry.message,
        pattern,
    })
}

/// 期待するトークンの表を読む。`_` で始まるキーは、textlintと同じく照合に使わず、`_skippable` だけを読む
fn expect(
    token: &serde_json::Map<String, serde_json::Value>,
) -> Result<Expect<OwnedFields>, String> {
    let mut expect = Expect {
        fields: Vec::new(),
        skippable: false,
    };
    for (key, value) in token {
        if key == "_skippable" {
            expect.skippable = value
                .as_bool()
                .ok_or_else(|| format!("`{key}` は `true` か `false` で書く"))?;
            continue;
        }
        if key.starts_with('_') {
            continue;
        }
        // `word_id`・`word_type`・`word_position` もkuromojiの項目だが、jafixのトークンは持たない
        let field = Field::from_kuromoji_name(key)
            .ok_or_else(|| format!("`{key}` はjafixのトークンの項目ではない"))?;
        let values = match value {
            serde_json::Value::String(value) => vec![value.clone()],
            serde_json::Value::Array(values) => values
                .iter()
                .map(|value| value.as_str().map(str::to_owned))
                .collect::<Option<_>>()
                .ok_or_else(|| format!("`{key}` は文字列か文字列の配列で書く"))?,
            _ => return Err(format!("`{key}` は文字列か文字列の配列で書く")),
        };
        expect.fields.push((field, values));
    }
    Ok(expect)
}

#[derive(Debug)]
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
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "maxLength" => rule.max_length = integer(key, value, 1)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug)]
pub struct Prh {
    /// `rulePaths` の辞書、`ruleContents` の辞書の順に読んでまとめた辞書
    pub dictionary: Dictionary,
    /// 辞書の出どころの、不正の理由に書く名前 (``rulePaths の `/path/prh.yml` `` か `ruleContents の2番目`)。
    /// `dictionary` に渡した出どころの一覧と同じ順に並ぶ
    pub source_names: Vec<String>,
    pub checks: Checks,
}

impl Prh {
    /// `config_dir` は、`rulePaths` の相対パスと `ruleContents` の中の `imports` を解決するディレクトリである。
    /// `rulePaths` と `ruleContents` に辞書が1つも無ければ理由を返す
    fn from_options(options: &Table, config_dir: &Path) -> Result<Self, String> {
        let mut checks = Checks::default();
        let mut rule_paths = Vec::new();
        let mut rule_contents = Vec::new();
        for (key, value) in options {
            match key.as_str() {
                "rulePaths" => rule_paths = non_empty_strings(key, value)?,
                "ruleContents" => {
                    rule_contents = strings(key, value)?
                        .into_iter()
                        .map(str::to_owned)
                        .collect();
                }
                "checkLink" => checks.link = boolean(key, value)?,
                "checkBlockQuote" => checks.block_quote = boolean(key, value)?,
                "checkEmphasis" => checks.emphasis = boolean(key, value)?,
                "checkHeader" => checks.header = boolean(key, value)?,
                "checkParagraph" => checks.paragraph = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        if rule_paths.is_empty() && rule_contents.is_empty() {
            return Err("`rulePaths` か `ruleContents` に辞書を1つ以上書く".to_owned());
        }
        let mut sources = Vec::new();
        let mut source_names = Vec::new();
        for path in &rule_paths {
            let path = resolve_path("rulePaths", path, config_dir)?;
            source_names.push(format!("rulePaths の `{}`", path.display()));
            sources.push(Source::Path(path));
        }
        for (i, yaml) in rule_contents.into_iter().enumerate() {
            source_names.push(format!("ruleContents の{}番目", i + 1));
            sources.push(Source::Contents {
                yaml,
                base: config_dir.to_path_buf(),
            });
        }
        let dictionary = Dictionary::load(&sources)
            .map_err(|e| format!("{}: {}", source_names[e.source], e.reason))?;
        Ok(Self {
            dictionary,
            source_names,
            checks,
        })
    }

    /// 辞書のルールの `expected` の、捕捉グループの参照を除いた部分
    fn checked_replacements(&self) -> impl Iterator<Item = CheckedReplacement> {
        self.dictionary.rules().iter().map(|rule| {
            CheckedReplacement::new(
                RuleId::Prh,
                format!(
                    "{}: `expected` の「{}」",
                    self.source_names[rule.source], rule.expected
                ),
                "このルールを辞書から外す",
                rule.fixed_parts(),
            )
        })
    }
}

/// 逆の置き換えかを確かめる、利用者の置き換えの1つ
struct CheckedReplacement {
    rule: RuleId,
    /// 理由の先頭に書く、置き換えの出どころと置き換え後の文字列
    shown: String,
    /// 理由の最後に書く、置き換えの直し方
    remedy: &'static str,
    /// 置き換え後の文字列のうち、捕捉グループの参照を除いた空でない部分
    parts: Vec<FixedPart>,
}

impl CheckedReplacement {
    fn new(rule: RuleId, shown: String, remedy: &'static str, parts: Vec<FixedPart>) -> Self {
        Self {
            rule,
            shown,
            remedy,
            parts,
        }
    }
}

/// `replacement` を、`$1`〜`$9` の参照で区切った各部分
fn fixed_parts(replacement: &str) -> Vec<FixedPart> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let bytes = replacement.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'$' && bytes.get(i + 1).is_some_and(|b| matches!(b, b'1'..=b'9')) {
            parts.push(Some(&replacement[start..i]));
            parts.push(None);
            i += 2;
            start = i;
        } else {
            i += 1;
        }
    }
    parts.push(Some(&replacement[start..]));
    js_regex::fixed_parts(parts)
}

/// 置き換えの固定の部分に当たった組み込みの修正ルール `fixes` と、固定の部分を指摘した辞書の指摘ルール `reports` から、
/// 置き換えが逆である理由を作る。理由は「どちらかを無効にするか」までで、呼び出し側が置き換えの直し方を続ける
fn opposite_reason(fixes: &[RuleId], reports: &[RuleId]) -> Option<String> {
    let mut rules: Vec<RuleId> = Vec::new();
    for &rule in fixes.iter().chain(reports) {
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    }
    let demand = match (fixes.is_empty(), reports.is_empty()) {
        (true, true) => return None,
        (false, true) => "が直す文字を含む",
        (true, false) => "が別の書き方にするよう指摘する文字を含む",
        (false, false) => "が直すか、別の書き方にするよう指摘する文字を含む",
    };
    let quoted: Vec<String> = rules
        .iter()
        .map(|rule| format!("`{}`", rule.name()))
        .collect();
    let disabled: Vec<String> = rules
        .iter()
        .map(|&rule| match rule {
            // 指摘したのは組み込みの辞書なので、利用者の `terms` を残して組み込みの辞書だけを切る
            RuleId::Terminology => "`terminology` の `defaultTerms = false`".to_owned(),
            _ => format!("`{} = false`", toml_key(rule.name())),
        })
        .collect();
    Some(format!(
        "{} {demand}。{} にするか",
        quoted.join("・"),
        disabled.join("・")
    ))
}

#[derive(Debug)]
pub struct TextlintRulePattern {
    /// `patterns` の要素。書いた順に並ぶ
    pub patterns: Vec<UserPattern>,
}

/// `@textlint-rule/pattern` の `patterns` の1つの要素
#[derive(Debug)]
pub struct UserPattern {
    pub message: String,
    pub pattern: Regex,
    /// 置き換え後の文字列。`$1`〜`$9` で捕捉グループを参照する。無ければ、箇所は指摘になる
    pub replace: Option<String>,
    /// 一致した文字列のどれかに当たれば、その一致を捨てる
    pub allows: Vec<Regex>,
    /// 本文の範囲がこの型のどれかの中にあれば、その範囲の一致を捨てる
    pub allow_node_types: Vec<PatternNodeType>,
}

/// `@textlint-rule/pattern` の `allowNodeTypes` に書けるtextlintのnodeの型。本文の範囲の祖先になりうる型だけを持つ
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternNodeType {
    Paragraph,
    Header,
    BlockQuote,
    /// `List` と `ListItem`
    ListItem,
    /// `Table`・`TableRow`・`TableCell`
    Table,
    Emphasis,
    Strong,
    Delete,
    Link,
    LinkReference,
}

/// 本文の範囲の祖先にならない型 (`Document`・`Str`・`Code` など) と知らない名前は `None` である
fn pattern_node_type(name: &str) -> Option<PatternNodeType> {
    Some(match name {
        "Paragraph" => PatternNodeType::Paragraph,
        "Header" => PatternNodeType::Header,
        "BlockQuote" => PatternNodeType::BlockQuote,
        "List" | "ListItem" => PatternNodeType::ListItem,
        "Table" | "TableRow" | "TableCell" => PatternNodeType::Table,
        "Emphasis" => PatternNodeType::Emphasis,
        "Strong" => PatternNodeType::Strong,
        "Delete" => PatternNodeType::Delete,
        "Link" => PatternNodeType::Link,
        "LinkReference" => PatternNodeType::LinkReference,
        _ => return None,
    })
}

impl TextlintRulePattern {
    /// `patterns` が無いか空なら理由を返す
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut patterns = Vec::new();
        for (key, value) in options {
            match key.as_str() {
                "patterns" => {
                    patterns = value
                        .as_array()
                        .ok_or("`patterns` は表の配列で書く")?
                        .iter()
                        .enumerate()
                        .map(|(i, value)| {
                            user_pattern(value)
                                .map_err(|reason| format!("`patterns` の{}番目: {reason}", i + 1))
                        })
                        .collect::<Result<_, _>>()?;
                }
                _ => return Err(unknown_option(key)),
            }
        }
        if patterns.is_empty() {
            return Err("`patterns` に要素を1つ以上書く".to_owned());
        }
        Ok(Self { patterns })
    }

    /// `replace` のある要素の、`replace` を `$1`〜`$9` で区切った各部分
    fn checked_replacements(&self) -> impl Iterator<Item = CheckedReplacement> {
        self.patterns.iter().enumerate().filter_map(|(i, pattern)| {
            let replace = pattern.replace.as_ref()?;
            Some(CheckedReplacement::new(
                RuleId::TextlintRulePattern,
                format!("`patterns` の{}番目: `replace` の「{replace}」", i + 1),
                "`replace` を変える",
                fixed_parts(replace),
            ))
        })
    }
}

fn user_pattern(value: &Value) -> Result<UserPattern, String> {
    let table = value.as_table().ok_or("表で書く")?;
    let mut message = None;
    let mut pattern = None;
    let mut replace = None;
    let mut allows = Vec::new();
    let mut allow_node_types = Vec::new();
    for (key, value) in table {
        match key.as_str() {
            "message" => message = Some(non_empty_string(key, value)?.to_owned()),
            "pattern" => {
                pattern = Some(
                    js_regex::pattern(non_empty_string(key, value)?)
                        .map_err(|reason| format!("`{key}`: {reason}"))?,
                );
            }
            "replace" => replace = Some(string(key, value)?.to_owned()),
            "allows" => {
                allows = non_empty_strings(key, value)?
                    .iter()
                    .map(|allow| {
                        js_regex::pattern(allow).map_err(|reason| format!("`{key}`: {reason}"))
                    })
                    .collect::<Result<_, _>>()?;
            }
            "allowNodeTypes" => {
                allow_node_types = strings(key, value)?
                    .into_iter()
                    .map(|name| {
                        pattern_node_type(name).ok_or_else(|| {
                            format!("`{key}` の `{name}` は本文の範囲の祖先になる型ではない")
                        })
                    })
                    .collect::<Result<_, _>>()?;
            }
            _ => return Err(unknown_option(key)),
        }
    }
    Ok(UserPattern {
        message: message.ok_or("`message` が無い")?,
        pattern: pattern.ok_or("`pattern` が無い")?,
        replace,
        allows,
        allow_node_types,
    })
}

#[derive(Debug, Default)]
pub struct JaSpaceBetweenHalfAndFullWidth {
    pub space: Space,
    /// 段落の直下でない文字列 (見出し・表のセル・引用・リンクの文字列・太字・強調の中) も見る
    pub lint_styled_node: bool,
    /// 判定に使った並びの先頭が、一致の範囲の中 (終わりの位置を含まない) にあれば外す
    pub allows: Vec<Regex>,
}

impl JaSpaceBetweenHalfAndFullWidth {
    /// `space` は `"never"`・`"always"`・`"alphabets"` と `"numbers"` と `"punctuation"` の配列のどれか。
    /// `exceptPunctuation` は受け付けず、知らないオプションとして理由を返す
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "space" => rule.space = Space::from_value(key, value)?,
                "lintStyledNode" => rule.lint_styled_node = boolean(key, value)?,
                "allows" => rule.allows = patterns(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `ja-space-between-half-and-full-width` の `space`。`alphabets` か `numbers` が真なら空白を求め、
/// どちらも偽なら空白を消す
#[derive(Debug)]
pub struct Space {
    pub alphabets: bool,
    pub numbers: bool,
    /// `、` `。` の隣も見る
    pub punctuation: bool,
    /// 設定に書いた値 (`"always"`、`["numbers"]`)。逆の書き方の理由に使う
    pub written: String,
}

impl Default for Space {
    fn default() -> Self {
        Self {
            alphabets: false,
            numbers: false,
            punctuation: false,
            written: "\"never\"".to_owned(),
        }
    }
}

impl Space {
    fn from_value(key: &str, value: &Value) -> Result<Self, String> {
        let invalid = || {
            format!(
                "`{key}` は \"never\"・\"always\"・\"alphabets\" と \"numbers\" と \"punctuation\" の配列のどれかで書く"
            )
        };
        if let Some(name) = value.as_str() {
            let demands = match name {
                "never" => false,
                "always" => true,
                _ => return Err(invalid()),
            };
            return Ok(Self {
                alphabets: demands,
                numbers: demands,
                punctuation: false,
                written: format!("\"{name}\""),
            });
        }
        let names = value
            .as_array()
            .and_then(|values| values.iter().map(Value::as_str).collect::<Option<Vec<_>>>())
            .ok_or_else(invalid)?;
        let mut space = Self {
            written: format!(
                "[{}]",
                names
                    .iter()
                    .map(|name| format!("\"{name}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ..Self::default()
        };
        for name in names {
            match name {
                "alphabets" => space.alphabets = true,
                "numbers" => space.numbers = true,
                "punctuation" => space.punctuation = true,
                _ => return Err(invalid()),
            }
        }
        Ok(space)
    }

    /// 英字か数字の隣に空白を求める
    pub fn demands_space(&self) -> bool {
        self.alphabets || self.numbers
    }
}

/// `ja-space-around-code`・`ja-space-around-link`・`ja-space-around-strong`・`ja-space-around-emphasis` の
/// オプション。`false` の側は空白を消す修正、`true` の側は空白を求める指摘になる
#[derive(Debug, Default)]
pub struct JaSpaceAround {
    pub before: bool,
    pub after: bool,
}

impl JaSpaceAround {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "before" => rule.before = boolean(key, value)?,
                "after" => rule.after = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

#[derive(Debug)]
pub struct NoMixedZenkakuAndHankakuAlphabet {
    pub prefer: AlphabetWidth,
}

impl NoMixedZenkakuAndHankakuAlphabet {
    /// `prefer` は `"全角"` か `"半角"` だけを受け付ける
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self {
            prefer: AlphabetWidth::Half,
        };
        for (key, value) in options {
            match key.as_str() {
                "prefer" => {
                    rule.prefer = match string(key, value)? {
                        "全角" => AlphabetWidth::Full,
                        "半角" => AlphabetWidth::Half,
                        _ => return Err(format!("`{key}` は \"全角\" か \"半角\" で書く")),
                    };
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `preset-jtf-style` の37ルールの有効・無効と、`3.3.かっこ類と隣接する文字の間のスペースの有無` のオプション
#[derive(Debug)]
pub struct JtfStyle {
    /// `JtfRule` の宣言の順に並べた、各ルールが有効か
    enabled: [bool; JtfRule::ALL.len()],
    /// `3.3` のオプション。`3.3` が無効なら既定値のままである
    pub parentheses: JtfParentheses,
}

impl Default for JtfStyle {
    fn default() -> Self {
        Self {
            enabled: [false; JtfRule::ALL.len()],
            parentheses: JtfParentheses::default(),
        }
    }
}

impl JtfStyle {
    pub fn enabled(&self, rule: JtfRule) -> bool {
        self.enabled[rule as usize]
    }

    /// `rule` を、`options` が無ければ無効にし、あればそのオプションで有効にする。`3.3` だけがオプションを持つ
    fn set(&mut self, rule: JtfRule, options: Option<&Table>) -> Result<(), String> {
        self.enabled[rule as usize] = match rule {
            JtfRule::R3_3 => {
                self.parentheses = options
                    .map(JtfParentheses::from_options)
                    .transpose()?
                    .unwrap_or_default();
                options.is_some()
            }
            _ => enabled(options)?,
        };
        Ok(())
    }
}

/// `3.3.かっこ類と隣接する文字の間のスペースの有無` のオプション
#[derive(Debug)]
pub struct JtfParentheses {
    /// `allowOutsideHalfParentheses`。真なら、半角の括弧の外側に接する半角空白を消さない
    pub allow_outside_half_parentheses: bool,
    /// `requireOutsideHalfParentheses`。真なら、日本語に接する半角の括弧の外側に半角空白を求める
    pub require_outside_half_parentheses: bool,
}

impl Default for JtfParentheses {
    fn default() -> Self {
        Self {
            allow_outside_half_parentheses: true,
            require_outside_half_parentheses: false,
        }
    }
}

impl JtfParentheses {
    /// `allowOutsideHalfParentheses = false` と `requireOutsideHalfParentheses = true` の組は、同じ空白を消す修正と
    /// 求める指摘が往復するので、2つのオプションの名前を含む理由を返す
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "allowOutsideHalfParentheses" => {
                    rule.allow_outside_half_parentheses = boolean(key, value)?;
                }
                "requireOutsideHalfParentheses" => {
                    rule.require_outside_half_parentheses = boolean(key, value)?;
                }
                _ => return Err(unknown_option(key)),
            }
        }
        if !rule.allow_outside_half_parentheses && rule.require_outside_half_parentheses {
            return Err(
                "`allowOutsideHalfParentheses = false` は半角の括弧の外側の空白を削除し、\
                 `requireOutsideHalfParentheses = true` は同じ空白を求める。どちらかを外す"
                    .to_owned(),
            );
        }
        Ok(rule)
    }
}

/// `ja-hiragana-hojodoushi` と `ja-hiragana-fukushi` のオプション
#[derive(Debug, Default)]
pub struct JaHiragana {
    /// `rulePath` の辞書。あれば、組み込みの辞書の代わりにこの辞書だけを使う
    pub dictionary: Option<HiraganaDictionary>,
}

impl JaHiragana {
    /// `config_dir` は、`rulePath` の相対パスを解決するディレクトリである
    fn from_options(options: &Table, config_dir: &Path) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "rulePath" => {
                    let path = resolve_path(key, non_empty_string(key, value)?, config_dir)?;
                    rule.dictionary =
                        Some(hiragana_dictionary(&path).map_err(|reason| {
                            format!("`rulePath` の {}: {reason}", path.display())
                        })?);
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `rulePath` の辞書。textlintの `ja-hiragana-hojodoushi` と `ja-hiragana-fukushi` の辞書と同じ形である
#[derive(Debug)]
pub struct HiraganaDictionary {
    /// 指摘の文面の先頭に使う
    pub message: String,
    pub entries: Vec<HiraganaWord>,
}

/// `rulePath` の辞書の1項目
#[derive(Debug)]
pub struct HiraganaWord {
    /// 置き換え後の語
    pub expected: String,
    pub pattern: Vec<Expect<OwnedFields>>,
}

/// `path` の `rulePath` の辞書 (`message` と、`expected` と `tokens` を持つ項目の配列 `dict` のYAML) を読む。
/// `tokens` の要素は `no-ai-words` の利用者の辞書と同じく `expect` で読み、項目のほかのキーは読まない。
/// 読めない、YAMLとして読めない、形の誤りは、理由を返す
fn hiragana_dictionary(path: &Path) -> Result<HiraganaDictionary, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("読めない: {e}"))?;
    let raw: RawHiraganaDictionary =
        serde_json::from_value(parse_yaml(&text)?).map_err(|e| format!("辞書の形でない: {e}"))?;
    if raw.message.is_empty() {
        return Err("`message` は空でない文字列で書く".to_owned());
    }
    if raw.dict.is_empty() {
        return Err("`dict` は空でない配列で書く".to_owned());
    }
    let entries = raw
        .dict
        .into_iter()
        .enumerate()
        .map(|(i, entry)| {
            hiragana_word(entry).map_err(|reason| format!("`dict` の{}番目: {reason}", i + 1))
        })
        .collect::<Result<_, _>>()?;
    Ok(HiraganaDictionary {
        message: raw.message,
        entries,
    })
}

#[derive(Deserialize)]
struct RawHiraganaDictionary {
    message: String,
    dict: Vec<RawHiraganaEntry>,
}

/// 項目の `expected` と `tokens` 以外のキー (`extensions` など) は、textlintと同じく読まない
#[derive(Deserialize)]
struct RawHiraganaEntry {
    expected: String,
    tokens: Vec<serde_json::Map<String, serde_json::Value>>,
}

fn hiragana_word(entry: RawHiraganaEntry) -> Result<HiraganaWord, String> {
    if entry.tokens.is_empty() {
        return Err("`tokens` は空でない配列で書く".to_owned());
    }
    let pattern = entry
        .tokens
        .iter()
        .enumerate()
        .map(|(j, token)| {
            expect(token).map_err(|reason| format!("`tokens` の{}番目: {reason}", j + 1))
        })
        .collect::<Result<_, _>>()?;
    Ok(HiraganaWord {
        expected: entry.expected,
        pattern,
    })
}

/// `ja-keishikimeishi` のオプション。2つは独立で、`false` にしたほうだけを止める
#[derive(Debug)]
pub struct JaKeishikimeishi {
    /// `detection_hou_kata`。偽なら、置き換え後の語が `ほう` の項目を当てない
    pub detection_hou_kata: bool,
    /// `detection_ue`。偽なら、置き換え後の語が `うえ` の項目を当てない
    pub detection_ue: bool,
}

impl Default for JaKeishikimeishi {
    fn default() -> Self {
        Self {
            detection_hou_kata: true,
            detection_ue: true,
        }
    }
}

impl JaKeishikimeishi {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "detection_hou_kata" => rule.detection_hou_kata = boolean(key, value)?,
                "detection_ue" => rule.detection_ue = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `@textlint-ja/no-synonyms` のオプション。`allows` と `preferWords` の要素は、パターンではなく、見出しと完全に
/// 一致させる文字列である
#[derive(Debug)]
pub struct NoSynonyms {
    /// 見出しがこのどれかである項目は、使われた項目に数えない。これを見出しに持つ集まりは、`preferWords` の指摘をしない
    pub allows: Vec<String>,
    /// 集まりの見出しにあれば、その集まりの使われたほかの項目を、最初の要素の語にするよう指摘する
    pub prefer_words: Vec<String>,
    /// 英字の表記の項目を、使われた項目に数えない
    pub allow_alphabet: bool,
    /// 見出しにASCIIの数字を含む項目と、見出しが漢数字1字の項目を、使われた項目に数えない
    pub allow_number: bool,
    /// 真なら、同義語のグループを語彙番号ごとの集まりに分け、偽ならグループの全体を1つの集まりにする
    pub allow_lexeme: bool,
}

impl Default for NoSynonyms {
    fn default() -> Self {
        Self {
            allows: Vec::new(),
            prefer_words: Vec::new(),
            allow_alphabet: true,
            allow_number: true,
            allow_lexeme: true,
        }
    }
}

impl NoSynonyms {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        let owned = |key: &str, value: &Value| -> Result<Vec<String>, String> {
            Ok(strings(key, value)?
                .into_iter()
                .map(str::to_owned)
                .collect())
        };
        for (key, value) in options {
            match key.as_str() {
                "allows" => rule.allows = owned(key, value)?,
                "preferWords" => rule.prefer_words = owned(key, value)?,
                "allowAlphabet" => rule.allow_alphabet = boolean(key, value)?,
                "allowNumber" => rule.allow_number = boolean(key, value)?,
                "allowLexeme" => rule.allow_lexeme = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `date-weekday-mismatch` のオプション
#[derive(Debug, Default)]
pub struct DateWeekdayMismatch {
    /// `lang`。あれば、その言語の曜日だけを確かめ、無ければ日本語と英語の両方の曜日を確かめる
    pub lang: Option<WeekdayLang>,
    /// `useCurrentYearIfMissing`。真なら、年の無い日付を `current_year` の年として確かめる
    pub use_current_year_if_missing: bool,
    /// `currentYear`。`今年`・`本年` と、年の無い日付に使う年である。無ければ実行した時点のシステムの年を使う
    pub current_year: Option<usize>,
}

impl DateWeekdayMismatch {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "lang" => {
                    rule.lang = Some(match string(key, value)? {
                        "ja" => WeekdayLang::Ja,
                        "en" => WeekdayLang::En,
                        _ => return Err(format!("`{key}` は \"ja\" か \"en\" で書く")),
                    });
                }
                "useCurrentYearIfMissing" => {
                    rule.use_current_year_if_missing = boolean(key, value)?;
                }
                "currentYear" => rule.current_year = Some(integer(key, value, 1)?),
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `date-weekday-mismatch` が確かめる曜日の言語
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeekdayLang {
    Ja,
    En,
}

/// `no-dead-link` のオプション。時間は秒である
#[derive(Debug)]
pub struct NoDeadLink {
    /// `checkRelative`。偽なら、相対のリンクを確かめない。`/` で始まる絶対パスは、偽でも確かめる
    pub check_relative: bool,
    /// `baseURI`。パスで書いたものは、末尾の `/` の有無によらずディレクトリの `file://` のURLにする
    pub base_uri: Option<Url>,
    /// `ignore`。解決する前の宛先の文字列に当たれば確かめない
    pub ignore: GlobSet,
    /// `ignoreRedirects`。真なら、リダイレクトの結果を指摘しない
    pub ignore_redirects: bool,
    /// `preferGET`。このoriginのURLは、最初から `GET` で確かめる
    pub prefer_get: Vec<Origin>,
    /// `retry`。2xxでない応答と接続の誤りを、この回数まで再試行する
    pub retry: usize,
    /// `concurrency`。1つのファイルの宛先を、最大この数だけ同時に確かめる
    pub concurrency: usize,
    /// `userAgent`
    pub user_agent: String,
    /// `maxRetryTime`。`Retry-After` の無い再試行の待ちの上限
    pub max_retry_time: usize,
    /// `maxRetryAfterTime`。`Retry-After` の待ちの上限
    pub max_retry_after_time: usize,
    /// `httpOnly`。真なら、すべての宛先を `base_uri` を基準にしたURLとして解決する
    pub http_only: bool,
}

impl Default for NoDeadLink {
    fn default() -> Self {
        Self {
            check_relative: true,
            base_uri: None,
            ignore: GlobSet::empty(),
            ignore_redirects: false,
            prefer_get: Vec::new(),
            retry: 3,
            concurrency: 8,
            user_agent: format!("jafix-no-dead-link/{}", env!("CARGO_PKG_VERSION")),
            max_retry_time: 10,
            max_retry_after_time: 10,
            http_only: false,
        }
    }
}

impl NoDeadLink {
    /// `config_dir` は、パスで書いた `baseURI` の相対パスを解決するディレクトリである。`dotInIgnore`・`interval`・
    /// `intervalCap`・`linkMaxAge` は受け付けず、知らないオプションとして理由を返す
    fn from_options(options: &Table, config_dir: &Path) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "checkRelative" => rule.check_relative = boolean(key, value)?,
                "baseURI" => rule.base_uri = Some(base_uri(key, value, config_dir)?),
                "ignore" => rule.ignore = link_globs(key, value)?,
                "ignoreRedirects" => rule.ignore_redirects = boolean(key, value)?,
                "preferGET" => {
                    rule.prefer_get = strings(key, value)?
                        .into_iter()
                        .map(|value| {
                            Url::parse(value)
                                .ok()
                                .map(|url| url.origin())
                                .filter(Origin::is_tuple)
                                .ok_or_else(|| {
                                    format!("`{key}` の `{value}` をoriginを持つURLとして読めない")
                                })
                        })
                        .collect::<Result<_, _>>()?;
                }
                "retry" => rule.retry = integer(key, value, 0)?,
                "concurrency" => rule.concurrency = integer(key, value, 1)?,
                "userAgent" => rule.user_agent = non_empty_string(key, value)?.to_owned(),
                "maxRetryTime" => rule.max_retry_time = integer(key, value, 0)?,
                "maxRetryAfterTime" => rule.max_retry_after_time = integer(key, value, 0)?,
                "httpOnly" => rule.http_only = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `baseURI` を読む。`http://`・`https://`・`file://` で始まればURL、それ以外はパスで、`config_dir` から解決して
/// ディレクトリの `file://` のURLにする
fn base_uri(key: &str, value: &Value, config_dir: &Path) -> Result<Url, String> {
    let value = non_empty_string(key, value)?;
    if ["http://", "https://", "file://"]
        .iter()
        .any(|scheme| value.starts_with(scheme))
    {
        return Url::parse(value)
            .map_err(|e| format!("`{key}` の `{value}` をURLとして読めない: {e}"));
    }
    let path = resolve_path(key, value, config_dir)?;
    Url::from_directory_path(&path)
        .map_err(|()| format!("`{key}` の `{value}` を絶対パスにできない"))
}

/// `ignore` のglobの配列を読む。`exclude` と同じく、`*` と `?` は `/` をまたがない
fn link_globs(key: &str, value: &Value) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for glob in strings(key, value)? {
        builder.add(
            GlobBuilder::new(glob)
                .literal_separator(true)
                .build()
                .map_err(|e| format!("`{key}` の `{glob}` をglobとして読めない: {e}"))?,
        );
    }
    builder
        .build()
        .map_err(|e| format!("`{key}` をglobとして読めない: {e}"))
}

/// `terminology` のオプションを読んだ結果。利用者の `terms` の項目と組み込みの辞書の項目を `exclude` と `defaultTerms` を
/// 当てて並べてある。組み込みの辞書の配列の項目の正規表現は、最初に当てるときに読む
#[derive(Debug)]
pub struct Terminology {
    /// 文字列の項目のすべてを `word_terms` の順に選択にまとめ、語の前後の条件で包んだ正規表現 (`i`・`m`・`u`)。
    /// 文字列の項目が無ければ `None`
    pub words: Option<Regex>,
    /// 文字列の項目。利用者の `terms` の項目を書いた順に、その後に組み込みの辞書の項目が並ぶ。置き換え後の語は、
    /// 一致を小文字にしたものと、小文字にして等しい最初の項目である
    pub word_terms: Vec<WordTerm>,
    /// 配列の項目。並びは `word_terms` と同じで、この順に1つずつ当てる
    pub pair_terms: Vec<PairTerm>,
    /// `skip`。本文の範囲がこの型のどれかの中にあれば当てない
    pub skip: Vec<PatternNodeType>,
}

/// `terminology` の文字列の項目
#[derive(Debug)]
pub struct WordTerm {
    /// 正しい表記
    pub term: Cow<'static, str>,
    /// 利用者の `terms` の項目か。真なら箇所は利用者の置き換え、偽なら指摘である
    pub user: bool,
}

/// `terminology` の配列の項目
#[derive(Debug)]
pub struct PairTerm {
    /// 1つ目の要素 (正規表現の元の文字列)
    first: Cow<'static, str>,
    /// `pattern` の正規表現。利用者の項目は読み込みのときに読み、組み込みの辞書の項目は最初に当てるときに読む。
    /// 数百の組み込みの項目をhookの1回ごとに読むと数msかかり、本文に当たりえない項目を読む必要は無いからである
    pattern: OnceLock<Regex>,
    /// `source` の正規表現。読む時期は `pattern` と同じである
    source: OnceLock<Regex>,
    /// 一致が必ずこの文字列で始まる、1つ目の要素の先頭のASCIIの英数字と空白の並びを小文字にしたもの。
    /// 求められなければ `None`
    pub prefix: Option<String>,
    /// 置き換え後の文字列。JavaScriptの `replace` の記法 (`js_regex::substitute`) で展開する
    pub replacement: Cow<'static, str>,
    /// 利用者の `terms` の項目か
    pub user: bool,
}

impl PairTerm {
    /// 本文の範囲に当てる正規表現 (`i`・`m`・`u`)。1つ目の要素が `(?<` で始まらず、`(?=`・`(?!` も含まなければ、
    /// 語の前後の条件で包んである
    pub fn pattern(&self) -> &Regex {
        self.pattern.get_or_init(|| {
            js_regex::compile(&pair_pattern(&self.first), TERM_FLAGS)
                .expect("the built-in terms compile")
        })
    }

    /// 置き換え後の語を求めるときに `xyz <一致> xyz` に1回だけ当てる、包まない1つ目の要素 (`i`・`u`)
    pub fn source(&self) -> &Regex {
        self.source.get_or_init(|| {
            js_regex::compile(&self.first, REPLACEMENT_FLAGS).expect("the built-in terms compile")
        })
    }
}

impl Terminology {
    /// `config_dir` は、文字列の `terms` (ファイルのパス) を解決するディレクトリである。`terms` のファイルは、JSONか、
    /// コメント (`strip-json-comments` の既定と同じく、文字列の外の `//` から行末までと `/* */`) を持つJSONである。
    /// `terms` の要素の形の誤り、読めない正規表現 (包む形と包まない形の両方)、`skip` の知らない型の名前、ファイルが
    /// 読めないことは、オプションの名前 (`terms` では何番目の要素かとファイルのパスも) を含む理由を返す
    fn from_options(options: &Table, config_dir: &Path) -> Result<Self, String> {
        let mut user_terms = Vec::new();
        let mut skip = vec![PatternNodeType::BlockQuote];
        let mut default_terms = true;
        let mut exclude = Vec::new();
        for (key, value) in options {
            match key.as_str() {
                "terms" => {
                    user_terms = match value {
                        Value::String(_) => {
                            let path =
                                resolve_path(key, non_empty_string(key, value)?, config_dir)?;
                            terms_file(&path).map_err(|reason| {
                                format!("`{key}` の {}: {reason}", path.display())
                            })?
                        }
                        Value::Array(items) => items
                            .iter()
                            .enumerate()
                            .map(|(i, item)| {
                                raw_term(
                                    item.as_str(),
                                    item.as_array()
                                        .map(|values| values.iter().map(Value::as_str).collect()),
                                )
                                .map_err(|reason| format!("`{key}` の{}番目: {reason}", i + 1))
                            })
                            .collect::<Result<_, _>>()?,
                        _ => {
                            return Err(format!(
                                "`{key}` は項目の配列か、項目の配列を書いたファイルのパスで書く"
                            ));
                        }
                    };
                }
                "skip" => {
                    skip = strings(key, value)?
                        .into_iter()
                        .map(|name| {
                            pattern_node_type(name).ok_or_else(|| {
                                format!("`{key}` の `{name}` は本文の範囲の祖先になる型ではない")
                            })
                        })
                        .collect::<Result<_, _>>()?;
                }
                "defaultTerms" => default_terms = boolean(key, value)?,
                "exclude" => exclude = strings(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        let default_terms = if default_terms { &TERMS[..] } else { &[] }
            .iter()
            .map(|term| match *term {
                Term::Word(word) => RawTerm::Word(Cow::Borrowed(word)),
                Term::Pair(pattern, replacement) => {
                    RawTerm::Pair(Cow::Borrowed(pattern), Cow::Borrowed(replacement))
                }
            });
        let mut words = Vec::new();
        let mut word_terms = Vec::new();
        let mut pair_terms = Vec::new();
        let terms = user_terms
            .into_iter()
            .enumerate()
            .map(|(i, term)| (Some(i), term))
            .chain(default_terms.map(|term| (None, term)));
        for (index, term) in terms {
            let first = match &term {
                RawTerm::Word(word) => word,
                RawTerm::Pair(pattern, _) => pattern,
            };
            if exclude.contains(&first.as_ref()) {
                continue;
            }
            let user = index.is_some();
            // 組み込みの辞書の項目はどれも読めるので、理由に書くのは利用者の項目の番号だけである
            let at = |reason: String| match index {
                Some(i) => format!("`terms` の{}番目: {reason}", i + 1),
                None => format!("組み込みの辞書の項目: {reason}"),
            };
            match term {
                RawTerm::Word(word) => {
                    if user {
                        js_regex::compile(&wrap_term(&word), TERM_FLAGS).map_err(at)?;
                        js_regex::compile(&word, REPLACEMENT_FLAGS).map_err(at)?;
                    }
                    words.push(word.clone());
                    word_terms.push(WordTerm { term: word, user });
                }
                RawTerm::Pair(pattern, replacement) => {
                    let (compiled, source) = if user {
                        (
                            OnceLock::from(
                                js_regex::compile(&pair_pattern(&pattern), TERM_FLAGS)
                                    .map_err(at)?,
                            ),
                            OnceLock::from(
                                js_regex::compile(&pattern, REPLACEMENT_FLAGS).map_err(at)?,
                            ),
                        )
                    } else {
                        (OnceLock::new(), OnceLock::new())
                    };
                    pair_terms.push(PairTerm {
                        prefix: required_prefix(&pattern),
                        first: pattern,
                        pattern: compiled,
                        source,
                        replacement,
                        user,
                    });
                }
            }
        }
        let words = if words.is_empty() {
            None
        } else {
            Some(
                js_regex::compile(&wrap_term(&format!("(?:{})", words.join("|"))), TERM_FLAGS)
                    .map_err(|reason| {
                        format!("`terms` の文字列の項目をまとめた正規表現を{reason}")
                    })?,
            )
        };
        Ok(Self {
            words,
            word_terms,
            pair_terms,
            skip,
        })
    }

    /// 利用者の `terms` の項目の置き換えの固定の部分 (文字列の項目では項目の全体、配列の項目では置き換えの記法を
    /// 除いた部分)
    fn checked_replacements(&self) -> impl Iterator<Item = CheckedReplacement> {
        let words = self.word_terms.iter().filter(|term| term.user).map(|term| {
            (
                term.term.as_ref(),
                js_regex::fixed_parts([Some(term.term.as_ref())]),
            )
        });
        let pairs = self.pair_terms.iter().filter(|term| term.user).map(|term| {
            (
                term.replacement.as_ref(),
                js_regex::substitution_fixed_parts(&term.replacement),
            )
        });
        words.chain(pairs).map(|(replacement, parts)| {
            CheckedReplacement::new(
                RuleId::Terminology,
                format!("`terms` の置き換えの「{replacement}」"),
                "`terms` から外す",
                parts,
            )
        })
    }
}

/// `terminology` の項目の正規表現のフラグ。textlint-rule-terminologyと同じ `i`・`m` に、`u` を付ける
const TERM_FLAGS: js_regex::Flags = js_regex::Flags {
    ignore_case: true,
    multiline: true,
    dot_all: false,
};

/// 配列の項目の置き換え後の語を求めるときに、包まない1つ目の要素に付けるフラグ (`i` と `u`)
const REPLACEMENT_FLAGS: js_regex::Flags = js_regex::Flags {
    ignore_case: true,
    multiline: false,
    dot_all: false,
};

/// 読み込んだ `terminology` の1項目
enum RawTerm {
    /// 正しい表記
    Word(Cow<'static, str>),
    /// 正規表現の元の文字列と、置き換え後の文字列
    Pair(Cow<'static, str>, Cow<'static, str>),
}

/// `terms` の要素を読む。`text` は要素が文字列ならその文字列、`array` は要素が配列ならその要素を文字列として読んだもの
/// (文字列でない要素は `None`) である。TOMLとJSONの値を同じ形で読むために、値そのものではなくこの2つを受ける
fn raw_term(text: Option<&str>, array: Option<Vec<Option<&str>>>) -> Result<RawTerm, String> {
    if let Some(text) = text {
        if text.is_empty() {
            return Err("空でない文字列で書く".to_owned());
        }
        return Ok(RawTerm::Word(Cow::Owned(text.to_owned())));
    }
    match array.as_deref() {
        Some([Some(pattern), Some(replacement)])
            if !pattern.is_empty() && !replacement.is_empty() =>
        {
            Ok(RawTerm::Pair(
                Cow::Owned((*pattern).to_owned()),
                Cow::Owned((*replacement).to_owned()),
            ))
        }
        _ => Err("空でない文字列か、空でない2つの文字列の配列で書く".to_owned()),
    }
}

/// `terms` のファイルを読む。読めない、JSONCとして読めない、項目の配列でないことは理由を返す
fn terms_file(path: &Path) -> Result<Vec<RawTerm>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("読めない: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&strip_json_comments(&text))
        .map_err(|e| format!("JSONCとして読めない: {e}"))?;
    value
        .as_array()
        .ok_or("項目の配列でない")?
        .iter()
        .enumerate()
        .map(|(i, item)| {
            raw_term(
                item.as_str(),
                item.as_array()
                    .map(|values| values.iter().map(serde_json::Value::as_str).collect()),
            )
            .map_err(|reason| format!("{}番目: {reason}", i + 1))
        })
        .collect()
}

/// `strip-json-comments` の既定と同じく、文字列の外の `//` から行末までと `/* */` を除く。末尾のカンマは除かない。
/// 閉じていない `/*` は、文書の末尾までを除く
fn strip_json_comments(text: &str) -> String {
    // コメントの文字を空白にして改行を残すので、JSONとして読めないときの理由の行が元のファイルと同じになる
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('"', _) => {
                out.push(c);
                while let Some(c) = chars.next() {
                    out.push(c);
                    match c {
                        '\\' => out.extend(chars.next()),
                        '"' => break,
                        _ => {}
                    }
                }
            }
            ('/', Some('/')) => {
                out.push(' ');
                // `strip-json-comments` と同じく、`\n` と `\r\n` で終え、単独の `\r` では終えない
                while let Some(&c) = chars.peek() {
                    if c == '\n' || c == '\r' && chars.clone().nth(1) == Some('\n') {
                        break;
                    }
                    out.push(if c == '\r' { c } else { ' ' });
                    chars.next();
                }
            }
            ('/', Some('*')) => {
                chars.next();
                out.push_str("  ");
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        out.push_str("  ");
                        break;
                    }
                    out.push(if c == '\n' || c == '\r' { c } else { ' ' });
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// 配列の項目の1つ目の要素 `pattern` から、本文の範囲に当てる正規表現の元の文字列を作る。`(?<` で始まるか、`(?=`・`(?!` を
/// 含めば、そのまま使い、ほかは語の前後の条件で包む
fn pair_pattern(pattern: &str) -> Cow<'_, str> {
    if pattern.starts_with("(?<") || pattern.contains("(?=") || pattern.contains("(?!") {
        Cow::Borrowed(pattern)
    } else {
        Cow::Owned(wrap_term(pattern))
    }
}

/// 正規表現の元の文字列 `pattern` の一致が必ず始まる文字列を、`i` の照合で比べられるよう小文字にして返す。先頭から続く
/// ASCIIの英数字と空白のうち、量指定子 (`?`・`*`・`{`) の付かないものである。括弧の外に `|` があれば、選択のどれで
/// 始まるか決まらないので `None` にする
fn required_prefix(pattern: &str) -> Option<String> {
    let mut depth = 0usize;
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '[' => {
                // 文字クラスの中の `(`・`)`・`|` は記号ではない
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            chars.next();
                        }
                        ']' => break,
                        _ => {}
                    }
                }
            }
            '(' => depth += 1,
            ')' => depth -= 1,
            '|' if depth == 0 => return None,
            _ => {}
        }
    }
    let mut prefix = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next_if(|&c| c.is_ascii_alphanumeric() || c == ' ') {
        match chars.peek() {
            Some('?' | '*' | '{') => break,
            Some('+') => {
                prefix.push(c.to_ascii_lowercase());
                break;
            }
            _ => prefix.push(c.to_ascii_lowercase()),
        }
    }
    (!prefix.is_empty()).then_some(prefix)
}

/// textlint-rule-terminologyの `getExactMatchRegExp` と同じく、項目を語の前後の条件で包む。直後の条件には、jafixが
/// `’` を除くASCIIでない文字1つを足す
fn wrap_term(source: &str) -> String {
    const PUNCTUATION: &str = r#"[\.,;:!?'"’”)]"#;
    format!(
        r"(?<=^|[^-\w])(?<!\.)\b{source}\b(?= |{PUNCTUATION} |{PUNCTUATION}{PUNCTUATION}|{PUNCTUATION}$|$|[^\x00-\x7F’])"
    )
}

/// `spelling` のオプション
#[derive(Debug)]
pub struct Spelling {
    /// `skipPatterns`。一致の範囲のどれかと重なる (端が接するものを含む) 語は指摘しない
    pub skip_patterns: Vec<Regex>,
    /// `wordDefinitionRegexp`。重ならない空でない一致の1つずつを1語にする
    pub word_definition: Regex,
    /// `suggestCorrections`。真なら、辞書の候補を文面に含める
    pub suggest_corrections: bool,
}

impl Spelling {
    /// `language` は `"en"` だけを受ける。`wordDefinitionRegexp` は `/source/flags` の形だけを受け、既定は `/\b[\w']+\b/g` である
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut skip_patterns = Vec::new();
        let mut word_definition = None;
        let mut suggest_corrections = true;
        for (key, value) in options {
            match key.as_str() {
                "language" => {
                    if string(key, value)? != "en" {
                        return Err(format!("`{key}` は \"en\" だけを書ける"));
                    }
                }
                "skipPatterns" => skip_patterns = patterns(key, value)?,
                "wordDefinitionRegexp" => {
                    word_definition = Some(
                        js_regex::slash_regex(string(key, value)?)
                            .map_err(|reason| format!("`{key}`: {reason}"))?,
                    );
                }
                "suggestCorrections" => suggest_corrections = boolean(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        let word_definition = match word_definition {
            Some(regex) => regex,
            None => js_regex::compile(r"\b[\w']+\b", js_regex::Flags::default())
                .expect("the default wordDefinitionRegexp compiles"),
        };
        Ok(Self {
            skip_patterns,
            word_definition,
            suggest_corrections,
        })
    }
}

/// `common-misspellings` のオプション
#[derive(Debug, Default)]
pub struct CommonMisspellings {
    /// `ignore` の語を小文字にしたもの。一致を小文字にしたものと等しければ指摘しない
    pub ignore: Vec<String>,
}

impl CommonMisspellings {
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut rule = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "ignore" => {
                    rule.ignore = strings(key, value)?
                        .into_iter()
                        .map(str::to_lowercase)
                        .collect();
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(rule)
    }
}

/// `write-good` の検査。宣言の順は、`write-good` 1.0.8 が検査を当てる順である
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteGoodCheck {
    Weasel,
    Illusion,
    So,
    ThereIs,
    Passive,
    Adverb,
    TooWordy,
    Cliches,
    Eprime,
}

impl WriteGoodCheck {
    pub const ALL: [WriteGoodCheck; 9] = [
        WriteGoodCheck::Weasel,
        WriteGoodCheck::Illusion,
        WriteGoodCheck::So,
        WriteGoodCheck::ThereIs,
        WriteGoodCheck::Passive,
        WriteGoodCheck::Adverb,
        WriteGoodCheck::TooWordy,
        WriteGoodCheck::Cliches,
        WriteGoodCheck::Eprime,
    ];

    /// オプションのキーと、文面に添える検査の名前
    pub fn name(self) -> &'static str {
        match self {
            WriteGoodCheck::Weasel => "weasel",
            WriteGoodCheck::Illusion => "illusion",
            WriteGoodCheck::So => "so",
            WriteGoodCheck::ThereIs => "thereIs",
            WriteGoodCheck::Passive => "passive",
            WriteGoodCheck::Adverb => "adverb",
            WriteGoodCheck::TooWordy => "tooWordy",
            WriteGoodCheck::Cliches => "cliches",
            WriteGoodCheck::Eprime => "eprime",
        }
    }
}

/// `write-good` のオプション
#[derive(Debug)]
pub struct WriteGood {
    /// 有効な検査。`WriteGoodCheck::ALL` の順に並ぶ
    pub checks: Vec<WriteGoodCheck>,
    /// `whitelist`。本文の範囲の中の出現と重なる提案は指摘しない
    pub whitelist: Vec<String>,
}

impl WriteGood {
    /// 検査の名前のキーは真偽値で、検査の有効・無効を変える。既定では `eprime` だけが無効である。
    /// `checks` (検査の差し替え) は知らないオプションである
    fn from_options(options: &Table) -> Result<Self, String> {
        let mut enabled = WriteGoodCheck::ALL.map(|check| (check, check != WriteGoodCheck::Eprime));
        let mut whitelist = Vec::new();
        for (key, value) in options {
            if key == "whitelist" {
                whitelist = non_empty_strings(key, value)?;
                continue;
            }
            let (_, on) = enabled
                .iter_mut()
                .find(|(check, _)| check.name() == key)
                .ok_or_else(|| unknown_option(key))?;
            *on = boolean(key, value)?;
        }
        Ok(Self {
            checks: enabled
                .into_iter()
                .filter_map(|(check, on)| on.then_some(check))
                .collect(),
            whitelist,
        })
    }
}

/// `no-mixed-zenkaku-and-hankaku-alphabet` が揃える英字の幅
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphabetWidth {
    /// `"全角"`
    Full,
    /// `"半角"` (既定)
    Half,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use toml::Table;

    use super::{Terminology, required_prefix, strip_json_comments};

    #[test]
    fn every_built_in_term_compiles() {
        // 組み込みの辞書の配列の項目は最初に当てるときに読むので、読めない項目があれば照合の途中で落ちる
        let terminology = Terminology::from_options(&Table::new(), Path::new(".")).unwrap();
        assert!(!terminology.pair_terms.is_empty());
        for term in &terminology.pair_terms {
            term.pattern();
            term.source();
        }
    }

    #[test]
    fn required_prefix_keeps_only_characters_every_match_starts_with() {
        assert_eq!(
            required_prefix("Apple[ -]?Script").as_deref(),
            Some("apple")
        );
        assert_eq!(required_prefix("a npm").as_deref(), Some("a npm"));
        assert_eq!(required_prefix("JSDocs?").as_deref(), Some("jsdoc"));
        assert_eq!(required_prefix("ab*c").as_deref(), Some("a"));
        assert_eq!(required_prefix("ab{0,2}").as_deref(), Some("a"));
        assert_eq!(required_prefix("ab+c").as_deref(), Some("ab"));
        assert_eq!(required_prefix("ab(c|d)").as_deref(), Some("ab"));
        assert_eq!(required_prefix(r"a[(|]b\|c").as_deref(), Some("a"));
    }

    #[test]
    fn required_prefix_gives_up_when_matches_may_start_differently() {
        assert_eq!(required_prefix("foo|bar"), None);
        assert_eq!(required_prefix("(a)|b"), None);
        assert_eq!(required_prefix("a?b"), None);
        assert_eq!(required_prefix(r"\(s\)he"), None);
        assert_eq!(required_prefix("(?<=x )npm"), None);
    }

    #[test]
    fn strip_json_comments_blanks_comments_outside_strings() {
        assert_eq!(
            strip_json_comments("[\"a\", // b\n/* c\nd */ \"e\"]"),
            "[\"a\",     \n    \n     \"e\"]"
        );
        assert_eq!(
            strip_json_comments(r#"["a//b", "c\"/*d*/", "\\"]"#),
            r#"["a//b", "c\"/*d*/", "\\"]"#
        );
    }

    #[test]
    fn strip_json_comments_blanks_multibyte_comments() {
        assert_eq!(strip_json_comments("/* 日本 */1"), "        1");
        assert_eq!(strip_json_comments("1// 語\r\n2"), "1    \r\n2");
    }

    #[test]
    fn strip_json_comments_does_not_end_a_line_comment_at_a_lone_carriage_return() {
        assert_eq!(strip_json_comments("1// a\r2\n3"), "1    \r \n3");
    }

    #[test]
    fn strip_json_comments_stops_at_the_end_of_the_text() {
        assert_eq!(strip_json_comments("1 /* 日本"), "1      ");
        assert_eq!(strip_json_comments("1 /*"), "1   ");
        assert_eq!(strip_json_comments("1 //"), "1   ");
        assert_eq!(strip_json_comments("1 /"), "1 /");
        assert_eq!(strip_json_comments("[\"a\\"), "[\"a\\");
        assert_eq!(strip_json_comments("[\"語"), "[\"語");
    }
}
