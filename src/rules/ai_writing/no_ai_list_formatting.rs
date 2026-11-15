use crate::prose::Document;
use crate::rules::Report;
use crate::rules::builtin::NoAiListFormatting;

/// リスト項目ごとに、先頭の「**太字**: 説明」型と、項目に属する段落の絵文字を、それぞれ最初の1件だけ指摘する
pub fn report(src: &str, document: &Document, rule: &NoAiListFormatting) -> Vec<Report<'static>> {
    let _ = (src, document, rule);
    todo!("skeleton: not implemented")
}
