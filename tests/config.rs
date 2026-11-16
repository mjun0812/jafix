mod common;

use std::path::Path;

use common::{
    AI_RULES_INPUT, AI_WORDS_PATTERN, HIRAGANA_AND_SYNONYMS_RULES, Sandbox,
    hiragana_and_synonyms_rules_config, read, stdout,
};
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
    sandbox.write_xdg_config(&format!(
        "exclude = [\"**/drafts/**\"]\n\n{AI_WORDS_PATTERN}"
    ));
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
fn hook_exits_1_on_an_unknown_key_in_a_pattern_element_without_changing_files() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    sandbox.write_xdg_config(&format!("{AI_WORDS_PATTERN}forceCode = true\n"));
    let output = sandbox.hook("claude", &claude_edit_input(&path, true));
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("forceCode"), "{stderr}");
    assert_eq!(read(&path), "日本語 Rust\n");
}

fn report_exclude_notes_with_ai_words() -> String {
    format!("report-exclude = [\"**/notes/**\"]\n\n{AI_WORDS_PATTERN}")
}

#[test]
fn check_reports_only_fix_rules_for_a_report_excluded_file() {
    let sandbox = Sandbox::new();
    sandbox.write("notes/a.md", "照合 Rust\n");
    sandbox.write_xdg_config(&report_exclude_notes_with_ai_words());
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
    sandbox.write_xdg_config(&report_exclude_notes_with_ai_words());
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

fn check_with_builtin(builtin: &str, text: &str) -> (Option<i32>, String) {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", text);
    sandbox.write_xdg_config(&format!("[builtin]\n{builtin}\n"));
    let output = sandbox.run(&["check", "t.md"]);
    (output.status.code(), stdout(&output).to_owned())
}

#[test]
fn builtin_option_patterns_use_javascript_regex() {
    // 後読みで `a` の直後の `b` だけを取り除くので、1つ目の段落は3文字になり、2つ目の段落は4文字のまま残る
    let (code, stdout) = check_with_builtin(
        r"sentence-length = { max = 3, skipPatterns = ['/(?<=a)b/'] }",
        "abcd\n\nxbcd\n",
    );
    assert_eq!(code, Some(1));
    assert_eq!(
        stdout,
        "t.md:3:1: 1文が3文字を超えている (4文字) [sentence-length]\n"
    );

    assert_check_rejects_builtin(
        r"no-exclamation-question-mark = { allow = ['/\-/'] }",
        &["allow"],
    );
}

#[test]
fn builtin_option_digit_class_is_ascii() {
    let (code, stdout) = check_with_builtin(
        r"no-exclamation-question-mark = { allow = ['/\d!/'] }",
        "1!\n１!\n",
    );
    assert_eq!(code, Some(1));
    assert_eq!(
        stdout,
        "t.md:2:2: 「!」を使わない [no-exclamation-question-mark]\n"
    );
}

#[test]
fn builtin_option_patterns_reject_rust_only_syntax() {
    for pattern in [r"/\p{Han}/", "/(?i)a/", "/[[:alpha:]]/", "/a{/"] {
        assert_check_rejects_builtin(
            &format!("ja-unnatural-alphabet = {{ allow = ['{pattern}'] }}"),
            &["ja-unnatural-alphabet", "allow", pattern],
        );
    }
    let (code, _) = check_with_builtin(
        r"ja-unnatural-alphabet = { allow = ['/\p{sc=Han}/'] }",
        "日本語\n",
    );
    assert_ne!(code, Some(2));
}

#[test]
fn builtin_option_slash_form_follows_pattern_rule() {
    // `//` はすべてを許す空の正規表現ではなく、文字列そのものの `//` である
    let (code, stdout) = check_with_builtin(
        r#"no-exclamation-question-mark = { allow = ["//"] }"#,
        "a!\n",
    );
    assert_eq!(code, Some(1));
    assert_eq!(
        stdout,
        "t.md:1:2: 「!」を使わない [no-exclamation-question-mark]\n"
    );

    // フラグ `d` を受けて無視するので、`/a!/d` は正規表現 `a!` になる
    let (code, stdout) = check_with_builtin(
        r#"no-exclamation-question-mark = { allow = ["/a!/d"] }"#,
        "a!\n",
    );
    assert_eq!((code, stdout.as_str()), (Some(0), ""));
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
        (
            "ja-no-successive-word = { allow = [\"\"] }",
            ["ja-no-successive-word", "allow"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

const MORPHOLOGICAL_RULES: [&str; 11] = [
    "max-ten",
    "no-doubled-joshi",
    "no-doubled-conjunctive-particle-ga",
    "no-doubled-conjunction",
    "no-mix-dearu-desumasu",
    "no-double-negative-ja",
    "no-dropping-the-ra",
    "ja-no-weak-phrase",
    "ja-no-successive-word",
    "ja-no-abusage",
    "ja-no-redundant-expression",
];

/// 形態素解析を使う11ルールのそれぞれに当たり、ほかの組み込みのルールには当たらない文書。
/// 段落を分けて、ルールどうしの指摘が重ならないようにしている
const MORPHOLOGICAL_RULES_INPUT: &str = "走って、食べて、寝て、起きて、歩く。

私は本は読む。

雨は降ったが、風が強いが、行く。

雨だ。しかし、行く。しかし、帰る。

これは本である。

知らないこともない。

食べれる。

問題があるかもしれない。

これはは問題です。

法律を適応する。

解析することができます。
";

#[test]
fn check_reports_each_morphological_builtin_rule_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", MORPHOLOGICAL_RULES_INPUT);
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    let stdout = stdout(&output);
    for rule in MORPHOLOGICAL_RULES {
        let suffix = format!("[{rule}]");
        assert!(
            stdout.lines().any(|line| line.ends_with(&suffix)),
            "{rule}: {stdout}"
        );
    }
}

#[test]
fn check_reports_nothing_when_every_morphological_builtin_rule_is_disabled() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", MORPHOLOGICAL_RULES_INPUT);
    let disabled: String = MORPHOLOGICAL_RULES
        .iter()
        .map(|rule| format!("{rule} = false\n"))
        .collect();
    sandbox.write_xdg_config(&format!("[builtin]\n{disabled}"));
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn fix_does_not_change_text_that_only_morphological_builtin_rules_report() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", MORPHOLOGICAL_RULES_INPUT);
    assert_eq!(sandbox.run(&["fix", "t.md"]).status.code(), Some(0));
    assert_eq!(read(&path), MORPHOLOGICAL_RULES_INPUT);
}

#[test]
fn max_ten_resets_the_count_at_the_kuten_before_comparing_it_with_max() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "あ、い。\n");
    sandbox.write_xdg_config("[builtin]\nmax-ten = { max = 0 }\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        stdout(&output),
        "t.md:1:2: 1文の「、」が0個を超えている [max-ten]\n"
    );

    // 読点と句点が同じ文字なら、数えた直後に0に戻るので、上限の0を超えない
    sandbox.write_xdg_config("[builtin]\nmax-ten = { max = 0, touten = \"。\" }\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

/// `stdout` のうち、`rule` の指摘の行
fn findings_of<'a>(stdout: &'a str, rule: &str) -> Vec<&'a str> {
    let suffix = format!(" [{rule}]");
    stdout
        .lines()
        .filter(|line| line.ends_with(&suffix))
        .collect()
}

const AI_WRITING_RULES: [&str; 5] = [
    "no-ai-list-formatting",
    "no-ai-hype-expressions",
    "no-ai-emphasis-patterns",
    "no-ai-colon-continuation",
    "ai-tech-writing-guideline",
];

/// `AI_RULES_INPUT` に当たる7ルール
const AI_RULES: [&str; 7] = [
    "no-ai-list-formatting",
    "no-ai-hype-expressions",
    "no-ai-emphasis-patterns",
    "no-ai-colon-continuation",
    "ai-tech-writing-guideline",
    "no-ai-words",
    "no-short-topic-comma",
];

