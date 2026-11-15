use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

mod ai_writing;
pub mod builtin;
mod morph;
mod paragraph;
mod prh;
mod spacing;
mod text;
mod textlint_rule_pattern;

use regex::Regex;

use crate::apply::{LineIndex, TargetLines};
use crate::prose::{Context, Document, Prose};

use self::builtin::Builtin;

/// 組み込みのルール。全角括弧と空白の3つ以外は、textlintのpreset (`preset-ja-technical-writing`・
/// `@textlint-ja/preset-ai-writing`・`preset-ai-words-ja`・`preset-ja-spacing`) の中の名前か、
/// textlintの設定に書くルールの名前と同じIDである。宣言の順は `ALL` と同じで、組み込みの修正ルールの箇所の
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
}

impl RuleId {
    /// 組み込みのルールのすべて。設定の `[builtin]` のキーをルールのIDとして読むのに使う
    pub const ALL: [RuleId; 48] = [
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
#[allow(dead_code)] // skeleton: not implemented
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
/// `@textlint-rule/pattern` の要素) に当たる箇所を、位置の順に返す。`prh` と `@textlint-rule/pattern` は、
/// `targets` と重なる本文の範囲だけに当てる
pub fn report(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    targets: &TargetLines,
) -> Vec<Report> {
    let mut reports = text::report(src, &document.texts, builtin);
    reports.extend(paragraph::report(src, &document.paragraphs, builtin));
    reports.extend(morph::report(src, document, builtin));
    reports.extend(ai_writing::report(src, document, builtin));
    let texts = target_texts(src, document, builtin, targets);
    if let Some(rule) = &builtin.textlint_rule_pattern {
        reports.extend(textlint_rule_pattern::report(src, &texts, rule));
    }
    reports.extend(
        user_replacements(src, &texts, builtin)
            .into_iter()
            .filter_map(|replacement| {
                let reason = unfixable(src, &replacement)?;
                let Finding {
                    range,
                    rule,
                    position,
                    message,
                    ..
                } = replacement.finding;
                Some(Report {
                    range: position..range.end,
                    rule: rule.name(),
                    message: Cow::Owned(format!("{message}。{reason}")),
                })
            }),
    );
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
    reports.sort_by_key(|report| report.range.start);
    reports
}

/// 利用者の置き換え (`prh` の箇所と、`replace` のある `@textlint-rule/pattern` の要素の箇所) の1箇所
pub struct UserReplacement {
    pub finding: Finding,
    /// 見出しの中の本文の範囲の箇所
    pub heading: bool,
}

/// 利用者の置き換えのうち直すものを、重なりを解いて位置の順に返す。見出しの中の箇所と改行を含む箇所は含めない
/// (`report` が指摘にする)。`targets` と重なる本文の範囲だけに当てる
pub fn replacements(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    targets: &TargetLines,
) -> Vec<Finding> {
    let texts = target_texts(src, document, builtin, targets);
    user_replacements(src, &texts, builtin)
        .into_iter()
        .filter(|replacement| unfixable(src, replacement).is_none())
        .map(|replacement| replacement.finding)
        .collect()
}

/// 利用者の置き換えの箇所を、重なりを解いて位置の順に返す。範囲が同じ箇所は、`prh`、`patterns` の前の要素の順に残す
fn user_replacements(src: &str, texts: &[&Prose], builtin: &Builtin) -> Vec<UserReplacement> {
    let mut replacements = Vec::new();
    if let Some(rule) = &builtin.prh {
        replacements.extend(prh::replacements(src, texts, rule));
    }
    if let Some(rule) = &builtin.textlint_rule_pattern {
        replacements.extend(textlint_rule_pattern::replacements(src, texts, rule));
    }
    drop_overlaps(&mut replacements, |replacement| &replacement.finding.range);
    replacements
}

/// 直さずに指摘にする利用者の置き換えなら、指摘の文面に続ける理由を返す。見出しの中の箇所は見出しの理由だけを返す
fn unfixable(src: &str, replacement: &UserReplacement) -> Option<&'static str> {
    if replacement.heading {
        return Some("見出しのアンカーが変わるので、見出しへのリンクも直す");
    }
    let finding = &replacement.finding;
    // hookは、直した後の内容でも対象行の番号が変わらないことを前提にする
    (src[finding.range.clone()].contains('\n') || finding.replacement.contains('\n'))
        .then_some("改行を含むので直さない")
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

/// 利用者の置き換えの固定の部分 `text` を、段落の直下にある本文の範囲1つだけの文書とみなして、
/// 有効な組み込みの修正ルールのうち当たるものを返す。当たれば、置き換えは組み込みの修正ルールと逆である
pub fn fix_rules_hitting(text: &str, builtin: &Builtin) -> Vec<RuleId> {
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
        list_items: Vec::new(),
        inline_nodes: Vec::new(),
    };
    let mut rules: Vec<RuleId> = Vec::new();
    for finding in find(text, &document, builtin) {
        if !rules.contains(&finding.rule) {
            rules.push(finding.rule);
        }
    }
    rules
}

/// 利用者の置き換えを当てる本文の範囲。最初の行から最後の行までのどれかが `targets` の対象行であるもの。
/// 行の索引を1回で作るために、`prh` と `@textlint-rule/pattern` で共有する。どちらも無効なら空
fn target_texts<'a>(
    src: &str,
    document: &'a Document,
    builtin: &Builtin,
    targets: &TargetLines,
) -> Vec<&'a Prose> {
    if builtin.prh.is_none() && builtin.textlint_rule_pattern.is_none() {
        return Vec::new();
    }
    let index = matches!(targets, TargetLines::Lines(_)).then(|| LineIndex::new(src));
    document
        .texts
        .iter()
        .filter(|prose| {
            index.as_ref().is_none_or(|index| {
                let (first, _) = index.line_col(prose.range.start);
                let (last, _) = index.line_col(prose.range.end);
                targets.overlaps(first..=last)
            })
        })
        .collect()
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

/// 有効な組み込みの修正ルールの箇所を、見出しの中のものも含めて、重なりを解かずに `RuleId` の順に安定に並べて返す
#[allow(dead_code)] // skeleton: not implemented
fn fix_rule_findings(src: &str, document: &Document, builtin: &Builtin) -> Vec<FixRuleFinding> {
    let _ = (src, document, builtin);
    todo!("skeleton: not implemented")
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
            inline_nodes: Vec::new(),
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
