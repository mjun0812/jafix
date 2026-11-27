use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;
use std::thread;

mod ai_writing;
pub mod builtin;
mod english;
pub mod general;
mod jtf_style;
mod morph;
mod paragraph;
mod prh;
mod spacing;
mod text;
mod textlint_rule_pattern;
mod yomiyasu;

use regex::Regex;

use crate::apply::{LineIndex, TargetLines};
use crate::filter::Ignored;
use crate::js_regex::FixedPart;
use crate::prose::{self, Context, Document, InlineNodeKind, Prose};

pub(crate) use self::prh::dictionary::parse_yaml;

use self::builtin::Builtin;
use self::general::Links;
use self::jtf_style::JtfRule;

/// 組み込みのルール。全角括弧と空白の3つ、およびyomiyasu由来のルール以外は、textlintのpreset (`preset-ja-technical-writing`・
/// `@textlint-ja/preset-ai-writing`・`preset-ai-words-ja`・`preset-ja-spacing`・`preset-jtf-style`) の中の名前か、
/// textlintの設定に書くルールの名前 (textlintが `textlint-rule-` を除いてruleIdとして出す名前) と同じIDである。宣言の順は `ALL` と同じで、組み込みの修正ルールの箇所の
/// 範囲が同じときに、前にあるルールの箇所を残すのに使う
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleId {
    FullwidthParen,
    SpaceBetweenJaAndAlnum,
    SpaceBetweenJaAndParen,
    SentenceLength,
    MaxComma,
    MaxKanjiContinuousLen,
    ArabicKanjiNumbers,
    JaNoMixedPeriod,
    NoNfd,
    NoInvalidControlCharacter,
    NoZeroWidthSpaces,
    NoExclamationQuestionMark,
    NoHankakuKana,
    JaUnnaturalAlphabet,
    NoUnmatchedPair,
    MaxTen,
    NoMixDearuDesumasu,
    NoDoubledConjunction,
    NoDoubledConjunctiveParticleGa,
    NoDoubleNegativeJa,
    NoDoubledJoshi,
    NoDroppingTheRa,
    JaNoWeakPhrase,
    JaNoSuccessiveWord,
    JaNoAbusage,
    JaNoRedundantExpression,
    NoAiListFormatting,
    NoAiHypeExpressions,
    NoAiEmphasisPatterns,
    NoAiColonContinuation,
    AiTechWritingGuideline,
    NoAiWords,
    NoShortTopicComma,
    Prh,
    TextlintRulePattern,
    JaNakaguroOrHalfwidthSpaceBetweenKatakana,
    JaNoSpaceAroundParentheses,
    JaNoSpaceAroundSlash,
    JaNoSpaceBetweenFullWidth,
    JaSpaceBetweenHalfAndFullWidth,
    JaSpaceAfterExclamation,
    JaSpaceAfterQuestion,
    JaSpaceAroundCode,
    JaSpaceAroundEmphasis,
    JaSpaceAroundLink,
    JaSpaceAroundStrong,
    NoMixedZenkakuAndHankakuAlphabet,
    NoKangxiRadicals,
    /// `preset-jtf-style` の37ルール
    JtfStyle(JtfRule),
    JaHiraganaHojodoushi,
    JaHiraganaKeishikimeishi,
    JaKeishikimeishi,
    JaHiraganaFukushi,
    JaHiraganaDaimeishi,
    NoInsertDroppingSa,
    NoSynonyms,
    NoTodo,
    DateWeekdayMismatch,
    NoDeadLink,
    Terminology,
    Spelling,
    CommonMisspellings,
    WriteGood,
    YomiyasuBoldNotRendered,
    YomiyasuInstructionResidue,
    YomiyasuChatResponse,
    YomiyasuMetaFiller,
    YomiyasuSummaryRestatement,
    YomiyasuMetaIntro,
    YomiyasuCountDeclaration,
    YomiyasuAiPhrase,
    YomiyasuPlaceholderResidue,
    YomiyasuShortMochiron,
    YomiyasuSayHedge,
    YomiyasuFragmentRun,
    YomiyasuCleftListEnding,
    YomiyasuCopyCommaNoun,
    YomiyasuBareVerbChain,
    YomiyasuDashDecoration,
    YomiyasuDashInsertion,
    YomiyasuDashListEnding,
    YomiyasuEmojiProhibited,
    YomiyasuRenyoNoun,
    YomiyasuSlopVocabulary,
    YomiyasuKatakanaTerm,
}

