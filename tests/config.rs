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
    sandbox.write("t.md", "日本語Rustです。\n");
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

#[test]
fn check_reports_a_builtin_report_rule_only_while_it_is_enabled() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "すごい！\n");
    sandbox.write_xdg_config("[builtin]\nno-exclamation-question-mark = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:1:4: 「！」を使わない [no-exclamation-question-mark]\n"
    );

    sandbox.write_xdg_config("[builtin]\nno-exclamation-question-mark = false\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn a_disabled_fullwidth_paren_is_neither_fixed_nor_reported() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "注（補足）です。\n");
    assert_eq!(sandbox.run(&["check", "t.md"]).status.code(), Some(1));

    sandbox.write_xdg_config("[builtin]\nfullwidth-paren = false\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(sandbox.run(&["fix", "t.md"]).status.code(), Some(0));
    assert_eq!(read(&path), "注（補足）です。\n");
}

#[test]
fn a_disabled_no_hankaku_kana_is_not_fixed() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "ﾊﾝｶｸです。\n");
    sandbox.write_xdg_config("[builtin]\nno-hankaku-kana = false\n");
    assert_eq!(sandbox.run(&["fix", "t.md"]).status.code(), Some(0));
    assert_eq!(read(&path), "ﾊﾝｶｸです。\n");

    sandbox.write_xdg_config("[builtin]\nno-hankaku-kana = true\n");
    assert_eq!(sandbox.run(&["fix", "t.md"]).status.code(), Some(0));
    assert_eq!(read(&path), "ハンカクです。\n");
}

#[test]
fn a_disabled_space_rule_is_not_reported_inside_a_heading() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "# API の設計\n");
    assert_eq!(sandbox.run(&["check", "t.md"]).status.code(), Some(1));

    sandbox.write_xdg_config("[builtin]\nspace-between-ja-and-alnum = false\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

/// `builtin` を `[builtin]` の表に書いた設定で `jafix check` を実行し、exit 2で標準エラーに `expected` をすべて含むことを確かめる
fn assert_check_rejects_builtin(builtin: &str, expected: &[&str]) {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config(&format!("[builtin]\n{builtin}\n"));
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2), "{builtin}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    for needle in expected {
        assert!(stderr.contains(needle), "{builtin}: {stderr}");
    }
}

#[test]
fn check_exits_2_on_an_unknown_builtin_rule() {
    assert_check_rejects_builtin("no-such-rule = false", &["no-such-rule"]);
}

#[test]
fn check_exits_2_on_a_builtin_value_that_is_neither_bool_nor_table() {
    assert_check_rejects_builtin("sentence-length = 5", &["sentence-length"]);
}

#[test]
fn check_exits_2_on_an_unknown_builtin_option() {
    assert_check_rejects_builtin(
        "sentence-length = { maxx = 5 }",
        &["sentence-length", "maxx"],
    );
    assert_check_rejects_builtin("no-nfd = { x = 1 }", &["no-nfd", "x"]);
}

#[test]
fn check_exits_2_on_textlint_options_that_jafix_does_not_support() {
    for (builtin, option) in [
        (
            "ja-no-mixed-period = { forceAppendPeriod = true }",
            "forceAppendPeriod",
        ),
        (
            "ja-no-mixed-period = { checkFootnote = true }",
            "checkFootnote",
        ),
        (
            "sentence-length = { exclusionPatterns = [] }",
            "exclusionPatterns",
        ),
        (
            "no-invalid-control-character = { checkCode = true }",
            "checkCode",
        ),
        (
            "no-invalid-control-character = { checkImage = false }",
            "checkImage",
        ),
    ] {
        assert_check_rejects_builtin(builtin, &[option]);
    }
}

#[test]
fn check_exits_2_on_a_builtin_option_of_the_wrong_type_or_value() {
    assert_check_rejects_builtin(
        "sentence-length = { max = \"a\" }",
        &["sentence-length", "max"],
    );
    assert_check_rejects_builtin("max-comma = { max = 0 }", &["max-comma", "max"]);
    assert_check_rejects_builtin(
        "max-kanji-continuous-len = { max = 0 }",
        &["max-kanji-continuous-len", "max"],
    );
    assert_check_rejects_builtin("sentence-length = { countBy = \"bytes\" }", &["countBy"]);
    assert_check_rejects_builtin(
        "no-invalid-control-character = { allow = [\"ab\"] }",
        &["allow"],
    );
}

#[test]
fn check_accepts_zero_as_the_sentence_length_max() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config("[builtin]\nsentence-length = { max = 0 }\n");
    assert_ne!(sandbox.run(&["check", "t.md"]).status.code(), Some(2));
}

#[test]
fn check_exits_2_on_a_pattern_that_cannot_be_read() {
    assert_check_rejects_builtin(
        "sentence-length = { skipPatterns = [\"/(/\"] }",
        &["skipPatterns"],
    );
    assert_check_rejects_builtin(
        "no-exclamation-question-mark = { allow = [\"/a/y\"] }",
        &["allow"],
    );
}

#[test]
fn check_exits_2_on_a_rule_named_after_a_ported_builtin_rule() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"max-comma\"\nmessage = \"m\"\nwords = [\"a\"]\n",
        "max-comma",
    );
}

#[test]
fn hook_exits_1_on_an_unknown_builtin_rule_without_changing_files() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    sandbox.write_xdg_config("[builtin]\nno-such-rule = false\n");
    let output = sandbox.hook("claude", &claude_edit_input(&path, true));
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("no-such-rule"), "{stderr}");
    assert_eq!(read(&path), "日本語 Rust\n");
}

#[test]
fn check_reports_a_morphological_builtin_rule_by_default_and_not_when_disabled() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "私は本は読む。\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:1:4: 同じ助詞「は」が続いている [no-doubled-joshi]\n"
    );

    sandbox.write_xdg_config("[builtin]\nno-doubled-joshi = false\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_exits_2_on_an_invalid_option_of_a_morphological_builtin_rule() {
    for (builtin, expected) in [
        ("max-ten = { max = -1 }", ["max-ten", "max"]),
        ("max-ten = { touten = \"\" }", ["max-ten", "touten"]),
        (
            "no-doubled-joshi = { min_interval = 0 }",
            ["no-doubled-joshi", "min_interval"],
        ),
        (
            "no-mix-dearu-desumasu = { preferInBody = \"敬体\" }",
            ["no-mix-dearu-desumasu", "preferInBody"],
        ),
        (
            "ja-no-redundant-expression = { allowNodeTypes = [\"Str\"] }",
            ["ja-no-redundant-expression", "allowNodeTypes"],
        ),
        (
            "ja-no-redundant-expression = { dictOptions = { dict7 = { disabled = true } } }",
            ["ja-no-redundant-expression", "dict7"],
        ),
        (
            "no-doubled-conjunctive-particle-ga = { separatorChars = [\"。\"] }",
            ["no-doubled-conjunctive-particle-ga", "separatorChars"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

#[test]
fn check_exits_2_on_a_rule_named_after_a_morphological_builtin_rule() {
    assert_check_rejects_rule(
        "[[rules]]\nid = \"max-ten\"\nmessage = \"m\"\nwords = [\"a\"]\n",
        "max-ten",
    );
}
