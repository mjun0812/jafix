use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

mod kangxi;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::js_regex;
use crate::prose::Prose;

use super::builtin::{AlphabetWidth, Builtin};
use super::{Finding, Report, RuleId};

/// 本文の範囲ごとに当てる修正ルール (`no-nfd`、`no-hankaku-kana`、`no-zero-width-spaces`、
/// `no-invalid-control-character`、`no-mixed-zenkaku-and-hankaku-alphabet`、`no-kangxi-radicals`) の箇所を返す。
/// 見出しの中の箇所も返し、指摘にするかは呼び出し側が決める。
/// `no-nfd` は、前の文字と合成して1文字になる組み合わせだけを返す
pub fn find<'a>(
    src: &str,
    texts: impl IntoIterator<Item = &'a Prose>,
    builtin: &Builtin,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for Prose { range, context } in texts {
        let text = &src[range.clone()];
        let at = |start: usize,
                  end: usize,
                  replacement: String,
                  rule: RuleId,
                  message: Cow<'static, str>| {
            Finding {
                range: range.start + start..range.start + end,
                replacement: Cow::Owned(replacement),
                rule,
                position: range.start + start,
                message,
            }
        };
        // textlintは、祖先に引用・強調・リンクがある文字列の濁点を見ない
        if builtin.no_nfd && !(context.blockquote || context.emphasis || context.link) {
            for mark in nfd_marks(text) {
                let Some(composed) = mark.composed else {
                    continue;
                };
                findings.push(Finding {
                    // textlintと同じく、置き換える2文字のうち濁点・半濁点の位置に出す
                    position: range.start + mark.position,
                    ..at(
                        mark.range.start,
                        mark.range.end,
                        composed.to_string(),
                        RuleId::NoNfd,
                        "濁点・半濁点を前の文字と合成する".into(),
                    )
                });
            }
        }
        if builtin.no_hankaku_kana && context.plain {
            for run in HANKAKU_KANA.find_iter(text) {
                findings.push(at(
                    run.start(),
                    run.end(),
                    run.as_str().nfkc().collect(),
                    RuleId::NoHankakuKana,
                    "半角カナを全角にする".into(),
                ));
            }
        }
        if builtin.no_zero_width_spaces {
            for (i, _) in text.match_indices('\u{200B}') {
                findings.push(at(
                    i,
                    i + '\u{200B}'.len_utf8(),
                    String::new(),
                    RuleId::NoZeroWidthSpaces,
                    "ゼロ幅スペースを削除する".into(),
                ));
            }
        }
        if let Some(rule) = &builtin.no_invalid_control_character {
            for (i, c) in text.char_indices() {
                if is_invalid_control_character(c) && !rule.allow.contains(&c) {
                    findings.push(at(
                        i,
                        i + c.len_utf8(),
                        String::new(),
                        RuleId::NoInvalidControlCharacter,
                        format!("制御文字 (U+{:04X}) を削除する", u32::from(c)).into(),
                    ));
                }
            }
        }
        // textlintは、祖先に引用・強調・リンクがある文字列の英字を見ない
        if let Some(rule) = &builtin.no_mixed_zenkaku_and_hankaku_alphabet
            && !(context.blockquote || context.emphasis || context.link)
        {
            // 全角英字は、同じ文字の半角英字にU+FEE0を足した位置にある
            let (alphabets, width, other, convert): (&Regex, _, _, fn(u32) -> u32) = match rule
                .prefer
            {
                AlphabetWidth::Half => (&FULLWIDTH_ALPHABETS, "全角", "半角", |c| c - 0xFEE0),
                AlphabetWidth::Full => (&HALFWIDTH_ALPHABETS, "半角", "全角", |c| c + 0xFEE0),
            };
            for run in alphabets.find_iter(text) {
                let replacement = run
                    .as_str()
                    .chars()
                    .map(|c| char::from_u32(convert(u32::from(c))))
                    .collect::<Option<String>>()
                    .expect("英字の幅を変えた文字は必ずある");
                findings.push(at(
                    run.start(),
                    run.end(),
                    replacement,
                    RuleId::NoMixedZenkakuAndHankakuAlphabet,
                    format!("{width}のアルファベット「{}」を{other}にする", run.as_str()).into(),
                ));
            }
        }
        if builtin.no_kangxi_radicals {
            for (i, c) in text.char_indices() {
                let Some(&kanji) = (u32::from(c))
                    .checked_sub(u32::from(kangxi::FIRST))
                    .and_then(|offset| kangxi::KANJI.get(offset as usize))
                else {
                    continue;
                };
                findings.push(at(
                    i,
                    i + c.len_utf8(),
                    kanji.to_string(),
                    RuleId::NoKangxiRadicals,
                    format!("康熙部首「{c}」を漢字「{kanji}」にする").into(),
                ));
            }
        }
    }
    findings
}

