mod common;

use std::path::Path;

use common::{Sandbox, read, stdout};
use serde_json::json;

#[test]
fn check_skips_a_file_listed_in_exclude() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    assert_eq!(sandbox.run(&["check", "t.md"]).status.code(), Some(1));

    sandbox.write_xdg_config(&format!("exclude = [{:?}]\n", path.to_str().unwrap()));
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[cfg(unix)]
#[test]
fn exclude_matches_a_symlink_to_the_listed_file() {
    let sandbox = Sandbox::new();
    let real = sandbox.write("real.md", "日本語 Rust\n");
    std::os::unix::fs::symlink(&real, sandbox.path("link.md")).unwrap();
    sandbox.write_xdg_config(&format!("exclude = [{:?}]\n", real.to_str().unwrap()));
    let output = sandbox.run(&["fix", "link.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&real), "日本語 Rust\n");
}

#[test]
fn exclude_expands_tilde_and_falls_back_to_home_config_without_xdg() {
    let sandbox = Sandbox::new();
    let excluded = sandbox.write("home/x.md", "日本語 Rust\n");
    let other = sandbox.write("home/y.md", "日本語 Rust\n");
    sandbox.write("home/.config/jafix/config.toml", "exclude = [\"~/x.md\"]\n");
    let output = sandbox
        .command()
        .env_remove("XDG_CONFIG_HOME")
        .args(["fix", "home/x.md", "home/y.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&excluded), "日本語 Rust\n");
    assert_eq!(read(&other), "日本語Rust\n");
}

#[test]
fn exclude_ignores_entries_for_missing_files() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    sandbox.write_xdg_config("exclude = [\"/nonexistent/x.md\"]\n");
    assert_eq!(sandbox.run(&["fix", "t.md"]).status.code(), Some(0));
    assert_eq!(read(&path), "日本語Rust\n");
}

#[test]
fn exclude_double_star_matches_files_in_any_drafts_directory() {
    let sandbox = Sandbox::new();
    let excluded = sandbox.write("drafts/a.md", "日本語 Rust\n");
    let other = sandbox.write("other/b.md", "日本語 Rust\n");
    sandbox.write_xdg_config("exclude = [\"**/drafts/**\"]\n");
    let output = sandbox.run(&["fix", "drafts/a.md", "other/b.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&excluded), "日本語 Rust\n");
    assert_eq!(read(&other), "日本語Rust\n");
}

#[test]
fn exclude_single_star_does_not_cross_a_directory() {
    let sandbox = Sandbox::new();
    let excluded = sandbox.write("drafts/a.md", "日本語 Rust\n");
    let nested = sandbox.write("drafts/sub/a.md", "日本語 Rust\n");
    sandbox.write_xdg_config("exclude = [\"**/drafts/*.md\"]\n");
    let output = sandbox.run(&["fix", "drafts/a.md", "drafts/sub/a.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&excluded), "日本語 Rust\n");
    assert_eq!(read(&nested), "日本語Rust\n");
}

#[cfg(unix)]
#[test]
fn exclude_with_tilde_matches_when_home_is_a_symlink() {
    let sandbox = Sandbox::new();
    let excluded = sandbox.write("real-home/notes/a.md", "日本語 Rust\n");
    let other = sandbox.write("real-home/b.md", "日本語 Rust\n");
    let home = sandbox.path("link-home");
    std::os::unix::fs::symlink(sandbox.path("real-home"), &home).unwrap();
    sandbox.write_xdg_config("exclude = [\"~/notes/*.md\"]\n");
    let output = sandbox
        .command()
        .env("HOME", &home)
        .arg("fix")
        .args([home.join("notes/a.md"), home.join("b.md")])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&excluded), "日本語 Rust\n");
    assert_eq!(read(&other), "日本語Rust\n");
}

#[test]
fn exclude_treats_glob_characters_in_the_home_path_literally() {
    let sandbox = Sandbox::new();
    let excluded = sandbox.write("h[o]me/a.md", "日本語 Rust\n");
    // `h[o]me` をglobとして読むと、このディレクトリに当たる
    let lookalike = sandbox.write("home/a.md", "日本語 Rust\n");
    sandbox.write_xdg_config("exclude = [\"~/a.md\"]\n");
    let output = sandbox
        .command()
        .env("HOME", sandbox.path("h[o]me"))
        .args(["fix", "h[o]me/a.md", "home/a.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&excluded), "日本語 Rust\n");
    assert_eq!(read(&lookalike), "日本語Rust\n");
}

#[cfg(unix)]
#[test]
fn exclude_glob_matches_the_target_of_a_symlink_outside_the_directory() {
    let sandbox = Sandbox::new();
    let real = sandbox.write("drafts/a.md", "日本語 Rust\n");
    std::os::unix::fs::symlink(&real, sandbox.path("link.md")).unwrap();
    sandbox.write_xdg_config("exclude = [\"**/drafts/**\"]\n");
    let output = sandbox.run(&["fix", "link.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&real), "日本語 Rust\n");
}

#[cfg(unix)]
#[test]
fn exclude_listing_a_symlink_matches_its_target() {
    let sandbox = Sandbox::new();
    let real = sandbox.write("real/a.md", "日本語 Rust\n");
    let link = sandbox.path("link.md");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    sandbox.write_xdg_config(&format!("exclude = [{:?}]\n", link.to_str().unwrap()));
    let output = sandbox.run(&["fix", "real/a.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&real), "日本語 Rust\n");
}

#[test]
fn check_reports_nothing_for_an_excluded_file() {
    let sandbox = Sandbox::new();
    sandbox.write("drafts/a.md", "照合する Rust\n");
    sandbox.write_xdg_config(
        "exclude = [\"**/drafts/**\"]\n\n[[rules]]\nid = \"ai-words\"\nmessage = \"AIが多用する語\"\nwords = [\"照合\"]\n",
    );
    let output = sandbox.run(&["check", "drafts/a.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_exits_2_on_a_relative_exclude() {
    let sandbox = Sandbox::new();
    sandbox.write("drafts/a.md", "日本語Rust\n");
    sandbox.write_xdg_config("exclude = [\"drafts/*.md\"]\n");
    let output = sandbox.run(&["check", "drafts/a.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
}

#[test]
fn check_exits_2_on_an_exclude_that_is_not_a_glob() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config("exclude = [\"/a/[b\"]\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
}

#[test]
fn check_exits_2_on_invalid_toml() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config("exclude = [\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
}

#[test]
fn fix_exits_2_on_unknown_key_without_changing_files() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    sandbox.write_xdg_config("exclud = []\n");
    let output = sandbox.run(&["fix", "t.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
    assert_eq!(read(&path), "日本語 Rust\n");
}

/// `rule` を `[[rules]]` の1つの表として書いた設定で `jafix check` を実行し、exit 2で標準エラーに `id` を含むことを確かめる
fn assert_check_rejects_rule(rule: &str, id: &str) {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config(rule);
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2), "{rule}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(id), "{stderr}");
}

#[test]
fn check_exits_2_on_a_rule_with_both_pattern_and_words() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"both-keys\"\nmessage = \"m\"\npattern = \"a\"\nwords = [\"b\"]\n",
        "both-keys",
    );
}

#[test]
fn check_exits_2_on_a_rule_without_pattern_or_words() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"no-matcher\"\nmessage = \"m\"\n",
        "no-matcher",
    );
}

#[test]
fn check_exits_2_on_a_rule_without_message() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"no-message\"\nwords = [\"a\"]\n",
        "no-message",
    );
}

#[test]
fn check_exits_2_on_a_rule_with_an_invalid_regex() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"bad-regex\"\nmessage = \"m\"\npattern = \"(\"\n",
        "bad-regex",
    );
}

#[test]
fn check_exits_2_on_a_rule_with_an_empty_word() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"empty-word\"\nmessage = \"m\"\nwords = [\"\"]\n",
        "empty-word",
    );
}

#[test]
fn check_exits_2_on_duplicate_rule_ids() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"dup-id\"\nmessage = \"m\"\nwords = [\"a\"]\n\n\
         [[rules]]\nid = \"dup-id\"\nmessage = \"m\"\nwords = [\"b\"]\n",
        "dup-id",
    );
}

#[test]
fn check_exits_2_on_a_rule_named_after_a_builtin_rule() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"fullwidth-paren\"\nmessage = \"m\"\nwords = [\"a\"]\n",
        "fullwidth-paren",
    );
}

#[test]
fn check_accepts_a_valid_rule() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config("[[rules]]\nid = \"ok\"\nmessage = \"m\"\nwords = [\"照合\"]\n");
    assert_eq!(sandbox.run(&["check", "t.md"]).status.code(), Some(0));
}