impl RuleId {
    /// 組み込みのルールのすべて。設定の `[builtin]` のキーをルールのIDとして読むのに使う
    pub const ALL: [RuleId; 121] = [
        RuleId::FullwidthParen,
        RuleId::SpaceBetweenJaAndAlnum,
        RuleId::SpaceBetweenJaAndParen,
        RuleId::SentenceLength,
        RuleId::MaxComma,
        RuleId::MaxKanjiContinuousLen,
        RuleId::ArabicKanjiNumbers,
        RuleId::JaNoMixedPeriod,
        RuleId::NoNfd,
        RuleId::NoInvalidControlCharacter,
        RuleId::NoZeroWidthSpaces,
        RuleId::NoExclamationQuestionMark,
        RuleId::NoHankakuKana,
        RuleId::JaUnnaturalAlphabet,
        RuleId::NoUnmatchedPair,
        RuleId::MaxTen,
        RuleId::NoMixDearuDesumasu,
        RuleId::NoDoubledConjunction,
        RuleId::NoDoubledConjunctiveParticleGa,
        RuleId::NoDoubleNegativeJa,
        RuleId::NoDoubledJoshi,
        RuleId::NoDroppingTheRa,
        RuleId::JaNoWeakPhrase,
        RuleId::JaNoSuccessiveWord,
        RuleId::JaNoAbusage,
        RuleId::JaNoRedundantExpression,
        RuleId::NoAiListFormatting,
        RuleId::NoAiHypeExpressions,
        RuleId::NoAiEmphasisPatterns,
        RuleId::NoAiColonContinuation,
        RuleId::AiTechWritingGuideline,
        RuleId::NoAiWords,
        RuleId::NoShortTopicComma,
        RuleId::Prh,
        RuleId::TextlintRulePattern,
        RuleId::JaNakaguroOrHalfwidthSpaceBetweenKatakana,
        RuleId::JaNoSpaceAroundParentheses,
        RuleId::JaNoSpaceAroundSlash,
        RuleId::JaNoSpaceBetweenFullWidth,
        RuleId::JaSpaceBetweenHalfAndFullWidth,
        RuleId::JaSpaceAfterExclamation,
        RuleId::JaSpaceAfterQuestion,
        RuleId::JaSpaceAroundCode,
        RuleId::JaSpaceAroundEmphasis,
        RuleId::JaSpaceAroundLink,
        RuleId::JaSpaceAroundStrong,
        RuleId::NoMixedZenkakuAndHankakuAlphabet,
        RuleId::NoKangxiRadicals,
        RuleId::JtfStyle(JtfRule::R1_1_1),
        RuleId::JtfStyle(JtfRule::R1_1_2),
        RuleId::JtfStyle(JtfRule::R1_1_3),
        RuleId::JtfStyle(JtfRule::R1_1_5),
        RuleId::JtfStyle(JtfRule::R1_2_1),
        RuleId::JtfStyle(JtfRule::R1_2_2),
        RuleId::JtfStyle(JtfRule::R2_1_2),
        RuleId::JtfStyle(JtfRule::R2_1_5),
        RuleId::JtfStyle(JtfRule::R2_1_6),
        RuleId::JtfStyle(JtfRule::R2_1_8),
        RuleId::JtfStyle(JtfRule::R2_1_9),
        RuleId::JtfStyle(JtfRule::R2_1_10),
        RuleId::JtfStyle(JtfRule::R2_2_1),
        RuleId::JtfStyle(JtfRule::R2_2_2),
        RuleId::JtfStyle(JtfRule::R2_2_3),
        RuleId::JtfStyle(JtfRule::R3_1_1),
        RuleId::JtfStyle(JtfRule::R3_1_2),
        RuleId::JtfStyle(JtfRule::R3_2),
        RuleId::JtfStyle(JtfRule::R3_3),
        RuleId::JtfStyle(JtfRule::R4_1_1),
        RuleId::JtfStyle(JtfRule::R4_1_3),
        RuleId::JtfStyle(JtfRule::R4_2_1),
        RuleId::JtfStyle(JtfRule::R4_2_2),
        RuleId::JtfStyle(JtfRule::R4_2_4),
        RuleId::JtfStyle(JtfRule::R4_2_5),
        RuleId::JtfStyle(JtfRule::R4_2_6),
        RuleId::JtfStyle(JtfRule::R4_2_7),
        RuleId::JtfStyle(JtfRule::R4_2_8),
        RuleId::JtfStyle(JtfRule::R4_2_9),
        RuleId::JtfStyle(JtfRule::R4_3_1),
        RuleId::JtfStyle(JtfRule::R4_3_2),
        RuleId::JtfStyle(JtfRule::R4_3_3),
        RuleId::JtfStyle(JtfRule::R4_3_4),
        RuleId::JtfStyle(JtfRule::R4_3_5),
        RuleId::JtfStyle(JtfRule::R4_3_6),
        RuleId::JtfStyle(JtfRule::R4_3_7),
        RuleId::JtfStyle(JtfRule::R4_3_8),
        RuleId::JaHiraganaHojodoushi,
        RuleId::JaHiraganaKeishikimeishi,
        RuleId::JaKeishikimeishi,
        RuleId::JaHiraganaFukushi,
        RuleId::JaHiraganaDaimeishi,
        RuleId::NoInsertDroppingSa,
        RuleId::NoSynonyms,
        RuleId::NoTodo,
        RuleId::DateWeekdayMismatch,
        RuleId::NoDeadLink,
        RuleId::Terminology,
        RuleId::Spelling,
        RuleId::CommonMisspellings,
        RuleId::WriteGood,
        RuleId::YomiyasuBoldNotRendered,
        RuleId::YomiyasuInstructionResidue,
        RuleId::YomiyasuChatResponse,
        RuleId::YomiyasuMetaFiller,
        RuleId::YomiyasuSummaryRestatement,
        RuleId::YomiyasuMetaIntro,
        RuleId::YomiyasuCountDeclaration,
        RuleId::YomiyasuAiPhrase,
        RuleId::YomiyasuPlaceholderResidue,
        RuleId::YomiyasuShortMochiron,
        RuleId::YomiyasuSayHedge,
        RuleId::YomiyasuFragmentRun,
        RuleId::YomiyasuCleftListEnding,
        RuleId::YomiyasuCopyCommaNoun,
        RuleId::YomiyasuBareVerbChain,
        RuleId::YomiyasuDashDecoration,
        RuleId::YomiyasuDashInsertion,
        RuleId::YomiyasuDashListEnding,
        RuleId::YomiyasuEmojiProhibited,
        RuleId::YomiyasuRenyoNoun,
        RuleId::YomiyasuSlopVocabulary,
        RuleId::YomiyasuKatakanaTerm,
    ];