/// textlintの正規表現の範囲をそのまま写す。タブ・改行・復帰と、U+2066からU+2069・U+200E・U+200Fは含めない
fn is_invalid_control_character(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{7F}'..='\u{9F}' | '\u{202A}'..='\u{202E}')
}

static FULLWIDTH_ALPHABETS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("[Ａ-Ｚａ-ｚ]+").unwrap());

static HALFWIDTH_ALPHABETS: LazyLock<Regex> = LazyLock::new(|| Regex::new("[A-Za-z]+").unwrap());

static HANKAKU_KANA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\x{FF61}-\x{FF9F}]+").unwrap());

/// 濁点・半濁点と、その前の文字との組み合わせ。範囲と位置は本文の範囲の中のバイト位置
struct NfdMark {
    /// 前の文字と濁点・半濁点を合わせた範囲
    range: Range<usize>,
    /// 濁点・半濁点の位置
    position: usize,
    /// 合成した1文字。1文字にならない組み合わせは `None`
    composed: Option<char>,
}

/// 本文の範囲の先頭以外にある濁点・半濁点 (結合文字と、単独のU+309B・U+309C) を返す
fn nfd_marks(text: &str) -> impl Iterator<Item = NfdMark> {
    text.char_indices().filter_map(|(i, c)| {
        if !matches!(c, '\u{3099}'..='\u{309C}') {
            return None;
        }
        let prev = text[..i].chars().next_back()?;
        let combining = match c {
            '\u{309B}' => '\u{3099}',
            '\u{309C}' => '\u{309A}',
            _ => c,
        };
        let mut composed = [prev, combining].into_iter().nfc();
        let composed = composed.next().filter(|_| composed.next().is_none());
        Some(NfdMark {
            range: i - prev.len_utf8()..i + c.len_utf8(),
            position: i,
            composed,
        })
    })
}