#[test]
fn new_rules_are_off_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", AI_RULES_INPUT);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in AI_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }

    // 同じ文書に7ルールのそれぞれが当たることを、すべて有効にして確かめる
    let enabled: String = AI_RULES
        .iter()
        .map(|rule| format!("{rule} = true\n"))
        .collect();
    sandbox.write_xdg_config(&format!("[builtin]\n{enabled}"));
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in AI_RULES {
        assert!(!findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
}

#[test]
fn preset_ai_writing_enables_its_rules() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", AI_RULES_INPUT);
    sandbox.write_xdg_config("[builtin]\n\"@textlint-ja/preset-ai-writing\" = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in AI_WRITING_RULES {
        assert!(!findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
    for rule in ["no-ai-words", "no-short-topic-comma"] {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
}

#[test]
fn rule_key_overrides_preset_key() {
    for builtin in [
        "\"@textlint-ja/preset-ai-writing\" = true\nno-ai-hype-expressions = false\n",
        "no-ai-hype-expressions = false\n\"@textlint-ja/preset-ai-writing\" = true\n",
    ] {
        let sandbox = Sandbox::new();
        sandbox.write("t.md", AI_RULES_INPUT);
        sandbox.write_xdg_config(&format!("[builtin]\n{builtin}"));
        let output = sandbox.run(&["check", "t.md"]);
        let found = stdout(&output);
        for rule in AI_WRITING_RULES {
            assert_eq!(
                findings_of(found, rule).is_empty(),
                rule == "no-ai-hype-expressions",
                "{builtin}: {rule}: {found}"
            );
        }
    }
}

#[test]
fn new_rule_options_are_validated() {
    for (builtin, expected) in [
        (
            "no-ai-list-formatting = { disableBoldListItem = true }",
            ["no-ai-list-formatting", "disableBoldListItem"],
        ),
        (
            "no-ai-list-formatting = { disableEmojiListItems = \"yes\" }",
            ["no-ai-list-formatting", "disableEmojiListItems"],
        ),
        (
            "no-ai-hype-expressions = { disablePredictivePatterns = true }",
            ["no-ai-hype-expressions", "disablePredictivePatterns"],
        ),
        (
            "no-ai-hype-expressions = { allows = \"許可\" }",
            ["no-ai-hype-expressions", "allows"],
        ),
        (
            "no-ai-emphasis-patterns = { disableHeadingPatterns = true }",
            ["no-ai-emphasis-patterns", "disableHeadingPatterns"],
        ),
        (
            "no-ai-emphasis-patterns = { disableInfoPatterns = 1 }",
            ["no-ai-emphasis-patterns", "disableInfoPatterns"],
        ),
        (
            "no-ai-colon-continuation = { disableHeading = true }",
            ["no-ai-colon-continuation", "disableHeading"],
        ),
        (
            "no-ai-colon-continuation = { disableTable = \"true\" }",
            ["no-ai-colon-continuation", "disableTable"],
        ),
        (
            "ai-tech-writing-guideline = { severity = \"warning\" }",
            ["ai-tech-writing-guideline", "severity"],
        ),
        (
            "ai-tech-writing-guideline = { enableDocumentAnalysis = 1 }",
            ["ai-tech-writing-guideline", "enableDocumentAnalysis"],
        ),
        (
            "no-ai-words = { dictionary = \"dict.json\" }",
            ["no-ai-words", "dictionary"],
        ),
        (
            "no-ai-words = { dictionaryMode = \"replace\" }",
            ["no-ai-words", "dictionaryMode"],
        ),
        (
            "no-short-topic-comma = { max = 3 }",
            ["no-short-topic-comma", "max"],
        ),
        (
            "no-short-topic-comma = { maxLength = 0 }",
            ["no-short-topic-comma", "maxLength"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

#[test]
fn preset_ai_words_ja_enables_only_no_ai_words() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", AI_RULES_INPUT);
    sandbox.write_xdg_config("[builtin]\npreset-ai-words-ja = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert!(!findings_of(found, "no-ai-words").is_empty(), "{found}");
    for rule in AI_WRITING_RULES.iter().chain(&["no-short-topic-comma"]) {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }

    // 同じ文書に `no-short-topic-comma` が当たることを、ルールのIDのキーで有効にして確かめる
    sandbox.write_xdg_config("[builtin]\npreset-ai-words-ja = true\nno-short-topic-comma = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert_eq!(
        findings_of(found, "no-short-topic-comma"),
        ["t.md:17:4: 主題を3文字示しただけで読点を打っている [no-short-topic-comma]"]
    );
}

/// `preset-ja-technical-writing` の23ルール
const TECHNICAL_WRITING_RULES: [&str; 23] = [
    "sentence-length",
    "max-comma",
    "max-kanji-continuous-len",
    "arabic-kanji-numbers",
    "ja-no-mixed-period",
    "no-nfd",
    "no-invalid-control-character",
    "no-zero-width-spaces",
    "no-exclamation-question-mark",
    "no-hankaku-kana",
    "ja-unnatural-alphabet",
    "no-unmatched-pair",
    "max-ten",
    "no-mix-dearu-desumasu",
    "no-doubled-conjunction",
    "no-doubled-conjunctive-particle-ga",
    "no-double-negative-ja",
    "no-doubled-joshi",
    "no-dropping-the-ra",
    "ja-no-weak-phrase",
    "ja-no-successive-word",
    "ja-no-abusage",
    "ja-no-redundant-expression",
];

/// jafix独自の修正ルール
const JAFIX_FIX_RULES: [&str; 3] = [
    "fullwidth-paren",
    "space-between-ja-and-alnum",
    "space-between-ja-and-paren",
];

/// `preset-ja-technical-writing` の23ルールと、jafix独自の修正ルール3つのそれぞれに当たる文書
fn technical_writing_rules_input() -> String {
    let others = [
        format!("{}。", "あ".repeat(101)),
        "a,b,c,d,e".to_owned(),
        "情報処理推進機です。".to_owned(),
        "二回実行する。".to_owned(),
        "これは文です".to_owned(),
        "か\u{3099}き。".to_owned(),
        "あ\u{1}い。".to_owned(),
        "あ\u{200b}い。".to_owned(),
        "すごい！".to_owned(),
        "ﾊﾝｶｸです。".to_owned(),
        "あgい。".to_owned(),
        "「閉じない文です。".to_owned(),
        "日本語 Rustです。".to_owned(),
        "注の文 （補足）です。".to_owned(),
    ];
    format!("{MORPHOLOGICAL_RULES_INPUT}\n{}\n", others.join("\n\n"))
}

#[test]
fn preset_ja_technical_writing_false_disables_its_rules() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", &technical_writing_rules_input());
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in TECHNICAL_WRITING_RULES.iter().chain(&JAFIX_FIX_RULES) {
        assert!(!findings_of(found, rule).is_empty(), "{rule}: {found}");
    }

    sandbox.write_xdg_config("[builtin]\npreset-ja-technical-writing = false\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in TECHNICAL_WRITING_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
    for rule in JAFIX_FIX_RULES {
        assert!(!findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
}

#[test]
fn rule_key_enables_a_rule_of_a_disabled_preset() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", &technical_writing_rules_input());
    sandbox.write_xdg_config(
        "[builtin]\npreset-ja-technical-writing = false\nsentence-length = true\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in TECHNICAL_WRITING_RULES {
        assert_eq!(
            findings_of(found, rule).is_empty(),
            rule != "sentence-length",
            "{rule}: {found}"
        );
    }
}

#[test]
fn preset_value_must_be_boolean() {
    assert_check_rejects_builtin("preset-ai-words-ja = { }", &["preset-ai-words-ja"]);
}

/// 「再考」の1項目を持つ利用者の辞書を、設定ファイルのディレクトリに `dict.json` として置く
fn write_user_dictionary(sandbox: &Sandbox) {
    let dictionary = json!({
        "entries": [
            {
                "message": "「再考」は言い換える",
                "tokens": [{ "pos": "名詞", "basic_form": "再考" }]
            }
        ]
    });
    sandbox.write("xdg/jafix/dict.json", &dictionary.to_string());
}

#[test]
fn no_ai_words_appends_user_dictionary() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "再考と照合を行う。\n");
    write_user_dictionary(&sandbox);
    sandbox.write_xdg_config("[builtin]\nno-ai-words = { dictionaryPath = \"dict.json\" }\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "no-ai-words"),
        [
            "t.md:1:1: 「再考」は言い換える [no-ai-words]",
            "t.md:1:4: AIが多用する語「照合」を使っている [no-ai-words]",
        ]
    );
}

#[test]
fn no_ai_words_override_uses_only_user_dictionary() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "再考と照合を行う。\n");
    write_user_dictionary(&sandbox);
    sandbox.write_xdg_config(
        "[builtin]\nno-ai-words = { dictionaryPath = \"dict.json\", dictionaryMode = \"override\" }\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "no-ai-words"),
        ["t.md:1:1: 「再考」は言い換える [no-ai-words]"]
    );
}

#[test]
fn no_ai_words_rejects_invalid_user_dictionary() {
    let token = |key: &str| {
        json!({ "entries": [{ "message": "m", "tokens": [{ key: "再考" }] }] }).to_string()
    };
    for dictionary in [
        None,
        Some("{ \"entries\": [".to_owned()),
        Some(token("part_of_speech")),
        Some(token("word_id")),
    ] {
        let sandbox = Sandbox::new();
        sandbox.write("t.md", "再考する。\n");
        if let Some(dictionary) = &dictionary {
            sandbox.write("xdg/jafix/dict.json", dictionary);
        }
        sandbox.write_xdg_config("[builtin]\nno-ai-words = { dictionaryPath = \"dict.json\" }\n");
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(output.status.code(), Some(2), "{dictionary:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("dictionaryPath"),
            "{dictionary:?}: {stderr}"
        );
    }
}

#[test]
fn no_ai_words_override_requires_dictionary_path() {
    assert_check_rejects_builtin(
        "no-ai-words = { dictionaryMode = \"override\" }",
        &["dictionaryMode"],
    );
}

#[test]
fn no_ai_words_expands_tilde_in_dictionary_path() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "再考する。\n");
    let dictionary = json!({
        "entries": [{ "message": "「再考」は言い換える", "tokens": [{ "basic_form": "再考" }] }]
    });
    sandbox.write("home/dict.json", &dictionary.to_string());
    sandbox.write_xdg_config("[builtin]\nno-ai-words = { dictionaryPath = \"~/dict.json\" }\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "no-ai-words"),
        ["t.md:1:1: 「再考」は言い換える [no-ai-words]"]
    );
}

#[test]
fn no_ai_words_rejects_a_skippable_that_is_not_boolean() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "再考する。\n");
    let dictionary = json!({
        "entries": [{ "message": "m", "tokens": [{ "basic_form": "再考", "_skippable": "yes" }] }]
    });
    sandbox.write("xdg/jafix/dict.json", &dictionary.to_string());
    sandbox.write_xdg_config("[builtin]\nno-ai-words = { dictionaryPath = \"dict.json\" }\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("dictionaryPath") && stderr.contains("_skippable"),
        "{stderr}"
    );
}

#[test]
fn replacement_rules_require_options() {
    assert_check_rejects_builtin("prh = true", &["prh", "rulePaths"]);
    assert_check_rejects_builtin(
        "\"@textlint-rule/pattern\" = true",
        &["@textlint-rule/pattern", "patterns"],
    );
}

/// `rulePaths` に `path` だけを書いた `prh` の設定
fn prh_rule_paths(path: &str) -> String {
    format!("[builtin]\nprh = {{ rulePaths = [{path:?}] }}\n")
}

/// `content` の `t.md` に `jafix fix` を実行し、直した後の内容を返す
fn fix_with_config(sandbox: &Sandbox, content: &str) -> String {
    let path = sandbox.write("t.md", content);
    let output = sandbox.run(&["fix", "t.md"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    read(&path)
}

#[test]
fn prh_reads_rule_paths_from_the_config_directory() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "xdg/jafix/dict/prh.yml",
        "rules:\n  - expected: Cookie\n    pattern: クッキー\n",
    );
    sandbox.write_xdg_config(&prh_rule_paths("dict/prh.yml"));
    assert_eq!(
        fix_with_config(&sandbox, "クッキーを食べる。\n"),
        "Cookieを食べる。\n"
    );

    sandbox.write(
        "home/prh.yml",
        "rules:\n  - expected: Biscuit\n    pattern: ビスケット\n",
    );
    sandbox.write_xdg_config(&prh_rule_paths("~/prh.yml"));
    assert_eq!(
        fix_with_config(&sandbox, "ビスケットを食べる。\n"),
        "Biscuitを食べる。\n"
    );
}

#[test]
fn prh_resolves_imports_relative_to_the_dictionary() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "xdg/jafix/dict/a.yml",
        "imports:
  - path: sub/b.yml
    ignoreRules:
      - expected: B1
      - pattern: /b2/gmu
  - path: sub/c.yml
    disableImports: true
rules:
  - expected: A1
    pattern: a1
",
    );
    sandbox.write(
        "xdg/jafix/dict/sub/b.yml",
        "rules:
  - expected: B1
    pattern: b1
  - expected: B2
    pattern: /b2/
  - expected: B3
    pattern: b3
",
    );
    sandbox.write(
        "xdg/jafix/dict/sub/c.yml",
        "imports:\n  - d.yml\nrules:\n  - expected: C1\n    pattern: c1\n",
    );
    sandbox.write(
        "xdg/jafix/dict/sub/d.yml",
        "rules:\n  - expected: D1\n    pattern: d1\n",
    );
    sandbox.write_xdg_config(&prh_rule_paths("dict/a.yml"));
    assert_eq!(
        fix_with_config(&sandbox, "a1 b1 b2 b3 c1 d1\n"),
        "A1 b1 b2 B3 C1 d1\n"
    );
}

#[test]
fn prh_keeps_the_first_rule_with_the_same_expected() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "xdg/jafix/prh.yml",
        "rules:\n  - expected: Cookie\n    pattern: クッキー\n",
    );
    sandbox.write_xdg_config(
        r#"[builtin.prh]
rulePaths = ["prh.yml"]
ruleContents = ["""
rules:
  - expected: Cookie
    pattern: ビスケット
  - expected: Pie
    pattern: パイ
"""]
"#,
    );
    assert_eq!(
        fix_with_config(&sandbox, "クッキーとビスケットとパイ\n"),
        "CookieとビスケットとPie\n"
    );
}

