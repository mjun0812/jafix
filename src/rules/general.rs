//! 言語によらない3つの指摘ルール (`no-todo`・`date-weekday-mismatch`・`no-dead-link`)

mod date;
mod date_weekday_mismatch;
mod no_dead_link;
mod no_todo;

use std::path::Path;

use crate::filter::Ignored;
use crate::prose::Document;
use crate::rules::Report;
use crate::rules::builtin::Builtin;

pub use self::no_dead_link::UrlChecker;

/// `no-dead-link` が宛先を確かめる文脈
pub struct Links<'a> {
    /// 検査するMarkdownのファイル。相対の宛先は、`baseURI` が無ければこのファイルを基準に解決する
    pub file: &'a Path,
    /// 外部のURLを確かめるなら、その結果をプロセスの中で共有する確認器。hookでは `None` で、`http:`・`https:` に
    /// 解決した宛先を確かめない
    pub urls: Option<&'a UrlChecker>,
}

/// 有効な3つのルールの指摘を返す。`ignored` に含まれる宛先は、確かめる前に飛ばす
pub fn report(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    links: Option<&Links<'_>>,
    ignored: &Ignored,
) -> Vec<Report> {
    let mut reports = Vec::new();
    if builtin.no_todo {
        reports.extend(no_todo::report(src, document));
    }
    if let Some(rule) = &builtin.date_weekday_mismatch {
        reports.extend(date_weekday_mismatch::report(src, &document.texts, rule));
    }
    if let (Some(rule), Some(links)) = (&builtin.no_dead_link, links) {
        reports.extend(no_dead_link::report(document, rule, links, ignored));
    }
    reports
}
