use crate::prose::Prose;
use crate::rules::Report;
use crate::rules::builtin::NoAiHypeExpressions;

/// 本文の範囲ごとに、誇張した表現の語の重ならない一致をすべて指摘する
pub fn report(src: &str, texts: &[Prose], rule: &NoAiHypeExpressions) -> Vec<Report<'static>> {
    let _ = (src, texts, rule);
    todo!("skeleton: not implemented")
}
