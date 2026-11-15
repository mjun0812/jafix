//! `comments`。HTMLのコメントの無効化から有効化までを、書いたルール (書かなければすべてのルール) の外す範囲にする

use toml::Table;

use super::{IgnoredRange, IgnoredRules, rule_id};
use crate::prose::HtmlComment;
use crate::rules::builtin::{non_empty_string, unknown_option};

/// `comments` のオプション
#[derive(Debug)]
pub struct Comments {
    /// `disablingComment`。コメントの中身がこれを含めば無効化である
    pub disabling_comment: String,
    /// `enablingComment`。コメントの中身がこれを含めば有効化で、無効化より先に判定する
    pub enabling_comment: String,
}

impl Default for Comments {
    fn default() -> Self {
        Self {
            disabling_comment: "textlint-disable".to_owned(),
            enabling_comment: "textlint-enable".to_owned(),
        }
    }
}

impl Comments {
    pub(super) fn from_options(options: &Table) -> Result<Self, String> {
        let mut filter = Self::default();
        for (key, value) in options {
            match key.as_str() {
                "disablingComment" => {
                    filter.disabling_comment = non_empty_string(key, value)?.to_owned();
                }
                "enablingComment" => {
                    filter.enabling_comment = non_empty_string(key, value)?.to_owned();
                }
                _ => return Err(unknown_option(key)),
            }
        }
        Ok(filter)
    }
}

/// `comments` (`src` のHTMLのコメントの出現順の一覧) を順に読み、外す範囲を返す。文書の末尾で開いたままの無効化は、
/// 文書の末尾 (`src.len()`) で閉じる。どのルールにも当たらないIDの無効化は、外す範囲にしない
pub(super) fn ranges(src: &str, comments: &[HtmlComment], filter: &Comments) -> Vec<IgnoredRange> {
    // 開いている無効化の始まりと、書いたID (`None` はIDを書かない、すべてのルールの無効化)
    let mut open: Vec<(usize, Option<String>)> = Vec::new();
    let mut closed = Vec::new();
    for comment in comments {
        let content = &src[comment.content.clone()];
        if content.contains(filter.enabling_comment.as_str()) {
            let ids = rule_ids(content, &filter.enabling_comment);
            if ids.is_empty() {
                closed.extend(open.drain(..).map(|(start, id)| (start..comment.start, id)));
                continue;
            }
            for id in ids {
                if let Some(i) = open
                    .iter()
                    .rposition(|(_, open_id)| open_id.as_ref() == Some(&id))
                {
                    let (start, id) = open.remove(i);
                    closed.push((start..comment.start, id));
                }
            }
        } else if content.contains(filter.disabling_comment.as_str()) {
            let ids = rule_ids(content, &filter.disabling_comment);
            if ids.is_empty() {
                open.push((comment.start, None));
            }
            open.extend(ids.into_iter().map(|id| (comment.start, Some(id))));
        }
    }
    closed.extend(open.into_iter().map(|(start, id)| (start..src.len(), id)));
    closed
        .into_iter()
        .filter_map(|(range, id)| {
            let rules = match id {
                None => IgnoredRules::All,
                Some(id) => {
                    let rules = rule_id::rules(&id);
                    if rules.is_empty() {
                        return None;
                    }
                    IgnoredRules::Only(rules)
                }
            };
            Some(IgnoredRange { range, rules })
        })
        .collect()
}

/// コメントの中身から最初の指示の文字列を除き、`--` から後ろを捨て、カンマで区切った空でないID
/// (textlintの `parse-comment.js` と同じ)
fn rule_ids(content: &str, directive: &str) -> Vec<String> {
    let rest = content.replacen(directive, "", 1);
    let ids = rest
        .split_once("--")
        .map_or(rest.as_str(), |(head, _)| head);
    ids.split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .collect()
}
