//! ひらがな系の5ルール (`ja-hiragana-hojodoushi`・`ja-hiragana-keishikimeishi`・`ja-keishikimeishi`・
//! `ja-hiragana-fukushi`・`ja-hiragana-daimeishi`)。本文の範囲のすべてに、辞書のトークンの並びを当てる。
//! 照合の状態は、本文の範囲をまたいで持ち越さない

use std::collections::HashMap;
use std::ops::Range;

use super::AnalyzedProse;
use super::hiragana_dict::{DAIMEISHI, Entry, FUKUSHI, HOJODOUSHI, KEISHIKIMEISHI};
use super::pattern::{self, Expect, Fields, Match, OwnedFields, StaticFields};
use crate::morph::Token;
use crate::rules::builtin::{Builtin, HiraganaDictionary, HiraganaWord, JaHiragana};
use crate::rules::{Report, RuleId};

/// 有効なひらがな系のルールの辞書 (組み込みの辞書か `rulePath` の辞書) と、その索引。`morph::report` の1回ごとに
/// 1つ作り、本文の範囲ごとに当てる
pub struct Matcher<'a> {
    builtin: SurfaceIndex<'static, StaticFields>,
    /// `builtin` の並びの番号ごとの、ルールと、文面に書く語の種類と、項目
    builtin_entries: Vec<(RuleId, &'static str, &'static Entry)>,
    user: SurfaceIndex<'a, OwnedFields>,
    /// `user` の並びの番号ごとの、ルールと、項目の辞書と、項目
    user_entries: Vec<(RuleId, &'a HiraganaDictionary, &'a HiraganaWord)>,
}

impl<'a> Matcher<'a> {
    /// ひらがな系のルールが1つも有効でなければ、呼ばない
    pub fn new(builtin: &'a Builtin) -> Self {
        let mut builtin_entries = Vec::new();
        let mut user_entries = Vec::new();
        let replaceable: [(RuleId, &Option<JaHiragana>, &'static str, &'static [Entry]); 2] = [
            (
                RuleId::JaHiraganaHojodoushi,
                &builtin.ja_hiragana_hojodoushi,
                "補助動詞",
                &HOJODOUSHI,
            ),
            (
                RuleId::JaHiraganaFukushi,
                &builtin.ja_hiragana_fukushi,
                "副詞",
                &FUKUSHI,
            ),
        ];
        for (rule, options, kind, entries) in replaceable {
            let Some(options) = options else {
                continue;
            };
            match &options.dictionary {
                Some(dictionary) => user_entries.extend(
                    dictionary
                        .entries
                        .iter()
                        .map(|word| (rule, dictionary, word)),
                ),
                None => builtin_entries.extend(entries.iter().map(|entry| (rule, kind, entry))),
            }
        }
        if builtin.ja_hiragana_keishikimeishi {
            builtin_entries.extend(
                KEISHIKIMEISHI
                    .iter()
                    .map(|entry| (RuleId::JaHiraganaKeishikimeishi, "形式名詞", entry)),
            );
        }
        if let Some(rule) = &builtin.ja_keishikimeishi {
            builtin_entries.extend(
                KEISHIKIMEISHI
                    .iter()
                    .filter(|entry| match entry.expected {
                        "ほう" => rule.detection_hou_kata,
                        "うえ" => rule.detection_ue,
                        _ => true,
                    })
                    .map(|entry| (RuleId::JaKeishikimeishi, "形式名詞", entry)),
            );
        }
        if builtin.ja_hiragana_daimeishi {
            // 辞書の副詞の語も、textlintと同じく「代名詞」とする
            builtin_entries.extend(
                DAIMEISHI
                    .iter()
                    .map(|entry| (RuleId::JaHiraganaDaimeishi, "代名詞", entry)),
            );
        }
        Self {
            builtin: SurfaceIndex::new(builtin_entries.iter().map(|(_, _, entry)| entry.pattern)),
            builtin_entries,
            user: SurfaceIndex::new(
                user_entries
                    .iter()
                    .map(|(_, _, word)| word.pattern.as_slice()),
            ),
            user_entries,
        }
    }

