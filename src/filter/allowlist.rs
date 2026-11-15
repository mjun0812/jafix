//! `allowlist`。許可したパターンに一致した文書の範囲を、すべてのルールの外す範囲にする

use std::ops::Range;
use std::path::Path;

use regress::Regex;
use toml::Table;

use crate::js_regex;
use crate::rules::builtin::{non_empty_strings, resolve_path, strings, unknown_option};
use crate::rules::parse_yaml;

/// `allowlist` のオプション
#[derive(Debug, Default)]
pub struct Allowlist {
    /// `allow` の要素と、`allowlistConfigPaths` のファイルの要素を、この順に連結したパターン
    #[allow(dead_code)] // skeleton: not implemented
    pub patterns: Vec<Regex>,
}

impl Allowlist {
    /// `config_dir` は、`allowlistConfigPaths` の相対パスを解決するディレクトリである
    pub(super) fn from_options(options: &Table, config_dir: &Path) -> Result<Self, String> {
        let mut allow = Vec::new();
        let mut config_paths = Vec::new();
        for (key, value) in options {
            match key.as_str() {
                "allow" => {
                    allow = strings(key, value)?
                        .into_iter()
                        .map(|value| pattern(value).map_err(|reason| format!("`{key}`: {reason}")))
                        .collect::<Result<_, _>>()?;
                }
                "allowlistConfigPaths" => config_paths = non_empty_strings(key, value)?,
                _ => return Err(unknown_option(key)),
            }
        }
        let mut patterns = allow;
        for path in &config_paths {
            let path = resolve_path("allowlistConfigPaths", path, config_dir)?;
            let values = read_strings(&path).map_err(|reason| {
                format!("`allowlistConfigPaths` の {}: {reason}", path.display())
            })?;
            for value in &values {
                patterns.push(pattern(value).map_err(|reason| {
                    format!("`allowlistConfigPaths` の {}: {reason}", path.display())
                })?);
            }
        }
        Ok(Self { patterns })
    }
}

/// 許可するパターンを、`@textlint-rule/pattern` と同じく読む。空の文字列は、どこにでも当たるので不正にする
fn pattern(value: &str) -> Result<Regex, String> {
    if value.is_empty() {
        return Err("空の文字列はパターンにできない".to_owned());
    }
    js_regex::pattern(value)
}

/// `path` の文字列の配列を読む。拡張子が `.json` ならJSON、`.yml` か `.yaml` ならYAMLとして読む
fn read_strings(path: &Path) -> Result<Vec<String>, String> {
    let yaml = match path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => false,
        Some("yml" | "yaml") => true,
        _ => return Err("拡張子は `.json`・`.yml`・`.yaml` のどれかにする".to_owned()),
    };
    let text = std::fs::read_to_string(path).map_err(|e| format!("読めない: {e}"))?;
    let value: serde_json::Value = if yaml {
        parse_yaml(&text)?
    } else {
        serde_json::from_str(&text).map_err(|e| format!("JSONとして読めない: {e}"))?
    };
    serde_json::from_value(value).map_err(|e| format!("文字列の配列でない: {e}"))
}

/// `src` の全体に `allowlist` の各パターンを当て、長さ0を含むすべての一致の範囲を返す
pub(super) fn ranges(src: &str, allowlist: &Allowlist) -> Vec<Range<usize>> {
    let _ = (src, allowlist);
    todo!("skeleton: not implemented")
}