#[test]
fn prh_keeps_rules_with_the_same_expected_in_one_dictionary() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(
        r#"[builtin.prh]
ruleContents = ["""
rules:
  - expected: Cookie
    pattern: クッキー
  - expected: Cookie
    pattern: ビスケット
"""]
"#,
    );
    assert_eq!(
        fix_with_config(&sandbox, "クッキーとビスケット\n"),
        "CookieとCookie\n"
    );
}

#[test]
fn prh_rejects_invalid_dictionaries() {
    for dictionary in [
        None,
        Some("rules: [\n"),
        Some("rules:\n  - pattern: a\n"),
        Some("rules:\n  - expected: A\n    pattern: /(/\n"),
        Some("rules:\n  - expected: A\n    patterns: [/a/i, /b/]\n"),
        Some("rules:\n  - expected: A\n    pattern: /(a)/\n    regexpMustEmpty: a\n"),
        Some(
            "rules:\n  - expected: A\n    pattern: a\n    specs:\n      - from: a\n        to: ''\n",
        ),
        Some("imports:\n  - prh.yml\nrules:\n  - expected: A\n    pattern: a\n"),
    ] {
        let sandbox = Sandbox::new();
        sandbox.write("t.md", "a\n");
        let path = sandbox.path("xdg/jafix/dict/prh.yml");
        if let Some(dictionary) = dictionary {
            sandbox.write("xdg/jafix/dict/prh.yml", dictionary);
        }
        sandbox.write_xdg_config(&prh_rule_paths("dict/prh.yml"));
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(output.status.code(), Some(2), "{dictionary:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        for needle in ["prh", "rulePaths", &path.display().to_string()] {
            assert!(stderr.contains(needle), "{dictionary:?}: {stderr}");
        }
    }
}

#[test]
fn prh_names_the_invalid_rule_contents() {
    assert_check_rejects_builtin(
        "prh = { ruleContents = [\"rules: []\", \"rules: [\"] }",
        &["prh", "ruleContents", "2番目"],
    );
}

#[test]
fn prh_reports_failed_specs() {
    assert_check_rejects_builtin(
        r#"[builtin.prh]
ruleContents = ["""
rules:
  - expected: Cookie
    pattern: クッキー
    specs:
      - from: クッキーを食べる
        to: Biscuitを食べる
"""]"#,
        &[
            "Cookie",
            "クッキーを食べる",
            "Biscuitを食べる",
            "Cookieを食べる",
        ],
    );
}

#[test]
fn prh_options_are_validated() {
    for (option, name) in [
        ("checkCodeComment = true", "checkCodeComment"),
        ("debug = true", "debug"),
        ("checkLinks = true", "checkLinks"),
        ("checkHeader = \"yes\"", "checkHeader"),
    ] {
        assert_check_rejects_builtin(
            &format!("prh = {{ ruleContents = [\"rules: []\"], {option} }}"),
            &["prh", name],
        );
    }
}

#[test]
fn textlint_rule_pattern_options_are_validated() {
    for (element, name) in [
        (
            "message = \"m\", pattern = \"a\", forceCode = true",
            "forceCode",
        ),
        (
            "message = \"m\", pattern = \"a\", replaces = \"b\"",
            "replaces",
        ),
        ("pattern = \"a\"", "message"),
        ("message = \"m\", pattern = \"/(/\"", "pattern"),
        ("message = \"m\", pattern = \"/a/y\"", "pattern"),
        ("message = \"m\", pattern = \"\"", "pattern"),
        (
            "message = \"m\", pattern = \"a\", allowNodeTypes = [\"Str\"]",
            "allowNodeTypes",
        ),
    ] {
        assert_check_rejects_builtin(
            &format!("\"@textlint-rule/pattern\" = {{ patterns = [{{ {element} }}] }}"),
            &["@textlint-rule/pattern", name],
        );
    }
    assert_check_rejects_builtin(
        "\"@textlint-rule/pattern\" = { patterns = [] }",
        &["@textlint-rule/pattern", "patterns"],
    );
}

const FULLWIDTH_PAREN_DICTIONARY: &str = r#"[builtin.prh]
ruleContents = ['''
rules:
  - expected: （$1）
    pattern: /\((.+?)\)/
''']
"#;

#[test]
fn prh_rejects_a_replacement_opposite_to_a_fix_rule() {
    assert_check_rejects_builtin(FULLWIDTH_PAREN_DICTIONARY, &["（$1）", "fullwidth-paren"]);

    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(&format!(
        "[builtin]\nfullwidth-paren = false\n{FULLWIDTH_PAREN_DICTIONARY}"
    ));
    assert_eq!(
        fix_with_config(&sandbox, "いいね(笑)ね\n"),
        "いいね（笑）ね\n"
    );
}

#[test]
fn textlint_rule_pattern_rejects_a_replacement_opposite_to_a_fix_rule() {
    assert_check_rejects_builtin(
        "\"@textlint-rule/pattern\" = { patterns = [{ message = \"m\", pattern = \"Rust言語\", replace = \"Rust 言語\" }] }",
        &[
            "@textlint-rule/pattern",
            "Rust 言語",
            "space-between-ja-and-alnum",
        ],
    );
}

#[test]
fn prh_ignore_rules_skip_the_opposite_check() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "xdg/jafix/paren.yml",
        "rules:
  - expected: （$1）
    pattern: /\\((.+?)\\)/
  - expected: Cookie
    pattern: クッキー
",
    );
    sandbox.write("t.md", "クッキー(笑)\n");
    sandbox.write_xdg_config("[builtin]\nprh = { ruleContents = [\"imports: [paren.yml]\"] }\n");
    assert_eq!(sandbox.run(&["check", "t.md"]).status.code(), Some(2));

    sandbox.write_xdg_config(
        r#"[builtin.prh]
ruleContents = ["""
imports:
  - path: paren.yml
    ignoreRules:
      - expected: （$1）
"""]
"#,
    );
    assert_eq!(fix_with_config(&sandbox, "クッキー(笑)\n"), "Cookie(笑)\n");
}

#[test]
fn rules_table_is_rejected() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "照合する。\n");
    sandbox.write_xdg_config("[[rules]]\nid = \"ai-words\"\nmessage = \"m\"\nwords = [\"照合\"]\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("rules"), "{stderr}");
}

/// `preset-ja-spacing` の11ルール。presetの既定で無効な `ja-space-around-emphasis` と `ja-space-around-strong` を含む
const JA_SPACING_RULES: [&str; 11] = [
    "ja-nakaguro-or-halfwidth-space-between-katakana",
    "ja-no-space-around-parentheses",
    "ja-no-space-around-slash",
    "ja-no-space-between-full-width",
    "ja-space-between-half-and-full-width",
    "ja-space-after-exclamation",
    "ja-space-after-question",
    "ja-space-around-code",
    "ja-space-around-emphasis",
    "ja-space-around-link",
    "ja-space-around-strong",
];

/// 空白と字種の13ルール
const SPACING_RULES: [&str; 13] = [
    "ja-nakaguro-or-halfwidth-space-between-katakana",
    "ja-no-space-around-parentheses",
    "ja-no-space-around-slash",
    "ja-no-space-between-full-width",
    "ja-space-between-half-and-full-width",
    "ja-space-after-exclamation",
    "ja-space-after-question",
    "ja-space-around-code",
    "ja-space-around-emphasis",
    "ja-space-around-link",
    "ja-space-around-strong",
    "no-mixed-zenkaku-and-hankaku-alphabet",
    "no-kangxi-radicals",
];

/// `SPACING_RULES` のそれぞれに当たる段落を、同じ順に並べた文書。`i` 番目の段落は `2 * i + 1` 行目にある
const SPACING_RULES_INPUT: &str = "サーバー\u{3000}クライアント

日本語 「引用」 です

a / b

日本語 日本語

日本語 Rust

驚き！ これ

本当？ これ

日本語 `c` 日本語

日本語 *x* 日本語

日本語 [x](https://example.com) 日本語

日本語 **x** 日本語

ＡＢＣです

⼈です
";

/// `SPACING_RULES` の `rule` に当たる段落の行
fn spacing_rule_line(rule: &str) -> usize {
    2 * SPACING_RULES.iter().position(|r| *r == rule).unwrap() + 1
}

/// `stdout` のうち、`rule` の指摘と箇所の行の行番号
fn finding_lines(stdout: &str, rule: &str) -> Vec<usize> {
    findings_of(stdout, rule)
        .iter()
        .map(|line| line.split(':').nth(1).unwrap().parse().unwrap())
        .collect()
}

#[test]
fn spacing_rules_are_off_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", SPACING_RULES_INPUT);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in SPACING_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }

    // 同じ文書の各段落に、その段落のルールが当たることを、すべて有効にして確かめる。
    // `日本語 Rust` の空白は `space-between-ja-and-alnum` の箇所になるので、それを無効にする
    let enabled: String = SPACING_RULES
        .iter()
        .map(|rule| format!("{rule} = true\n"))
        .collect();
    sandbox.write_xdg_config(&format!(
        "[builtin]\nspace-between-ja-and-alnum = false\n{enabled}"
    ));
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in SPACING_RULES {
        assert!(
            finding_lines(found, rule).contains(&spacing_rule_line(rule)),
            "{rule}: {found}"
        );
    }
}

#[test]
fn preset_ja_spacing_enables_its_rules() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", SPACING_RULES_INPUT);
    sandbox.write_xdg_config("[builtin]\npreset-ja-spacing = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in SPACING_RULES {
        let enabled_by_preset = JA_SPACING_RULES.contains(&rule)
            && ![
                "ja-space-around-emphasis",
                "ja-space-around-strong",
                "ja-space-between-half-and-full-width",
            ]
            .contains(&rule);
        assert_eq!(
            finding_lines(found, rule).contains(&spacing_rule_line(rule)),
            enabled_by_preset,
            "{rule}: {found}"
        );
    }
    assert!(
        finding_lines(found, "space-between-ja-and-alnum")
            .contains(&spacing_rule_line("ja-space-between-half-and-full-width")),
        "{found}"
    );

    sandbox.write_xdg_config(
        "[builtin]\npreset-ja-spacing = true\nspace-between-ja-and-alnum = false\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert_eq!(
        finding_lines(found, "ja-space-between-half-and-full-width"),
        [spacing_rule_line("ja-space-between-half-and-full-width")],
        "{found}"
    );
}

#[test]
fn rule_key_overrides_preset_ja_spacing() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", SPACING_RULES_INPUT);
    sandbox
        .write_xdg_config("[builtin]\npreset-ja-spacing = true\nja-space-around-strong = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert!(
        finding_lines(found, "ja-space-around-strong")
            .contains(&spacing_rule_line("ja-space-around-strong")),
        "{found}"
    );
    assert!(
        findings_of(found, "ja-space-around-emphasis").is_empty(),
        "{found}"
    );

    sandbox.write_xdg_config(
        "[builtin]\npreset-ja-spacing = false\nja-no-space-around-slash = true\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in JA_SPACING_RULES {
        assert_eq!(
            findings_of(found, rule).is_empty(),
            rule != "ja-no-space-around-slash",
            "{rule}: {found}"
        );
    }
}

