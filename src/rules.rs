use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

mod ai_writing;
pub mod builtin;
mod morph;
mod paragraph;
mod text;

use regex::Regex;

use crate::prose::{Document, Prose};

use self::builtin::Builtin;

/// 組み込みのルール。全角括弧と空白の3つ以外は、textlintのpreset (`preset-ja-technical-writing`・
/// `@textlint-ja/preset-ai-writing`・`preset-ai-words-ja`) の中の名前と同じIDである
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

impl RuleId {
    /// 組み込みのルールのすべて。設定の `[builtin]` のキーと、設定の指摘ルールの `id` と重ならないことを確かめるのに使う
    pub const ALL: [RuleId; 33] = [
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

/// 設定の `[[rules]]` で足した指摘ルール
#[derive(Debug)]
pub struct CustomRule {
    id: String,
    message: String,
    regex: Regex,
}

impl CustomRule {
    /// `pattern` をRustの `regex` crateの正規表現として読む。読めなければ理由を返す
    pub fn pattern(id: String, message: String, pattern: &str) -> Result<Self, String> {
        let regex =
            Regex::new(pattern).map_err(|e| format!("`pattern` を正規表現として読めない: {e}"))?;
        Ok(Self { id, message, regex })
    }

    /// `words` の文字列をそのまま探す。同じ位置で重なる語は、長いほうに当てる。空の文字列があれば理由を返す
    pub fn words(id: String, message: String, words: &[String]) -> Result<Self, String> {
        if words.iter().any(String::is_empty) {
            return Err("`words` に空の文字列がある".to_owned());
        }
        // 選択は左から順に試すので、長い語を先に置くと、同じ位置で重なる語は長いほうに当たる
        let mut sorted: Vec<&str> = words.iter().map(String::as_str).collect();
        sorted.sort_by_key(|word| std::cmp::Reverse(word.len()));
        let alternation: Vec<String> = sorted.into_iter().map(regex::escape).collect();
        let regex = Regex::new(&alternation.join("|"))
            .map_err(|e| format!("`words` が長すぎて正規表現にできない: {e}"))?;
        Ok(Self { id, message, regex })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// 指摘ルールに当たった1箇所。直さずに指摘として出す。`range` は元のソース上のバイト範囲
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report<'r> {
    pub range: Range<usize>,
    /// `check` の出力の `[<rule-id>]` に使う名前
    pub rule: &'r str,
    pub message: Cow<'r, str>,
}

/// 指摘ルール (組み込みの指摘ルール、見出しの中で直さない修正ルール、設定の `custom`) に当たる箇所を、位置の順に返す
pub fn report<'r>(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    custom: &'r [CustomRule],
) -> Vec<Report<'r>> {
    let mut reports = text::report(src, &document.texts, builtin);
    reports.extend(paragraph::report(src, &document.paragraphs, builtin));
    reports.extend(morph::report(src, document, builtin));
    reports.extend(ai_writing::report(src, document, builtin));
    let headings = document.texts.iter().filter(|prose| prose.context.heading);
    reports.extend(
        anchor_changing_findings(src, headings, builtin)
            .into_iter()
            .map(|finding| Report {
                range: finding.position..finding.range.end,
                rule: finding.rule.name(),
                message: Cow::Owned(format!(
                    "{}。見出しのアンカーが変わるので、見出しへのリンクも直す",
                    finding.message
                )),
            }),
    );
    for Prose { range, .. } in &document.texts {
        let text = &src[range.clone()];
        for rule in custom {
            for m in rule.regex.find_iter(text).filter(|m| !m.is_empty()) {
                reports.push(Report {
                    range: range.start + m.start()..range.start + m.end(),
                    rule: &rule.id,
                    message: Cow::Owned(format!("「{}」: {}", m.as_str(), rule.message)),
                });
            }
        }
    }
    reports.sort_by_key(|report| report.range.start);
    reports
}

static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new("[ \u{3000}]+").unwrap());

