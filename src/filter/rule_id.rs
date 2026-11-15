//! コメントに書いたルールのIDを、jafixのルールに読み替える。textlintの `normalizeTextlintKeyPath` と同じく
//! presetの部分とルールの部分に分け、`textlint-rule-`・`textlint-rule-preset-`・`preset-` の前置きを除いて比べる。
//! presetの部分は、`Preset::ALL` の名前から同じく前置きを除いたもの (`ja-technical-writing`・`@textlint-ja/ai-writing`・
//! `ai-words-ja`・`ja-spacing`・`jtf-style`) と比べる

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::js_regex::{self, Flags, Regex};
use crate::rules::RuleId;
use crate::rules::builtin::Preset;

/// `id` が当たるjafixのルール。presetの部分が無ければ、ルールの部分と名前が等しいルール、あればそのpresetが束ねる
/// ルールのうち、ルールの部分と名前が等しいものである。どれにも当たらなければ空
pub(super) fn rules(id: &str) -> Vec<RuleId> {
    let (preset, rule) = split(id);
    let rule = remove_prefix(&["textlint-rule-"], rule);
    let Some(preset) = preset else {
        return RuleId::ALL
            .into_iter()
            .filter(|id| id.name() == rule)
            .collect();
    };
    let preset = preset_key(preset);
    Preset::ALL
        .into_iter()
        .filter(|candidate| preset_key(candidate.name()) == preset)
        .flat_map(|candidate| candidate.rules())
        .map(|&(id, _)| id)
        .filter(|id| id.name() == rule)
        .collect()
}

// textlintの `splitKeyToPresetSubRule` の正規表現を、同じJavaScriptの方言で順に当てる
static KEY_PATHS: LazyLock<[Regex; 4]> = LazyLock::new(|| {
    [
        "^(@.*?/.*?)/(@.*?/.*?)$",
        "^(@.*?/.*?)/(.*?)$",
        "^(.*?)/(@.*?)$",
        "^([^@].*?)/(.*?)$",
    ]
    .map(|source| js_regex::compile(source, Flags::default()).unwrap())
});

/// `@org/preset/@org/rule`・`@org/preset/rule`・`preset/@org/rule`・`preset/rule` を、presetの部分とルールの部分に
/// 分ける。どれにも当たらなければ、ルールの部分だけである
fn split(id: &str) -> (Option<&str>, &str) {
    KEY_PATHS
        .iter()
        .find_map(|regex| {
            let m = regex.find(id)?;
            Some((&id[m.group(1)?], &id[m.group(2)?]))
        })
        .map_or((None, id), |(preset, rule)| (Some(preset), rule))
}

fn preset_key(name: &str) -> Cow<'_, str> {
    remove_prefix(&["textlint-rule-preset-", "preset-"], name)
}

/// textlintの `removePrefixFromPackageName` と同じく、前置きを除いた名前を `name` の部分の文字列で返す。`@` で
/// 始まる名前は、最初の `/` の後の部分から除き、除いたときは2つ目の `/` から後ろを捨てる
fn remove_prefix<'a>(prefixes: &[&str], name: &'a str) -> Cow<'a, str> {
    let Some(scoped) = name.strip_prefix('@') else {
        return Cow::Borrowed(
            prefixes
                .iter()
                .find_map(|prefix| name.strip_prefix(prefix))
                .unwrap_or(name),
        );
    };
    let mut parts = scoped.split('/');
    let namespace = parts.next().unwrap_or_default();
    let Some(part) = parts.next() else {
        return Cow::Borrowed(name);
    };
    prefixes
        .iter()
        .find_map(|prefix| part.strip_prefix(prefix))
        .map_or(Cow::Borrowed(name), |rest| {
            Cow::Owned(format!("@{namespace}/{rest}"))
        })
}