#[test]
fn spacing_rule_options_are_validated() {
    for (builtin, expected) in [
        (
            "ja-space-between-half-and-full-width = { spaces = \"never\" }",
            ["ja-space-between-half-and-full-width", "spaces"],
        ),
        (
            "ja-space-between-half-and-full-width = { exceptPunctuation = true }",
            ["ja-space-between-half-and-full-width", "exceptPunctuation"],
        ),
        (
            "ja-space-between-half-and-full-width = { space = true }",
            ["ja-space-between-half-and-full-width", "space"],
        ),
        (
            "ja-space-between-half-and-full-width = { space = \"sometimes\" }",
            ["ja-space-between-half-and-full-width", "space"],
        ),
        (
            "ja-space-between-half-and-full-width = { space = [\"letters\"] }",
            ["ja-space-between-half-and-full-width", "space"],
        ),
        (
            "ja-space-between-half-and-full-width = { lintStyledNode = \"yes\" }",
            ["ja-space-between-half-and-full-width", "lintStyledNode"],
        ),
        (
            "ja-space-between-half-and-full-width = { allows = \"Rust\" }",
            ["ja-space-between-half-and-full-width", "allows"],
        ),
        (
            "ja-space-between-half-and-full-width = { allows = [\"/(/\"] }",
            ["ja-space-between-half-and-full-width", "allows"],
        ),
        (
            "ja-space-around-code = { beforeSpace = true }",
            ["ja-space-around-code", "beforeSpace"],
        ),
        (
            "ja-space-around-link = { before = 1 }",
            ["ja-space-around-link", "before"],
        ),
        (
            "ja-space-around-strong = { after = \"true\" }",
            ["ja-space-around-strong", "after"],
        ),
        (
            "ja-space-around-emphasis = { before = [true] }",
            ["ja-space-around-emphasis", "before"],
        ),
        (
            "no-mixed-zenkaku-and-hankaku-alphabet = { prefer = \"zenkaku\" }",
            ["no-mixed-zenkaku-and-hankaku-alphabet", "prefer"],
        ),
        (
            "no-mixed-zenkaku-and-hankaku-alphabet = { prefer = 1 }",
            ["no-mixed-zenkaku-and-hankaku-alphabet", "prefer"],
        ),
        (
            "no-mixed-zenkaku-and-hankaku-alphabet = { preferred = \"全角\" }",
            ["no-mixed-zenkaku-and-hankaku-alphabet", "preferred"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

#[test]
fn half_and_full_width_space_is_opposite_to_space_between_ja_and_alnum() {
    for space in ["\"always\"", "[\"numbers\"]"] {
        assert_check_rejects_builtin(
            &format!("ja-space-between-half-and-full-width = {{ space = {space} }}"),
            &[
                "ja-space-between-half-and-full-width",
                "space",
                "space-between-ja-and-alnum",
            ],
        );
    }

    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config(
        "[builtin]\nja-space-between-half-and-full-width = { space = \"always\" }\nspace-between-ja-and-alnum = false\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        findings_of(stdout(&output), "ja-space-between-half-and-full-width"),
        [
            "t.md:1:3: 全角文字と半角文字の間に半角空白を入れる [ja-space-between-half-and-full-width]"
        ]
    );

    sandbox.write_xdg_config(
        "[builtin]\nja-space-between-half-and-full-width = { space = [\"punctuation\"] }\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    assert_ne!(
        output.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn space_around_true_is_not_opposite_to_fix_rules() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語`code`日本語\n");
    let enabled: String = [
        "ja-space-around-code",
        "ja-space-around-link",
        "ja-space-around-strong",
        "ja-space-around-emphasis",
    ]
    .iter()
    .map(|rule| format!("{rule} = {{ before = true, after = true }}\n"))
    .collect();
    sandbox.write_xdg_config(&format!("[builtin]\n{enabled}"));
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        findings_of(stdout(&output), "ja-space-around-code").len(),
        2,
        "{}",
        stdout(&output)
    );
}

#[test]
fn overlapping_fix_rules_fix_once() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語 Rust と語 （補足） です\n");
    sandbox.write_xdg_config("[builtin]\npreset-ja-spacing = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert_eq!(
        findings_of(found, "space-between-ja-and-alnum"),
        [
            "t.md:1:4: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]",
            "t.md:1:9: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]",
        ]
    );
    assert_eq!(
        findings_of(found, "space-between-ja-and-paren"),
        [
            "t.md:1:12: 日本語と括弧の間の空白を削除する [space-between-ja-and-paren]",
            "t.md:1:17: 日本語と括弧の間の空白を削除する [space-between-ja-and-paren]",
        ]
    );
    for rule in [
        "ja-space-between-half-and-full-width",
        "ja-no-space-around-parentheses",
    ] {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
    assert_eq!(
        fix_with_config(&sandbox, "日本語 Rust と語 （補足） です\n"),
        "日本語Rustと語(補足)です\n"
    );
}

#[test]
fn fix_rules_repeat_until_the_content_is_stable() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nno-mixed-zenkaku-and-hankaku-alphabet = true\n");
    assert_eq!(
        fix_with_config(&sandbox, "日本語 Ｒｕｓｔ です\n"),
        "日本語Rustです\n"
    );

    let sandbox = Sandbox::new();
    assert_eq!(
        fix_with_config(&sandbox, "日本語 \u{200b}Rust\n"),
        "日本語Rust\n"
    );
}

/// `expected` が `expected` で、`pattern` が `pattern` のルールを1つ持つ `prh` の辞書
fn prh_rule(expected: &str, pattern: &str) -> String {
    format!(
        "[builtin.prh]\nruleContents = ['''\nrules:\n  - expected: {expected}\n    pattern: {pattern}\n''']\n"
    )
}

#[test]
fn prh_rejects_a_replacement_opposite_to_a_ported_fix_rule() {
    let fullwidth_alphabets = prh_rule("ＡＢＣ", "エービーシー");
    assert_check_rejects_builtin(
        &format!("no-mixed-zenkaku-and-hankaku-alphabet = true\n{fullwidth_alphabets}"),
        &["ＡＢＣ", "no-mixed-zenkaku-and-hankaku-alphabet"],
    );
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(&format!("[builtin]\n{fullwidth_alphabets}"));
    assert_eq!(
        fix_with_config(&sandbox, "エービーシーを使う。\n"),
        "ＡＢＣを使う。\n"
    );

    assert_check_rejects_builtin(
        &format!(
            "preset-ja-spacing = true\n{}",
            prh_rule("日本語 Rust", "日本語Rust")
        ),
        &[
            "space-between-ja-and-alnum",
            "ja-space-between-half-and-full-width",
        ],
    );
}

const JTF_PARENTHESES: &str = "3.3.かっこ類と隣接する文字の間のスペースの有無";

#[test]
fn preset_jtf_style_alone_conflicts_with_fullwidth_paren() {
    assert_check_rejects_builtin(
        "preset-jtf-style = true",
        &[
            "4.3.1.丸かっこ（）",
            "fullwidth-paren",
            "`fullwidth-paren = false` か `\"4.3.1.丸かっこ（）\" = false` にする",
        ],
    );
    assert_check_rejects_builtin(
        "ja-space-between-half-and-full-width = { space = \"always\" }",
        &[
            "`space-between-ja-and-alnum = false` か `ja-space-between-half-and-full-width = false` にする",
        ],
    );
}

const JTF_STYLE_RULES: [&str; 37] = [
    "1.1.1.本文",
    "1.1.2.見出し",
    "1.1.3.箇条書き",
    "1.1.5.図表のキャプション",
    "1.2.1.句点(。)と読点(、)",
    "1.2.2.ピリオド(.)とカンマ(,)",
    "2.1.2.漢字",
    "2.1.5.カタカナ",
    "2.1.6.カタカナの長音",
    "2.1.8.算用数字",
    "2.1.9.アルファベット",
    "2.1.10.算用数字の位取りの表記",
    "2.2.1.ひらがなと漢字の使い分け",
    "2.2.2.算用数字と漢数字の使い分け",
    "2.2.3.一部の助数詞の表記",
    "3.1.1.全角文字と半角文字の間",
    "3.1.2.全角文字どうし",
    "3.2.カタカナ語間のスペースの有無",
    JTF_PARENTHESES,
    "4.1.1.句点(。)",
    "4.1.3.ピリオド(.)、カンマ(,)",
    "4.2.1.感嘆符(！)",
    "4.2.2.疑問符(？)",
    "4.2.4.中黒(・)",
    "4.2.5.波線(〜)",
    "4.2.6.ハイフン(-)",
    "4.2.7.コロン(：)",
    "4.2.8.セミコロン(;)",
    "4.2.9.ダッシュ(-)",
    "4.3.1.丸かっこ（）",
    "4.3.2.大かっこ［］",
    "4.3.3.かぎかっこ「」",
    "4.3.4.二重かぎかっこ『』",
    "4.3.5.二重引用符",
    "4.3.6.中かっこ{ }",
    "4.3.7.山かっこ<>",
    "4.3.8.一重引用符",
];

/// `JTF_STYLE_RULES` の各ルールに当たる段落を、同じ順に並べた文書。`4.3.8.一重引用符` は何も指摘しない
const JTF_STYLE_RULES_INPUT: &str = "本文は敬体です。本文は常体である。

# 見出し。

- 項目です。
- 項目である

![図は敬体です。図は常体である。](a.png)

日本語,です

これは，テスト

彼は躓いた

ｻｰﾊﾞｰとアジヤ

サーバ

全角の１２３

全角のＡＢＣ

値は0,5です

予め決める

三回実行する

3ヶ月

日本語 Rust

日本語 日本語

カタカナ\u{3000}カタカナ

日本語 「引用」 です

「これは。」と言う

日本語.

すごい!

本当?

ア･イ

1~10

日本語-日本語

例:これ

日本語;です

日本語—です

日本語(補足)です

日本語[補足]です

「閉じない

『閉じない

\"閉じない

{閉じない

<閉じない

'一重'
";

/// `preset-jtf-style = true` でも有効にならない、presetの既定で無効な4ルール
const JTF_STYLE_PRESET_OFF_RULES: [&str; 4] = [
    "2.1.2.漢字",
    "2.1.5.カタカナ",
    "2.1.6.カタカナの長音",
    "2.2.1.ひらがなと漢字の使い分け",
];

#[test]
fn jtf_style_rules_are_off_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", JTF_STYLE_RULES_INPUT);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in JTF_STYLE_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
}

#[test]
fn preset_jtf_style_enables_its_rules() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", JTF_STYLE_RULES_INPUT);
    let preset = "[builtin]\npreset-jtf-style = true\nfullwidth-paren = false\n";
    sandbox.write_xdg_config(preset);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    // 同じ範囲に当たる `space-between-ja-and-alnum`・`no-hankaku-kana` の箇所が、重なりの規則で残る
    let covered_by_fix_rules = ["3.1.1.全角文字と半角文字の間", "4.2.4.中黒(・)"];
    for rule in JTF_STYLE_RULES {
        let expected = !JTF_STYLE_PRESET_OFF_RULES.contains(&rule)
            && rule != "4.3.8.一重引用符"
            && !covered_by_fix_rules.contains(&rule);
        assert_eq!(
            !findings_of(found, rule).is_empty(),
            expected,
            "{rule}: {found}"
        );
    }

    sandbox.write_xdg_config(&format!(
        "{preset}space-between-ja-and-alnum = false\nno-hankaku-kana = false\n"
    ));
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in covered_by_fix_rules {
        assert!(!findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
    for rule in JTF_STYLE_PRESET_OFF_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
}

#[test]
fn rule_key_overrides_preset_jtf_style() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", JTF_STYLE_RULES_INPUT);
    let preset = "preset-jtf-style = true\nfullwidth-paren = false\n";
    sandbox.write_xdg_config(&format!("[builtin]\n{preset}"));
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert!(findings_of(found, "2.1.2.漢字").is_empty(), "{found}");
    assert!(
        !findings_of(found, "4.2.7.コロン(：)").is_empty(),
        "{found}"
    );

    sandbox.write_xdg_config(&format!("[builtin]\n{preset}\"2.1.2.漢字\" = true\n"));
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert!(!findings_of(found, "2.1.2.漢字").is_empty(), "{found}");
    assert!(
        !findings_of(found, "4.2.7.コロン(：)").is_empty(),
        "{found}"
    );

    sandbox.write_xdg_config("[builtin]\npreset-jtf-style = false\n\"4.2.7.コロン(：)\" = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in JTF_STYLE_RULES {
        assert_eq!(
            findings_of(found, rule).is_empty(),
            rule != "4.2.7.コロン(：)",
            "{rule}: {found}"
        );
    }
}

#[test]
fn jtf_style_rule_options_are_validated() {
    for (builtin, expected) in [
        (
            "\"4.2.7.コロン(：)\" = { allowOutsideHalfParentheses = true }".to_owned(),
            vec!["4.2.7.コロン(：)", "allowOutsideHalfParentheses"],
        ),
        (
            format!("\"{JTF_PARENTHESES}\" = {{ allowOutside = true }}"),
            vec![JTF_PARENTHESES, "allowOutside"],
        ),
        (
            format!("\"{JTF_PARENTHESES}\" = {{ requireOutsideHalfParentheses = \"yes\" }}"),
            vec![JTF_PARENTHESES, "requireOutsideHalfParentheses"],
        ),
        (
            format!(
                "\"{JTF_PARENTHESES}\" = {{ allowOutsideHalfParentheses = false, requireOutsideHalfParentheses = true }}\nspace-between-ja-and-paren = false"
            ),
            vec![
                JTF_PARENTHESES,
                "allowOutsideHalfParentheses",
                "requireOutsideHalfParentheses",
            ],
        ),
    ] {
        assert_check_rejects_builtin(&builtin, &expected);
    }
}

#[test]
fn jtf_style_opposite_pairs_are_rejected() {
    let parentheses = format!("\"{JTF_PARENTHESES}\" = {{ requireOutsideHalfParentheses = true }}");
    let half_and_full_width = "ja-space-between-half-and-full-width = { space = \"always\" }";
    let alphabet = "no-mixed-zenkaku-and-hankaku-alphabet = { prefer = \"全角\" }";
    // 組の2つのルールの設定、組に関わらない共通の設定、理由に出る2つのルールのID
    for (rule, fix_rule, common, ids) in [
        (
            parentheses.as_str(),
            "space-between-ja-and-paren = true",
            "",
            [JTF_PARENTHESES, "space-between-ja-and-paren"],
        ),
        (
            half_and_full_width,
            "\"3.1.1.全角文字と半角文字の間\" = true",
            "space-between-ja-and-alnum = false",
            [
                "ja-space-between-half-and-full-width",
                "3.1.1.全角文字と半角文字の間",
            ],
        ),
        (
            alphabet,
            "\"2.1.9.アルファベット\" = true",
            "",
            [
                "no-mixed-zenkaku-and-hankaku-alphabet",
                "2.1.9.アルファベット",
            ],
        ),
    ] {
        assert_check_rejects_builtin(&format!("{rule}\n{fix_rule}\n{common}"), &ids);
        let off = |line: &str| format!("{} = false", line.split_once(" = ").unwrap().0);
        for loadable in [
            format!("{}\n{fix_rule}\n{common}", off(rule)),
            format!("{rule}\n{}\n{common}", off(fix_rule)),
        ] {
            let sandbox = Sandbox::new();
            sandbox.write("t.md", "日本語Rust\n");
            sandbox.write_xdg_config(&format!("[builtin]\n{loadable}\n"));
            let output = sandbox.run(&["check", "t.md"]);
            assert_ne!(
                output.status.code(),
                Some(2),
                "{loadable}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    assert_check_rejects_builtin(&parentheses, &["requireOutsideHalfParentheses"]);
}

#[test]
fn jtf_style_fix_rules_repeat_with_other_fix_rules() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\n\"2.1.9.アルファベット\" = true\n");
    assert_eq!(
        fix_with_config(&sandbox, "日本語 ＡＢＣ です\n"),
        "日本語ABCです\n"
    );
}

#[test]
fn overlapping_jtf_style_fix_rules_fix_once() {
    let sandbox = Sandbox::new();
    let config =
        "[builtin]\npreset-jtf-style = true\nfullwidth-paren = false\n\"2.1.5.カタカナ\" = true\n";
    sandbox.write_xdg_config(config);
    let input = "日本語 Rust、ｻｰﾊﾞｰ、項目 （補足）\n";
    sandbox.write("t.md", input);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert_eq!(
        findings_of(found, "space-between-ja-and-alnum"),
        ["t.md:1:4: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]"],
        "{found}"
    );
    assert_eq!(
        findings_of(found, "no-hankaku-kana"),
        ["t.md:1:10: 半角カナを全角にする [no-hankaku-kana]"],
        "{found}"
    );
    assert_eq!(
        findings_of(found, "space-between-ja-and-paren"),
        ["t.md:1:18: 日本語と括弧の間の空白を削除する [space-between-ja-and-paren]"],
        "{found}"
    );
    for rule in [
        "3.1.1.全角文字と半角文字の間",
        "2.1.5.カタカナ",
        JTF_PARENTHESES,
    ] {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
    assert_eq!(
        fix_with_config(&sandbox, input),
        "日本語Rust、サーバー、項目（補足）\n"
    );

    sandbox.write_xdg_config(&format!("{config}no-hankaku-kana = false\n"));
    sandbox.write("t.md", input);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert_eq!(
        findings_of(found, "2.1.5.カタカナ"),
        ["t.md:1:10: 半角カナ「ｻｰﾊﾞｰ」を全角にする [2.1.5.カタカナ]"],
        "{found}"
    );
}

#[test]
fn prh_rejects_a_replacement_opposite_to_a_jtf_style_fix_rule() {
    let fullwidth_digits = prh_rule("１２３", "いちにさん");
    assert_check_rejects_builtin(
        &format!("\"2.1.8.算用数字\" = true\n{fullwidth_digits}"),
        &["１２３", "`\"2.1.8.算用数字\" = false`"],
    );
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(&format!("[builtin]\n{fullwidth_digits}"));
    assert_eq!(fix_with_config(&sandbox, "いちにさん\n"), "１２３\n");
}

/// `HIRAGANA_AND_SYNONYMS_RULES` のそれぞれに当たる行。最後の2行は、同じ同義語の集まりの別の表記である
const HIRAGANA_AND_SYNONYMS_RULES_INPUT: &str = "お願い致します。

見た事がある。

既に終わった。

彼処に行く。

このカレーは辛さそうだ。

サーバを使う。

サーバーを使う。
";

#[test]
fn hiragana_and_synonyms_rules_are_off_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", HIRAGANA_AND_SYNONYMS_RULES_INPUT);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in HIRAGANA_AND_SYNONYMS_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }

    // 同じ文書に7つのそれぞれが当たることを、有効にして確かめる
    sandbox.write_xdg_config(&hiragana_and_synonyms_rules_config());
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in HIRAGANA_AND_SYNONYMS_RULES {
        assert!(!findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
}

#[test]
fn hiragana_and_synonyms_rules_do_not_fix() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(&hiragana_and_synonyms_rules_config());
    assert_eq!(
        fix_with_config(&sandbox, HIRAGANA_AND_SYNONYMS_RULES_INPUT),
        HIRAGANA_AND_SYNONYMS_RULES_INPUT
    );
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in HIRAGANA_AND_SYNONYMS_RULES {
        assert!(!findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
}

#[test]
fn hiragana_and_synonyms_rule_options_are_validated() {
    for (builtin, expected) in [
        (
            "ja-hiragana-hojodoushi = { dictPath = \"hojo.yml\" }",
            ["ja-hiragana-hojodoushi", "dictPath"],
        ),
        (
            "ja-hiragana-hojodoushi = { rulePath = 1 }",
            ["ja-hiragana-hojodoushi", "rulePath"],
        ),
        (
            "ja-hiragana-fukushi = { dictPath = \"fukushi.yml\" }",
            ["ja-hiragana-fukushi", "dictPath"],
        ),
        (
            "ja-hiragana-fukushi = { rulePath = 1 }",
            ["ja-hiragana-fukushi", "rulePath"],
        ),
        (
            "ja-keishikimeishi = { detection_hou = false }",
            ["ja-keishikimeishi", "detection_hou"],
        ),
        (
            "ja-keishikimeishi = { detection_ue = \"false\" }",
            ["ja-keishikimeishi", "detection_ue"],
        ),
        (
            "\"@textlint-ja/no-synonyms\" = { allow = [] }",
            ["@textlint-ja/no-synonyms", "allow"],
        ),
        (
            "\"@textlint-ja/no-synonyms\" = { allows = [1] }",
            ["@textlint-ja/no-synonyms", "allows"],
        ),
        (
            "\"@textlint-ja/no-synonyms\" = { preferWords = [true] }",
            ["@textlint-ja/no-synonyms", "preferWords"],
        ),
        (
            "\"@textlint-ja/no-synonyms\" = { allowLexeme = \"true\" }",
            ["@textlint-ja/no-synonyms", "allowLexeme"],
        ),
        (
            "ja-hiragana-keishikimeishi = { detection_ue = false }",
            ["ja-hiragana-keishikimeishi", "detection_ue"],
        ),
        (
            "ja-hiragana-daimeishi = { rulePath = \"daimeishi.yml\" }",
            ["ja-hiragana-daimeishi", "rulePath"],
        ),
        (
            "\"@textlint-ja/no-insert-dropping-sa\" = { strict = true }",
            ["@textlint-ja/no-insert-dropping-sa", "strict"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

#[test]
fn hiragana_rule_path_replaces_the_builtin_dictionary() {
    let dictionary = "message: 漢字で書かない語
dict:
  - expected: さいこう
    tokens:
      - basic_form: 再考
";
    for rule in ["ja-hiragana-hojodoushi", "ja-hiragana-fukushi"] {
        let sandbox = Sandbox::new();
        // `致し` は補助動詞の、`既に` は副詞の組み込みの辞書の語である
        sandbox.write("t.md", "再考する。お願い致します。既に終わった。\n");
        sandbox.write("xdg/jafix/hiragana.yml", dictionary);
        sandbox.write_xdg_config(&format!(
            "[builtin]\n{rule} = {{ rulePath = \"hiragana.yml\" }}\n"
        ));
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(
            findings_of(stdout(&output), rule),
            [format!(
                "t.md:1:1: 漢字で書かない語: 「再考」を「さいこう」にする [{rule}]"
            )],
            "{rule}"
        );
    }
}

#[test]
fn hiragana_rule_path_rejects_invalid_dictionary() {
    let token = |key: &str| {
        format!("message: m\ndict:\n  - expected: いたし\n    tokens:\n      - {key}: 致し\n")
    };
    // `rulePath` の値と、置く辞書 (無ければ置かない) と、理由
    for (rule_path, dictionary, reason) in [
        ("", None, "`rulePath` は空でない文字列で書く"),
        ("hojo.yml", None, "読めない"),
        (
            "hojo.yml",
            Some("message: [\n".to_owned()),
            "YAMLとして読めない",
        ),
        (
            "hojo.yml",
            Some(token("word_id")),
            "`word_id` はjafixのトークンの項目ではない",
        ),
        (
            "hojo.yml",
            Some(token("part_of_speech")),
            "`part_of_speech` はjafixのトークンの項目ではない",
        ),
    ] {
        let sandbox = Sandbox::new();
        sandbox.write("t.md", "お願い致します。\n");
        if let Some(dictionary) = &dictionary {
            sandbox.write("xdg/jafix/hojo.yml", dictionary);
        }
        sandbox.write_xdg_config(&format!(
            "[builtin]\nja-hiragana-hojodoushi = {{ rulePath = \"{rule_path}\" }}\n"
        ));
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(output.status.code(), Some(2), "{dictionary:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("rulePath") && stderr.contains(rule_path) && stderr.contains(reason),
            "{dictionary:?}: {stderr}"
        );
    }
}

#[test]
fn both_keishikimeishi_rules() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "見る事が出来る。\n");
    sandbox.write_xdg_config(
        "[builtin]\nja-hiragana-keishikimeishi = true\nja-keishikimeishi = true\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in ["ja-hiragana-keishikimeishi", "ja-keishikimeishi"] {
        assert_eq!(
            findings_of(found, rule),
            [format!(
                "t.md:1:3: 形式名詞「事」はひらがなの「こと」にする [{rule}]"
            )],
            "{found}"
        );
    }
}

#[test]
fn hiragana_rules_and_2_2_1_report_both() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "提出して下さい。\n");
    sandbox.write_xdg_config(
        "[builtin]\n\"2.2.1.ひらがなと漢字の使い分け\" = true\nja-hiragana-hojodoushi = true\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    let found = stdout(&output);
    for rule in ["2.2.1.ひらがなと漢字の使い分け", "ja-hiragana-hojodoushi"] {
        let findings = findings_of(found, rule);
        assert_eq!(findings.len(), 1, "{rule}: {found}");
        assert!(findings[0].starts_with("t.md:1:5: "), "{rule}: {found}");
    }
}

/// 言語によらない3つの指摘ルール。どれも既定で無効である
const GENERAL_RULES: [&str; 3] = ["no-todo", "date-weekday-mismatch", "no-dead-link"];

/// 1行目・3行目・5行目が `GENERAL_RULES` の3つのそれぞれに当たり、`<!-- textlint-disable -->` の後の9行目が修正ルールと
/// 指摘ルールに当たる文書
const GENERAL_RULES_INPUT: &str = "TODO: 手順を書く。

2024年4月23日(月)に会う。

[設定](missing.md)を見る。

<!-- textlint-disable -->

日本語 Rust の（補足）を読む！
";

#[test]
fn general_rules_and_filters_are_off_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", GENERAL_RULES_INPUT);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in GENERAL_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }
    // `[filters]` を書かなければ `comments` は無効なので、コメントの後の箇所と指摘も出る
    let after_comment: Vec<&str> = found
        .lines()
        .filter(|line| line.starts_with("t.md:9:"))
        .collect();
    assert_eq!(
        after_comment,
        [
            "t.md:9:4: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]",
            "t.md:9:9: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]",
            "t.md:9:11: 全角括弧を半角にする [fullwidth-paren]",
            "t.md:9:14: 全角括弧を半角にする [fullwidth-paren]",
            "t.md:9:18: 「！」を使わない [no-exclamation-question-mark]",
        ],
        "{found}"
    );

    // 同じ文書に3つのそれぞれが当たることを、有効にして確かめる
    let enabled: String = GENERAL_RULES
        .iter()
        .map(|rule| format!("{rule} = true\n"))
        .collect();
    sandbox.write_xdg_config(&format!("[builtin]\n{enabled}"));
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for (rule, line) in GENERAL_RULES.iter().zip([1, 3, 5]) {
        let findings = findings_of(found, rule);
        assert_eq!(findings.len(), 1, "{rule}: {found}");
        assert!(
            findings[0].starts_with(&format!("t.md:{line}:")),
            "{rule}: {found}"
        );
    }
}

#[test]
fn general_rule_options_are_validated() {
    for (builtin, expected) in [
        ("no-todo = { allow = [] }", ["no-todo", "allow"]),
        (
            "date-weekday-mismatch = { locale = \"ja\" }",
            ["date-weekday-mismatch", "locale"],
        ),
        (
            "date-weekday-mismatch = { lang = \"es\" }",
            ["date-weekday-mismatch", "lang"],
        ),
        (
            "date-weekday-mismatch = { useCurrentYearIfMissing = \"true\" }",
            ["date-weekday-mismatch", "useCurrentYearIfMissing"],
        ),
        (
            "date-weekday-mismatch = { currentYear = 0 }",
            ["date-weekday-mismatch", "currentYear"],
        ),
        (
            "no-dead-link = { dotInIgnore = true }",
            ["no-dead-link", "dotInIgnore"],
        ),
        (
            "no-dead-link = { interval = 1 }",
            ["no-dead-link", "interval"],
        ),
        (
            "no-dead-link = { intervalCap = 1 }",
            ["no-dead-link", "intervalCap"],
        ),
        (
            "no-dead-link = { linkMaxAge = 1 }",
            ["no-dead-link", "linkMaxAge"],
        ),
        (
            "no-dead-link = { checkRelative = 1 }",
            ["no-dead-link", "checkRelative"],
        ),
        (
            "no-dead-link = { baseURI = \"\" }",
            ["no-dead-link", "baseURI"],
        ),
        (
            "no-dead-link = { ignore = [1] }",
            ["no-dead-link", "ignore"],
        ),
        (
            "no-dead-link = { ignore = [\"[\"] }",
            ["no-dead-link", "ignore"],
        ),
        (
            "no-dead-link = { preferGET = [\"example.com\"] }",
            ["no-dead-link", "preferGET"],
        ),
        ("no-dead-link = { retry = -1 }", ["no-dead-link", "retry"]),
        (
            "no-dead-link = { concurrency = 0 }",
            ["no-dead-link", "concurrency"],
        ),
        (
            "no-dead-link = { userAgent = \"\" }",
            ["no-dead-link", "userAgent"],
        ),
        (
            "no-dead-link = { maxRetryTime = 1.5 }",
            ["no-dead-link", "maxRetryTime"],
        ),
        (
            "no-dead-link = { maxRetryAfterTime = \"10\" }",
            ["no-dead-link", "maxRetryAfterTime"],
        ),
        (
            "no-dead-link = { httpOnly = \"false\" }",
            ["no-dead-link", "httpOnly"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

/// `[filters]` に `filters` を書いた設定で `jafix check` がexit 2で終わり、標準エラーが `expected` をすべて含むことを確かめる
fn assert_check_rejects_filters(filters: &str, expected: &[&str]) {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rust\n");
    sandbox.write_xdg_config(&format!("[filters]\n{filters}\n"));
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2), "{filters}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    for needle in expected {
        assert!(stderr.contains(needle), "{filters}: {stderr}");
    }
}

#[test]
fn filter_options_are_validated() {
    for (filters, expected) in [
        ("no-such-filter = true", &["filters", "no-such-filter"][..]),
        ("comments = 1", &["filters", "comments"]),
        (
            "comments = { disable = \"x\" }",
            &["filters", "comments", "disable"],
        ),
        (
            "comments = { disablingComment = \"\" }",
            &["filters", "comments", "disablingComment"],
        ),
        (
            "allowlist = { allow = [\"/(/\"] }",
            &["filters", "allowlist", "allow", "/(/"],
        ),
        (
            "allowlist = { allow = [\"\"] }",
            &["filters", "allowlist", "allow"],
        ),
        (
            "allowlist = { allowlistConfigPaths = [\"\"] }",
            &["filters", "allowlist", "allowlistConfigPaths"],
        ),
    ] {
        assert_check_rejects_filters(filters, expected);
    }
}

#[test]
fn allowlist_reads_config_paths() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "TODO: a\n\nTODO: b\n\nTODO: c\n");
    sandbox.write("xdg/jafix/a.json", "[\"TODO: a\"]\n");
    sandbox.write("xdg/jafix/b.yml", "- \"TODO: b\"\n");
    sandbox.write_xdg_config(
        "[builtin]\nno-todo = true\n\n[filters]\nallowlist = { allowlistConfigPaths = [\"a.json\", \"b.yml\"] }\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "no-todo"),
        ["t.md:5:1: TODOが残っている: 「TODO: c」 [no-todo]"]
    );
}

#[test]
fn allowlist_rejects_invalid_config_paths() {
    for (name, content) in [
        ("missing.json", None),
        ("allow.txt", Some("TODO: a\n")),
        ("table.json", Some("{\"TODO\": \"a\"}\n")),
    ] {
        let sandbox = Sandbox::new();
        if let Some(content) = content {
            sandbox.write(&format!("xdg/jafix/{name}"), content);
        }
        sandbox.write("t.md", "TODO: a\n");
        sandbox.write_xdg_config(&format!(
            "[builtin]\nno-todo = true\n\n[filters]\nallowlist = {{ allowlistConfigPaths = [\"{name}\"] }}\n"
        ));
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(output.status.code(), Some(2), "{name}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        let path = sandbox.path(&format!("xdg/jafix/{name}"));
        assert!(
            stderr.contains(&path.display().to_string()),
            "{name}: {stderr}"
        );
    }
}

#[test]
fn comments_filter_keeps_fixes_inside_disabled_range() {
    let sandbox = Sandbox::new();
    let path = sandbox.write(
        "t.md",
        "日本語 Rust の（補足）\n\n<!-- textlint-disable -->\n\n日本語 Rust の（補足）\n\n<!-- textlint-enable -->\n\n日本語 Rust の（補足）\n",
    );
    sandbox.write_xdg_config("[filters]\ncomments = true\n");
    let output = sandbox.run(&["fix", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        read(&path),
        "日本語Rustの(補足)\n\n<!-- textlint-disable -->\n\n日本語 Rust の（補足）\n\n<!-- textlint-enable -->\n\n日本語Rustの(補足)\n"
    );
}

#[test]
fn filtered_fix_findings_do_not_win_overlaps() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\npreset-ja-spacing = true\n\n[filters]\ncomments = true\n");
    // 無効化が無ければ、同じ空白の箇所は `RuleId` で前にある `space-between-ja-and-alnum` が残る
    sandbox.write("t.md", "日本語 Rust\n");
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert_eq!(
        findings_of(found, "space-between-ja-and-alnum"),
        ["t.md:1:4: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]"],
        "{found}"
    );
    assert!(
        findings_of(found, "ja-space-between-half-and-full-width").is_empty(),
        "{found}"
    );

    sandbox.write(
        "t.md",
        "<!-- textlint-disable space-between-ja-and-alnum -->\n\n日本語 Rust\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    assert!(
        findings_of(found, "space-between-ja-and-alnum").is_empty(),
        "{found}"
    );
    assert_eq!(
        findings_of(found, "ja-space-between-half-and-full-width"),
        ["t.md:3:4: 全角文字と半角文字の間の空白を削除する [ja-space-between-half-and-full-width]"],
        "{found}"
    );
}

#[test]
fn comments_filter_accepts_textlint_rule_ids() {
    let long_sentence = format!("{}。", "あ".repeat(101));
    for (id, rule, body) in [
        (
            "ja-technical-writing/sentence-length",
            "sentence-length",
            long_sentence.as_str(),
        ),
        ("sentence-length", "sentence-length", long_sentence.as_str()),
        ("textlint-rule-no-todo", "no-todo", "TODO: 手順を書く"),
    ] {
        let sandbox = Sandbox::new();
        sandbox.write_xdg_config(
            "[builtin]\nsentence-length = true\nno-todo = true\n\n[filters]\ncomments = true\n",
        );
        sandbox.write("t.md", &format!("{body}\n"));
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(findings_of(stdout(&output), rule).len(), 1, "{id}");

        sandbox.write(
            "t.md",
            &format!("<!-- textlint-disable {id} -->\n\n{body}\n"),
        );
        let output = sandbox.run(&["check", "t.md"]);
        assert!(
            findings_of(stdout(&output), rule).is_empty(),
            "{id}: {}",
            stdout(&output)
        );
    }
}

#[test]
fn no_todo_reports_a_task_box_followed_only_by_trailing_spaces() {
    for content in ["- [ ] \n", "- [ ]\t\n", "- 項目 [ ] \n"] {
        let sandbox = Sandbox::new();
        sandbox.write("t.md", content);
        sandbox.write_xdg_config("[builtin]\nno-todo = true\n");
        let output = sandbox.run(&["check", "t.md"]);
        let column = content.find('[').map(|i| content[..i].chars().count() + 1);
        assert_eq!(
            findings_of(stdout(&output), "no-todo"),
            [format!(
                "t.md:1:{}: 未完了のタスクが残っている [no-todo]",
                column.unwrap()
            )],
            "{content:?}"
        );
    }
}

/// 英語の4つのルール
const ENGLISH_RULES: [&str; 4] = [
    "terminology",
    "spelling",
    "common-misspellings",
    "write-good",
];

/// `terminology` (`github`)、`spelling` (`recieve`)、`common-misspellings` (`teh`)、`write-good` (`clearly`) の
/// それぞれに当たる英語の行
const ENGLISH_RULES_INPUT: &str = "Use github here.

We recieve it.

I saw teh cat.

It is clearly wrong.
";

#[test]
fn english_rules_are_off_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", ENGLISH_RULES_INPUT);
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for rule in ENGLISH_RULES {
        assert!(findings_of(found, rule).is_empty(), "{rule}: {found}");
    }

    // 同じ文書に当たることを、有効にして確かめる
    sandbox.write_xdg_config(
        "[builtin]\nterminology = true\nspelling = true\ncommon-misspellings = true\nwrite-good = true\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    let found = stdout(&output);
    for (rule, line) in [
        ("terminology", 1),
        ("spelling", 3),
        ("common-misspellings", 5),
        ("write-good", 7),
    ] {
        assert!(
            findings_of(found, rule)
                .iter()
                .any(|finding| finding.starts_with(&format!("t.md:{line}:"))),
            "{rule}: {found}"
        );
    }
}

#[test]
fn english_rule_options_are_validated() {
    for (builtin, expected) in [
        ("terminology = { term = [] }", ["terminology", "term"]),
        (
            "terminology = { defaultTerms = \"false\" }",
            ["terminology", "defaultTerms"],
        ),
        (
            "terminology = { exclude = \"GitHub\" }",
            ["terminology", "exclude"],
        ),
        ("terminology = { terms = 1 }", ["terminology", "terms"]),
        (
            "spelling = { dictionary = \"en\" }",
            ["spelling", "dictionary"],
        ),
        (
            "spelling = { suggestCorrections = 1 }",
            ["spelling", "suggestCorrections"],
        ),
        (
            "spelling = { skipPatterns = \"a\" }",
            ["spelling", "skipPatterns"],
        ),
        (
            "common-misspellings = { ignores = [] }",
            ["common-misspellings", "ignores"],
        ),
        (
            "common-misspellings = { ignore = \"teh\" }",
            ["common-misspellings", "ignore"],
        ),
        ("write-good = { checks = [] }", ["write-good", "checks"]),
        (
            "write-good = { weasel = \"false\" }",
            ["write-good", "weasel"],
        ),
        (
            "write-good = { whitelist = [1] }",
            ["write-good", "whitelist"],
        ),
        (
            "write-good = { whitelist = [\"\"] }",
            ["write-good", "whitelist"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

#[test]
fn terminology_rejects_invalid_terms() {
    for (builtin, expected) in [
        ("terminology = { terms = [1] }", ["terminology", "terms"]),
        (
            "terminology = { terms = [[\"a\", \"b\", \"c\"]] }",
            ["terminology", "terms"],
        ),
        (
            "terminology = { terms = [\"(\"] }",
            ["terminology", "terms"],
        ),
        (
            "terminology = { terms = [[\"a\", \"b\"], [\"[\", \"c\"]] }",
            ["terminology", "terms"],
        ),
        (
            "terminology = { skip = [\"Code\"] }",
            ["terminology", "skip"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

#[test]
fn terminology_reads_terms_file() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "xdg/jafix/terms.jsonc",
        r#"// 利用者の表記
[
  "Jafix", /* 文字列の項目 */
  ["front[- ]end(\\w*)", "frontend$1"] // 配列の項目
]
"#,
    );
    sandbox.write_xdg_config("[builtin]\nterminology = { terms = \"terms.jsonc\" }\n");
    assert_eq!(
        fix_with_config(&sandbox, "Use jafix for front-ends.\n"),
        "Use Jafix for frontends.\n"
    );

    for (name, content) in [
        ("missing.jsonc", None),
        ("broken.jsonc", Some("[\"Jafix\",]")),
        ("table.json", Some("{\"terms\": [\"Jafix\"]}")),
    ] {
        let sandbox = Sandbox::new();
        if let Some(content) = content {
            sandbox.write(&format!("xdg/jafix/{name}"), content);
        }
        let path = sandbox.path(&format!("xdg/jafix/{name}"));
        assert_check_rejects_builtin_in(
            &sandbox,
            &format!("terminology = {{ terms = {name:?} }}"),
            &["terminology", "terms", &path.display().to_string()],
        );
    }
}

/// `sandbox` に `builtin` を `[builtin]` の表に書いた設定で `jafix check` を実行し、exit 2で標準エラーに `expected` を
/// すべて含むことを確かめる
fn assert_check_rejects_builtin_in(sandbox: &Sandbox, builtin: &str, expected: &[&str]) {
    sandbox.write("t.md", "Use jafix.\n");
    sandbox.write_xdg_config(&format!("[builtin]\n{builtin}\n"));
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(2), "{builtin}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    for needle in expected {
        assert!(stderr.contains(needle), "{builtin}: {stderr}");
    }
}

#[test]
fn terminology_user_terms_are_user_replacements() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(
        r#"[builtin]
terminology = { terms = ["GitHub", ["front[- ]end(\\w*)", "frontend$1"]] }
"#,
    );
    let input = "# Use github\n\nUse github for the front-end.\n";
    sandbox.write("t.md", input);
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "terminology"),
        [
            "t.md:1:7: 「github」を「GitHub」にする。見出しのアンカーが変わるので、見出しへのリンクも直す [terminology]",
            "t.md:3:5: 「github」を「GitHub」にする [terminology]",
            "t.md:3:20: 「front-end」を「frontend」にする [terminology]",
        ],
        "{}",
        stdout(&output)
    );
    assert_eq!(
        fix_with_config(&sandbox, input),
        "# Use github\n\nUse GitHub for the frontend.\n"
    );
}

#[test]
fn terminology_rejects_opposite_user_terms() {
    assert_check_rejects_builtin(
        "terminology = { terms = [\"Ｗｅｂ\"] }\nno-mixed-zenkaku-and-hankaku-alphabet = true",
        &[
            "terminology",
            "Ｗｅｂ",
            "no-mixed-zenkaku-and-hankaku-alphabet",
        ],
    );

    let (code, _) = check_with_builtin("terminology = { terms = [\"Ｗｅｂ\"] }", "Use Ｗｅｂ.\n");
    assert_ne!(code, Some(2));
}

#[test]
fn terminology_default_terms_are_reported() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nterminology = true\n");
    let input = "Use javascript here.\n";
    assert_eq!(fix_with_config(&sandbox, input), input);
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "terminology"),
        ["t.md:1:5: 「javascript」を「JavaScript」にする [terminology]"]
    );
}

#[test]
fn terminology_matches_after_space_removal() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nterminology = true\n");
    assert_eq!(
        fix_with_config(&sandbox, "日本語の javascript を使う\n"),
        "日本語のjavascriptを使う\n"
    );
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "terminology"),
        ["t.md:1:5: 「javascript」を「JavaScript」にする [terminology]"]
    );
}

#[test]
fn terminology_user_term_same_as_a_default_term_is_one_fix() {
    for terms in [
        r#"["JavaScript"]"#,
        r#"[["java[ -]?script", "JavaScript"]]"#,
    ] {
        let sandbox = Sandbox::new();
        sandbox.write_xdg_config(&format!("[builtin]\nterminology = {{ terms = {terms} }}\n"));
        sandbox.write("t.md", "Use javascript here.\n");
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(
            findings_of(stdout(&output), "terminology"),
            ["t.md:1:5: 「javascript」を「JavaScript」にする [terminology]"],
            "{terms}"
        );
        assert_eq!(
            fix_with_config(&sandbox, "Use javascript here.\n"),
            "Use JavaScript here.\n",
            "{terms}"
        );
    }

    // 組み込みの辞書の文字列の項目 `GitHub` と、利用者の配列の項目が同じ箇所に当たる
    let sandbox = Sandbox::new();
    sandbox
        .write_xdg_config("[builtin]\nterminology = { terms = [[\"git[ -]?hub\", \"GitHub\"]] }\n");
    sandbox.write("t.md", "Use github here.\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "terminology"),
        ["t.md:1:5: 「github」を「GitHub」にする [terminology]"]
    );
    assert_eq!(
        fix_with_config(&sandbox, "Use github here.\n"),
        "Use GitHub here.\n"
    );
}

#[test]
fn spelling_rejects_unsupported_options() {
    for (builtin, expected) in [
        (
            "spelling = { language = \"en-GB\" }",
            ["spelling", "language"],
        ),
        (
            "spelling = { wordDefinitionRegexp = \"[a-z]+\" }",
            ["spelling", "wordDefinitionRegexp"],
        ),
        (
            "spelling = { wordDefinitionRegexp = \"/[a-z/g\" }",
            ["spelling", "wordDefinitionRegexp"],
        ),
        (
            "spelling = { wordDefinitionRegexp = \"/[a-z]+/y\" }",
            ["spelling", "wordDefinitionRegexp"],
        ),
        (
            "spelling = { skipPatterns = [\"/(/\"] }",
            ["spelling", "skipPatterns"],
        ),
    ] {
        assert_check_rejects_builtin(builtin, &expected);
    }
}

#[test]
fn spelling_skips_identifiers_and_abbreviations() {
    let (_, found) = check_with_builtin(
        "spelling = true",
        "We recieve x86_64 50ms README PostToolUse macOS ruleId.\n",
    );
    assert_eq!(
        findings_of(&found, "spelling"),
        [
            "t.md:1:4: 英語の辞書に無い語「recieve」を使っている。候補: receive、relieve、reverie [spelling]"
        ]
    );
}

#[test]
fn spelling_suggests_candidates() {
    let input = "Run jafix to recieve it.\n";
    let (_, found) = check_with_builtin("spelling = { skipPatterns = [\"jafix\"] }", input);
    assert_eq!(
        findings_of(&found, "spelling"),
        [
            "t.md:1:14: 英語の辞書に無い語「recieve」を使っている。候補: receive、relieve、reverie [spelling]"
        ]
    );
    let (_, found) = check_with_builtin(
        "spelling = { skipPatterns = [\"jafix\"], suggestCorrections = false }",
        input,
    );
    assert_eq!(
        findings_of(&found, "spelling"),
        ["t.md:1:14: 英語の辞書に無い語「recieve」を使っている [spelling]"]
    );
}

#[test]
fn spelling_candidates_are_stable_across_runs() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nspelling = true\n");
    sandbox.write(
        "t.md",
        "Use textlint to seperate the colour of hyperfine untill now.\n",
    );
    let first = sandbox.run(&["check", "t.md"]);
    let second = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&first), "spelling").len(),
        5,
        "{}",
        stdout(&first)
    );
    assert_eq!(stdout(&first), stdout(&second));
}