    /// `prose` に当たる、有効なひらがな系のルールの指摘を返す。同じルールの、範囲と文面が同じ指摘は1件にする
    pub fn report(&self, prose: &AnalyzedProse) -> Vec<Report> {
        let builtin = self
            .builtin
            .find(prose.tokens)
            .into_iter()
            .filter_map(|(i, found)| {
                let (rule, kind, entry) = self.builtin_entries[i];
                let range = matched_range(prose, &found, entry.offset)?;
                let message = format!(
                    "{kind}「{}」はひらがなの「{}」にする",
                    &prose.text[range.clone()],
                    entry.expected
                );
                Some(prose.report(range, rule, message))
            });
        let user = self
            .user
            .find(prose.tokens)
            .into_iter()
            .filter_map(|(i, found)| {
                let (rule, dictionary, word) = self.user_entries[i];
                let range = matched_range(prose, &found, 0)?;
                let message = format!(
                    "{}: 「{}」を「{}」にする",
                    dictionary.message,
                    &prose.text[range.clone()],
                    word.expected
                );
                Some(prose.report(range, rule, message))
            });
        let mut reports = Vec::new();
        for report in builtin.chain(user) {
            if !reports.contains(&report) {
                reports.push(report);
            }
        }
        reports
    }
}

/// 一致の `from` 番目の期待するトークン以降に一致したトークンの、最初の先頭から最後の末尾まで (`prose.text` の中の
/// バイト範囲)。そこから後がすべて飛ばされたときは無い
pub fn matched_range(prose: &AnalyzedProse, found: &Match, from: usize) -> Option<Range<usize>> {
    let mut matched = found.tokens[from..]
        .iter()
        .flatten()
        .map(|&i| &prose.tokens[i]);
    let first = matched.next()?;
    let end = matched.next_back().unwrap_or(first).range.end;
    Some(first.range.start..end)
}

/// 表層を書いた期待するトークンで、並びの照合の開始位置を引く索引。並びのうち、前に飛ばせる期待するトークンの無い、
/// 表層を書いた最初のトークンの表層で引き、本文のトークンの位置からその並びの中の位置を引いた開始位置で当てる。
/// 表層を書いたトークンを持たない並びは、すべての開始位置で当てる。結果は、並びをすべての開始位置から当てたもの
/// (`pattern::find`) と同じである
pub struct SurfaceIndex<'d, F> {
    patterns: Vec<&'d [Expect<F>]>,
    /// 表層から、(並びの番号, 表層を書いたトークンの並びの中の位置) の一覧
    by_surface: HashMap<&'d str, Vec<(usize, usize)>>,
    /// 表層を書いたトークンを持たない並びの番号
    unindexed: Vec<usize>,
}

impl<'d, F: Fields> SurfaceIndex<'d, F> {
    pub fn new(patterns: impl IntoIterator<Item = &'d [Expect<F>]>) -> Self {
        let patterns: Vec<&'d [Expect<F>]> = patterns.into_iter().collect();
        let mut by_surface: HashMap<&'d str, Vec<(usize, usize)>> = HashMap::new();
        let mut unindexed = Vec::new();
        for (i, &pattern) in patterns.iter().enumerate() {
            let Some((k, surfaces)) = first_surface(pattern) else {
                unindexed.push(i);
                continue;
            };
            for surface in surfaces {
                let starts = by_surface.entry(surface).or_default();
                // 値に同じ表層を2度書いた並びを、同じ開始位置で2度当てない
                if !starts.contains(&(i, k)) {
                    starts.push((i, k));
                }
            }
        }
        Self {
            patterns,
            by_surface,
            unindexed,
        }
    }

    /// `tokens` に当たる一致を、(`new` に渡した並びの番号, 一致) で返す
    pub fn find(&self, tokens: &[Token]) -> Vec<(usize, Match)> {
        let mut starts: Vec<(usize, usize)> = tokens
            .iter()
            .enumerate()
            .flat_map(|(t, token)| {
                self.by_surface
                    .get(token.surface)
                    .into_iter()
                    .flatten()
                    .filter_map(move |&(i, k)| Some((i, t.checked_sub(k)?)))
            })
            .collect();
        starts.extend(
            self.unindexed
                .iter()
                .flat_map(|&i| (0..tokens.len()).map(move |start| (i, start))),
        );
        starts.sort_unstable();
        starts
            .into_iter()
            .filter_map(|(i, start)| Some((i, pattern::match_at(tokens, self.patterns[i], start)?)))
            .collect()
    }
}

