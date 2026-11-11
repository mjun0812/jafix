use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::Prose;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleId {
    FullwidthParen,
    SpaceBetweenJaAndAlnum,
    SpaceBetweenJaAndParen,
}

impl RuleId {
    /// 組み込みのルールのすべて。設定の指摘ルールの `id` と重ならないことを確かめるのに使う
    pub const ALL: [RuleId; 3] = [
        RuleId::FullwidthParen,
        RuleId::SpaceBetweenJaAndAlnum,
        RuleId::SpaceBetweenJaAndParen,
    ];

    /// `check` の出力の `[<rule-id>]` に使う名前
    pub fn name(self) -> &'static str {
        match self {
            RuleId::FullwidthParen => "fullwidth-paren",
            RuleId::SpaceBetweenJaAndAlnum => "space-between-ja-and-alnum",
            RuleId::SpaceBetweenJaAndParen => "space-between-ja-and-paren",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            RuleId::FullwidthParen => "全角括弧を半角にする",
            RuleId::SpaceBetweenJaAndAlnum => "日本語と英数字の間の空白を削除する",
            RuleId::SpaceBetweenJaAndParen => "日本語と括弧の間の空白を削除する",
        }
    }
}

/// 空白を削除するルール。見出しの中では直さず指摘にするので、`RuleId` とは別に持つ
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

    /// 見出しの中で当たったときの指摘の文面。空白を消すとアンカーが変わることを伝える
    fn heading_message(self) -> &'static str {
        match self {
            SpaceRule::JaAndAlnum => {
                "日本語と英数字の間の空白を削除する。見出しのアンカーが変わるので、見出しへのリンクも直す"
            }
            SpaceRule::JaAndParen => {
                "日本語と括弧の間の空白を削除する。見出しのアンカーが変わるので、見出しへのリンクも直す"
            }
        }
    }
}

/// 修正ルールに当たった1箇所。`range` は元のソース上のバイト範囲で、`replacement` に置き換える
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub range: Range<usize>,
    pub replacement: &'static str,
    pub rule: RuleId,
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

/// 本文の範囲の中で指摘ルール (見出しの中の空白と、設定の `custom`) に当たる箇所を、位置の順に返す
pub fn report<'r>(src: &str, prose: &[Prose], custom: &'r [CustomRule]) -> Vec<Report<'r>> {
    let mut reports = Vec::new();
    for Prose { range, in_heading } in prose {
        let text = &src[range.clone()];
        if *in_heading {
            reports.extend(space_findings(text).map(|(spaces, rule)| Report {
                range: range.start + spaces.start..range.start + spaces.end,
                rule: rule.id().name(),
                message: Cow::Borrowed(rule.heading_message()),
            }));
        }
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

/// 本文の範囲の中で修正ルールに当たる箇所を、位置の順に返す
pub fn find(src: &str, prose: &[Prose]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for Prose { range, in_heading } in prose {
        let text = &src[range.clone()];
        for (i, paren) in text.match_indices(['（', '）']) {
            let start = range.start + i;
            findings.push(Finding {
                range: start..start + paren.len(),
                replacement: if paren == "（" { "(" } else { ")" },
                rule: RuleId::FullwidthParen,
            });
        }
        // 見出しの空白を消すとアンカーが変わり、見出しへのリンクが切れるので、`report` で指摘にする
        if *in_heading {
            continue;
        }
        findings.extend(space_findings(text).map(|(spaces, rule)| Finding {
            range: range.start + spaces.start..range.start + spaces.end,
            replacement: "",
            rule: rule.id(),
        }));
    }
    findings.sort_by_key(|finding| finding.range.start);
    findings
}

/// `text` の中で空白のルールに当たる空白の範囲 (`text` の中のバイト範囲) とルールを返す
fn space_findings(text: &str) -> impl Iterator<Item = (Range<usize>, SpaceRule)> {
    SPACES.find_iter(text).filter_map(|spaces| {
        let prev = text[..spaces.start()].chars().next_back()?;
        let next = text[spaces.end()..].chars().next()?;
        space_rule(prev, next).map(|rule| (spaces.range(), rule))
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

    /// 見出しの外にある本文の範囲を1つだけ持つ配列
    fn body(range: Range<usize>) -> [Prose; 1] {
        [Prose {
            range,
            in_heading: false,
        }]
    }

    fn find_all(src: &str) -> Vec<(&str, &'static str, RuleId)> {
        find(src, &body(0..src.len()))
            .into_iter()
            .map(|f| (&src[f.range], f.replacement, f.rule))
            .collect()
    }

    #[test]
    fn replaces_fullwidth_parens_with_halfwidth() {
        assert_eq!(
            find_all("注（補足）"),
            vec![
                ("（", "(", RuleId::FullwidthParen),
                ("）", ")", RuleId::FullwidthParen),
            ]
        );
    }

    #[test]
    fn removes_space_between_japanese_and_alnum_in_both_orders() {
        assert_eq!(
            find_all("日本語 Rust で"),
            vec![
                (" ", "", RuleId::SpaceBetweenJaAndAlnum),
                (" ", "", RuleId::SpaceBetweenJaAndAlnum),
            ]
        );
        assert_eq!(
            find_all("版 2 です"),
            vec![
                (" ", "", RuleId::SpaceBetweenJaAndAlnum),
                (" ", "", RuleId::SpaceBetweenJaAndAlnum),
            ]
        );
    }

    #[test]
    fn treats_a_run_of_halfwidth_and_fullwidth_spaces_as_one_finding() {
        assert_eq!(
            find_all("日本語\u{3000} \u{3000}Rust"),
            vec![("\u{3000} \u{3000}", "", RuleId::SpaceBetweenJaAndAlnum)]
        );
    }

    #[test]
    fn counts_kanji_hiragana_katakana_and_long_vowel_mark_as_japanese() {
        for src in ["漢 A", "ひ A", "カ A", "ー A", "々 A"] {
            assert_eq!(
                find_all(src),
                vec![(" ", "", RuleId::SpaceBetweenJaAndAlnum)],
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
                (" ", "", RuleId::SpaceBetweenJaAndParen),
                (" ", "", RuleId::SpaceBetweenJaAndParen),
            ]
        );
        assert_eq!(find_all("語) (語"), vec![]);
    }

    #[test]
    fn removes_space_around_fullwidth_parens_and_replaces_them() {
        assert_eq!(
            find_all("で （補足） です"),
            vec![
                (" ", "", RuleId::SpaceBetweenJaAndParen),
                ("（", "(", RuleId::FullwidthParen),
                ("）", ")", RuleId::FullwidthParen),
                (" ", "", RuleId::SpaceBetweenJaAndParen),
            ]
        );
    }

    #[test]
    fn ignores_spaces_at_the_edges_of_a_range() {
        let src = "日本語 Rust";
        assert_eq!(find(src, &body(0.."日本語 ".len())), vec![]);
        assert_eq!(find(src, &body("日本語".len()..src.len())), vec![]);
    }

    #[test]
    fn ignores_text_outside_the_ranges_and_offsets_findings_by_range_start() {
        let src = "日本語 Rust と（外） 語 A";
        let start = src.find("語 A").unwrap();
        assert_eq!(
            find(src, &body(start..src.len())),
            vec![Finding {
                range: start + "語".len()..start + "語 ".len(),
                replacement: "",
                rule: RuleId::SpaceBetweenJaAndAlnum,
            }]
        );
    }

    #[test]
    fn does_not_treat_a_newline_as_a_space() {
        assert_eq!(find_all("日本語\nRust"), vec![]);
        assert_eq!(find_all("日本語\n Rust"), vec![]);
    }
}