#[test]
fn spelling_does_not_report_empty_words() {
    let (_, found) = check_with_builtin(
        "spelling = { suggestCorrections = false, wordDefinitionRegexp = \"/[a-z]*/g\" }",
        "日本語のteh、é\n",
    );
    assert_eq!(
        findings_of(&found, "spelling"),
        ["t.md:1:5: 英語の辞書に無い語「teh」を使っている [spelling]"]
    );
}

#[test]
fn spelling_skips_character_reference_names() {
    let (_, found) = check_with_builtin(
        "spelling = { suggestCorrections = false }",
        "Hello&nbsp;world&mdash;and &copy; here.

We recieve&nbsp;it &#x2014;teh &#160;and &pacakge; here.

日本語&nbsp;テスト&;と&nbsp
",
    );
    // `&pacakge;` は実在しない名前なので文字参照でなく、範囲の末尾の `&nbsp` は `;` が無いので文字参照でない
    assert_eq!(
        findings_of(&found, "spelling"),
        [
            "t.md:3:4: 英語の辞書に無い語「recieve」を使っている [spelling]",
            "t.md:3:28: 英語の辞書に無い語「teh」を使っている [spelling]",
            "t.md:3:43: 英語の辞書に無い語「pacakge」を使っている [spelling]",
            "t.md:5:17: 英語の辞書に無い語「nbsp」を使っている [spelling]",
        ]
    );
}