    /// `check` の出力の `[<rule-id>]` に使う名前
    pub fn name(self) -> &'static str {
        match self {
            RuleId::FullwidthParen => "fullwidth-paren",
            RuleId::SpaceBetweenJaAndAlnum => "space-between-ja-and-alnum",
            RuleId::SpaceBetweenJaAndParen => "space-between-ja-and-paren",
            RuleId::SentenceLength => "sentence-length",
            RuleId::MaxComma => "max-comma",
            RuleId::MaxKanjiContinuousLen => "max-kanji-continuous-len",
            RuleId::ArabicKanjiNumbers => "arabic-kanji-numbers",
            RuleId::JaNoMixedPeriod => "ja-no-mixed-period",
            RuleId::NoNfd => "no-nfd",
            RuleId::NoInvalidControlCharacter => "no-invalid-control-character",
            RuleId::NoZeroWidthSpaces => "no-zero-width-spaces",
            RuleId::NoExclamationQuestionMark => "no-exclamation-question-mark",
            RuleId::NoHankakuKana => "no-hankaku-kana",
            RuleId::JaUnnaturalAlphabet => "ja-unnatural-alphabet",
            RuleId::NoUnmatchedPair => "no-unmatched-pair",
            RuleId::MaxTen => "max-ten",
            RuleId::NoMixDearuDesumasu => "no-mix-dearu-desumasu",
            RuleId::NoDoubledConjunction => "no-doubled-conjunction",
            RuleId::NoDoubledConjunctiveParticleGa => "no-doubled-conjunctive-particle-ga",
            RuleId::NoDoubleNegativeJa => "no-double-negative-ja",
            RuleId::NoDoubledJoshi => "no-doubled-joshi",
            RuleId::NoDroppingTheRa => "no-dropping-the-ra",
            RuleId::JaNoWeakPhrase => "ja-no-weak-phrase",
            RuleId::JaNoSuccessiveWord => "ja-no-successive-word",
            RuleId::JaNoAbusage => "ja-no-abusage",
            RuleId::JaNoRedundantExpression => "ja-no-redundant-expression",
            RuleId::NoAiListFormatting => "no-ai-list-formatting",
            RuleId::NoAiHypeExpressions => "no-ai-hype-expressions",
            RuleId::NoAiEmphasisPatterns => "no-ai-emphasis-patterns",
            RuleId::NoAiColonContinuation => "no-ai-colon-continuation",
            RuleId::AiTechWritingGuideline => "ai-tech-writing-guideline",
            RuleId::NoAiWords => "no-ai-words",
            RuleId::NoShortTopicComma => "no-short-topic-comma",
            RuleId::Prh => "prh",
            RuleId::TextlintRulePattern => "@textlint-rule/pattern",
            RuleId::JaNakaguroOrHalfwidthSpaceBetweenKatakana => {
                "ja-nakaguro-or-halfwidth-space-between-katakana"
            }
            RuleId::JaNoSpaceAroundParentheses => "ja-no-space-around-parentheses",
            RuleId::JaNoSpaceAroundSlash => "ja-no-space-around-slash",
            RuleId::JaNoSpaceBetweenFullWidth => "ja-no-space-between-full-width",
            RuleId::JaSpaceBetweenHalfAndFullWidth => "ja-space-between-half-and-full-width",
            RuleId::JaSpaceAfterExclamation => "ja-space-after-exclamation",
            RuleId::JaSpaceAfterQuestion => "ja-space-after-question",
            RuleId::JaSpaceAroundCode => "ja-space-around-code",
            RuleId::JaSpaceAroundEmphasis => "ja-space-around-emphasis",
            RuleId::JaSpaceAroundLink => "ja-space-around-link",
            RuleId::JaSpaceAroundStrong => "ja-space-around-strong",
            RuleId::NoMixedZenkakuAndHankakuAlphabet => "no-mixed-zenkaku-and-hankaku-alphabet",
            RuleId::NoKangxiRadicals => "no-kangxi-radicals",
            RuleId::JtfStyle(rule) => rule.name(),
            RuleId::JaHiraganaHojodoushi => "ja-hiragana-hojodoushi",
            RuleId::JaHiraganaKeishikimeishi => "ja-hiragana-keishikimeishi",
            RuleId::JaKeishikimeishi => "ja-keishikimeishi",
            RuleId::JaHiraganaFukushi => "ja-hiragana-fukushi",
            RuleId::JaHiraganaDaimeishi => "ja-hiragana-daimeishi",
            RuleId::NoInsertDroppingSa => "@textlint-ja/no-insert-dropping-sa",
            RuleId::NoSynonyms => "@textlint-ja/no-synonyms",
            RuleId::NoTodo => "no-todo",
            RuleId::DateWeekdayMismatch => "date-weekday-mismatch",
            RuleId::NoDeadLink => "no-dead-link",
            RuleId::Terminology => "terminology",
            RuleId::Spelling => "spelling",
            RuleId::CommonMisspellings => "common-misspellings",
            RuleId::WriteGood => "write-good",
            RuleId::YomiyasuBoldNotRendered => "yomiyasu-bold-not-rendered",
            RuleId::YomiyasuInstructionResidue => "yomiyasu-instruction-residue",
            RuleId::YomiyasuChatResponse => "yomiyasu-chat-response",
            RuleId::YomiyasuMetaFiller => "yomiyasu-meta-filler",
            RuleId::YomiyasuSummaryRestatement => "yomiyasu-summary-restatement",
            RuleId::YomiyasuMetaIntro => "yomiyasu-meta-intro",
            RuleId::YomiyasuCountDeclaration => "yomiyasu-count-declaration",
            RuleId::YomiyasuAiPhrase => "yomiyasu-ai-phrase",
            RuleId::YomiyasuPlaceholderResidue => "yomiyasu-placeholder-residue",
            RuleId::YomiyasuShortMochiron => "yomiyasu-short-mochiron",
            RuleId::YomiyasuSayHedge => "yomiyasu-say-hedge",
            RuleId::YomiyasuFragmentRun => "yomiyasu-fragment-run",
            RuleId::YomiyasuCleftListEnding => "yomiyasu-cleft-list-ending",
            RuleId::YomiyasuCopyCommaNoun => "yomiyasu-copy-comma-noun",
            RuleId::YomiyasuBareVerbChain => "yomiyasu-bare-verb-chain",
            RuleId::YomiyasuDashDecoration => "yomiyasu-dash-decoration",
            RuleId::YomiyasuDashInsertion => "yomiyasu-dash-insertion",
            RuleId::YomiyasuDashListEnding => "yomiyasu-dash-list-ending",
            RuleId::YomiyasuEmojiProhibited => "yomiyasu-emoji-prohibited",
            RuleId::YomiyasuRenyoNoun => "yomiyasu-renyo-noun",
            RuleId::YomiyasuSlopVocabulary => "yomiyasu-slop-vocabulary",
            RuleId::YomiyasuKatakanaTerm => "yomiyasu-katakana-term",
        }
    }
}

/// 空白を削除する2つのルール。空白の前後の文字から決まるルールを、この2つに限って表すので、`RuleId` とは別に持つ
#[derive(Clone, Copy)]
enum SpaceRule {
    JaAndAlnum,
    JaAndParen,
}

impl SpaceRule {
    fn id(self) -> RuleId {
        match self {
            SpaceRule::JaAndAlnum => RuleId::SpaceBetweenJaAndAlnum,
            SpaceRule::JaAndParen => RuleId::SpaceBetweenJaAndParen,
        }
    }

    fn enabled(self, builtin: &Builtin) -> bool {
        match self {
            SpaceRule::JaAndAlnum => builtin.space_between_ja_and_alnum,
            SpaceRule::JaAndParen => builtin.space_between_ja_and_paren,
        }
    }

    fn message(self) -> &'static str {
        match self {
            SpaceRule::JaAndAlnum => "日本語と英数字の間の空白を削除する",
            SpaceRule::JaAndParen => "日本語と括弧の間の空白を削除する",
        }
    }
}

