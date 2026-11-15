//! `date-weekday-mismatch`。本文の範囲の中の日付と、その直後の括弧の中の曜日が食い違えば指摘する

use std::borrow::Cow;
use std::cell::LazyCell;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use regress::Regex;

use super::date::{self, Year};
use crate::prose::Prose;
use crate::rules::builtin::DateWeekdayMismatch;
use crate::rules::{Report, RuleId, one_line};

/// textlintと同じく、日付の直後の最初の括弧だけを見る。中身は前後の空白を除いて曜日として読む
static PAREN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*?[(（])([^(（)]+)([)）])").unwrap());

/// `texts` のそれぞれで、日付の直後の括弧の中身を曜日として読み、日付から求めた曜日と違えば指摘を返す。同じ
/// 範囲と文面の指摘は1件にする
pub fn report(src: &str, texts: &[Prose], rule: &DateWeekdayMismatch) -> Vec<Report> {
    let current_year = LazyCell::new(|| {
        rule.current_year
            .map_or_else(system_year, |year| year as i64)
    });
    let mut reports = Vec::new();
    for prose in texts {
        let text = &src[prose.range.clone()];
        // 日付の書式を当てるのは重いので、曜日の括弧を持たない本文の範囲を先に除く
        if !text.contains(['(', '（']) {
            continue;
        }
        for found in date::find_dates(text) {
            let after = &text[found.range.end..];
            let Some(paren) = PAREN.find(after) else {
                continue;
            };
            let opening = paren.group(1).unwrap();
            let inside = paren.group(2).unwrap();
            let written = after[inside.clone()].trim();
            let Some(names) = date::weekday_name(written, rule.lang) else {
                continue;
            };
            let year = match found.year {
                Year::Written(year) => year,
                Year::Current => *current_year,
                Year::Missing if rule.use_current_year_if_missing => *current_year,
                Year::Missing | Year::Unknown => continue,
            };
            let Some(weekday) = date::weekday(year, found.month, found.day) else {
                continue;
            };
            let expected = names[weekday];
            if expected == written {
                continue;
            }
            let leading = after[inside.clone()].len() - after[inside.clone()].trim_start().len();
            let start = prose.range.start + found.range.end + inside.start + leading;
            let report = Report {
                range: start..start + written.len(),
                rule: RuleId::DateWeekdayMismatch.name(),
                message: Cow::Owned(format!(
                    "「{}」の曜日は「{expected}」で、「{written}」ではない",
                    one_line(&text[found.range.start..found.range.end + opening.end])
                )),
            };
            if !reports.contains(&report) {
                reports.push(report);
            }
        }
    }
    reports
}

/// 実行した時点のシステムの年 (UTC)
fn system_year() -> i64 {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() / 86_400) as i64;
    // 1970年1月1日からの日数を、3月始まりの年に直して年を求める
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let march_based_month = (5 * day_of_year + 2) / 153;
    era * 400 + year_of_era + i64::from(march_based_month >= 10)
}