#[test]
fn common_misspellings_and_write_good_skip_character_reference_names() {
    let (_, found) =
        check_with_builtin("common-misspellings = true", "Use &andd; and andd here.\n");
    assert_eq!(
        findings_of(&found, "common-misspellings"),
        [
            "t.md:1:16: よくある綴りの誤り「andd」を使っている。正しくは「and」 [common-misspellings]"
        ]
    );
    let (_, found) = check_with_builtin(
        "write-good = true",
        "We said &therefore; and therefore left.\n",
    );
    assert_eq!(
        findings_of(&found, "write-good"),
        ["t.md:1:25: 「therefore」は冗長か不要な句である(tooWordy) [write-good]"]
    );
}

#[test]
fn terminology_does_not_fix_inside_character_references() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nterminology = { terms = [\"Copy\"] }\n");
    sandbox.write("t.md", "&copy; copy\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(
        findings_of(stdout(&output), "terminology"),
        ["t.md:1:8: 「copy」を「Copy」にする [terminology]"]
    );
    assert_eq!(fix_with_config(&sandbox, "&copy; copy\n"), "&copy; Copy\n");
}

#[test]
fn user_replacements_leave_text_already_in_the_replaced_form() {
    let pattern = r#""@textlint-rule/pattern" = { patterns = [{ message = "m", pattern = "/JS/", replace = "JavaScript (JS)" }] }"#;
    let terminology = r#"terminology = { terms = [["JS", "JavaScript (JS)"]] }"#;
    for (builtin, rule, input, fixed) in [
        (
            prh_rule("サーバー", "サーバ"),
            "prh",
            "サーバーとサーバを使う。\n",
            "サーバーとサーバーを使う。\n",
        ),
        (
            prh_rule("Node.js", "Node"),
            "prh",
            "Node.jsとNodeを使う。\n",
            "Node.jsとNode.jsを使う。\n",
        ),
        (
            pattern.to_owned(),
            "@textlint-rule/pattern",
            "Use JavaScript (JS) and JS.\n",
            "Use JavaScript (JS) and JavaScript (JS).\n",
        ),
        (
            terminology.to_owned(),
            "terminology",
            "Use JavaScript (JS) and JS.\n",
            "Use JavaScript (JS) and JavaScript (JS).\n",
        ),
    ] {
        let sandbox = Sandbox::new();
        sandbox.write_xdg_config(&format!("[builtin]\n{builtin}\n"));
        assert_eq!(fix_with_config(&sandbox, input), fixed, "{builtin}");
        let path = sandbox.path("t.md");
        assert_eq!(sandbox.run(&["fix", "t.md"]).status.code(), Some(0));
        assert_eq!(read(&path), fixed, "{builtin}");
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(
            findings_of(stdout(&output), rule),
            Vec::<&str>::new(),
            "{builtin}"
        );
    }
}

