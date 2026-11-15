//! 英語の4つのルール (`terminology`・`spelling`・`common-misspellings`・`write-good`)。`terminology` の利用者の
//! `terms` の項目の箇所は利用者の置き換えで、ほかはすべて指摘である

mod common_misspellings;
mod spelling;
mod terminology;
pub mod terms;
mod write_good;

use std::ops::Range;

use crate::prose::{Document, Prose};

use super::builtin::{Builtin, Terminology};
use super::{Report, UserReplacement};

/// 有効な4つのルールの指摘を返す。`texts` と `alts` は、対象行と重なる本文の範囲と、インラインの画像のaltの元のソース上の
/// バイト範囲である。`terminology` は `texts` に当たる組み込みの辞書の項目の箇所を、`spelling` は `texts` と `alts` の語を、
/// `common-misspellings` と `write-good` は文書の本文の範囲を見る。利用者の `terms` の項目の直さない箇所は、
/// `rules::report` が利用者の置き換えとして指摘にする
pub fn report(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    texts: &[&Prose],
    alts: &[Range<usize>],
) -> Vec<Report> {
    let mut reports = Vec::new();
    if let Some(rule) = &builtin.terminology {
        reports.extend(terminology::report(src, texts, rule));
    }
    if let Some(rule) = &builtin.spelling {
        reports.extend(spelling::report(src, texts, alts, rule));
    }
    if let Some(rule) = &builtin.common_misspellings {
        reports.extend(common_misspellings::report(src, &document.texts, rule));
    }
    if let Some(rule) = &builtin.write_good {
        reports.extend(write_good::report(src, &document.texts, rule));
    }
    reports
}

/// `terminology` の利用者の `terms` の項目の箇所を返す。重なりは `rules::replacements` が解く
pub fn replacements(src: &str, texts: &[&Prose], rule: &Terminology) -> Vec<UserReplacement> {
    terminology::replacements(src, texts, rule)
}
