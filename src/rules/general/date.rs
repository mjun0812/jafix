//! `date-weekday-mismatch` が読む日付の書式と、曜日の一覧。日付の書式は `chrono-node` 1.4.9 の日本語の
//! `JPStandardParser` と、英語の `ENISOFormatParser`・`ENSlashDateFormatStartWithYearParser`・`ENSlashDateFormatParser`・
//! `ENMonthNameMiddleEndianParser`・`ENMonthNameLittleEndianParser` から写す。曜日の一覧は `moment` 2.31.0 の
//! `ja` と `en` の `weekdaysShort`・`weekdays`・`weekdaysMin` である

use std::ops::Range;
use std::sync::LazyLock;

use regress::{Match, Regex};

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

/// 1つの書式の正規表現と、一致から日付を読む関数
struct Format {
    regex: Regex,
    read: fn(&str, &Match) -> Read,
    /// 書式に当たる文字列が必ず含むもの。含まない文字列には、重い正規表現を当てない
    needs: Needs,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Needs {
    /// 半角か全角の数字
    Digit,
    /// 英語の月名の先頭の3文字 (大文字と小文字を区別しない)
    MonthName,
}

/// 一致を読んだ結果
enum Read {
    Date(DateMatch),
    /// 日付にしない。chronoと同じく、一致の始まりの次の文字から探し直す
    Reject,
    /// 日付にせず、一致の終わりの次の文字から探し直す
    Skip,
}

/// chrono-node の casual の順に並べる。同じ位置から同じ長さで当たれば、前の書式の一致が残る
static FORMATS: LazyLock<[Format; 6]> = LazyLock::new(|| {
    let format = |pattern: String, flags: &str, read, needs| Format {
        regex: Regex::with_flags(&pattern, flags).unwrap(),
        read,
        needs,
    };
    let month = format!(
        "(?:{})",
        MONTH_NAMES
            .iter()
            .map(|(name, _)| name.replace('.', "\\."))
            .collect::<Vec<_>>()
            .join("|")
    );
    let ordinal = format!(
        "(?:{})",
        ORDINAL_WORDS
            .iter()
            .map(|(word, _)| word.replace(' ', "[ -]"))
            .collect::<Vec<_>>()
            .join("|")
    );
    let day = format!("(([0-9]{{1,2}})(?:st|nd|rd|th)?|{ordinal})");
    [
        format(
            r"(\W|^)([0-9]{4})-([0-9]{1,2})-([0-9]{1,2})(?:T[0-9]{1,2}:[0-9]{1,2}(?::[0-9]{1,2}(?:\.\d{1,4})?)?(?:Z|[+-]\d{2}:?(?:\d{2})?)?)?(?=\W|$)".to_owned(),
            "i",
            read_iso,
            Needs::Digit,
        ),
        format(
            format!(
                r"(\W|^)(?:on\s*?)?(?:(Sunday|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sun|Mon|Tue|Wed|Thu|Fri|Sat)\s*,?\s*)?{day}(?:\s*(?:to|\-|\u2013|until|through|till|\s)\s*{day})?\s*(?:of)?\s*({month})(?:,?\s*([0-9]{{1,4}}(?![^\s]\d))(\s*(?:BE|AD|BC))?)?(?=\W|$)"
            ),
            "i",
            read_little_endian,
            Needs::MonthName,
        ),
        format(
            format!(
                r"(\W|^)(?:(?:on\s*?)?(Sunday|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sun\.?|Mon\.?|Tue\.?|Wed\.?|Thu\.?|Fri\.?|Sat\.?)\s*,?\s*)?({month})(?:-|/|\s*,?\s*){day}(?!\s*(?:am|pm))\s*(?:(?:to|\-)\s*{day}\s*)?(?:(?:-|/|\s*,?\s*)(?:([0-9]{{4}})\s*(BE|AD|BC)?|([0-9]{{1,4}})\s*(AD|BC))\s*)?(?=\W|$)(?!:\d)"
            ),
            "i",
            read_middle_endian,
            Needs::MonthName,
        ),
        format(
            r"(\W|^)(?:(?:on\s*?)?((?:sun|mon|tues?|wed(?:nes)?|thu(?:rs?)?|fri|sat(?:ur)?)(?:day)?)\s*,?\s*)?([0-3]?[0-9])[/.\-]([0-3]?[0-9])(?:[/.\-]([0-9]{4}\s*,?\s*|[0-9]{2}\s*,?\s*))?(\W|$)".to_owned(),
            "i",
            read_month_first,
            Needs::Digit,
        ),
        format(
            format!(
                r"(\W|^)([0-9]{{4}})[\-./]({month}|[0-9]{{1,2}})[\-./]([0-9]{{1,2}})(?=\W|$)"
            ),
            "i",
            read_year_first,
            Needs::Digit,
        ),
        format(
            r"(?:(?:(同|今|本|元)|(昭和|平成|令和)([0-9０-９]{1,4}|元)|([0-9０-９]{1,4}))年\s*)?([0-9０-９]{1,2})月\s*([0-9０-９]{1,2})日".to_owned(),
            "",
            read_japanese,
            Needs::Digit,
        ),
    ]
});

/// `text` に日付の書式を左から重ねずに当てる。同じ位置から複数の書式が当たれば、長い一致を使う
pub fn find_dates(text: &str) -> Vec<DateMatch> {
    let has_digit = text.contains(|c: char| c.is_ascii_digit() || ('０'..='９').contains(&c));
    let lowered = text.to_ascii_lowercase();
    let has_month_name = MONTH_NAMES
        .iter()
        .any(|(name, _)| lowered.contains(&name[..3]));
    let mut dates = Vec::new();
    for format in FORMATS.iter() {
        let possible = match format.needs {
            Needs::Digit => has_digit,
            Needs::MonthName => has_month_name,
        };
        if !possible {
            continue;
        }
        // chronoと同じく、探し直すたびに残りの文字列の先頭を `^` に当てる
        let mut start = 0;
        while start <= text.len() {
            let rest = &text[start..];
            let Some(found) = format.regex.find(rest) else {
                break;
            };
            match (format.read)(rest, &found) {
                Read::Date(mut date) => {
                    date.range = start + date.range.start..start + date.range.end;
                    start = date.range.end;
                    dates.push(date);
                }
                Read::Reject => start += next_char(rest, found.start()),
                Read::Skip => start += next_char(rest, found.end()),
            }
        }
    }
    // chronoの `OverlapRemovalRefiner` と同じく、位置の順に並べ、前の一致と重なれば長いほうを残す
    dates.sort_by_key(|date| date.range.start);
    let mut kept: Vec<DateMatch> = Vec::new();
    for date in dates {
        match kept.last_mut() {
            Some(last) if date.range.start < last.range.end => {
                if date.range.len() > last.range.len() {
                    *last = date;
                }
            }
            _ => kept.push(date),
        }
    }
    kept
}

/// `i` の次の文字の位置。`i` が末尾なら末尾の1つ先である
fn next_char(text: &str, i: usize) -> usize {
    i + text[i..].chars().next().map_or(1, char::len_utf8)
}

/// 一致の `group` 番目の組の文字列
fn group<'t>(text: &'t str, found: &Match, group: usize) -> Option<&'t str> {
    found.group(group).map(|range| &text[range])
}

