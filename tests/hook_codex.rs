mod common;

use std::path::Path;

use common::{
    AI_RULES_CONFIG, AI_RULES_INPUT, AI_WORDS_PATTERN, JTF_STYLE_RULES_CONFIG,
    JTF_STYLE_RULES_HOOK_FIXED, JTF_STYLE_RULES_HOOK_INPUT, SPACING_RULES_CONFIG,
    SPACING_RULES_FIXED, SPACING_RULES_INPUT, Sandbox, USER_REPLACEMENTS_CONFIG,
    USER_REPLACEMENTS_FIXED, USER_REPLACEMENTS_INPUT, jtf_style_rule_lines, jtf_style_rule_reports,
    read, spacing_rule_lines, spacing_rule_reports, stdout, user_replacement_reports,
};
use serde_json::json;

/// Codexが実際に渡すapply_patchの入力の形
fn apply_patch_input(cwd: &Path, patch: &str) -> String {
    json!({
        "session_id": "s",
        "turn_id": "t",
        "transcript_path": "/tmp/rollout.jsonl",
        "cwd": cwd,
        "hook_event_name": "PostToolUse",
        "model": "gpt-5",
        "permission_mode": "default",
        "tool_name": "apply_patch",
        "tool_input": {"command": patch},
        "tool_response": "Success. Updated the following files:\nM t.md\n",
        "tool_use_id": "call_1",
    })
    .to_string()
}

fn update_patch(path: &str, chunks: &str) -> String {
    format!("*** Begin Patch\n*** Update File: {path}\n{chunks}*** End Patch\n")
}

/// `content` を編集後のファイルとして `t.md` に置き、そのファイルへのUpdate Fileで `jafix hook codex` を実行する
fn run_update(content: &str, chunks: &str) -> (String, std::process::Output) {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", content);
    let output = sandbox.hook(
        "codex",
        &apply_patch_input(&sandbox.path(""), &update_patch("t.md", chunks)),
    );
    (read(&path), output)
}

#[test]
fn update_with_a_path_relative_to_cwd_fixes_only_the_plus_lines() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("work/docs/t.md", "語 A\n語 B\n語 C\n");
    let patch = update_patch("docs/t.md", "@@\n 語 A\n-語 x\n+語 B\n 語 C\n");
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path("work"), &patch));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(&path), "語 A\n語B\n語 C\n");
}

#[test]
fn an_identical_line_before_the_chunk_is_left_unchanged() {
    let (content, output) = run_update(
        "語 B\n語 A\n語 B\n語 C\n",
        "@@\n 語 A\n-語 x\n+語 B\n 語 C\n",
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(content, "語 B\n語 A\n語B\n語 C\n");
}

#[test]
fn append_only_chunk_before_a_context_chunk_fixes_both() {
    let (content, output) = run_update(
        "語 A\n語 B\n語 C\n語 D。\n",
        "@@\n+語 D。\n@@\n 語 A\n-語 x\n+語 B\n",
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(content, "語 A\n語B\n語 C\n語D。\n");
}

#[test]
fn two_append_only_chunks_fix_the_last_lines() {
    let (content, output) = run_update("語 A\n語 B\n語 C。\n", "@@\n+語 B\n@@\n+語 C。\n");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(content, "語 A\n語B\n語C。\n");
}

#[test]
fn end_of_file_chunk_fixes_only_the_last_lines() {
    let (content, output) = run_update(
        "語 A\n語 B。\n語 A\n語 B。\n",
        "@@\n 語 A\n-語 x\n+語 B。\n*** End of File\n",
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(content, "語 A\n語 B。\n語 A\n語B。\n");
}

#[test]
fn chunk_without_context_matching_twice_leaves_the_file_and_exits_1() {
    let (content, output) = run_update("語 B\n語 A\n語 B\n", "@@\n-語 x\n+語 B\n");
    assert_eq!(output.status.code(), Some(1));
    assert!(!output.stderr.is_empty());
    assert_eq!(content, "語 B\n語 A\n語 B\n");
}

#[test]
fn add_file_fixes_every_line() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("new.md", "日本語 Rust\n（補足） です。\n");
    let patch =
        "*** Begin Patch\n*** Add File: new.md\n+日本語 Rust\n+（補足） です。\n*** End Patch\n";
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), patch));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&path), "日本語Rust\n(補足)です。\n");
}

