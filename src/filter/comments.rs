//! `comments`。HTMLのコメントの無効化から有効化までを、書いたルール (書かなければすべてのルール) の外す範囲にする

use toml::Table;

use super::{IgnoredRange, rule_id};
use crate::prose::HtmlComment;
use crate::rules::builtin::{non_empty_string, unknown_option};

/// `comments` のオプション
#[derive(Debug)]
pub struct Comments {
    /// `disablingComment`。コメントの中身がこれを含めば無効化である
    #[allow(dead_code)] // skeleton: not implemented
    pub disabling_comment: String,
    /// `enablingComment`。コメントの中身がこれを含めば有効化で、無効化より先に判定する
    #[allow(dead_code)] // skeleton: not implemented
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
    let _ = (src, comments, filter, rule_id::rules);
    todo!("skeleton: not implemented")
}
