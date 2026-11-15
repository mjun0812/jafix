//! 形態素解析を使う20ルール

mod dict;
mod hiragana;
mod hiragana_dict;
mod ja_no_abusage;
mod ja_no_redundant_expression;
mod ja_no_successive_word;
mod ja_no_weak_phrase;
mod max_ten;
mod no_ai_words;
mod no_double_negative_ja;
mod no_doubled_conjunction;
mod no_doubled_conjunctive_particle_ga;
mod no_doubled_joshi;
mod no_dropping_the_ra;
mod no_insert_dropping_sa;
mod no_mix_dearu_desumasu;
mod no_short_topic_comma;
mod no_synonyms;
pub(super) mod pattern;
pub(super) mod projection;

use std::borrow::Cow;
use std::ops::Range;

use crate::morph::{self, Token};
use crate::prose::{Context, Document, Inline, InlineKind};
use crate::sentence::{self, Sentence};

use self::no_mix_dearu_desumasu::{Kind, Styles};
use self::projection::Projection;
use super::builtin::Builtin;
use super::{Report, RuleId};

/// 有効なルールの指摘を返す。順は問わない (`rules::report` が位置の順に並べる)
pub fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    let mut styles = Styles::default();
    // `ja-no-redundant-expression` 以外の文の単位のルールは、引用の中の段落を対象にしない
    let unquoted_rules = builtin.max_ten.is_some()
        || builtin.no_doubled_joshi.is_some()
        || builtin.no_doubled_conjunctive_particle_ga
        || builtin.no_doubled_conjunction
        || builtin.no_mix_dearu_desumasu.is_some();
    for paragraph in &document.paragraphs {
        let redundant = builtin
            .ja_no_redundant_expression
            .as_ref()
            .filter(|rule| ja_no_redundant_expression::targets(rule, paragraph));
        let unquoted = unquoted_rules && !paragraph.blockquote;
        if !unquoted && redundant.is_none() {
            continue;
        }
        let sentences = sentence::split(src, paragraph);
        let projections: Vec<Projection> = sentences
            .iter()
            .map(|sentence| Projection::new(src, sentence))
            .collect();
        let analyzed: Vec<Analyzed> = sentences
            .iter()
            .zip(&projections)
            .map(|(sentence, projection)| Analyzed::new(sentence, projection))
            .collect();
        if let Some(rule) = redundant {
            for sentence in &analyzed {
                reports.extend(ja_no_redundant_expression::report(sentence, rule));
            }
        }
        if !unquoted {
            continue;
        }
        for sentence in &analyzed {
            if let Some(rule) = &builtin.max_ten {
                reports.extend(max_ten::report(sentence, rule));
            }
            if let Some(rule) = &builtin.no_doubled_joshi {
                reports.extend(no_doubled_joshi::report(sentence, rule));
            }
            if builtin.no_doubled_conjunctive_particle_ga {
                reports.extend(no_doubled_conjunctive_particle_ga::report(sentence));
            }
            if let Some(rule) = &builtin.no_mix_dearu_desumasu {
                let kind = if paragraph.list_item_index.is_some() {
                    Kind::List
                } else {
                    Kind::Body
                };
                styles.count(kind, sentence, rule.strict);
            }
        }
        if builtin.no_doubled_conjunction {
            reports.extend(no_doubled_conjunction::report(&analyzed));
        }
    }
    if let Some(rule) = &builtin.no_mix_dearu_desumasu {
        for heading in document
            .headings
            .iter()
            .filter(|heading| !heading.blockquote)
        {
            for sentence in sentence::split(src, heading) {
                let projection = Projection::new(src, &sentence);
                let analyzed = Analyzed::new(&sentence, &projection);
                styles.count(Kind::Header, &analyzed, rule.strict);
            }
        }
        reports.extend(styles.reports(rule));
    }
    let hiragana = (builtin.ja_hiragana_hojodoushi.is_some()
        || builtin.ja_hiragana_keishikimeishi
        || builtin.ja_keishikimeishi.is_some()
        || builtin.ja_hiragana_fukushi.is_some()
        || builtin.ja_hiragana_daimeishi)
        .then(|| hiragana::Matcher::new(builtin));
    // `@textlint-ja/no-synonyms` は、文書の全体の出現を集めてから指摘を決める
    let mut synonyms = builtin
        .no_synonyms
        .as_ref()
        .map(|rule| (rule, no_synonyms::Occurrences::default()));
    let prose_rules = builtin.no_double_negative_ja
        || builtin.no_dropping_the_ra
        || builtin.ja_no_weak_phrase
        || builtin.ja_no_successive_word.is_some()
        || builtin.ja_no_abusage
        || builtin.no_ai_words.is_some()
        || builtin.no_short_topic_comma.is_some()
        || hiragana.is_some()
        || builtin.no_insert_dropping_sa
        || synonyms.is_some();
    if !prose_rules {
        return reports;
    }
    for prose in &document.texts {
        let text = &src[prose.range.clone()];
        let tokens = morph::tokenize(text);
        let analyzed = AnalyzedProse {
            start: prose.range.start,
            context: prose.context,
            text,
            tokens: &tokens,
        };
        if builtin.no_double_negative_ja {
            reports.extend(no_double_negative_ja::report(&analyzed));
        }
        if builtin.no_dropping_the_ra {
            reports.extend(no_dropping_the_ra::report(&analyzed));
        }
        if builtin.ja_no_weak_phrase {
            reports.extend(ja_no_weak_phrase::report(&analyzed));
        }
        if let Some(rule) = &builtin.ja_no_successive_word {
            reports.extend(ja_no_successive_word::report(&analyzed, rule));
        }
        if builtin.ja_no_abusage {
            reports.extend(ja_no_abusage::report(&analyzed));
        }
        if let Some(rule) = &builtin.no_ai_words {
            reports.extend(no_ai_words::report(&analyzed, rule));
        }
        if let Some(rule) = &builtin.no_short_topic_comma {
            reports.extend(no_short_topic_comma::report(&analyzed, rule));
        }
        if let Some(matcher) = &hiragana {
            reports.extend(matcher.report(&analyzed));
        }
        if builtin.no_insert_dropping_sa {
            reports.extend(no_insert_dropping_sa::report(&analyzed));
        }
        if let Some((_, occurrences)) = &mut synonyms {
            occurrences.collect(&analyzed);
        }
    }
    if let Some((rule, occurrences)) = synonyms {
        reports.extend(occurrences.report(rule));
    }
    reports
}