/// 本文の範囲ごとに当てる指摘ルール (`max-kanji-continuous-len`、`arabic-kanji-numbers`、
/// `no-exclamation-question-mark`、`ja-unnatural-alphabet`) の指摘と、`no-nfd` のうち合成できない組み合わせの指摘を返す
pub fn report(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    for Prose { range, context } in texts {
        let text = &src[range.clone()];
        let at = |start: usize, end: usize, rule: RuleId, message: String| Report {
            range: range.start + start..range.start + end,
            rule: rule.name(),
            message: Cow::Owned(message),
        };
        // textlintの4つのルールは、祖先に引用・強調・リンクがある文字列を見ない
        let in_quote_emphasis_or_link = context.blockquote || context.emphasis || context.link;
        if let Some(rule) = &builtin.max_kanji_continuous_len
            && !in_quote_emphasis_or_link
        {
            for run in KANJI_RUN.find_iter(text) {
                let len: usize = run.as_str().chars().map(char::len_utf16).sum();
                if len > rule.max && !rule.allow.iter().any(|word| word == run.as_str()) {
                    reports.push(at(
                        run.start(),
                        run.end(),
                        RuleId::MaxKanjiContinuousLen,
                        format!("漢字が{}文字以上続いている: {}", rule.max + 1, run.as_str()),
                    ));
                }
            }
        }
        if builtin.no_nfd && !in_quote_emphasis_or_link {
            for mark in nfd_marks(text).filter(|mark| mark.composed.is_none()) {
                reports.push(at(
                    mark.position,
                    mark.range.end,
                    RuleId::NoNfd,
                    "濁点・半濁点を前の文字と合成できない".to_owned(),
                ));
            }
        }
        if builtin.arabic_kanji_numbers && context.plain {
            for pattern in NUMBER_PATTERNS.iter() {
                if pattern.ignore.iter().any(|ignore| ignore.is_match(text)) {
                    continue;
                }
                let Some(m) = pattern.regex.find_iter(text).find(|m| {
                    pattern
                        .not_followed_by
                        .is_none_or(|c| !text[m.end()..].starts_with(c))
                }) else {
                    continue;
                };
                reports.push(at(
                    m.start(),
                    m.end(),
                    RuleId::ArabicKanjiNumbers,
                    format!("{}: {}", pattern.message, m.as_str()),
                ));
            }
        }
        if let Some(rule) = &builtin.no_exclamation_question_mark
            && !in_quote_emphasis_or_link
        {
            for (i, c) in text.char_indices() {
                let allowed_kind = match c {
                    '!' => rule.allow_half_width_exclamation,
                    '！' => rule.allow_full_width_exclamation,
                    '?' => rule.allow_half_width_question,
                    '？' => rule.allow_full_width_question,
                    _ => continue,
                };
                if allowed_kind || is_allowed(&rule.allow, text, i) {
                    continue;
                }
                reports.push(at(
                    i,
                    i + c.len_utf8(),
                    RuleId::NoExclamationQuestionMark,
                    format!("「{c}」を使わない"),
                ));
            }
        }
        if let Some(rule) = &builtin.ja_unnatural_alphabet {
            let mut start = 0;
            while let Some(captures) = UNNATURAL_ALPHABET.captures_at(text, start) {
                let whole = captures.get(0).expect("一致の全体は必ずある");
                let alphabet = captures.get(1).expect("アルファベットは必ず捕まえる");
                // textlintはUTF-16で照合するので、後ろの文字がサロゲートペアなら上位だけを使い、
                // 下位のサロゲートが次の一致の前の文字になる。その文字から次の一致を探して合わせる
                let next_is_surrogate_pair = whole
                    .as_str()
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.len_utf16() == 2);
                start = if next_is_surrogate_pair {
                    alphabet.end()
                } else {
                    whole.end()
                };
                if is_allowed(&rule.allow, text, alphabet.start()) {
                    continue;
                }
                reports.push(at(
                    alphabet.start(),
                    alphabet.end(),
                    RuleId::JaUnnaturalAlphabet,
                    format!("不自然なアルファベットがある: {}", alphabet.as_str()),
                ));
            }
        }
    }
    reports
}

/// `allow` のどれかの一致の先頭から直後の位置まで (直後の位置を含む) に、`text` の位置 `i` があればtrue。
/// textlintは直後の1文字まで許すので、指摘の有無を合わせる
fn is_allowed(allow: &[js_regex::Regex], text: &str, i: usize) -> bool {
    allow.iter().any(|regex| {
        regex
            .find_iter(text)
            .any(|m| m.start() <= i && i <= m.end())
    })
}

// textlintの正規表現の範囲をそのまま写す。U+20000からU+2FFFFは、textlintがサロゲートペアで書いている範囲である
static KANJI_RUN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[々〇〻\x{3400}-\x{9FFF}\x{F900}-\x{FAFF}\x{20000}-\x{2FFFF}]+").unwrap()
});

