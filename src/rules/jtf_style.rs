//! `preset-jtf-style` (textlint-rule-preset-jtf-style 3.0.3) の37ルール。段落の直下の本文の範囲に当てる
//! 24ルール (`texts`)、文体の3ルールと見出しの `1.1.2` (`style`)、段落ごとに対を数える5ルール (`pairs`)、
//! 辞書の4ルール (`dictionaries`) に分ける。日本語の文字の範囲は、ルールごとのtextlintの範囲である

mod dictionaries;
mod hankaku;
mod joyo;
mod pairs;
mod style;
mod texts;

use std::ops::RangeInclusive;

use crate::prose::{Document, Prose};

use super::builtin::Builtin;
use super::{FixRuleFinding, Report};

/// `preset-jtf-style` の37ルール。宣言の順は `ALL` と同じで、`RuleId` の順を決める
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JtfRule {
    R1_1_1,
    R1_1_2,
    R1_1_3,
    R1_1_5,
    R1_2_1,
    R1_2_2,
    R2_1_2,
    R2_1_5,
    R2_1_6,
    R2_1_8,
    R2_1_9,
    R2_1_10,
    R2_2_1,
    R2_2_2,
    R2_2_3,
    R3_1_1,
    R3_1_2,
    R3_2,
    R3_3,
    R4_1_1,
    R4_1_3,
    R4_2_1,
    R4_2_2,
    R4_2_4,
    R4_2_5,
    R4_2_6,
    R4_2_7,
    R4_2_8,
    R4_2_9,
    R4_3_1,
    R4_3_2,
    R4_3_3,
    R4_3_4,
    R4_3_5,
    R4_3_6,
    R4_3_7,
    R4_3_8,
}

impl JtfRule {
    pub const ALL: [JtfRule; 37] = [
        JtfRule::R1_1_1,
        JtfRule::R1_1_2,
        JtfRule::R1_1_3,
        JtfRule::R1_1_5,
        JtfRule::R1_2_1,
        JtfRule::R1_2_2,
        JtfRule::R2_1_2,
        JtfRule::R2_1_5,
        JtfRule::R2_1_6,
        JtfRule::R2_1_8,
        JtfRule::R2_1_9,
        JtfRule::R2_1_10,
        JtfRule::R2_2_1,
        JtfRule::R2_2_2,
        JtfRule::R2_2_3,
        JtfRule::R3_1_1,
        JtfRule::R3_1_2,
        JtfRule::R3_2,
        JtfRule::R3_3,
        JtfRule::R4_1_1,
        JtfRule::R4_1_3,
        JtfRule::R4_2_1,
        JtfRule::R4_2_2,
        JtfRule::R4_2_4,
        JtfRule::R4_2_5,
        JtfRule::R4_2_6,
        JtfRule::R4_2_7,
        JtfRule::R4_2_8,
        JtfRule::R4_2_9,
        JtfRule::R4_3_1,
        JtfRule::R4_3_2,
        JtfRule::R4_3_3,
        JtfRule::R4_3_4,
        JtfRule::R4_3_5,
        JtfRule::R4_3_6,
        JtfRule::R4_3_7,
        JtfRule::R4_3_8,
    ];

    /// ルールのID。textlintのpresetの `rules` のキーと同じ文字列である
    pub fn name(self) -> &'static str {
        match self {
            JtfRule::R1_1_1 => "1.1.1.本文",
            JtfRule::R1_1_2 => "1.1.2.見出し",
            JtfRule::R1_1_3 => "1.1.3.箇条書き",
            JtfRule::R1_1_5 => "1.1.5.図表のキャプション",
            JtfRule::R1_2_1 => "1.2.1.句点(。)と読点(、)",
            JtfRule::R1_2_2 => "1.2.2.ピリオド(.)とカンマ(,)",
            JtfRule::R2_1_2 => "2.1.2.漢字",
            JtfRule::R2_1_5 => "2.1.5.カタカナ",
            JtfRule::R2_1_6 => "2.1.6.カタカナの長音",
            JtfRule::R2_1_8 => "2.1.8.算用数字",
            JtfRule::R2_1_9 => "2.1.9.アルファベット",
            JtfRule::R2_1_10 => "2.1.10.算用数字の位取りの表記",
            JtfRule::R2_2_1 => "2.2.1.ひらがなと漢字の使い分け",
            JtfRule::R2_2_2 => "2.2.2.算用数字と漢数字の使い分け",
            JtfRule::R2_2_3 => "2.2.3.一部の助数詞の表記",
            JtfRule::R3_1_1 => "3.1.1.全角文字と半角文字の間",
            JtfRule::R3_1_2 => "3.1.2.全角文字どうし",
            JtfRule::R3_2 => "3.2.カタカナ語間のスペースの有無",
            JtfRule::R3_3 => "3.3.かっこ類と隣接する文字の間のスペースの有無",
            JtfRule::R4_1_1 => "4.1.1.句点(。)",
            JtfRule::R4_1_3 => "4.1.3.ピリオド(.)、カンマ(,)",
            JtfRule::R4_2_1 => "4.2.1.感嘆符(！)",
            JtfRule::R4_2_2 => "4.2.2.疑問符(？)",
            JtfRule::R4_2_4 => "4.2.4.中黒(・)",
            JtfRule::R4_2_5 => "4.2.5.波線(〜)",
            JtfRule::R4_2_6 => "4.2.6.ハイフン(-)",
            JtfRule::R4_2_7 => "4.2.7.コロン(：)",
            JtfRule::R4_2_8 => "4.2.8.セミコロン(;)",
            JtfRule::R4_2_9 => "4.2.9.ダッシュ(-)",
            JtfRule::R4_3_1 => "4.3.1.丸かっこ（）",
            JtfRule::R4_3_2 => "4.3.2.大かっこ［］",
            JtfRule::R4_3_3 => "4.3.3.かぎかっこ「」",
            JtfRule::R4_3_4 => "4.3.4.二重かぎかっこ『』",
            JtfRule::R4_3_5 => "4.3.5.二重引用符",
            JtfRule::R4_3_6 => "4.3.6.中かっこ{ }",
            JtfRule::R4_3_7 => "4.3.7.山かっこ<>",
            JtfRule::R4_3_8 => "4.3.8.一重引用符",
        }
    }
}