/// 修正ルールに当たった1箇所。`range` は元のソース上のバイト範囲で、`replacement` に置き換える
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub range: Range<usize>,
    pub replacement: Cow<'static, str>,
    pub rule: RuleId,
    /// `check` の出力に使う位置。置き換える範囲の先頭とは限らない (`no-nfd` は濁点の位置に出す)
    pub position: usize,
    /// `check` の出力に使う文面
    pub message: Cow<'static, str>,
}

/// 組み込みの修正ルールに当たった1箇所。重なりを解く前の一覧の要素である
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixRuleFinding {
    pub finding: Finding,
    /// 見出しの中の箇所で、直さずに `report` で指摘にする。見出しの中でも直す `fullwidth-paren` の箇所は含めない
    pub heading: bool,
}

/// 指摘ルールに当たった1箇所。直さずに指摘として出す。`range` は元のソース上のバイト範囲
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub range: Range<usize>,
    /// `check` の出力の `[<rule-id>]` に使う名前
    pub rule: &'static str,
    pub message: Cow<'static, str>,
}

/// 指摘ルール (組み込みの指摘ルール、見出しの中で直さない修正ルール、直さない利用者の置き換え、`replace` の無い
/// `@textlint-rule/pattern` の要素) に当たる箇所を返す。`prh` と `@textlint-rule/pattern` と
/// `preset-jtf-style` の辞書の4ルールと `terminology` と `spelling` は、`targets` と重なる本文の範囲 (`spelling` は画像のaltも)
/// だけに当てる。`fixes` は、同じ `src` と `document` の `fix_findings` の結果と、検査で求めた利用者の置き換えである。
/// 利用者の置き換えを渡さないときは、修正後の `src` に当て直す。
/// `ignored` に含まれる指摘は除く。`links` は、`no-dead-link` が宛先を確かめる文脈である
pub fn report(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    targets: &TargetLines,
    fixes: (&[FixRuleFinding], Option<&[UserReplacement]>),
    ignored: &Ignored,
    links: Option<&Links<'_>>,
) -> Vec<Report> {
    let (fixes, replacements) = fixes;
    let (texts, alts) = target_texts(src, document, builtin, targets);
    // 英語の4ルールは、`spelling` の辞書の読み込みと候補の計算に時間がかかるので、どれかが有効なときだけ1つのthreadで、
    // ほかのルールと並行に当てる (threadを立てる箇所は `english::report` に書いた)。指摘は、ほかのルールの指摘の後に足すので、
    // 並びは並行にしない場合と同じである
    let mut reports = thread::scope(|scope| {
        let english = english::enabled(builtin)
            .then(|| scope.spawn(|| english::report(src, document, builtin, &texts, &alts)));
        let mut reports = text::report(src, &document.texts, builtin);
        reports.extend(paragraph::report(src, &document.paragraphs, builtin));
        reports.extend(morph::report(src, document, builtin));
        reports.extend(ai_writing::report(src, document, builtin));
        reports.extend(yomiyasu::report(src, document, builtin));
        if let Some(rule) = &builtin.textlint_rule_pattern {
            reports.extend(textlint_rule_pattern::report(src, &texts, rule));
        }
        let calculated;
        let replacements = match replacements {
            Some(replacements) => replacements,
            None => {
                calculated = user_replacements_in(src, &texts, builtin, ignored);
                &calculated
            }
        };
        reports.extend(replacements.iter().filter_map(|replacement| {
            let reason = replacement.unfixable(src)?;
            let finding = &replacement.finding;
            Some(Report {
                range: finding.position..finding.range.end,
                rule: finding.rule.name(),
                message: Cow::Owned(format!("{}。{reason}", finding.message)),
            })
        }));
        reports.extend(fixes.iter().filter(|fix| fix.heading).map(
            |FixRuleFinding { finding, .. }| Report {
                range: finding.position..finding.range.end,
                rule: finding.rule.name(),
                message: Cow::Owned(format!(
                    "{}。見出しのアンカーが変わるので、見出しへのリンクも直す",
                    finding.message
                )),
            },
        ));
        reports.extend(spacing::report(src, document, builtin));
        reports.extend(jtf_style::report(src, document, builtin, &texts));
        reports.extend(general::report(src, document, builtin, links, ignored));
        if let Some(english) = english {
            reports.extend(english.join().unwrap());
        }
        reports
    });
    reports.retain(|report| !ignored.covers(&report.range, report.rule));
    reports
}

/// 利用者の置き換え (`prh` の箇所と、`replace` のある `@textlint-rule/pattern` の要素の箇所と、`terminology` の利用者の
/// `terms` の項目の箇所) の1箇所
pub struct UserReplacement {
    pub finding: Finding,
    /// 見出しの中の本文の範囲の箇所
    pub heading: bool,
}

impl UserReplacement {
    /// 直さずに指摘にするときの理由。見出しの中の箇所には、見出しの理由だけを返す
    pub fn unfixable(&self, src: &str) -> Option<&'static str> {
        if self.heading {
            return Some("見出しのアンカーが変わるので、見出しへのリンクも直す");
        }
        // hookは、直した後の内容でも対象行の番号が変わらないことを前提にする
        (src[self.finding.range.clone()].contains('\n') || self.finding.replacement.contains('\n'))
            .then_some("改行を含むので直さない")
    }
}

/// 利用者の置き換えを、対象行と重なる本文の範囲に当てる
pub fn user_replacements(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    targets: &TargetLines,
    ignored: &Ignored,
) -> Vec<UserReplacement> {
    if builtin.prh.is_none()
        && builtin.textlint_rule_pattern.is_none()
        && builtin.terminology.is_none()
    {
        return Vec::new();
    }
    let (texts, _) = target_texts(src, document, builtin, targets);
    user_replacements_in(src, &texts, builtin, ignored)
}

