//! コメントに書いたルールのIDを、jafixのルールに読み替える。textlintの `normalizeTextlintKeyPath` と同じく
//! presetの部分とルールの部分に分け、`textlint-rule-`・`textlint-rule-preset-`・`preset-` の前置きを除いて比べる。
//! presetの部分は、`Preset::ALL` の名前から同じく前置きを除いたもの (`ja-technical-writing`・`@textlint-ja/ai-writing`・
//! `ai-words-ja`・`ja-spacing`・`jtf-style`) と比べる

use crate::rules::RuleId;

/// `id` が当たるjafixのルール。presetの部分が無ければ、ルールの部分と名前が等しいルール、あればそのpresetが束ねる
/// ルールのうち、ルールの部分と名前が等しいものである。どれにも当たらなければ空
pub(super) fn rules(id: &str) -> Vec<RuleId> {
    let _ = (id, crate::rules::builtin::Preset::rules);
    todo!("skeleton: not implemented")
}