#[test]
fn move_to_fixes_the_moved_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("new.md", "語 A\n語 B。\n");
    let patch = "*** Begin Patch\n*** Update File: old.md\n*** Move to: new.md\n@@\n 語 A\n-語 x\n+語 B。\n*** End Patch\n";
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), patch));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&path), "語 A\n語B。\n");
}

fn bash_input(cwd: &Path, command: &str) -> String {
    json!({
        "cwd": cwd,
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": command},
        "tool_response": "",
    })
    .to_string()
}

#[test]
fn bash_command_that_only_writes_patch_text_leaves_the_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("z.md", "一 abc\n");
    let command = "cat > fixture.txt <<'EOF'\n*** Begin Patch\n*** Add File: z.md\n+一 abc\n*** End Patch\nEOF\n";
    let output = sandbox.hook("codex", &bash_input(&sandbox.path(""), command));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert!(output.stderr.is_empty());
    assert_eq!(read(&path), "一 abc\n");
}

#[test]
fn bash_apply_patch_after_cd_leaves_both_files() {
    let sandbox = Sandbox::new();
    let content = "語 A\n語 B\n語 C\n";
    let in_cwd = sandbox.write("t.md", content);
    let in_sub = sandbox.write("sub/t.md", content);
    let command = format!(
        "cd sub && apply_patch <<'EOF'\n{}EOF\n",
        update_patch("t.md", "@@\n 語 A\n-語 x\n+語 B\n 語 C\n")
    );
    let output = sandbox.hook("codex", &bash_input(&sandbox.path(""), &command));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&in_cwd), content);
    assert_eq!(read(&in_sub), content);
}

#[test]
fn delete_only_patch_does_nothing() {
    let sandbox = Sandbox::new();
    let patch = "*** Begin Patch\n*** Delete File: gone.md\n*** End Patch\n";
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), patch));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn a_file_whose_chunk_is_missing_is_left_while_the_other_is_fixed() {
    let sandbox = Sandbox::new();
    let missing = sandbox.write("a.md", "語 A\n語 B\n");
    let found = sandbox.write("b.md", "語 A\n語 B。\n");
    let patch = "\
*** Begin Patch
*** Update File: a.md
@@
 語 A
-語 x
+語 Z
*** Update File: b.md
@@
 語 A
-語 x
+語 B。
*** End Patch
";
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), patch));
    assert_eq!(output.status.code(), Some(1));
    assert!(!output.stderr.is_empty());
    assert_eq!(read(&missing), "語 A\n語 B\n");
    assert_eq!(read(&found), "語 A\n語B。\n");
}

#[test]
fn apply_patch_reports_the_plus_line_to_stderr_and_exits_2() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(AI_WORDS_PATTERN);
    let path = sandbox.write("t.md", "語 A\n照合 B\n");
    let patch = update_patch("t.md", "@@\n 語 A\n-語 x\n+照合 B\n");
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), &patch));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(&path), "語 A\n照合B\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let expected = format!(
        "{}:2:1: 「照合」: AIが多用する語 [@textlint-rule/pattern]",
        path.display()
    );
    assert!(stderr.lines().any(|line| line == expected), "{stderr}");
}

#[test]
fn reports_and_unlocated_chunks_are_both_written_with_exit_2() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(AI_WORDS_PATTERN);
    let missing = sandbox.write("a.md", "語 A\n語 B\n");
    let reported = sandbox.write("b.md", "語 A\n照合 B\n");
    let patch = "\
*** Begin Patch
*** Update File: a.md
@@
 語 A
-語 x
+語 Z
*** Update File: b.md
@@
 語 A
-語 x
+照合 B
*** End Patch
";
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), patch));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(read(&missing), "語 A\n語 B\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let missing_path = missing.display().to_string();
    assert!(
        stderr
            .lines()
            .any(|line| line.contains(&missing_path) && !line.ends_with("[@textlint-rule/pattern]")),
        "{stderr}"
    );
    let report = format!(
        "{}:2:1: 「照合」: AIが多用する語 [@textlint-rule/pattern]",
        reported.display()
    );
    assert!(stderr.lines().any(|line| line == report), "{stderr}");
}

