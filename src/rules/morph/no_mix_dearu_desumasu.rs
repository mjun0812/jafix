use std::borrow::Cow;
use std::ops::Range;

use crate::morph::Token;
use crate::prose::InlineKind;
use crate::rules::builtin::{NoMixDearuDesumasu, Style};
use crate::rules::{Report, RuleId};

use super::Analyzed;

/// 文体を別々に数える種類
#[derive(Clone, Copy)]
pub(super) enum Kind {
    Header,
    Body,
    List,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Header => "見出し",
            Kind::Body => "本文",
            Kind::List => "箇条書き",
        }
    }
}

/// 文書の全体で数えた「である」と「ですます」の箇所。種類ごとに持つ
#[derive(Default)]
pub(super) struct Styles {
    header: Counts,
    body: Counts,
    list: Counts,
}

/// 文体ごとの箇所の元のソースでの範囲。`None` は、数えるが指摘しない箇所 (リンク・画像・強調の中) である
#[derive(Default)]
struct Counts {
    dearu: Vec<Option<Range<usize>>>,
    desumasu: Vec<Option<Range<usize>>>,
}

impl Styles {
    pub(super) fn count(&mut self, kind: Kind, sentence: &Analyzed, strict: bool) {
        let counts = match kind {
            Kind::Header => &mut self.header,
            Kind::Body => &mut self.body,
            Kind::List => &mut self.list,
        };
        let tokens = &sentence.tokens;
        for (i, token) in tokens.iter().enumerate() {
            let Some(style) = style(token, tokens.get(i + 1)) else {
                continue;
            };
            if !strict && !at_end(&tokens[i + 1..]) {
                continue;
            }
            let range = sentence.source(token);
            let reportable = !sentence.inside(range.start, |kind| {
                matches!(
                    kind,
                    InlineKind::Link { .. } | InlineKind::Image | InlineKind::Emphasis
                )
            });
            let found = reportable.then_some(range);
            match style {
                Style::Dearu => counts.dearu.push(found),
                Style::Desumasu => counts.desumasu.push(found),
            }
        }
    }

    /// 種類ごとに、優先する文体と違う側の箇所に指摘を出す
    pub(super) fn reports(&self, rule: &NoMixDearuDesumasu) -> Vec<Report> {
        let mut reports = Vec::new();
        for (kind, counts, prefer) in [
            (Kind::Header, &self.header, rule.prefer_in_header),
            (Kind::Body, &self.body, rule.prefer_in_body),
            (Kind::List, &self.list, rule.prefer_in_list),
        ] {
            let (style, found) = match counts.mixed(prefer) {
                Some(Style::Dearu) => ("である", &counts.dearu),
                Some(Style::Desumasu) => ("ですます", &counts.desumasu),
                None => continue,
            };
            reports.extend(found.iter().flatten().map(|range| Report {
                range: range.clone(),
                rule: RuleId::NoMixDearuDesumasu.name(),
                message: Cow::Owned(format!("{}に「{style}」調が混ざっている", kind.name())),
            }));
        }
        reports
    }
}

impl Counts {
    /// 指摘する側の文体。`prefer` が無ければ、両方があるときに少ない側 (同数なら「である」) を指摘する
    fn mixed(&self, prefer: Option<Style>) -> Option<Style> {
        match prefer {
            Some(Style::Desumasu) => Some(Style::Dearu),
            Some(Style::Dearu) => Some(Style::Desumasu),
            None if self.dearu.is_empty() || self.desumasu.is_empty() => None,
            None if self.desumasu.len() < self.dearu.len() => Some(Style::Desumasu),
            None => Some(Style::Dearu),
        }
    }
}

/// `token` が「である」の「で」か、「です」「ます」なら、その文体
fn style(token: &Token, next: Option<&Token>) -> Option<Style> {
    if token.conjugated_type == "特殊・ダ"
        && token.pos == "助動詞"
        && token.conjugated_form == "連用形"
        && next.is_some_and(|next| next.conjugated_type == "五段・ラ行アル")
    {
        return Some(Style::Dearu);
    }
    let desumasu = ["特殊・デス", "特殊・マス"].contains(&token.conjugated_type)
        && token.conjugated_form == "基本形"
        && token.basic_form != "やす";
    desumasu.then_some(Style::Desumasu)
}

/// 候補の後ろの `rest` から見て、候補が文末にあるか。後ろで最初の切れ目が無いか、文の終わりの記号なら文末である
fn at_end(rest: &[Token]) -> bool {
    let boundary = rest.iter().find(|token| {
        token.surface.contains(['、', '。'])
            || token.conjugated_type.contains("特殊")
            || token.pos == "名詞"
    });
    boundary.is_none_or(|token| token.surface.contains(['!', '?', '！', '？', '。']))
}