// `\p{Han}` のような省略形はScript_Extensionsで判定し、`、` や `。` まで含んでしまうので、Scriptで判定する
static JAPANESE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{sc=Han}\p{sc=Hiragana}\p{sc=Katakana}ー]$").unwrap());

/// 有効な修正ルールに当たる箇所を、位置の順に返す。見出しの中で直さない箇所は含めない
pub fn find(src: &str, document: &Document, builtin: &Builtin) -> Vec<Finding> {
    let mut findings = Vec::new();
    if builtin.fullwidth_paren {
        for Prose { range, .. } in &document.texts {
            for (i, paren) in src[range.clone()].match_indices(['（', '）']) {
                let start = range.start + i;
                findings.push(Finding {
                    range: start..start + paren.len(),
                    replacement: Cow::Borrowed(if paren == "（" { "(" } else { ")" }),
                    rule: RuleId::FullwidthParen,
                    position: start,
                    message: Cow::Borrowed("全角括弧を半角にする"),
                });
            }
        }
    }
    let body = document.texts.iter().filter(|prose| !prose.context.heading);
    findings.extend(anchor_changing_findings(src, body, builtin));
    findings.sort_by_key(|finding| finding.range.start);
    findings
}

/// `fullwidth-paren` 以外の修正ルールの、`texts` の中の箇所。見出しの文字を変えるとアンカーが変わり、
/// 見出しへのリンクが切れるので、見出しの中の箇所は直さずに `report` で指摘にする。
/// 全角括弧は、置き換えてもアンカーが変わらないので、見出しの中でも直す
fn anchor_changing_findings<'a>(
    src: &str,
    texts: impl Iterator<Item = &'a Prose> + Clone,
    builtin: &Builtin,
) -> Vec<Finding> {
    let mut findings = text::find(src, texts.clone(), builtin);
    for Prose { range, .. } in texts {
        findings.extend(
            space_findings(&src[range.clone()], builtin).map(|(spaces, rule)| Finding {
                range: range.start + spaces.start..range.start + spaces.end,
                replacement: Cow::Borrowed(""),
                rule: rule.id(),
                position: range.start + spaces.start,
                message: Cow::Borrowed(rule.message()),
            }),
        );
    }
    findings
}

/// `text` の中で、有効な空白のルールに当たる空白の範囲 (`text` の中のバイト範囲) とルールを返す
fn space_findings(
    text: &str,
    builtin: &Builtin,
) -> impl Iterator<Item = (Range<usize>, SpaceRule)> {
    SPACES.find_iter(text).filter_map(move |spaces| {
        let prev = text[..spaces.start()].chars().next_back()?;
        let next = text[spaces.end()..].chars().next()?;
        let rule = space_rule(prev, next).filter(|rule| rule.enabled(builtin))?;
        Some((spaces.range(), rule))
    })
}

/// 空白の直前の文字 `prev` と直後の文字 `next` から、空白を削除するルールを決める
fn space_rule(prev: char, next: char) -> Option<SpaceRule> {
    let alnum = |c: char| c.is_ascii_alphanumeric();
    if (is_japanese(prev) && alnum(next)) || (alnum(prev) && is_japanese(next)) {
        return Some(SpaceRule::JaAndAlnum);
    }
    if (is_japanese(prev) && matches!(next, '(' | '（'))
        || (matches!(prev, ')' | '）') && is_japanese(next))
    {
        return Some(SpaceRule::JaAndParen);
    }
    None
}

fn is_japanese(c: char) -> bool {
    JAPANESE.is_match(c.encode_utf8(&mut [0; 4]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prose::Context;

    /// 見出しの外にある本文の範囲を1つだけ持つ文書
    fn body(range: Range<usize>) -> Document {
        Document {
            texts: vec![Prose {
                range,
                context: Context::default(),
            }],
            paragraphs: Vec::new(),
            headings: Vec::new(),
            list_items: Vec::new(),
        }
    }

    fn find_in(src: &str, range: Range<usize>) -> Vec<Finding> {
        find(src, &body(range), &Builtin::default())
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
}