/// `path` の1行目を書き換えたClaude CodeのEditの入力
fn claude_edit_input(path: &Path, structured_patch: bool) -> String {
    let mut response = json!({"filePath": path, "originalFile": "x\n"});
    if structured_patch {
        response["structuredPatch"] = json!([
            {"oldStart": 1, "oldLines": 1, "newStart": 1, "newLines": 1,
             "lines": ["-x", "+日本語 Rust"]},
        ]);
    }
    json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Edit",
        "tool_input": {"file_path": path, "old_string": "x", "new_string": "日本語 Rust"},
        "tool_response": response,
    })
    .to_string()
}

#[cfg(unix)]
#[test]
fn hook_skips_a_symlink_to_an_excluded_file() {
    let sandbox = Sandbox::new();
    let real = sandbox.write("real.md", "日本語 Rust\n");
    let link = sandbox.path("link.md");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    sandbox.write_xdg_config(&format!("exclude = [{:?}]\n", real.to_str().unwrap()));
    let output = sandbox.hook("claude", &claude_edit_input(&link, true));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read(&real), "日本語 Rust\n");
}

#[test]
fn hook_ignores_a_missing_structured_patch_for_an_excluded_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    sandbox.write_xdg_config(&format!("exclude = [{:?}]\n", path.to_str().unwrap()));
    let output = sandbox.hook("claude", &claude_edit_input(&path, false));
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(read(&path), "日本語 Rust\n");
}

