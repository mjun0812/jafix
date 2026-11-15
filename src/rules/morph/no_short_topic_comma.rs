use super::AnalyzedProse;
use crate::rules::Report;
use crate::rules::builtin::NoShortTopicComma;

/// 係助詞の `は` の直後の読点のうち、文頭から読点の手前までの文字数が `maxLength` 以下のものを指摘する
pub fn report(prose: &AnalyzedProse, rule: &NoShortTopicComma) -> Vec<Report<'static>> {
    let _ = (prose, rule);
    todo!("skeleton: not implemented")
}