/// 利用者の置き換えの箇所を、`ignored` に含まれるものと、すでに置き換え後の形になっているものを除き、重なりを解いて
/// 位置の順に返す。範囲が同じ箇所は、`prh`、`patterns` の前の要素、`terminology` の順に残す
fn user_replacements_in(
    src: &str,
    texts: &[&Prose],
    builtin: &Builtin,
    ignored: &Ignored,
) -> Vec<UserReplacement> {
    let mut replacements = Vec::new();
    if let Some(rule) = &builtin.prh {
        replacements.extend(prh::replacements(src, texts, rule));
    }
    if let Some(rule) = &builtin.textlint_rule_pattern {
        replacements.extend(textlint_rule_pattern::replacements(src, texts, rule));
    }
    if let Some(rule) = &builtin.terminology {
        replacements.extend(english::replacements(src, texts, rule));
    }
    replacements.retain(|replacement| {
        !ignored.covers(&replacement.finding.range, replacement.finding.rule.name())
            && !already_replaced(src, &replacement.finding)
    });
    drop_overlaps(&mut replacements, |replacement| &replacement.finding.range);
    replacements
}

/// 置き換え後の文字列の中に一致した文字列が現れる位置のどれかで、元のソースがすでに置き換え後の文字列になっているか。
/// `サーバ` を `サーバー` にする置き換えは `サーバー` の `サーバ` にも当たり、直すたびに `ー` が増えるので、その一致は直さない
fn already_replaced(src: &str, finding: &Finding) -> bool {
    let matched = &src[finding.range.clone()];
    let replacement: &str = &finding.replacement;
    replacement.char_indices().any(|(offset, _)| {
        replacement[offset..].starts_with(matched)
            && finding
                .range
                .start
                .checked_sub(offset)
                .and_then(|start| src.get(start..start + replacement.len()))
                == Some(replacement)
    })
}

/// 箇所を開始の位置の順に、同じ位置なら長いものを先に並べ、残した最後の箇所の終わりより前に始まる箇所を捨てる。
/// 並べ替えは安定なので、範囲が同じ箇所は `items` で前にあるものが残る
fn drop_overlaps<T>(items: &mut Vec<T>, range: impl Fn(&T) -> &Range<usize>) {
    items.sort_by_key(|item| {
        let range = range(item);
        (range.start, std::cmp::Reverse(range.end))
    });
    let mut end = 0;
    items.retain(|item| {
        let range = range(item);
        if range.start < end {
            return false;
        }
        end = range.end;
        true
    });
}

/// 1つの利用者の置き換えの固定の部分 `parts` に当たる、有効な組み込みの修正ルールを返す。当たれば、置き換えは
/// 組み込みの修正ルールと逆である。`yomiyasu-bold-not-rendered` は置き換えの結果の全体に当てる (`bold_fix_hits`)
pub fn fix_rules_hitting(parts: &[FixedPart], builtin: &Builtin) -> Vec<RuleId> {
    let mut rules: Vec<RuleId> = parts
        .iter()
        .flat_map(|part| fix_rules_hitting_part(part, builtin))
        .collect();
    if bold_fix_hits(parts, builtin) {
        rules.push(RuleId::YomiyasuBoldNotRendered);
    }
    rules
}

/// 固定の部分 `part` を、段落の直下にある本文の範囲1つだけの文書とみなして、有効な組み込みの修正ルールのうち当たる
/// ものを `RuleId` の順に返す。段落の範囲を持たない文書なので、太字の修正は当たらない。参照に接する端の空白の修正は、
/// 字下げかどうかが決まらないので除く。同じ箇所の重なりは解かない
fn fix_rules_hitting_part(part: &FixedPart, builtin: &Builtin) -> Vec<RuleId> {
    let text = &part.text;
    let document = Document {
        texts: vec![Prose {
            range: 0..text.len(),
            context: Context {
                plain: true,
                paragraph: true,
                ..Context::default()
            },
        }],
        paragraphs: Vec::new(),
        headings: Vec::new(),
        table_cells: Vec::new(),
        list_items: Vec::new(),
        inline_nodes: Vec::new(),
        comments: Vec::new(),
        links: Vec::new(),
    };
    let mut rules: Vec<RuleId> = fix_rule_findings(text, &document, builtin)
        .into_iter()
        .filter(|fix| {
            // 字種だけの修正は前後の参照によらず必要になる。空白の修正は、参照が改行や字下げかで変わる
            matches!(
                fix.finding.rule,
                RuleId::FullwidthParen
                    | RuleId::NoNfd
                    | RuleId::NoInvalidControlCharacter
                    | RuleId::NoZeroWidthSpaces
                    | RuleId::NoHankakuKana
                    | RuleId::NoMixedZenkakuAndHankakuAlphabet
                    | RuleId::NoKangxiRadicals
                    | RuleId::JtfStyle(
                        JtfRule::R2_1_5 | JtfRule::R2_1_8 | JtfRule::R2_1_9 | JtfRule::R4_2_4
                    )
            ) || part.contains(&fix.finding.range)
        })
        .map(|fix| fix.finding.rule)
        .collect();
    rules.dedup();
    rules
}

/// 固定の部分 `parts` と捕捉参照をつないだ置き換えの結果に、`yomiyasu-bold-not-rendered` が当たるか。参照には、
/// 太字になるかの判定でいちばん有利な中身を入れて当てるので、当たる組は参照の中身によらず太字にならない。
/// 組の内側 (前に閉じていない `**` があり、後ろに `**` がある位置) の参照は日本語の文字、組の外側で `**` に接する
/// 参照は空白とみなす。空白の後の `**` は次が空白でなければ開きに、空白の前の `**` は前が空白でなければ閉じに
/// なれる。置き換えの結果は日本語の文の中に入るので、参照に接しない端の外側には日本語の文字があるとみなす
fn bold_fix_hits(parts: &[FixedPart], builtin: &Builtin) -> bool {
    const NEIGHBOR: &str = "あ";
    if !builtin.yomiyasu_bold_not_rendered || !parts.iter().any(|part| part.text.contains("**")) {
        return false;
    }
    // `before` は参照より前の結果、`rest` は参照より後の固定の部分である
    let stand_in = |before: &str, rest: &[FixedPart]| {
        let touches =
            before.ends_with("**") || rest.first().is_some_and(|part| part.text.starts_with("**"));
        // `*` が2つ以上続く並びを1つの記号と数え、前の記号が奇数個なら、開いたまま閉じていない組がある
        let opened = before
            .split(|c| c != '*')
            .filter(|run| run.len() >= 2)
            .count()
            % 2
            == 1;
        let inside = opened && rest.iter().any(|part| part.text.contains("**"));
        if touches && !inside { " " } else { NEIGHBOR }
    };
    // 隣り合う固定の部分の間には参照が1つ以上あり、後の部分の `leading_reference` が立つ。続いた参照は1つにまとめる
    let mut text = String::new();
    if !parts[0].leading_reference {
        text.push_str(NEIGHBOR);
    }
    for (i, part) in parts.iter().enumerate() {
        if part.leading_reference {
            text.push_str(stand_in(&text, &parts[i..]));
        }
        text.push_str(&part.text);
    }
    text.push_str(if parts.last().unwrap().trailing_reference {
        stand_in(&text, &[])
    } else {
        NEIGHBOR
    });
    !yomiyasu::find(&text, &prose::parse(&text), builtin).is_empty()
}