#[test]
fn hook_ignores_an_ambiguous_codex_chunk_for_an_excluded_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "語 B\n語 A\n語 B\n");
    sandbox.write_xdg_config(&format!("exclude = [{:?}]\n", path.to_str().unwrap()));
    let input = json!({
        "cwd": sandbox.path(""),
        "hook_event_name": "PostToolUse",
        "tool_name": "apply_patch",
        "tool_input": {
            "command": "*** Begin Patch\n*** Update File: t.md\n@@\n-語 x\n+語 B\n*** End Patch\n",
        },
        "tool_response": "",
    })
    .to_string();
    let output = sandbox.hook("codex", &input);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(read(&path), "語 B\n語 A\n語 B\n");
}

#[test]
fn hook_exits_1_on_unknown_key_without_changing_files() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    sandbox.write_xdg_config("exclud = []\n");
    let output = sandbox.hook("claude", &claude_edit_input(&path, true));
    assert_eq!(output.status.code(), Some(1));
    assert!(!output.stderr.is_empty());
    assert_eq!(read(&path), "日本語 Rust\n");
}

#[test]
fn hook_exits_1_on_an_unknown_key_in_a_rule_without_changing_files() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    sandbox.write_xdg_config(
        "[[rules]]\nid = \"ai-words\"\nmessage = \"m\"\nwords = [\"a\"]\nreplace = \"b\"\n",
    );
    let output = sandbox.hook("claude", &claude_edit_input(&path, true));
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("replace"), "{stderr}");
    assert_eq!(read(&path), "日本語 Rust\n");
}

const REPORT_EXCLUDE_NOTES_WITH_WORDS: &str = "report-exclude = [\"**/notes/**\"]\n\n[[rules]]\nid = \"ai-words\"\nmessage = \"AIが多用する語\"\nwords = [\"照合\"]\n";

#[test]
fn check_reports_only_fix_rules_for_a_report_excluded_file() {
    let sandbox = Sandbox::new();
    sandbox.write("notes/a.md", "照合 Rust\n");
    sandbox.write_xdg_config(REPORT_EXCLUDE_NOTES_WITH_WORDS);
    let output = sandbox.run(&["check", "notes/a.md"]);
    assert_eq!(output.status.code(), Some(1));
    let lines: Vec<&str> = stdout(&output).lines().collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].ends_with(" [space-between-ja-and-alnum]"),
        "{lines:?}"
    );
}

#[test]
fn hook_fixes_a_report_excluded_file_without_reporting() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("notes/a.md", "照合 Rust\n");
    sandbox.write_xdg_config(REPORT_EXCLUDE_NOTES_WITH_WORDS);
    let output = sandbox.hook("claude", &claude_edit_input(&path, true));
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(read(&path), "照合Rust\n");
}

#[test]
fn check_does_not_report_heading_spaces_in_a_report_excluded_file() {
    let sandbox = Sandbox::new();
    sandbox.write("notes/a.md", "# API の設計\n");
    sandbox.write_xdg_config("report-exclude = [\"**/notes/**\"]\n");
    let output = sandbox.run(&["check", "notes/a.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_exits_2_on_a_relative_report_exclude_and_accepts_a_double_star_one() {
    let sandbox = Sandbox::new();
    sandbox.write("sub/a.md", "日本語Rust\n");
    sandbox.write_xdg_config("report-exclude = [\"sub/*.md\"]\n");
    let output = sandbox.run(&["check", "sub/a.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
    sandbox.write_xdg_config("report-exclude = [\"**/sub/*.md\"]\n");
    assert_eq!(sandbox.run(&["check", "sub/a.md"]).status.code(), Some(0));
}
