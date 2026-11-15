//! ルールごとの日本語の文字の範囲。textlintの正規表現の範囲を写す。`[\uD840-\uD87F][\uDC00-\uDFFF]` は
//! U+20000からU+2FFFFの文字である

use std::ops::RangeInclusive;

/// `ja-space-between-half-and-full-width` の全角文字。`ー`・`々`・`・` を含まない
pub const HALF_AND_FULL_WIDTH: &[RangeInclusive<char>] = &[
    '、'..='、',
    '。'..='。',
    '\u{3400}'..='\u{4DBF}',
    '\u{4E00}'..='\u{9FFF}',
    '\u{F900}'..='\u{FAFF}',
    '\u{20000}'..='\u{2FFFF}',
    'ぁ'..='ん',
    'ァ'..='ヶ',
];

/// `ja-no-space-between-full-width` の全角文字
pub const BETWEEN_FULL_WIDTH: &[RangeInclusive<char>] = &[
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

/// 4つの `ja-space-around-*` が、nodeの隣で日本語とみなす文字
pub const AROUND_NODE: &[RangeInclusive<char>] = &[
    '\u{3400}'..='\u{4DBF}',
    '\u{4E00}'..='\u{9FFF}',
    '\u{F900}'..='\u{FAFF}',
    '\u{20000}'..='\u{2FFFF}',
    'ぁ'..='ん',
    'ァ'..='ヶ',
];

/// `ja-nakaguro-or-halfwidth-space-between-katakana` のカタカナ
pub const KATAKANA_WORD: &[RangeInclusive<char>] = &['ァ'..='ヶ', 'ー'..='ー'];

/// `ja-no-space-between-full-width` が、前後がどちらも当たれば外すカタカナ
pub const KATAKANA: RangeInclusive<char> = 'ァ'..='ヶ';

/// `c` が `ranges` のどれかに入るか
pub fn contains(ranges: &[RangeInclusive<char>], c: char) -> bool {
    let _ = (ranges, c);
    todo!("skeleton: not implemented")
}