/// 辞書で語を別の書き方にするよう求める組み込みの指摘ルール。利用者の置き換えの結果をこれらが指摘すると、hookが直した語を
/// agentが指摘に従って戻し、次の編集でhookがまた直す往復になる。ほかの指摘ルールは、別の言い方で両方を満たせる
pub const DICTIONARY_REPORT_RULES: [RuleId; 13] = [
    RuleId::JtfStyle(JtfRule::R2_1_5),
    RuleId::JtfStyle(JtfRule::R2_1_6),
    RuleId::JtfStyle(JtfRule::R2_2_1),
    RuleId::JtfStyle(JtfRule::R2_2_3),
    RuleId::JaHiraganaHojodoushi,
    RuleId::JaHiraganaKeishikimeishi,
    RuleId::JaKeishikimeishi,
    RuleId::JaHiraganaFukushi,
    RuleId::JaHiraganaDaimeishi,
    RuleId::JaNoAbusage,
    RuleId::NoSynonyms,
    RuleId::Terminology,
    RuleId::CommonMisspellings,
];

/// 利用者の置き換えの固定の部分 `parts` のそれぞれを、段落の直下にある本文の範囲1つだけの文書とみなして、
/// `DICTIONARY_REPORT_RULES` のうち有効で指摘を出すものを、`parts` と同じ順に返す。指摘すれば、置き換えは指摘ルールと
/// 逆である。部分は数千になりうるので、辞書の数百の正規表現を部分ごとに当て直さず、部分を改行でつないだ文字列に当てて、
/// 指摘と重なる部分だけを1つずつ当て直す。参照に接する端の指摘は除く。英語の2つは、ASCIIの英字を含む部分だけに当てる
pub fn dictionary_rules_reporting(parts: &[&FixedPart], builtin: &Builtin) -> Vec<Vec<RuleId>> {
    if parts.is_empty() {
        return Vec::new();
    }
    let texts: Vec<&str> = parts.iter().map(|part| part.text.as_str()).collect();
    let all = Joined::new(&texts, 0..parts.len());
    let english = Joined::new(
        &texts,
        (0..parts.len()).filter(|&i| texts[i].bytes().any(|b| b.is_ascii_alphabetic())),
    );
    let jtf = |text: &str, _: &[Range<usize>]| {
        jtf_style::dictionary_report(text, &[&whole_prose(text)], builtin)
    };
    let english_rules = |text: &str, _: &[Range<usize>]| {
        english::dictionary_report(text, &whole_prose(text), builtin)
    };
    let morph = |text: &str, parts: &[Range<usize>]| morph::dictionary_report(text, parts, builtin);
    // hookは設定を読み終えるまで文書を解析しないので、この時間はそのままhookの時間に足される。3つの組を並行に当てる
    let candidates = thread::scope(|scope| {
        let jtf_candidates = scope.spawn(|| all.candidates(&jtf));
        let english_candidates = scope.spawn(|| english.candidates(&english_rules));
        let morph_candidates = all.candidates(&morph);
        [
            jtf_candidates.join().unwrap(),
            english_candidates.join().unwrap(),
            morph_candidates,
        ]
    });
    let checks: [&DictionaryCheck; 3] = [&jtf, &english_rules, &morph];
    let mut found = vec![Vec::new(); parts.len()];
    for (candidates, check) in candidates.into_iter().zip(checks) {
        for i in candidates {
            for report in check(texts[i], std::slice::from_ref(&(0..texts[i].len()))) {
                if !parts[i].contains(&report.range) {
                    continue;
                }
                let rule = DICTIONARY_REPORT_RULES
                    .into_iter()
                    .find(|rule| rule.name() == report.rule)
                    .expect("辞書の指摘ルールだけを当てた");
                if !found[i].contains(&rule) {
                    found[i].push(rule);
                }
            }
        }
    }
    found
}

/// 辞書の指摘ルールの組を、つないだ文字列と、その中の部分のバイト範囲に当てて、指摘を返すもの
type DictionaryCheck<'a> = dyn Fn(&str, &[Range<usize>]) -> Vec<Report> + 'a;

/// `text` の全体を、段落の直下にある本文の範囲1つとしたもの
fn whole_prose(text: &str) -> Prose {
    Prose {
        range: 0..text.len(),
        context: Context {
            plain: true,
            paragraph: true,
            ..Context::default()
        },
    }
}

/// 利用者の置き換えの固定の部分のいくつかを、改行でつないだ文字列
struct Joined {
    text: String,
    /// つないだ部分の `text` の中の範囲と、`parts` での番号
    parts: Vec<(Range<usize>, usize)>,
}

impl Joined {
    /// `parts` のうち、番号が `indices` のものをつなぐ
    fn new(parts: &[&str], indices: impl Iterator<Item = usize>) -> Self {
        let mut text = String::new();
        let mut joined = Vec::new();
        for i in indices {
            if !joined.is_empty() {
                text.push('\n');
            }
            joined.push((text.len()..text.len() + parts[i].len(), i));
            text.push_str(parts[i]);
        }
        Self {
            text,
            parts: joined,
        }
    }

    /// `check` (つないだ文字列と部分の範囲から指摘を返すもの) を `text` に当て、指摘と重なる部分の `parts` での番号を、
    /// 番号の順に返す。つないだ文字列では、正規表現が部分の境の改行をまたいで当たり (`サーバ([^ーァ-ン]{1,2}|(?=$))` が
    /// `サーバ\nT` に当たる)、部分ごとに当てたときと違う指摘になるので、指摘を部分に割り当てずに、当て直す部分を選ぶのに使う
    fn candidates(&self, check: &DictionaryCheck) -> Vec<usize> {
        let ranges: Vec<Range<usize>> = self.parts.iter().map(|(range, _)| range.clone()).collect();
        let mut candidates: Vec<usize> = check(&self.text, &ranges)
            .iter()
            .flat_map(|report| {
                let first = self
                    .parts
                    .partition_point(|(range, _)| range.end < report.range.start);
                let last = self.parts.partition_point(|(range, _)| {
                    range.start < report.range.end.max(report.range.start + 1)
                });
                self.parts[first..last].iter().map(|&(_, part)| part)
            })
            .collect();
        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }
}

