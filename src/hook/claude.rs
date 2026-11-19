use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use super::{Target, Targets};
use crate::apply::TargetLines;

#[derive(Deserialize)]
struct Input {
    tool_name: String,
    #[serde(default)]
    tool_input: Value,
    #[serde(default)]
    tool_response: Value,
}

#[derive(Deserialize)]
struct EditInput {
    file_path: PathBuf,
}

/// `structuredPatch` と `originalFile` は文書化されていないので、無ければ対象行を決めずに知らせる
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditResponse {
    structured_patch: Option<Vec<Hunk>>,
    original_file: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Hunk {
    new_start: usize,
    lines: Vec<String>,
}

/// Claude CodeのPostToolUseの入力JSONから、直すMarkdownファイルと対象行を決める。
/// `exclude` に当たるファイルは、対象行を決めずに外す
pub fn targets(input: &str, mut is_excluded: impl FnMut(&Path) -> Result<bool, String>) -> Targets {
    let mut targets = Targets::default();
    let input: Input = match serde_json::from_str(input) {
        Ok(input) => input,
        Err(e) => {
            targets
                .errors
                .push(format!("hookの入力JSONを読めない: {e}"));
            return targets;
        }
    };
    if input.tool_name != "Edit" && input.tool_name != "Write" {
        return targets;
    }
    let path = match EditInput::deserialize(input.tool_input) {
        Ok(edit) => edit.file_path,
        Err(e) => {
            targets
                .errors
                .push(format!("hookの入力の `tool_input` を読めない: {e}"));
            return targets;
        }
    };
    if !targets.includes(&path, &mut is_excluded) {
        return targets;
    }
    match target_lines(input.tool_response) {
        Ok(Some(lines)) => targets.files.push(Target { path, lines }),
        Ok(None) => {}
        Err(reason) => targets.errors.push(format!(
            "{}: 対象行を決められない: {reason}",
            path.display()
        )),
    }
    targets
}

/// 変更の無い上書きでは `None` を返す
fn target_lines(response: Value) -> Result<Option<TargetLines>, String> {
    let response = EditResponse::deserialize(response)
        .map_err(|e| format!("`tool_response` を読めない: {e}"))?;
    let hunks = response
        .structured_patch
        .ok_or("`tool_response.structuredPatch` が無い")?;
    if hunks.is_empty() {
        return Ok(response.original_file.is_none().then_some(TargetLines::All));
    }
    let mut lines = BTreeSet::new();
    for hunk in hunks {
        let mut line = hunk.new_start;
        for patch_line in &hunk.lines {
            match patch_line.chars().next() {
                Some(' ') => line += 1,
                Some('+') => {
                    lines.insert(line);
                    line += 1;
                }
                // `\` は `\ No newline at end of file` で、行として数えない
                Some('-' | '\\') => {}
                _ => return Err(format!("hunkに解釈できない行がある: {patch_line:?}")),
            }
        }
    }
    Ok(Some(TargetLines::Lines(lines)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn targets(input: &str) -> Targets {
        super::targets(input, |_| Ok(false))
    }

    fn edit(response: Value) -> String {
        json!({
            "tool_name": "Edit",
            "tool_input": {"file_path": "/tmp/t.md"},
            "tool_response": response,
        })
        .to_string()
    }

    fn lines(lines: &[usize]) -> TargetLines {
        TargetLines::Lines(lines.iter().copied().collect())
    }

    #[test]
    fn plus_lines_become_target_lines_counted_from_new_start() {
        let targets = targets(&edit(json!({
            "originalFile": "a\nb\nc\nd\n",
            "structuredPatch": [
                {"oldStart": 2, "oldLines": 2, "newStart": 2, "newLines": 3,
                 "lines": [" b", "-c", "+C1", "+C2", " d"]},
            ],
        })));
        assert!(targets.errors.is_empty());
        assert_eq!(
            targets.files,
            [Target {
                path: "/tmp/t.md".into(),
                lines: lines(&[3, 4])
            }]
        );
    }

    #[test]
    fn no_newline_marker_does_not_count_as_a_line() {
        let targets = targets(&edit(json!({
            "originalFile": "a\nb",
            "structuredPatch": [
                {"oldStart": 1, "oldLines": 2, "newStart": 1, "newLines": 3,
                 "lines": [" a", "-b", "\\ No newline at end of file", "+B", "+c"]},
            ],
        })));
        assert_eq!(targets.files[0].lines, lines(&[2, 3]));
    }

    #[test]
    fn each_hunk_starts_at_its_own_new_start() {
        let targets = targets(&edit(json!({
            "originalFile": "",
            "structuredPatch": [
                {"oldStart": 1, "oldLines": 1, "newStart": 1, "newLines": 1, "lines": ["-a", "+A"]},
                {"oldStart": 9, "oldLines": 1, "newStart": 10, "newLines": 2, "lines": [" i", "+J"]},
            ],
        })));
        assert_eq!(targets.files[0].lines, lines(&[1, 11]));
    }

    #[test]
    fn new_file_targets_all_lines() {
        let targets = targets(&edit(json!({"structuredPatch": [], "originalFile": null})));
        assert_eq!(targets.files[0].lines, TargetLines::All);
    }

    #[test]
    fn unchanged_overwrite_has_no_target() {
        let targets = targets(&edit(json!({"structuredPatch": [], "originalFile": "a\n"})));
        assert!(targets.files.is_empty());
        assert!(targets.errors.is_empty());
    }

    #[test]
    fn missing_or_non_array_structured_patch_is_an_error() {
        for response in [
            json!({"originalFile": "a\n"}),
            json!({"structuredPatch": "x"}),
        ] {
            let targets = targets(&edit(response));
            assert!(targets.files.is_empty());
            assert_eq!(targets.errors.len(), 1);
        }
    }

    #[test]
    fn other_tools_and_non_markdown_files_have_no_target() {
        let bash = json!({"tool_name": "Bash", "tool_input": {"command": "ls"}}).to_string();
        let txt = json!({
            "tool_name": "Write",
            "tool_input": {"file_path": "/tmp/t.txt"},
            "tool_response": {"structuredPatch": [], "originalFile": null},
        })
        .to_string();
        for input in [bash, txt] {
            let targets = targets(&input);
            assert!(targets.files.is_empty());
            assert!(targets.errors.is_empty());
        }
    }
}
