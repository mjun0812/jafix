use super::Targets;

/// CodexのPostToolUseの入力JSONの `tool_input.command` にあるpatch本文から、直すMarkdownファイルと対象行を決める。
/// Update Fileの対象行は、編集後のファイルを読んでchunkを位置づけて決める
pub fn targets(input: &str) -> Targets {
    let _ = input;
    todo!("skeleton: not implemented")
}