/// 英語の書式の一致のうち、先頭の区切り (1番目の組) を除いた範囲
fn after_opening(found: &Match) -> Range<usize> {
    found.group(1).map_or(found.start(), |opening| opening.end)..found.end()
}

/// 全角の数字を半角として読む
fn number(digits: &str) -> i64 {
    digits
        .chars()
        .filter_map(|c| match c {
            '０'..='９' => Some(c as u32 - '０' as u32),
            _ => c.to_digit(10),
        })
        .fold(0, |n, digit| n * 10 + i64::from(digit))
}

fn month_name(name: &str) -> u32 {
    let name = name.to_ascii_lowercase();
    MONTH_NAMES
        .iter()
        .find(|(month, _)| *month == name)
        .map_or(0, |&(_, month)| month)
}

/// `([0-9]{1,2})(?:st|nd|rd|th)?` か序数の語の日。`digits` は数字の組である
fn day_of(word: &str, digits: Option<&str>) -> u32 {
    if let Some(digits) = digits {
        return number(digits) as u32;
    }
    let word = word.to_ascii_lowercase().replace('-', " ");
    ORDINAL_WORDS
        .iter()
        .find(|(ordinal, _)| *ordinal == word)
        .map_or(0, |&(_, day)| day)
}

fn date(range: Range<usize>, year: Year, month: u32, day: u32) -> Read {
    Read::Date(DateMatch {
        range,
        year,
        month,
        day,
    })
}

/// `ENISOFormatParser`。`2024-04-23` と、続く時刻
fn read_iso(text: &str, found: &Match) -> Read {
    let field = |i| number(group(text, found, i).unwrap());
    date(
        after_opening(found),
        Year::Written(field(2)),
        field(3) as u32,
        field(4) as u32,
    )
}

/// `ENSlashDateFormatStartWithYearParser`。`2024/04/23`・`2024.04.23`・`2024/Apr/23`
fn read_year_first(text: &str, found: &Match) -> Read {
    let month = group(text, found, 3).unwrap();
    let month = if month.starts_with(|c: char| c.is_ascii_digit()) {
        number(month) as u32
    } else {
        month_name(month)
    };
    date(
        after_opening(found),
        Year::Written(number(group(text, found, 2).unwrap())),
        month,
        number(group(text, found, 4).unwrap()) as u32,
    )
}

/// `ENSlashDateFormatParser`。`04/23/2024`・`23/04/2024`・`4/23/24`・`4.23.2024`・`4/23`
fn read_month_first(text: &str, found: &Match) -> Read {
    let opening = group(text, found, 1).unwrap();
    let ending = group(text, found, 6).unwrap();
    if opening == "/" || ending == "/" {
        return Read::Skip;
    }
    let range = found.group(1).unwrap().end..found.group(6).unwrap().start;
    let matched = &text[range.clone()];
    let year = group(text, found, 5);
    if is_decimal_like(matched) || (year.is_none() && !text[found.range()].contains('/')) {
        return Read::Reject;
    }
    let mut month = number(group(text, found, 3).unwrap()) as u32;
    let mut day = number(group(text, found, 4).unwrap()) as u32;
    if month > 12 {
        if !(1..=12).contains(&day) || month > 31 {
            return Read::Reject;
        }
        std::mem::swap(&mut month, &mut day);
    }
    if !(1..=31).contains(&day) {
        return Read::Reject;
    }
    let year = match year {
        Some(year) => written_year(year),
        None => Year::Missing,
    };
    date(range, year, month, day)
}

