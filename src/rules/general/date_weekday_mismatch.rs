//! `date-weekday-mismatch`。本文の範囲の中の日付と、その直後の括弧の中の曜日が食い違えば指摘する

use super::date::{self, DateMatch};
use crate::prose::Prose;
use crate::rules::Report;
use crate::rules::builtin::DateWeekdayMismatch;

/// `texts` のそれぞれで、日付の直後の括弧の中身を曜日として読み、日付から求めた曜日と違えば指摘を返す。同じ
/// 範囲と文面の指摘は1件にする
pub fn report(src: &str, texts: &[Prose], rule: &DateWeekdayMismatch) -> Vec<Report> {
    let _ = (
        src,
        texts,
        rule,
        date::find_dates,
        date::weekday,
        date::weekday_name,
    );
    let _ = |found: DateMatch| (found.range, found.year, found.month, found.day);
    todo!("skeleton: not implemented")
}