#[test]
fn ja_no_successive_word_reports_repeated_punctuation_but_not_other_symbols() {
    let (_, found) = check_with_builtin(
        "",
        "これは──例です。○○の場合です。これは……例です。\n\nあ、、い。。\n",
    );
    assert_eq!(
        findings_of(&found, "ja-no-successive-word"),
        [
            "t.md:3:3: 「、」が2回続いている [ja-no-successive-word]",
            "t.md:3:6: 「。」が2回続いている [ja-no-successive-word]",
        ]
    );
}

#[test]
fn jtf_style_angle_brackets_do_not_count_arrows() {
    let (_, found) = check_with_builtin(
        "\"4.3.7.山かっこ<>\" = true",
        "処理A -> 処理B => C <- D <= E <a\n",
    );
    assert_eq!(
        findings_of(&found, "4.3.7.山かっこ<>"),
        ["t.md:1:27: < に対応する > が無い [4.3.7.山かっこ<>]"]
    );
}

#[test]
fn no_todo_reports_an_unchecked_box_whatever_follows_it() {
    for (input, position) in [
        ("- [ ]\n", "1:3"),
        ("- [ ] \n", "1:3"),
        ("- [ ]", "1:3"),
        ("- [ ]\n\n  b\n", "1:3"),
        ("- [ ]\n  - [x] child\n", "1:3"),
        ("- a [ ]\n", "1:5"),
        ("- a [ ]", "1:5"),
        ("- a [ ]\n\n  b\n", "1:5"),
        ("- a [ ] b\n", "1:5"),
    ] {
        let (_, found) = check_with_builtin("no-todo = true", input);
        assert_eq!(
            findings_of(&found, "no-todo"),
            [format!(
                "t.md:{position}: 未完了のタスクが残っている [no-todo]"
            )],
            "{input:?}"
        );
    }
    for input in [
        "- [ ](https://example.com)\n",
        "- a [ ](https://example.com)\n",
        "- a `[ ]`\n",
    ] {
        let (_, found) = check_with_builtin("no-todo = true", input);
        assert_eq!(
            findings_of(&found, "no-todo"),
            Vec::<&str>::new(),
            "{input:?}"
        );
    }
}