/// 利用者の置き換えと辞書の4ルールと `spelling` を当てる本文の範囲と、`spelling` が語を取り出すインラインの画像のaltの、
/// 元のソース上のバイト範囲。どちらも、最初の行から最後の行までのどれかが `targets` の対象行であるもの。行の索引を1回で
/// 作るために、`prh` と `@textlint-rule/pattern` と `terminology` と辞書の4ルールと `spelling` で共有する。
/// 本文の範囲は、どれも無効なら空で、altは `spelling` が無効なら空
fn target_texts<'a>(
    src: &str,
    document: &'a Document,
    builtin: &Builtin,
    targets: &TargetLines,
) -> (Vec<&'a Prose>, Vec<Range<usize>>) {
    if builtin.prh.is_none()
        && builtin.textlint_rule_pattern.is_none()
        && builtin.terminology.is_none()
        && builtin.spelling.is_none()
        && !jtf_style::uses_dictionaries(builtin)
    {
        return (Vec::new(), Vec::new());
    }
    let index = matches!(targets, TargetLines::Lines(_)).then(|| LineIndex::new(src));
    let overlaps = |range: &Range<usize>| {
        index
            .as_ref()
            .is_none_or(|index| targets.overlaps(index.line(range.start)..=index.line(range.end)))
    };
    let texts = document
        .texts
        .iter()
        .filter(|prose| overlaps(&prose.range))
        .collect();
    if builtin.spelling.is_none() {
        return (texts, Vec::new());
    }
    let alts = document
        .inline_nodes
        .iter()
        .flat_map(|node| match &node.kind {
            InlineNodeKind::Image { alt } => alt.as_slice(),
            _ => &[],
        })
        .filter(|alt| overlaps(alt))
        .cloned()
        .collect();
    (texts, alts)
}

/// 文面に埋め込む文字列。出力は1件1行なので、改行を `\n` の2文字にする
fn one_line(text: &str) -> Cow<'_, str> {
    if text.contains('\n') {
        Cow::Owned(text.replace('\n', "\\n"))
    } else {
        Cow::Borrowed(text)
    }
}

static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new("[ \u{3000}]+").unwrap());

// `\p{Han}` のような省略形はScript_Extensionsで判定し、`、` や `。` まで含んでしまうので、Scriptで判定する
static JAPANESE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{sc=Han}\p{sc=Hiragana}\p{sc=Katakana}ー]$").unwrap());

/// `fix_findings` のうち、見出しの中で直さない箇所を除いた、直す箇所を返す
pub fn fixable(fixes: &[FixRuleFinding]) -> Vec<Finding> {
    fixes
        .iter()
        .filter(|fix| !fix.heading)
        .map(|fix| fix.finding.clone())
        .collect()
}

/// 有効な修正ルールに当たる箇所を、見出しの中のものも含めて、重なりを解いて位置の順に返す。範囲が同じ箇所は、
/// `RuleId` で前にあるルールの箇所が残る。`ignored` に含まれる箇所は、重なりを解く前に除く
pub fn fix_findings(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    ignored: &Ignored,
) -> Vec<FixRuleFinding> {
    let mut findings = fix_rule_findings(src, document, builtin);
    findings.retain(|fix| !ignored.covers(&fix.finding.range, fix.finding.rule.name()));
    drop_overlaps(&mut findings, |fix| &fix.finding.range);
    findings
}

/// 有効な組み込みの修正ルールの箇所を、見出しの中のものも含めて、重なりを解かずに `RuleId` の順に安定に並べて返す。
/// 見出しの文字を変えるとアンカーが変わり、見出しへのリンクが切れるので、`fullwidth-paren` 以外の見出しの中の
/// 箇所は直さずに `report` で指摘にする。全角括弧は、置き換えてもアンカーが変わらないので、見出しの中でも直す
fn fix_rule_findings(src: &str, document: &Document, builtin: &Builtin) -> Vec<FixRuleFinding> {
    let mut findings = Vec::new();
    for prose in &document.texts {
        let Prose { range, context } = prose;
        let heading = context.heading;
        if builtin.fullwidth_paren {
            for (i, paren) in src[range.clone()].match_indices(['（', '）']) {
                let start = range.start + i;
                findings.push(FixRuleFinding {
                    finding: Finding {
                        range: start..start + paren.len(),
                        replacement: Cow::Borrowed(if paren == "（" { "(" } else { ")" }),
                        rule: RuleId::FullwidthParen,
                        position: start,
                        message: Cow::Borrowed("全角括弧を半角にする"),
                    },
                    heading: false,
                });
            }
        }
        findings.extend(
            text::find(src, [prose], builtin)
                .into_iter()
                .map(|finding| FixRuleFinding { finding, heading }),
        );
        findings.extend(
            space_findings(&src[range.clone()], builtin).map(|(spaces, rule)| FixRuleFinding {
                finding: Finding {
                    range: range.start + spaces.start..range.start + spaces.end,
                    replacement: Cow::Borrowed(""),
                    rule: rule.id(),
                    position: range.start + spaces.start,
                    message: Cow::Borrowed(rule.message()),
                },
                heading,
            }),
        );
    }
    findings.extend(spacing::find(src, document, builtin));
    findings.extend(jtf_style::find(src, &document.texts, builtin));
    findings.extend(yomiyasu::find(src, document, builtin));
    findings.sort_by_key(|fix| fix.finding.rule);
    findings
}

/// `text` の中で、有効な空白のルールに当たる空白の範囲 (`text` の中のバイト範囲) とルールを返す
fn space_findings(
    text: &str,
    builtin: &Builtin,
) -> impl Iterator<Item = (Range<usize>, SpaceRule)> {
    (builtin.space_between_ja_and_alnum || builtin.space_between_ja_and_paren)
        .then(|| SPACES.find_iter(text))
        .into_iter()
        .flatten()
        .filter_map(move |spaces| {
            let prev = text[..spaces.start()].chars().next_back()?;
            let next = text[spaces.end()..].chars().next()?;
            let rule = space_rule(prev, next).filter(|rule| rule.enabled(builtin))?;
            Some((spaces.range(), rule))
        })
}

