use crate::prose::Document;
use crate::rules::Report;
use crate::rules::builtin::NoAiEmphasisPatterns;

/// 段落の絵文字と太字・太字の情報プレフィックスと、見出しの中の太字を指摘する
pub fn report(src: &str, document: &Document, rule: &NoAiEmphasisPatterns) -> Vec<Report<'static>> {
    let _ = (src, document, rule);
    todo!("skeleton: not implemented")
}
