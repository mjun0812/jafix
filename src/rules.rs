use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleId {
    FullwidthParen,
    SpaceBetweenJaAndAlnum,
    SpaceBetweenJaAndParen,
}

impl RuleId {
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

/// 修正ルールに当たった1箇所。`range` は元のソース上のバイト範囲で、`replacement` に置き換える
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub range: Range<usize>,
    pub replacement: &'static str,
    pub rule: RuleId,
}

static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new("[ \u{3000}]+").unwrap());

// `\p{Han}` のような省略形はScript_Extensionsで判定し、`、` や `。` まで含んでしまうので、Scriptで判定する
static JAPANESE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{sc=Han}\p{sc=Hiragana}\p{sc=Katakana}ー]$").unwrap());

/// 本文の範囲の中で修正ルールに当たる箇所を、位置の順に返す
pub fn find(src: &str, prose: &[Range<usize>]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for range in prose {
        let text = &src[range.clone()];
        for (i, paren) in text.match_indices(['（', '）']) {
            let start = range.start + i;
            findings.push(Finding {
                range: start..start + paren.len(),
                replacement: if paren == "（" { "(" } else { ")" },
                rule: RuleId::FullwidthParen,
            });
        }
        for spaces in SPACES.find_iter(text) {
            let prev = text[..spaces.start()].chars().next_back();
            let next = text[spaces.end()..].chars().next();
            let (Some(prev), Some(next)) = (prev, next) else {
                continue;
            };
            if let Some(rule) = space_rule(prev, next) {
                findings.push(Finding {
                    range: range.start + spaces.start()..range.start + spaces.end(),
                    replacement: "",
                    rule,
                });
            }
        }
    }
    findings.sort_by_key(|finding| finding.range.start);
    findings
}

/// 空白の直前の文字 `prev` と直後の文字 `next` から、空白を削除するルールを決める
fn space_rule(prev: char, next: char) -> Option<RuleId> {
    let alnum = |c: char| c.is_ascii_alphanumeric();
    if (is_japanese(prev) && alnum(next)) || (alnum(prev) && is_japanese(next)) {
        return Some(RuleId::SpaceBetweenJaAndAlnum);
    }
    if (is_japanese(prev) && matches!(next, '(' | '（'))
        || (matches!(prev, ')' | '）') && is_japanese(next))
    {
        return Some(RuleId::SpaceBetweenJaAndParen);
    }
    None
}

fn is_japanese(c: char) -> bool {
    JAPANESE.is_match(c.encode_utf8(&mut [0; 4]))
}

#[cfg(test)]
// 本文の範囲を1つだけ渡すテストで、範囲の配列を意図して書いている
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;

    fn find_all(src: &str) -> Vec<(&str, &'static str, RuleId)> {
        find(src, &[0..src.len()])
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
        assert_eq!(find(src, &[0.."日本語 ".len()]), vec![]);
        assert_eq!(find(src, &["日本語".len()..src.len()]), vec![]);
    }

    #[test]
    fn ignores_text_outside_the_ranges_and_offsets_findings_by_range_start() {
        let src = "日本語 Rust と（外） 語 A";
        let start = src.find("語 A").unwrap();
        assert_eq!(
            find(src, &[start..src.len()]),
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
