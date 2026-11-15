//! 辞書の4ルール (`2.1.5`・`2.1.6`・`2.2.1`・`2.2.3`)。textlint-rule-preset-jtf-style 3.0.3 の `dict/*.yml` を元の
//! ファイルのまま埋め込み、`prh` の辞書の読み込みと照合で当てる。箇所はすべて指摘にする

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::prose::Prose;

use super::super::builtin::JtfStyle;
use super::super::prh::Checks;
use super::super::prh::dictionary::{Dictionary, Replacement, Source};
use super::super::{Report, one_line};
use super::JtfRule;

/// 辞書を持つルールと、その辞書のYAML
const DICTIONARIES: [(JtfRule, &str); 4] = [
    (JtfRule::R2_1_5, include_str!("dict/2.1.5.yml")),
    (JtfRule::R2_1_6, include_str!("dict/2.1.6.yml")),
    (JtfRule::R2_2_1, include_str!("dict/2.2.1.yml")),
    (JtfRule::R2_2_3, include_str!("dict/2.2.3.yml")),
];

/// 辞書の4ルールのどれかが有効か
pub fn any_enabled(rules: &JtfStyle) -> bool {
    DICTIONARIES.iter().any(|&(rule, _)| rules.enabled(rule))
}

/// 有効な辞書のルールを、`texts` のうち `Checks::default().covers` が真の本文の範囲に当てた指摘。
/// 文面は `prh` と同じ `「<一致>」を「<置き換え後>」にする` である。見出しの中の箇所も、文面を変えずに指摘にする
pub fn report(src: &str, texts: &[&Prose], rules: &JtfStyle) -> Vec<Report> {
    let checks = Checks::default();
    let mut reports = Vec::new();
    for &(rule, _) in DICTIONARIES
        .iter()
        .filter(|&&(rule, _)| rules.enabled(rule))
    {
        let dictionary = dictionary(rule);
        for prose in texts.iter().filter(|prose| checks.covers(&prose.context)) {
            let text = &src[prose.range.clone()];
            for Replacement {
                range,
                text: replaced,
                ..
            } in dictionary.replacements(text)
            {
                reports.push(Report {
                    range: prose.range.start + range.start..prose.range.start + range.end,
                    rule: rule.name(),
                    message: Cow::Owned(format!(
                        "「{}」を「{}」にする",
                        one_line(&text[range]),
                        one_line(&replaced)
                    )),
                });
            }
        }
    }
    reports
}

/// `rule` の辞書。最初に呼ばれたときに1回だけ読む。埋め込んだ辞書は読めることがテストで決まっているので、
/// 読めなければ不具合として止める
fn dictionary(rule: JtfRule) -> &'static Dictionary {
    static LOADED: [LazyLock<Dictionary>; 4] = [
        LazyLock::new(|| load(0)),
        LazyLock::new(|| load(1)),
        LazyLock::new(|| load(2)),
        LazyLock::new(|| load(3)),
    ];
    let index = DICTIONARIES
        .iter()
        .position(|&(dictionary_rule, _)| dictionary_rule == rule)
        .expect("辞書を持つルールだけを渡す");
    &LOADED[index]
}

/// `DICTIONARIES` の `index` 番目の辞書を読む。辞書は `imports` を持たないので、相対パスを解決するディレクトリは使われない
fn load(index: usize) -> Dictionary {
    let (rule, yaml) = DICTIONARIES[index];
    Dictionary::load(&[Source::Contents {
        yaml: yaml.to_owned(),
        base: PathBuf::new(),
    }])
    .unwrap_or_else(|error| panic!("`{}` の辞書を読めない: {}", rule.name(), error.reason))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use toml::{Table, Value};

    use super::super::super::RuleId;
    use super::super::super::builtin::Builtin;
    use super::*;

    #[test]
    fn embedded_dictionaries_load_and_pass_their_specs() {
        for (rule, _) in DICTIONARIES {
            assert!(!dictionary(rule).rules().is_empty(), "{}", rule.name());
        }
    }

    /// 埋め込んだ辞書の置き換え後の文字列を、組み込みの修正ルールが戻さない。戻すなら、辞書の指摘に従って直した
    /// 文書をhookが元に戻し、指摘と修正が往復する。`prh` の逆の置き換えの判定に、辞書をそのまま渡して確かめる
    #[test]
    fn no_fix_rule_reverts_an_embedded_dictionary() {
        for (rule, yaml) in DICTIONARIES {
            let mut table = Table::new();
            for id in RuleId::ALL {
                // `4.3.1` は既定で有効な `fullwidth-paren` と組になって設定の不正になる。指摘ルールなので、
                // 外しても判定に入る修正ルールは変わらない
                if !matches!(
                    id,
                    RuleId::Prh | RuleId::TextlintRulePattern | RuleId::JtfStyle(JtfRule::R4_3_1)
                ) {
                    table.insert(id.name().to_owned(), Value::Boolean(true));
                }
            }
            let mut prh = Table::new();
            prh.insert(
                "ruleContents".to_owned(),
                Value::Array(vec![Value::String(yaml.to_owned())]),
            );
            table.insert(RuleId::Prh.name().to_owned(), Value::Table(prh));
            if let Err(reason) = Builtin::from_table(&table, Path::new(".")) {
                panic!("{}: {reason}", rule.name());
            }
        }
    }
}
