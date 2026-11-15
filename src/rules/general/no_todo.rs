//! `no-todo`。本文の範囲の `todo:` と、リスト項目の未完了のタスクの記号を指摘する

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regress::Regex;

use crate::prose::{Document, InlineKind, ListItem};
use crate::rules::{Report, RuleId, one_line};

/// textlintと同じく、`\s` はJavaScriptの空白で、全角空白を含む
static TASK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\s+\]\s").unwrap());

/// リンクの文字列と引用の中を除く本文の範囲の `todo:` (大文字と小文字を区別しない) のすべての一致と、リスト項目ごとの
/// 最初の未完了のタスクの記号 (項目の先頭のGFMのタスクの記号か、項目の直下の段落の直下の文字列の `[ ]`) を返す
pub fn report(src: &str, document: &Document) -> Vec<Report> {
    let mut reports = Vec::new();
    for prose in &document.texts {
        if prose.context.link || prose.context.blockquote {
            continue;
        }
        // ASCIIだけを小文字にするので、バイト位置は元の文字列と同じである
        let lowered = src[prose.range.clone()].to_ascii_lowercase();
        for (offset, todo) in lowered.match_indices("todo:") {
            let start = prose.range.start + offset;
            reports.push(Report {
                range: start..start + todo.len(),
                rule: RuleId::NoTodo.name(),
                message: Cow::Owned(format!(
                    "TODOが残っている: 「{}」",
                    one_line(&src[start..prose.range.end])
                )),
            });
        }
    }
    // 項目の直下の段落の最初の記号を、項目ごとに1回の走査で集める
    let mut paragraph_boxes = vec![None; document.list_items.len()];
    for paragraph in &document.paragraphs {
        let Some(index) = paragraph.list_item_index else {
            continue;
        };
        if paragraph_boxes[index].is_some() {
            continue;
        }
        paragraph_boxes[index] = paragraph
            .children
            .iter()
            .filter(|inline| inline.kind == InlineKind::Text)
            .find_map(|inline| unchecked_box(src, inline.range.clone()));
    }
    for (item, paragraph_box) in document.list_items.iter().zip(paragraph_boxes) {
        let task = task_marker(src, item).or(paragraph_box);
        if let Some(range) = task {
            reports.push(Report {
                range,
                rule: RuleId::NoTodo.name(),
                message: Cow::Borrowed("未完了のタスクが残っている"),
            });
        }
    }
    reports
}

/// 項目の先頭のGFMのタスクの記号が未完了で、後ろに空白が続くなら、`[` から `]` までの範囲。markdown-rsは `- [ ]` のように
/// 記号の後ろに何も無い項目もタスクにするが、textlintの正規表現は後ろの空白を求めるので当てない
fn task_marker(src: &str, item: &ListItem) -> Option<Range<usize>> {
    if item.checked != Some(false) {
        return None;
    }
    let marker_end = src[item.range.clone()]
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '*' | '+' | '.' | ')')))?;
    let rest = &src[item.range.start + marker_end..item.range.end];
    let start = item.range.start + marker_end + (rest.len() - rest.trim_start().len());
    unchecked_box(src, start..item.range.end).filter(|range| range.start == start)
}

/// `range` の中の最初の `[ ]` (角括弧の間が1文字以上の空白で、後ろに空白が続くもの) の、`[` から `]` までの範囲
fn unchecked_box(src: &str, range: Range<usize>) -> Option<Range<usize>> {
    let found = TASK.find(&src[range.clone()])?;
    let start = range.start + found.start();
    let close = src[start..range.start + found.end()].rfind(']')?;
    Some(start..start + close + 1)
}
