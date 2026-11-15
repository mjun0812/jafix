//! 指摘を外す2つのfilter (`comments`・`allowlist`)。filterは指摘を出さず、文書から外す範囲を求める。外す範囲に
//! 完全に含まれる指摘ルールの指摘と、組み込みの修正ルールと利用者の置き換えの箇所を、`rules` が外す

mod allowlist;
mod comments;
mod rule_id;

use std::ops::Range;
use std::path::Path;

use toml::{Table, Value};

use crate::prose::Document;
use crate::rules::RuleId;

pub use self::allowlist::Allowlist;
pub use self::comments::Comments;

/// `[filters]` で決めた、2つのfilterの有効・無効とオプション。既定ではどちらも無効である
#[derive(Debug, Default)]
pub struct Filters {
    pub comments: Option<Comments>,
    pub allowlist: Option<Allowlist>,
}

impl Filters {
    /// 設定の `[filters]` の表から作る。キーはfilterのIDで、値は `false` (無効)・`true` (既定のオプションで有効)・
    /// オプションの表 (有効) である。`config_dir` は設定ファイルのあるディレクトリで、`allowlistConfigPaths` の
    /// 相対パスをここから解決する。知らないキー、値の型の誤り、知らないオプション、値の誤りは、キーとオプションの
    /// 名前を含む理由を返す
    pub fn from_table(table: &Table, config_dir: &Path) -> Result<Self, String> {
        let mut filters = Filters::default();
        let defaults = Table::new();
        for (key, value) in table {
            if !matches!(key.as_str(), "comments" | "allowlist") {
                return Err(format!(
                    "`{key}` はfilterのIDでない (`comments` か `allowlist` を書く)"
                ));
            }
            let options = match value {
                Value::Boolean(false) => None,
                Value::Boolean(true) => Some(&defaults),
                Value::Table(options) => Some(options),
                _ => {
                    return Err(format!(
                        "`{key}`: 値は `true`・`false`・オプションの表のどれかで書く"
                    ));
                }
            };
            let in_key = |reason: String| format!("`{key}`: {reason}");
            if key == "comments" {
                filters.comments = options
                    .map(Comments::from_options)
                    .transpose()
                    .map_err(in_key)?;
            } else {
                filters.allowlist = options
                    .map(|options| Allowlist::from_options(options, config_dir))
                    .transpose()
                    .map_err(in_key)?;
            }
        }
        Ok(filters)
    }
}

/// 文書から外す範囲の一覧
#[derive(Debug, Default)]
pub struct Ignored {
    ranges: Vec<IgnoredRange>,
}

/// 外す範囲の1つ
#[derive(Debug)]
struct IgnoredRange {
    /// 元のソース上のバイト範囲
    range: Range<usize>,
    rules: IgnoredRules,
}

/// 外す範囲の対象のルール
#[derive(Debug)]
enum IgnoredRules {
    All,
    Only(Vec<RuleId>),
}

impl Ignored {
    /// `src` (`document` はその解析の結果) から、有効なfilterの外す範囲を求める。どちらのfilterも無効なら空である
    pub fn new(src: &str, document: &Document, filters: &Filters) -> Self {
        let mut ranges = Vec::new();
        if let Some(filter) = &filters.comments {
            ranges.extend(comments::ranges(src, &document.comments, filter));
        }
        if let Some(filter) = &filters.allowlist {
            ranges.extend(
                allowlist::ranges(src, filter)
                    .into_iter()
                    .map(|range| IgnoredRange {
                        range,
                        rules: IgnoredRules::All,
                    }),
            );
        }
        Self { ranges }
    }

    /// `range` が外す範囲のどれかに完全に含まれ (始まりと終わりがどちらも外す範囲の中か端にあり)、名前が `rule` の
    /// ルールがその範囲の対象なら真
    pub fn covers(&self, range: &Range<usize>, rule: &str) -> bool {
        self.ranges.iter().any(|ignored| {
            ignored.range.start <= range.start
                && range.end <= ignored.range.end
                && match &ignored.rules {
                    IgnoredRules::All => true,
                    IgnoredRules::Only(rules) => rules.iter().any(|id| id.name() == rule),
                }
        })
    }
}