/// トークンに分けた本文の範囲。トークンの位置は `text` の中のバイト位置である
struct AnalyzedProse<'a> {
    /// `text` の、元のソースでの開始位置
    start: usize,
    context: Context,
    text: &'a str,
    tokens: &'a [Token<'a>],
}

impl AnalyzedProse<'_> {
    /// `range` (`text` の中のバイト範囲) への指摘
    fn report(&self, range: Range<usize>, rule: RuleId, message: String) -> Report {
        Report {
            range: self.start + range.start..self.start + range.end,
            rule: rule.name(),
            message: Cow::Owned(message),
        }
    }

    /// textlintの `no-dropping-the-ra` と `ja-no-abusage` の文字列の辞書は、祖先に引用・強調・リンクがある文字列を見ない
    fn in_quote_emphasis_or_link(&self) -> bool {
        self.context.blockquote || self.context.emphasis || self.context.link
    }
}

/// トークンに分けた文
struct Analyzed<'a> {
    sentence: &'a Sentence,
    projection: &'a Projection,
    tokens: Vec<Token<'a>>,
}

impl<'a> Analyzed<'a> {
    fn new(sentence: &'a Sentence, projection: &'a Projection) -> Self {
        Self {
            sentence,
            projection,
            tokens: morph::tokenize(&projection.text),
        }
    }

    /// `token` の元のソースでの範囲
    fn source(&self, token: &Token) -> Range<usize> {
        self.projection.source(token.range.start)..self.projection.source(token.range.end)
    }

    fn report(&self, token: &Token, rule: RuleId, message: String) -> Report {
        Report {
            range: self.source(token),
            rule: rule.name(),
            message: Cow::Owned(message),
        }
    }

    /// 元のソースの位置 `position` が、文の子孫のうち `kind` に当たるインラインの要素の範囲の中にあるか
    fn inside(&self, position: usize, kind: impl Fn(&InlineKind) -> bool + Copy) -> bool {
        inside(&self.sentence.children, position, kind)
    }
}

/// 元のソースの位置 `position` が、`children` とその子孫のうち `kind` に当たるインラインの要素の範囲の中にあるか
pub(super) fn inside(
    children: &[Inline],
    position: usize,
    kind: impl Fn(&InlineKind) -> bool + Copy,
) -> bool {
    children.iter().any(|child| {
        (kind(&child.kind) && child.range.contains(&position))
            || inside(&child.children, position, kind)
    })
}