/// 並びのうち、前に飛ばせる期待するトークンが無く、それ自身も飛ばせない、表層を書いた最初の期待するトークンの位置と
/// 表層。このトークンは、一致すれば必ず開始位置からこの位置だけ後のトークンに当たる
fn first_surface<F: Fields>(pattern: &[Expect<F>]) -> Option<(usize, Vec<&str>)> {
    for (k, expect) in pattern.iter().enumerate() {
        if expect.skippable {
            return None;
        }
        if let Some(surfaces) = expect.fields.surfaces() {
            return Some((k, surfaces));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::hiragana_dict::SA_PATTERNS;
    use super::super::pattern::Field;
    use super::*;

    /// `fields` の各項目の最初の値を持ち、書いていない項目は `*` のトークン
    fn token_for(fields: StaticFields) -> Token<'static> {
        let value = |field: Field| {
            fields
                .iter()
                .find(|(f, _)| *f == field)
                .map_or("*", |(_, values)| values[0])
        };
        Token {
            range: 0..0,
            surface: value(Field::Surface),
            pos: value(Field::Pos),
            pos_detail_1: value(Field::PosDetail1),
            pos_detail_2: value(Field::PosDetail2),
            pos_detail_3: value(Field::PosDetail3),
            conjugated_type: value(Field::ConjugatedType),
            conjugated_form: value(Field::ConjugatedForm),
            basic_form: value(Field::BasicForm),
            reading: value(Field::Reading),
            pronunciation: value(Field::Pronunciation),
        }
    }

    fn find_from_every_start<F: Fields>(
        tokens: &[Token],
        patterns: &[&[Expect<F>]],
    ) -> Vec<(usize, Match)> {
        patterns
            .iter()
            .enumerate()
            .flat_map(|(i, pattern)| {
                pattern::find(tokens, pattern)
                    .into_iter()
                    .map(move |found| (i, found))
            })
            .collect()
    }

    #[test]
    fn surface_index_finds_the_same_matches_as_every_start_for_the_builtin_dictionaries() {
        let patterns: Vec<&[Expect]> = HOJODOUSHI
            .iter()
            .chain(&FUKUSHI)
            .chain(&DAIMEISHI)
            .chain(&KEISHIKIMEISHI)
            .map(|entry| entry.pattern)
            .chain(SA_PATTERNS.iter().map(|sa| sa.pattern))
            .collect();
        // 各並びに当たるトークンを、順に並べる
        let tokens: Vec<Token> = patterns
            .iter()
            .flat_map(|pattern| pattern.iter().map(|expect| token_for(expect.fields)))
            .collect();
        let expected = find_from_every_start(&tokens, &patterns);
        assert!(expected.len() >= patterns.len(), "{}", expected.len());
        assert_eq!(
            SurfaceIndex::new(patterns.iter().copied()).find(&tokens),
            expected
        );
    }

    #[test]
    fn surface_index_tries_every_start_for_a_pattern_without_a_surface_or_after_a_skippable() {
        let owned = |fields: &[(Field, &str)], skippable: bool| Expect::<OwnedFields> {
            fields: fields
                .iter()
                .map(|(field, value)| (*field, vec![(*value).to_owned()]))
                .collect(),
            skippable,
        };
        let without_surface = vec![owned(&[(Field::Pos, "名詞")], false)];
        let after_skippable = vec![
            owned(&[(Field::Pos, "記号")], true),
            owned(&[(Field::Surface, "x")], false),
        ];
        let surface_later = vec![
            owned(&[(Field::Pos, "名詞")], false),
            owned(&[(Field::Surface, "x")], false),
        ];
        let patterns = [
            without_surface.as_slice(),
            after_skippable.as_slice(),
            surface_later.as_slice(),
        ];
        let tokens = [
            token_for(&[(Field::Surface, &["x"]), (Field::Pos, &["名詞"])]),
            token_for(&[(Field::Pos, &["記号"])]),
            token_for(&[(Field::Surface, &["x"]), (Field::Pos, &["名詞"])]),
            token_for(&[(Field::Surface, &["x"]), (Field::Pos, &["動詞"])]),
        ];
        let expected = find_from_every_start(&tokens, &patterns);
        // 名詞の2つ、飛ばせる記号の後の `x` の4つ (記号を飛ばす開始位置を含む)、名詞の後の `x` の1つ
        assert_eq!(
            expected.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            [0, 0, 1, 1, 1, 1, 2]
        );
        assert_eq!(SurfaceIndex::new(patterns).find(&tokens), expected);
    }
}