#[test]
fn apply_patch_reports_a_builtin_report_rule_on_the_plus_line() {
    let (_, output) = run_update("語 A\nすごい！\n", "@@\n 語 A\n-語 x\n+すごい！\n");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr
            .lines()
            .any(|line| line.ends_with("[no-exclamation-question-mark]")),
        "{stderr}"
    );
}

#[test]
fn apply_patch_reports_a_doubled_particle_only_on_the_added_line() {
    let (_, output) = run_update(
        "私は本は読む。\n\n彼は本は読む。\n",
        "@@\n 私は本は読む。\n \n+彼は本は読む。\n",
    );
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr
            .lines()
            .any(|line| line.ends_with(":3:4: 同じ助詞「は」が続いている [no-doubled-joshi]")),
        "{stderr}"
    );
    assert!(!stderr.contains(":1:4:"), "{stderr}");
}

#[test]
fn reports_ai_writing_findings_on_target_lines() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(AI_RULES_CONFIG);
    let path = sandbox.write("t.md", AI_RULES_INPUT);
    let chunk: String = AI_RULES_INPUT
        .lines()
        .enumerate()
        .map(|(i, line)| match i + 1 {
            3 => format!("-これは普通です。\n+{line}\n"),
            _ => format!(" {line}\n"),
        })
        .collect();
    let patch = update_patch("t.md", &format!("@@\n{chunk}"));
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), &patch));
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        lines[1..],
        [format!(
            "{}:3:4: 誇張した表現「革命的な」を使っている [no-ai-hype-expressions]",
            path.display()
        )],
        "{lines:?}"
    );
}

#[test]
fn fixes_user_replacements_on_target_lines() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(USER_REPLACEMENTS_CONFIG);
    let path = sandbox.write("t.md", USER_REPLACEMENTS_INPUT);
    let chunk: String = USER_REPLACEMENTS_INPUT
        .lines()
        .enumerate()
        .map(|(i, line)| match i + 1 {
            1 | 5 => format!("-x\n+{line}\n"),
            _ => format!(" {line}\n"),
        })
        .collect();
    let patch = update_patch("t.md", &format!("@@\n{chunk}"));
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), &patch));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(read(&path), USER_REPLACEMENTS_FIXED);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines[1..], user_replacement_reports(&path), "{lines:?}");
}

#[test]
fn fixes_spacing_rules_on_target_lines() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(SPACING_RULES_CONFIG);
    let path = sandbox.write("t.md", SPACING_RULES_INPUT);
    let chunk: String = SPACING_RULES_INPUT
        .lines()
        .enumerate()
        .map(|(i, line)| match i + 1 {
            1 | 5 => format!("-x\n+{line}\n"),
            _ => format!(" {line}\n"),
        })
        .collect();
    let patch = update_patch("t.md", &format!("@@\n{chunk}"));
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), &patch));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(read(&path), SPACING_RULES_FIXED);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        spacing_rule_lines(&stderr),
        spacing_rule_reports(&path),
        "{stderr}"
    );
}

#[test]
fn reports_jtf_style_findings_on_target_lines() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(JTF_STYLE_RULES_CONFIG);
    let path = sandbox.write("t.md", JTF_STYLE_RULES_HOOK_INPUT);
    let chunk: String = JTF_STYLE_RULES_HOOK_INPUT
        .lines()
        .enumerate()
        .map(|(i, line)| match i + 1 {
            3 | 7 => format!("-x\n+{line}\n"),
            _ => format!(" {line}\n"),
        })
        .collect();
    let patch = update_patch("t.md", &format!("@@\n{chunk}"));
    let output = sandbox.hook("codex", &apply_patch_input(&sandbox.path(""), &patch));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(read(&path), JTF_STYLE_RULES_HOOK_FIXED);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        jtf_style_rule_lines(&stderr),
        jtf_style_rule_reports(&path),
        "{stderr}"
    );
}