// textlintはUTF-16のコードユニットで照合し、前後の文字に単独のサロゲートも入れている。前の文字は下位のサロゲートで
// U+10000以上のどの文字にも当たり、後ろの文字は上位のサロゲート (U+D840からU+D87F) でU+20000からU+2FFFFに当たる
static UNNATURAL_ALPHABET: LazyLock<Regex> = LazyLock::new(|| {
    let japanese = r"々〇〻\x{3400}-\x{4DBF}\x{4E00}-\x{9FFF}\x{F900}-\x{FAFF}\x{FF00}-\x{FFEF}ぁ-んァ-ヶー。、・−";
    Regex::new(&format!(
        r"[{japanese}\x{{10000}}-\x{{10FFFF}}]([a-zA-Zａ-ｚＡ-Ｚ])[{japanese}\x{{20000}}-\x{{2FFFF}}]"
    ))
    .unwrap()
});

/// `arabic-kanji-numbers` と `2.2.2.算用数字と漢数字の使い分け` の1つのパターン。`arabic-kanji-numbers` は
/// 範囲ごとに最初の一致だけを、`2.2.2` はすべての一致を指摘する
pub struct NumberPattern {
    pub regex: Regex,
    /// 無視パターン。`arabic-kanji-numbers` は範囲のどこかに一致すればこのパターンを指摘せず、`2.2.2` は
    /// 一致の範囲と重なる無視パターンの一致があればその一致だけを指摘しない
    pub ignore: Vec<Regex>,
    /// 一致の直後にこの文字があれば、その一致を使わない。`regex` crateが先読みを持たないので分けて持つ
    pub not_followed_by: Option<char>,
    pub message: &'static str,
}

pub static NUMBER_PATTERNS: LazyLock<Vec<NumberPattern>> = LazyLock::new(|| {
    const K: &str = "[一二三四五六七八九十壱弐参拾百〇]+";
    let pattern = |regex: &str, ignore: &[&str], message| NumberPattern {
        regex: Regex::new(regex).unwrap(),
        ignore: ignore
            .iter()
            .map(|ignore| Regex::new(ignore).unwrap())
            .collect(),
        not_followed_by: None,
        message,
    };
    let to_arabic = |regex: &str, ignore: &[&str]| pattern(regex, ignore, "算用数字で書く");
    let to_kanji = |regex: &str| pattern(regex, &[], "漢数字で書く");
    vec![
        to_arabic(&format!("{K}[兆億万]"), &[&format!("(数|何){K}[兆億万]")]),
        to_arabic(
            &format!("{K}つ"),
            &[
                "[一二三四五六七八九]つ(返事|子|ひとつ|星|編|葉|橋|と[無な]い|に一つ)",
                "(ただ|唯|[女男]手|穴|瓜|馬鹿の)[一二]つ",
            ],
        ),
        to_arabic(&format!("{K}回"), &[]),
        to_arabic(&format!("{K}か月"), &[]),
        to_arabic(&format!("{K}番目"), &[]),
        to_arabic(&format!("{K}進法"), &[]),
        to_arabic(&format!("{K}次元"), &[]),
        to_arabic(&format!("第{K}章"), &[]),
        to_arabic(&format!("第{K}節"), &[]),
        to_kanji("世界1"),
        to_kanji("1時的"),
        to_kanji("1部分"),
        to_kanji("第3者"),
        NumberPattern {
            not_followed_by: Some('類'),
            ..to_kanji("[^0-9]1種")
        },
        to_kanji("1部の"),
        to_kanji("1番に"),
        to_kanji("数10+倍"),
        to_kanji("数10+[兆億万]"),
        to_kanji("数10+年"),
        to_kanji("[0-9]+次関数"),
        to_kanji("5大陸"),
    ]
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prose::parse;

    #[test]
    fn a_surrogate_pair_after_an_alphabet_is_also_the_japanese_character_before_the_next_one() {
        let src = "あg𠮷hい";
        let builtin = Builtin {
            ja_unnatural_alphabet: Some(Default::default()),
            ..Builtin::default()
        };
        let reports = report(src, &parse(src).texts, &builtin);
        let alphabets: Vec<&str> = reports.iter().map(|r| &src[r.range.clone()]).collect();
        assert_eq!(alphabets, vec!["g", "h"]);
    }
}
