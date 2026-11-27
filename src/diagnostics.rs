use std::borrow::Cow;

use schemars::JsonSchema;
use serde::Serialize;

use crate::apply::{LineIndex, TargetLines};
use crate::config::Config;
use crate::filter::Ignored;
use crate::rules::general::Links;
use crate::{prose, rules};

#[derive(Serialize, JsonSchema)]
pub struct Diagnostic {
    /// 1始まり
    pub line: usize,
    /// 文字で数える1始まりの列
    pub column: usize,
    pub rule_id: &'static str,
    pub message: Cow<'static, str>,
    pub fixable: bool,
}

pub fn check(
    src: &str,
    config: &Config,
    targets: &TargetLines,
    links: Option<&Links<'_>>,
    report_excluded: bool,
) -> Vec<Diagnostic> {
    let document = prose::parse(src);
    let builtin = config.builtin();
    let ignored = Ignored::new(src, &document, config.filters());
    let replacements = rules::user_replacements(src, &document, builtin, targets, &ignored);
    let fixes = rules::fix_findings(src, &document, builtin, &ignored);
    let reports = if report_excluded {
        Vec::new()
    } else {
        rules::report(
            src,
            &document,
            builtin,
            targets,
            (&fixes, Some(&replacements)),
            &ignored,
            links,
        )
    };
    let mut spots: Vec<_> = replacements
        .into_iter()
        .filter(|replacement| replacement.unfixable(src).is_none())
        .map(|replacement| replacement.finding)
        .chain(rules::fixable(&fixes))
        .map(|f| (f.position, f.message, f.rule.name(), true))
        .chain(
            reports
                .into_iter()
                .map(|r| (r.range.start, r.message, r.rule, false)),
        )
        .collect();
    spots.sort_by_key(|&(start, ..)| start);
    let index = LineIndex::new(src);
    spots
        .into_iter()
        .filter_map(|(start, message, rule_id, fixable)| {
            let (line, column) = index.line_col(start);
            targets.contains(line).then_some(Diagnostic {
                line,
                column,
                rule_id,
                message,
                fixable,
            })
        })
        .collect()
}
