use crate::prose::Paragraph;
use crate::rules::Report;
use crate::rules::builtin::AiTechWritingGuideline;

/// 段落の文字列にパターンを当てた指摘と、リストの直前の段落の指摘を返す。`enableDocumentAnalysis` が有効で
/// 1件以上あれば、文書の先頭に件数の要約の指摘を足す
pub fn report(
    src: &str,
    paragraphs: &[Paragraph],
    rule: &AiTechWritingGuideline,
) -> Vec<Report<'static>> {
    let _ = (src, paragraphs, rule);
    todo!("skeleton: not implemented")
}
