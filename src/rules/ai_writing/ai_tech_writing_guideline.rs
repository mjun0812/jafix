use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{BlockKind, Paragraph};
use crate::rules::builtin::AiTechWritingGuideline;
use crate::rules::morph::projection::Projection;
use crate::rules::{Report, RuleId};

use super::allowed;
use super::table::{GUIDELINES, GuidelineCategory, LIST_INTRO};

static PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    GUIDELINES
        .iter()
        .map(|guideline| Regex::new(guideline.pattern).unwrap())
        .collect()
});

static INTRO: LazyLock<Regex> = LazyLock::new(|| Regex::new(LIST_INTRO).unwrap());

/// 要約の内訳に並べる順
const CATEGORIES: [GuidelineCategory; 5] = [
    GuidelineCategory::Redundancy,
    GuidelineCategory::Voice,
    GuidelineCategory::Clarity,
    GuidelineCategory::Consistency,
    GuidelineCategory::Structure,
];

/// 段落の文字列にパターンを当てた指摘と、リストの直前の段落の指摘を返す。`enableDocumentAnalysis` が有効で
/// 1件以上あれば、文書の先頭に件数の要約の指摘を足す
pub fn report(
    src: &str,
    paragraphs: &[Paragraph],
    rule: &AiTechWritingGuideline,
) -> Vec<Report<'static>> {
    let enabled = |category: GuidelineCategory| match category {
        GuidelineCategory::Redundancy => !rule.disable_redundancy_guidance,
        GuidelineCategory::Voice => !rule.disable_voice_guidance,
        GuidelineCategory::Clarity => !rule.disable_clarity_guidance,
        GuidelineCategory::Consistency => !rule.disable_consistency_guidance,
        GuidelineCategory::Structure => !rule.disable_structure_guidance,
    };
    let mut reports = Vec::new();
    let mut counts = [0; CATEGORIES.len()];
    let mut push = |range: Range<usize>, category: GuidelineCategory, message: String| {
        counts[index(category)] += 1;
        reports.push(Report {
            range,
            rule: RuleId::AiTechWritingGuideline.name(),
            message: Cow::Owned(format!("{}: {message}", name(category))),
        });
    };
    for paragraph in paragraphs {
        // 21個のパターンは `ユーザー` を含むので、`ー` にすると `ユーザ` の直後のコードがパターンの語の一部になる。
        // 半角空白なら語の一部にならず、末尾の `\s*$` も、textlintがコードを空にしたときと同じく当たる
        let projection = Projection::paragraph(src, paragraph, ' ');
        let text = &projection.text;
        if allowed(&rule.allows, text) {
            continue;
        }
        if enabled(GuidelineCategory::Structure)
            && paragraph.next_block == Some(BlockKind::List)
            && let Some(intro) = INTRO.find(text)
        {
            let word = intro.as_str().trim_end().trim_end_matches('。');
            push(
                paragraph.range.clone(),
                GuidelineCategory::Structure,
                format!("段落を「{word}。」で終えてリストを続けている"),
            );
        }
        for (guideline, pattern) in GUIDELINES.iter().zip(PATTERNS.iter()) {
            if !enabled(guideline.category) {
                continue;
            }
            for m in pattern.find_iter(text) {
                push(
                    projection.source(m.start())..projection.source(m.end()),
                    guideline.category,
                    guideline.message.to_owned(),
                );
            }
        }
    }
    let total: usize = counts.iter().sum();
    if rule.enable_document_analysis && total > 0 {
        let details: Vec<String> = CATEGORIES
            .into_iter()
            .filter(|&category| counts[index(category)] > 0)
            .map(|category| format!("{}: {}件", name(category), counts[index(category)]))
            .collect();
        reports.insert(
            0,
            Report {
                range: 0..0,
                rule: RuleId::AiTechWritingGuideline.name(),
                message: Cow::Owned(format!(
                    "この文書でai-tech-writing-guidelineの指摘が{total}件ある({})",
                    details.join("、")
                )),
            },
        );
    }
    reports
}

/// `CATEGORIES` の並びでの、カテゴリーの位置
fn index(category: GuidelineCategory) -> usize {
    match category {
        GuidelineCategory::Redundancy => 0,
        GuidelineCategory::Voice => 1,
        GuidelineCategory::Clarity => 2,
        GuidelineCategory::Consistency => 3,
        GuidelineCategory::Structure => 4,
    }
}

fn name(category: GuidelineCategory) -> &'static str {
    match category {
        GuidelineCategory::Redundancy => "簡潔性",
        GuidelineCategory::Voice => "明確性",
        GuidelineCategory::Clarity => "具体性",
        GuidelineCategory::Consistency => "一貫性",
        GuidelineCategory::Structure => "構造化",
    }
}