/// 空白の直前の文字 `prev` と直後の文字 `next` から、空白を削除するルールを決める
fn space_rule(prev: char, next: char) -> Option<SpaceRule> {
    let alnum = |c: char| c.is_ascii_alphanumeric();
    if (alnum(next) && is_japanese(prev)) || (alnum(prev) && is_japanese(next)) {
        return Some(SpaceRule::JaAndAlnum);
    }
    if (matches!(next, '(' | '（') && is_japanese(prev))
        || (matches!(prev, ')' | '）') && is_japanese(next))
    {
        return Some(SpaceRule::JaAndParen);
    }
    None
}

fn is_japanese(c: char) -> bool {
    !c.is_ascii() && JAPANESE.is_match(c.encode_utf8(&mut [0; 4]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 見出しの外にある本文の範囲を1つだけ持つ文書
    fn body(range: Range<usize>) -> Document {
        Document {
            texts: vec![Prose {
                range,
                context: Context::default(),
            }],
            paragraphs: Vec::new(),
            headings: Vec::new(),
            table_cells: Vec::new(),
            list_items: Vec::new(),
            inline_nodes: Vec::new(),
            comments: Vec::new(),
            links: Vec::new(),
        }
    }

    fn find_in(src: &str, range: Range<usize>) -> Vec<Finding> {
        fixable(&fix_findings(
            src,
            &body(range),
            &Builtin::default(),
            &Ignored::default(),
        ))
    }

    fn find_all(src: &str) -> Vec<(&str, String, RuleId)> {
        find_in(src, 0..src.len())
            .into_iter()
            .map(|f| (&src[f.range], f.replacement.into_owned(), f.rule))
            .collect()
    }

    #[test]
    fn replaces_fullwidth_parens_with_halfwidth() {
        assert_eq!(
            find_all("注（補足）"),
            vec![
                ("（", "(".to_owned(), RuleId::FullwidthParen),
                ("）", ")".to_owned(), RuleId::FullwidthParen),
            ]
        );
    }

    #[test]
    fn removes_space_between_japanese_and_alnum_in_both_orders() {
        assert_eq!(
            find_all("日本語 Rust で"),
            vec![
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndAlnum),
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndAlnum),
            ]
        );
        assert_eq!(
            find_all("版 2 です"),
            vec![
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndAlnum),
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndAlnum),
            ]
        );
    }

    #[test]
    fn treats_a_run_of_halfwidth_and_fullwidth_spaces_as_one_finding() {
        assert_eq!(
            find_all("日本語\u{3000} \u{3000}Rust"),
            vec![(
                "\u{3000} \u{3000}",
                "".to_owned(),
                RuleId::SpaceBetweenJaAndAlnum
            )]
        );
    }

    #[test]
    fn counts_kanji_hiragana_katakana_and_long_vowel_mark_as_japanese() {
        for src in ["漢 A", "ひ A", "カ A", "ー A", "々 A"] {
            assert_eq!(
                find_all(src),
                vec![(" ", "".to_owned(), RuleId::SpaceBetweenJaAndAlnum)],
                "{src}"
            );
        }
    }

    #[test]
    fn does_not_count_punctuation_or_fullwidth_alnum() {
        assert_eq!(find_all("です。 Rust"), vec![]);
        assert_eq!(find_all("です、 Rust"), vec![]);
        assert_eq!(find_all("日本語 Ｒｕｓｔ"), vec![]);
        assert_eq!(find_all("Rust - の"), vec![]);
    }

    #[test]
    fn removes_space_between_japanese_and_parens_only_on_the_outer_side() {
        assert_eq!(
            find_all("語 (a) 語"),
            vec![
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndParen),
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndParen),
            ]
        );
        assert_eq!(find_all("語) (語"), vec![]);
    }

    #[test]
    fn removes_space_around_fullwidth_parens_and_replaces_them() {
        assert_eq!(
            find_all("で （補足） です"),
            vec![
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndParen),
                ("（", "(".to_owned(), RuleId::FullwidthParen),
                ("）", ")".to_owned(), RuleId::FullwidthParen),
                (" ", "".to_owned(), RuleId::SpaceBetweenJaAndParen),
            ]
        );
    }

    #[test]
    fn ignores_spaces_at_the_edges_of_a_range() {
        let src = "日本語 Rust";
        assert_eq!(find_in(src, 0.."日本語 ".len()), vec![]);
        assert_eq!(find_in(src, "日本語".len()..src.len()), vec![]);
    }

    #[test]
    fn ignores_text_outside_the_ranges_and_offsets_findings_by_range_start() {
        let src = "日本語 Rust と（外） 語 A";
        let start = src.find("語 A").unwrap();
        assert_eq!(
            find_in(src, start..src.len()),
            vec![Finding {
                range: start + "語".len()..start + "語 ".len(),
                replacement: Cow::Borrowed(""),
                rule: RuleId::SpaceBetweenJaAndAlnum,
                position: start + "語".len(),
                message: Cow::Borrowed("日本語と英数字の間の空白を削除する"),
            }]
        );
    }

    #[test]
    fn does_not_treat_a_newline_as_a_space() {
        assert_eq!(find_all("日本語\nRust"), vec![]);
        assert_eq!(find_all("日本語\n Rust"), vec![]);
    }

    /// `src` の中で最初に出る `matched` を `replacement` に置き換える箇所
    fn replacing(src: &str, matched: &str, replacement: &'static str) -> Finding {
        let start = src.find(matched).unwrap();
        Finding {
            range: start..start + matched.len(),
            replacement: Cow::Borrowed(replacement),
            rule: RuleId::Prh,
            position: start,
            message: Cow::Borrowed(""),
        }
    }

    #[test]
    fn already_replaced_reads_only_inside_the_source() {
        assert!(already_replaced(
            "サーバーを",
            &replacing("サーバーを", "サーバ", "サーバー")
        ));
        // 置き換え後の文字列を合わせる位置が、元のソースの先頭より前、末尾より後ろ、多バイト文字の途中にかかる
        assert!(!already_replaced(
            "サーバ",
            &replacing("サーバ", "サーバ", "サーバー")
        ));
        assert!(!already_replaced("js", &replacing("js", "js", "Node.js")));
        assert!(!already_replaced(
            "éサーバ",
            &replacing("éサーバ", "サーバ", "xサーバ")
        ));
    }
}
