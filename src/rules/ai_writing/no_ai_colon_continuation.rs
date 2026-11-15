use crate::prose::Paragraph;
use crate::rules::Report;
use crate::rules::builtin::NoAiColonContinuation;

/// 文書の直下の段落のうち、述語とコロンで終わり、次にコードブロック・リスト・引用・表が続くものの、そのコロンを指摘する
pub fn report(
    src: &str,
    paragraphs: &[Paragraph],
    rule: &NoAiColonContinuation,
) -> Vec<Report<'static>> {
    let _ = (src, paragraphs, rule);
    todo!("skeleton: not implemented")
}