#[test]
fn sentence_length_ends_sentences_at_hard_breaks() {
    let line = format!("– {}", "あ".repeat(60));
    for input in [
        format!("{line}  \n{line}  \n{line}\n"),
        format!("{line}\\\n{line}\\\n{line}\n"),
    ] {
        let (_, found) = check_with_builtin("", &input);
        assert_eq!(
            findings_of(&found, "sentence-length"),
            Vec::<&str>::new(),
            "{input:?}"
        );
    }
    let (_, found) = check_with_builtin("", &format!("{line}{}\n", "い".repeat(40)));
    assert_eq!(
        findings_of(&found, "sentence-length"),
        ["t.md:1:1: 1文が100文字を超えている (102文字) [sentence-length]"]
    );
}

#[test]
fn sentence_length_does_not_count_the_indentation_of_continuation_lines() {
    let half = "あ".repeat(40);
    let indented = format!("- {half}\n{}{half}\n", " ".repeat(30));
    let (_, found) = check_with_builtin("", &indented);
    assert_eq!(
        findings_of(&found, "sentence-length"),
        Vec::<&str>::new(),
        "{indented:?}"
    );
    let long = format!("- {half}\n{}{half}{}\n", " ".repeat(30), "い".repeat(20));
    let (_, found) = check_with_builtin("", &long);
    assert_eq!(
        findings_of(&found, "sentence-length"),
        ["t.md:1:3: 1文が100文字を超えている (101文字) [sentence-length]"]
    );
}

#[test]
fn terminology_suggests_colocated_for_co_located() {
    let (_, found) = check_with_builtin(
        "terminology = true",
        "Use co-located files and co-location.\n",
    );
    assert_eq!(
        findings_of(&found, "terminology"),
        [
            "t.md:1:5: 「co-located」を「colocated」にする [terminology]",
            "t.md:1:26: 「co-location」を「colocation」にする [terminology]",
        ]
    );
}

#[test]
fn no_unmatched_pair_does_not_split_sentences_at_hard_breaks() {
    for input in [
        "「一行目の文で  \n二行目の文」と書いた。\n",
        "彼は（注記を書く\\\n続き）と言った。\n",
        "「a  \n「b」」と書いた。\n",
        "（注：詳細は  \n（別表）を参照）。\n",
    ] {
        let (_, found) =
            check_with_builtin("no-unmatched-pair = true\nfullwidth-paren = false", input);
        assert_eq!(
            findings_of(&found, "no-unmatched-pair"),
            Vec::<&str>::new(),
            "{input:?}"
        );
    }
    let (_, found) = check_with_builtin("no-unmatched-pair = true", "(注記  \n続き\n");
    assert_eq!(
        findings_of(&found, "no-unmatched-pair"),
        ["t.md:1:2: ( に対応する ) が無い [no-unmatched-pair]"]
    );
}