/// 書かれた英語の書式の年。2桁までの年は、50より大きければ1900年代、それ以外は2000年代とする
fn written_year(digits: &str) -> Year {
    let year = number(digits);
    Year::Written(match year {
        0..=50 => year + 2000,
        51..=99 => year + 1900,
        _ => year,
    })
}

/// `d.d` と `d.dd.dd` の形は、日付ではなく番号か小数とみなす
fn is_decimal_like(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    let digits = |part: &str, max: usize| {
        (1..=max).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_digit())
    };
    match parts.as_slice() {
        [a, b] => digits(a, 1) && digits(b, 1),
        [a, b, c] => digits(a, 1) && digits(b, 2) && digits(c, 2),
        _ => false,
    }
}

/// `ENMonthNameMiddleEndianParser`。`April 23, 2024`・`Tuesday, Apr. 23rd 2024`・`April 23`
fn read_middle_endian(text: &str, found: &Match) -> Read {
    // 範囲の2つ目の日 (6番目の組) と、`BE`・`AD`・`BC` の付く年 (9番目と11番目の組) は日付にしない
    if [6, 9, 11].iter().any(|&i| found.group(i).is_some()) {
        return Read::Reject;
    }
    let year = group(text, found, 8).map_or(Year::Missing, |year| Year::Written(number(year)));
    date(
        after_opening(found),
        year,
        month_name(group(text, found, 3).unwrap()),
        day_of(group(text, found, 4).unwrap(), group(text, found, 5)),
    )
}

/// `ENMonthNameLittleEndianParser`。`23 April 2024`・`23rd of April, 2024`・`twenty-third of April`
fn read_little_endian(text: &str, found: &Match) -> Read {
    // 範囲の2つ目の日 (5番目の組) と、`BE`・`AD`・`BC` の付く年 (9番目の組) は日付にしない
    if [5, 9].iter().any(|&i| found.group(i).is_some()) {
        return Read::Reject;
    }
    let year = group(text, found, 8).map_or(Year::Missing, written_year);
    date(
        after_opening(found),
        year,
        month_name(group(text, found, 7).unwrap()),
        day_of(group(text, found, 3).unwrap(), group(text, found, 4)),
    )
}

/// `JPStandardParser`。`2024年4月23日`・`令和6年4月23日`・`今年4月23日`・`4月23日`
fn read_japanese(text: &str, found: &Match) -> Read {
    let year = match (
        group(text, found, 1),
        group(text, found, 2),
        group(text, found, 4),
    ) {
        (Some("今" | "本"), ..) => Year::Current,
        // `同年` は文脈の年を指し、元号の無い `元年` は年を決めない
        (Some(_), ..) => Year::Unknown,
        (_, Some(era), _) => {
            let year = match group(text, found, 3).unwrap() {
                "元" => 1,
                year => number(year),
            };
            Year::Written(
                year + match era {
                    "令和" => 2018,
                    "平成" => 1988,
                    _ => 1925,
                },
            )
        }
        (_, _, Some(year)) => Year::Written(number(year)),
        _ => Year::Missing,
    };
    date(
        found.range(),
        year,
        number(group(text, found, 5).unwrap()) as u32,
        number(group(text, found, 6).unwrap()) as u32,
    )
}

/// 年・月・日の曜日 (日曜が0) を、グレゴリオ暦を年0より前にも延ばした暦で求める。存在しない日付 (月が1〜12でない、
/// 日がその月の日数を超える) なら `None`
pub fn weekday(year: i64, month: u32, day: u32) -> Option<usize> {
    let leap = year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if !(1..=days_in_month).contains(&day) {
        return None;
    }
    // 1970年1月1日 (木曜) からの日数。3月始まりの年で数えると、閏日が年の末尾に来る
    let (month, day) = (i64::from(month), i64::from(day));
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some((days + 4).rem_euclid(7) as usize)
}

/// `text` (括弧の中の前後の空白を除いた文字列) と完全に一致する曜日の一覧を返す。大文字と小文字を区別する。`lang` が
/// あれば、その言語の一覧だけを見る
pub fn weekday_name(text: &str, lang: Option<WeekdayLang>) -> Option<&'static [&'static str; 7]> {
    WEEKDAYS
        .iter()
        .filter(|(names_lang, _)| lang.is_none_or(|lang| lang == *names_lang))
        .map(|(_, names)| names)
        .find(|names| names.contains(&text))
}

/// 曜日の一覧と、その言語。各一覧は日曜から土曜の順である
pub static WEEKDAYS: [(WeekdayLang, [&str; 7]); 5] = [
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
