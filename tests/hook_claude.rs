mod common;

use std::path::Path;

use common::{AI_RULES_CONFIG, AI_RULES_INPUT, Sandbox, read, stdout};
use serde_json::{Value, json};

/// Claude Code (v2.1.319) が実際に渡すEditの入力の形
fn edit_input(path: &Path, tool_response: Value) -> String {
    json!({
        "session_id": "s",
        "transcript_path": "/tmp/transcript.jsonl",
        "cwd": "/tmp",
        "permission_mode": "default",
        "hook_event_name": "PostToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": path,
            "old_string": "x",
            "new_string": "y",
            "replace_all": false,
        },
        "tool_response": tool_response,
        "tool_use_id": "toolu_1",
        "duration_ms": 4,
    })
    .to_string()
}

#[test]
fn edit_fixes_only_the_plus_lines_of_the_structured_patch() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "前 A\n語 B\n後 C\n");
    let input = edit_input(
        &path,
        json!({
            "filePath": path,
            "originalFile": "前 A\n語 x\n後 C\n",
            "structuredPatch": [
                {"oldStart": 1, "oldLines": 3, "newStart": 1, "newLines": 3,
                 "lines": [" 前 A", "-語 x", "+語 B", " 後 C"]},
            ],
            "userModified": false,
            "replaceAll": false,
        }),
    );
    let output = sandbox.hook("claude", &input);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(&path), "前 A\n語B\n後 C\n");
}

#[test]
fn write_of_a_new_file_fixes_every_line() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n（補足） です。\n");
    let input = json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": path, "content": "..."},
        "tool_response": {
            "type": "create",
            "filePath": path,
            "content": "...",
            "structuredPatch": [],
            "originalFile": null,
            "userModified": false,
        },
    })
    .to_string();
    let output = sandbox.hook("claude", &input);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&path), "日本語Rust\n(補足)です。\n");
}

#[test]
fn edit_without_structured_patch_leaves_the_file_and_exits_1() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    let input = edit_input(&path, json!({"filePath": path, "originalFile": ""}));
    let output = sandbox.hook("claude", &input);
    assert_eq!(output.status.code(), Some(1));
    assert!(!output.stderr.is_empty());
    assert_eq!(read(&path), "日本語 Rust\n");
}

#[test]
fn no_newline_marker_after_a_minus_line_is_not_counted() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "前 A\n語 B\n後 C\n");
    let input = edit_input(
        &path,
        json!({
            "filePath": path,
            "originalFile": "前 A\n語 x",
            "structuredPatch": [
                {"oldStart": 1, "oldLines": 2, "newStart": 1, "newLines": 2,
                 "lines": [" 前 A", "-語 x", "\\ No newline at end of file", "+語 B"]},
            ],
        }),
    );
    let output = sandbox.hook("claude", &input);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&path), "前 A\n語B\n後 C\n");
}

#[test]
fn edit_of_a_non_markdown_file_is_ignored() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.txt", "日本語 Rust\n");
    let input = edit_input(
        &path,
        json!({
            "filePath": path,
            "originalFile": "x\n",
            "structuredPatch": [
                {"oldStart": 1, "oldLines": 1, "newStart": 1, "newLines": 1,
                 "lines": ["-x", "+日本語 Rust"]},
            ],
        }),
    );
    let output = sandbox.hook("claude", &input);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(&path), "日本語 Rust\n");
}

#[test]
fn other_tools_are_ignored() {
    let sandbox = Sandbox::new();
    let input = json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"},
        "tool_response": {"stdout": "", "stderr": "", "interrupted": false},
    })
    .to_string();
    let output = sandbox.hook("claude", &input);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn unreadable_json_exits_1() {
    let sandbox = Sandbox::new();
    let output = sandbox.hook("claude", "not json");
    assert_eq!(output.status.code(), Some(1));
    assert!(!output.stderr.is_empty());
}

const AI_WORDS_RULE: &str =
    "[[rules]]\nid = \"ai-words\"\nmessage = \"AIが多用する語\"\nwords = [\"照合\"]\n";

fn stderr(output: &std::process::Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

/// ファイルの行のうち `changed` 行目だけを `+` にした `structuredPatch` を持つEditの入力
fn edit_changing_line(path: &Path, changed: usize) -> String {
    let content = read(path);
    let lines: Vec<String> = content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let marker = if i + 1 == changed { '+' } else { ' ' };
            format!("{marker}{line}")
        })
        .collect();
    edit_input(
        path,
        json!({
            "filePath": path,
            "originalFile": "",
            "structuredPatch": [
                {"oldStart": 1, "oldLines": lines.len() - 1, "newStart": 1,
                 "newLines": lines.len(), "lines": lines},
            ],
        }),
    )
}

#[test]
fn edit_reports_only_on_the_changed_line_to_stderr_and_exits_2() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(AI_WORDS_RULE);
    let path = sandbox.write("t.md", "照合 Rust\n照合 Rust\n照合 Rust\n");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 2));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(&path), "照合 Rust\n照合Rust\n照合 Rust\n");
    let lines: Vec<&str> = stderr(&output).lines().collect();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[0].contains("適用済み") && lines[0].contains("誤検知"),
        "{lines:?}"
    );
    assert_eq!(
        lines[1],
        format!(
            "{}:2:1: 「照合」: AIが多用する語 [ai-words]",
            path.display()
        )
    );
}