/// 37ルールのうち修正ルールになる7ルール (`2.1.5` の半角カナの部分、`2.1.8`、`2.1.9`、`3.1.1`、`3.1.2`、
/// `3.3` の空白を消す側、`4.2.4`) の箇所を返す。どれも `Context::plain` の本文の範囲だけに当てるので、
/// 見出しの中の箇所は無い。重なりは解かない
#[allow(dead_code)] // skeleton: not implemented
pub fn find(src: &str, texts: &[Prose], builtin: &Builtin) -> Vec<FixRuleFinding> {
    let _ = (src, texts, builtin, texts::find);
    todo!("skeleton: not implemented")
}

/// 37ルールのうち指摘ルールになる側の指摘を返す。辞書の4ルールは、`target_texts` (対象行と重なる本文の範囲) だけに
/// 当てる。ほかのルールは文書の全体で数え、対象行で絞るのは呼び出し側である。同じルールの、位置と文面が同じ指摘は1件にする
#[allow(dead_code)] // skeleton: not implemented
pub fn report(
    src: &str,
    document: &Document,
    builtin: &Builtin,
    target_texts: &[&Prose],
) -> Vec<Report> {
    let _ = (
        src,
        document,
        builtin,
        target_texts,
        texts::report,
        style::report,
        pairs::report,
        dictionaries::report,
    );
    todo!("skeleton: not implemented")
}

/// textlintの `japaneseRegExp`。`1.2.1`・`3.1.2`・`3.3`・`4.2.2`・`4.2.4`・`4.2.6`〜`4.2.9`・`4.3.1`・`4.3.2` が使う。
/// `[\uD840-\uD87F][\uDC00-\uDFFF]` はU+20000からU+2FFFFの文字である。`ー` を含まない
#[allow(dead_code)] // skeleton: not implemented
const JAPANESE: &[RangeInclusive<char>] = &[
    '々'..='々',
    '〇'..='〇',
    '〻'..='〻',
    '\u{3400}'..='\u{4DBF}',
    '\u{4E00}'..='\u{9FFF}',
    '\u{F900}'..='\u{FAFF}',
    '\u{20000}'..='\u{2FFFF}',
    'ぁ'..='ん',
    'ァ'..='ヶ',
];

/// `JAPANESE` から `々` `〇` `〻` を除いたもの。`3.1.1`・`4.1.3`・`4.2.1` のルールのファイルに直接書かれた範囲である
#[allow(dead_code)] // skeleton: not implemented
const JAPANESE_WITHOUT_MARKS: &[RangeInclusive<char>] = &[
    '\u{3400}'..='\u{4DBF}',
    '\u{4E00}'..='\u{9FFF}',
    '\u{F900}'..='\u{FAFF}',
    '\u{20000}'..='\u{2FFFF}',
    'ぁ'..='ん',
    'ァ'..='ヶ',
];

/// textlintの `kanjiRegExp`。`2.1.2` が使う。`々` を含まない
#[allow(dead_code)] // skeleton: not implemented
const KANJI: &[RangeInclusive<char>] = &[
    '〇'..='〇',
    '〻'..='〻',
    '\u{3400}'..='\u{9FFF}',
    '\u{F900}'..='\u{FAFF}',
    '\u{20000}'..='\u{2FFFF}',
];

/// `3.2` のカタカナ語の文字
#[allow(dead_code)] // skeleton: not implemented
const KATAKANA_WORD: &[RangeInclusive<char>] = &['ァ'..='ヶ', 'ー'..='ー'];

/// `2.1.5` の半角カナ
#[allow(dead_code)] // skeleton: not implemented
const HANKAKU_KANA: RangeInclusive<char> = '\u{FF65}'..='\u{FF9F}';

/// `c` が `ranges` のどれかに入るか
#[allow(dead_code)] // skeleton: not implemented
fn contains(ranges: &[RangeInclusive<char>], c: char) -> bool {
    let _ = (ranges, c);
    todo!("skeleton: not implemented")
}
