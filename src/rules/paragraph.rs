#![expect(dead_code, unused_variables, reason = "skeleton: not implemented")]

use crate::prose::Paragraph;

use super::Report;
use super::builtin::Builtin;

/// 段落ごとに当てる指摘ルール (`sentence-length`、`max-comma`、`ja-no-mixed-period`、`no-unmatched-pair`) の指摘を返す
pub fn report(src: &str, paragraphs: &[Paragraph], builtin: &Builtin) -> Vec<Report<'static>> {
    todo!("skeleton: not implemented")
}