#[test]
fn write_of_a_new_heading_reports_the_space_and_leaves_the_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "# API の設計\n");
    let input = json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": path, "content": "# API の設計\n"},
        "tool_response": {
            "type": "create",
            "filePath": path,
            "content": "# API の設計\n",
            "structuredPatch": [],
            "originalFile": null,
        },
    })
    .to_string();
    let output = sandbox.hook("claude", &input);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(read(&path), "# API の設計\n");
    assert!(
        stderr(&output)
            .lines()
            .any(|line| line.contains(":1:6:") && line.ends_with("[space-between-ja-and-alnum]")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn report_column_is_counted_on_the_fixed_line() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(AI_WORDS_RULE);
    let path = sandbox.write("t.md", "語 A、照合\n");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 1));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(read(&path), "語A、照合\n");
    assert!(
        stderr(&output).contains(&format!("{}:1:4: 「照合」", path.display())),
        "{}",
        stderr(&output)
    );
}

#[test]
fn report_outside_the_changed_lines_is_not_returned() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(AI_WORDS_RULE);
    let path = sandbox.write("t.md", "照合する\n語 B\n照合する\n");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 2));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr(&output), "");
    assert_eq!(read(&path), "照合する\n語B\n照合する\n");
}

#[test]
fn edit_fixes_halfwidth_kana_only_on_the_changed_line() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "最初はﾊﾝｶｸです。\n\n次もﾊﾝｶｸです。\n");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 3));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr(&output), "");
    assert_eq!(read(&path), "最初はﾊﾝｶｸです。\n\n次もハンカクです。\n");
}

/// 1行目が短い文、3行目が101文字の文のファイル。`third_line_prefix` は3行目の先頭に足す
fn write_long_sentence_file(sandbox: &Sandbox, third_line_prefix: &str) -> std::path::PathBuf {
    sandbox.write(
        "t.md",
        &format!(
            "短い文です。\n\n{third_line_prefix}{}。\n",
            "あいうえ".repeat(25)
        ),
    )
}

#[test]
fn edit_reports_a_long_sentence_on_the_changed_line() {
    let sandbox = Sandbox::new();
    let path = write_long_sentence_file(&sandbox, "");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 3));
    assert_eq!(output.status.code(), Some(2));
    let lines: Vec<&str> = stderr(&output).lines().collect();
    assert_eq!(
        lines.get(1).copied(),
        Some(
            format!(
                "{}:3:1: 1文が100文字を超えている (101文字) [sentence-length]",
                path.display()
            )
            .as_str()
        ),
        "{lines:?}"
    );
}

#[test]
fn edit_does_not_report_a_long_sentence_outside_the_changed_line() {
    let sandbox = Sandbox::new();
    let path = write_long_sentence_file(&sandbox, "");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 1));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr(&output), "");
}

#[test]
fn edit_does_not_report_a_disabled_builtin_rule() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nsentence-length = false\n");
    let path = write_long_sentence_file(&sandbox, "");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 3));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr(&output), "");
}

#[test]
fn edit_of_a_report_excluded_file_fixes_but_does_not_report_builtin_rules() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("report-exclude = [\"**/t.md\"]\n");
    let path = write_long_sentence_file(&sandbox, "ﾊﾝｶｸと");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 3));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr(&output), "");
    assert_eq!(
        read(&path),
        format!("短い文です。\n\nハンカクと{}。\n", "あいうえ".repeat(25))
    );
}

#[test]
fn edit_reports_a_doubled_particle_only_on_the_changed_line() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "私は本は読む。\n\n彼は本は読む。\n");
    let output = sandbox.hook("claude", &edit_changing_line(&path, 3));
    assert_eq!(output.status.code(), Some(2));
    let lines: Vec<&str> = stderr(&output).lines().collect();
    assert_eq!(
        lines[1..],
        [format!(
            "{}:3:4: 同じ助詞「は」が続いている [no-doubled-joshi]",
            path.display()
        )],
        "{lines:?}"
    );
}

/// 1行目と3行目が「ですます」、5行目が「である」の文のファイル。本文で多いほうの文体を優先する設定で使う
fn write_mixed_style_file(sandbox: &Sandbox) -> std::path::PathBuf {
    sandbox.write_xdg_config("[builtin]\nno-mix-dearu-desumasu = { preferInBody = \"\" }\n");
    sandbox.write(
        "t.md",
        "これは本です。\n\nそれも本です。\n\nあれは本である。\n",
    )
}

#[test]
fn edit_reports_a_mixed_style_counted_over_the_whole_file_on_the_changed_line() {
    let sandbox = Sandbox::new();
    let path = write_mixed_style_file(&sandbox);
    let output = sandbox.hook("claude", &edit_changing_line(&path, 5));
    assert_eq!(output.status.code(), Some(2));
    let lines: Vec<&str> = stderr(&output).lines().collect();
    assert_eq!(
        lines.get(1).copied(),
        Some(
            format!(
                "{}:5:5: 本文に「である」調が混ざっている [no-mix-dearu-desumasu]",
                path.display()
            )
            .as_str()
        ),
        "{lines:?}"
    );
}

#[test]
fn edit_does_not_report_a_mixed_style_outside_the_changed_line() {
    let sandbox = Sandbox::new();
    let path = write_mixed_style_file(&sandbox);
    let output = sandbox.hook("claude", &edit_changing_line(&path, 1));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr(&output), "");
}

#[test]
fn reports_ai_writing_findings_on_target_lines() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(AI_RULES_CONFIG);
    let path = sandbox.write("t.md", AI_RULES_INPUT);
    let output = sandbox.hook("claude", &edit_changing_line(&path, 3));
    assert_eq!(output.status.code(), Some(2));
    let lines: Vec<&str> = stderr(&output).lines().collect();
    assert_eq!(
        lines[1..],
        [format!(
            "{}:3:4: 誇張した表現「革命的な」を使っている [no-ai-hype-expressions]",
            path.display()
        )],
        "{lines:?}"
    );
}
