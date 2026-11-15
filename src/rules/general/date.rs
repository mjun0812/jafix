//! `date-weekday-mismatch` が読む日付の書式と、曜日の一覧。日付の書式は `chrono-node` 1.4.9 の日本語の
//! `JPStandardParser` と、英語の `ENISOFormatParser`・`ENSlashDateFormatStartWithYearParser`・`ENSlashDateFormatParser`・
//! `ENMonthNameMiddleEndianParser`・`ENMonthNameLittleEndianParser` から写す。曜日の一覧は `moment` 2.31.0 の
//! `ja` と `en` の `weekdaysShort`・`weekdays`・`weekdaysMin` である

use std::ops::Range;

use crate::rules::builtin::WeekdayLang;

/// 本文の範囲の中で日付の書式に当たった1つ
pub struct DateMatch {
    /// 本文の範囲の文字列の中のバイト範囲。英語の書式の先頭の区切りの1文字は含めない
    pub range: Range<usize>,
    pub year: Year,
    pub month: u32,
    pub day: u32,
}

/// 日付の年の書き方
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // skeleton: not implemented
pub enum Year {
    /// 書かれた年。元号は西暦にし、2桁の年は50より大きければ1900年代、それ以外は2000年代にする
    Written(i64),
    /// `今年`・`本年`。`currentYear` の年である
    Current,
    /// 年が書かれていない。`useCurrentYearIfMissing` が真なら `currentYear` の年である
    Missing,
    /// `同年` と元号の無い `元年`。年を決めず、確かめない
    Unknown,
}

/// `text` に日付の書式を左から重ねずに当てる。同じ位置から複数の書式が当たれば、長い一致を使う
pub fn find_dates(text: &str) -> Vec<DateMatch> {
    let _ = (text, &MONTH_NAMES, &ORDINAL_WORDS);
    todo!("skeleton: not implemented")
}

/// 年・月・日の曜日 (日曜が0) を、グレゴリオ暦を年0より前にも延ばした暦で求める。存在しない日付 (月が1〜12でない、
/// 日がその月の日数を超える) なら `None`
pub fn weekday(year: i64, month: u32, day: u32) -> Option<usize> {
    let _ = (year, month, day);
    todo!("skeleton: not implemented")
}

/// `text` (括弧の中の前後の空白を除いた文字列) と完全に一致する曜日の一覧を返す。大文字と小文字を区別する。`lang` が
/// あれば、その言語の一覧だけを見る
pub fn weekday_name(text: &str, lang: Option<WeekdayLang>) -> Option<&'static [&'static str; 7]> {
    let _ = (text, lang, &WEEKDAYS);
    todo!("skeleton: not implemented")
}

/// 曜日の一覧と、その言語。各一覧は日曜から土曜の順である
pub const WEEKDAYS: [(WeekdayLang, [&str; 7]); 5] = [
    (WeekdayLang::Ja, ["日", "月", "火", "水", "木", "金", "土"]),
    (
        WeekdayLang::Ja,
        [
            "日曜日",
            "月曜日",
            "火曜日",
            "水曜日",
            "木曜日",
            "金曜日",
            "土曜日",
        ],
    ),
    (WeekdayLang::En, ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]),
    (
        WeekdayLang::En,
        ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
    ),
    (
        WeekdayLang::En,
        [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ],
    ),
];

/// 英語の月名と、月の番号。chrono-node 1.4.9 の `MONTH_OFFSET` の順で、正規表現では `.` を退避して `|` でつなぐ
pub const MONTH_NAMES: [(&str, u32); 36] = [
    ("january", 1),
    ("jan", 1),
    ("jan.", 1),
    ("february", 2),
    ("feb", 2),
    ("feb.", 2),
    ("march", 3),
    ("mar", 3),
    ("mar.", 3),
    ("april", 4),
    ("apr", 4),
    ("apr.", 4),
    ("may", 5),
    ("june", 6),
    ("jun", 6),
    ("jun.", 6),
    ("july", 7),
    ("jul", 7),
    ("jul.", 7),
    ("august", 8),
    ("aug", 8),
    ("aug.", 8),
    ("september", 9),
    ("sep", 9),
    ("sep.", 9),
    ("sept", 9),
    ("sept.", 9),
    ("october", 10),
    ("oct", 10),
    ("oct.", 10),
    ("november", 11),
    ("nov", 11),
    ("nov.", 11),
    ("december", 12),
    ("dec", 12),
    ("dec.", 12),
];

/// 日を書く序数の語と、日。chrono-node 1.4.9 の `ORDINAL_WORDS` の順で、正規表現では空白を `[ -]` にして `|` でつなぐ
pub const ORDINAL_WORDS: [(&str, u32); 31] = [
    ("first", 1),
    ("second", 2),
    ("third", 3),
    ("fourth", 4),
    ("fifth", 5),
    ("sixth", 6),
    ("seventh", 7),
    ("eighth", 8),
    ("ninth", 9),
    ("tenth", 10),
    ("eleventh", 11),
    ("twelfth", 12),
    ("thirteenth", 13),
    ("fourteenth", 14),
    ("fifteenth", 15),
    ("sixteenth", 16),
    ("seventeenth", 17),
    ("eighteenth", 18),
    ("nineteenth", 19),
    ("twentieth", 20),
    ("twenty first", 21),
    ("twenty second", 22),
    ("twenty third", 23),
    ("twenty fourth", 24),
    ("twenty fifth", 25),
    ("twenty sixth", 26),
    ("twenty seventh", 27),
    ("twenty eighth", 28),
    ("twenty ninth", 29),
    ("thirtieth", 30),
    ("thirty first", 31),
];
