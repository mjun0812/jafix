mod common;

use common::{AI_WORDS_PATTERN, Sandbox, finding_rule, fixture, read, stdout};

fn fix_one_file(content: &str) -> String {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", content);
    let output = sandbox.run(&["fix", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    read(&path)
}

#[test]
fn fix_normalizes_parentheses_and_spaces_in_japanese_prose() {
    let content = concat!(
        "日本語 Rust で （補足） です\n\n",
        "日本語\u{3000}Rust  です\n\n",
        "サーバー API を使う\n\n",
        "です。 Rust は\n",
    );
    let expected = concat!(
        "日本語Rustで(補足)です\n\n",
        "日本語Rustです\n\n",
        "サーバーAPIを使う\n\n",
        "です。 Rustは\n",
    );
    assert_eq!(fix_one_file(content), expected);
}

#[test]
fn fix_applies_rules_to_every_kind_of_prose() {
    assert_eq!(
        fix_one_file(&fixture("prose.txt")),
        fixture("prose.expected.txt")
    );
}

#[test]
fn fix_leaves_code_urls_front_matter_html_and_element_boundaries_unchanged() {
    for name in ["non-prose-yaml.txt", "non-prose-toml.txt"] {
        let original = fixture(name);
        assert_eq!(fix_one_file(&original), original, "{name}");
    }
}

#[test]
fn fix_leaves_shortcut_and_collapsed_reference_link_text_but_fixes_full_reference_link_text() {
    let original = "詳細は[設定 API]を参照。\n\n別に[設定 API][]と[設定 API][設定 API]も。\n\n[設定 API]: https://example.com/api\n";
    let expected = "詳細は[設定 API]を参照。\n\n別に[設定 API][]と[設定API][設定 API]も。\n\n[設定 API]: https://example.com/api\n";
    assert_eq!(fix_one_file(original), expected);
    assert_eq!(fix_one_file(expected), expected);
}

#[test]
fn fix_replaces_fullwidth_parens_around_a_bare_url() {
    let cases = [
        (
            "公式サイト（https://example.com）を参照する。\n",
            "公式サイト(https://example.com)を参照する。\n",
        ),
        (
            "公式サイト（https://ja.wikipedia.org/wiki/東京（駅））を参照する。\n",
            "公式サイト(https://ja.wikipedia.org/wiki/東京（駅）)を参照する。\n",
        ),
    ];
    for (original, expected) in cases {
        assert_eq!(fix_one_file(original), expected);
        assert_eq!(fix_one_file(expected), expected);
    }
}

#[test]
fn check_and_fix_detect_shift_jis_and_fix_preserves_the_encoding_and_crlf() {
    let sandbox = Sandbox::new();
    let path = sandbox.path("t.txt");
    // CP932で書いた「表記を（確認）します。」
    let original = b"\x95\x5c\x8b\x4c\x82\xf0\x81\x69\x8a\x6d\x94\x46\x81\x6a\x82\xb5\x82\xdc\x82\xb7\x81\x42\r\n";
    std::fs::write(&path, original).unwrap();
    let check = sandbox.run(&["check", "t.txt"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert_eq!(
        stdout(&check),
        "t.txt:1:4: [fullwidth-paren] 全角括弧を半角にする\n\
         t.txt:1:7: [fullwidth-paren] 全角括弧を半角にする\n"
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let utf8 = sandbox.write("u.txt", "表記を（確認）します。\n");
    let fix = sandbox.run(&["fix", "t.txt", "u.txt"]);
    assert_eq!(fix.status.code(), Some(0), "{fix:?}");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"\x95\x5c\x8b\x4c\x82\xf0(\x8a\x6d\x94\x46)\x82\xb5\x82\xdc\x82\xb7\x81\x42\r\n"
    );
    assert_eq!(read(&utf8), "表記を(確認)します。\n");
}

#[test]
fn explicit_shift_jis_overrides_valid_utf8() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nno-hankaku-kana = true\n");
    // UTF-8では「¶」、CP932では「ﾂｶ」になる。
    std::fs::write(sandbox.path("t.txt"), b"\xc2\xb6\n").unwrap();
    let auto = sandbox.run(&["check", "t.txt"]);
    assert_eq!(auto.status.code(), Some(0), "{auto:?}");
    assert_eq!(stdout(&auto), "");

    let explicit = sandbox.run(&["check", "--encoding", "shift-jis", "t.txt"]);
    assert_eq!(explicit.status.code(), Some(1), "{explicit:?}");
    assert_eq!(
        stdout(&explicit),
        "t.txt:1:1: [no-hankaku-kana] 半角カナを全角にする\n"
    );
}

#[test]
fn fix_keeps_shift_jis_source_when_replacement_cannot_be_read_back() {
    let sandbox = Sandbox::new();
    let path = sandbox.path("t.txt");
    let original = b"\x93\xfa\x96\x7b\x8c\xea\x82\xcc\x95\xb6\x8f\xcd\x82\xf0\x8d\x5a\x90\xb3\x82\xb5\x82\xdc\x82\xb7\x81\x42";
    std::fs::write(&path, original).unwrap();
    for replacement in ["¥", "😀"] {
        sandbox.write_xdg_config(&format!(
            "[[builtin.\"@textlint-rule/pattern\".patterns]]\npattern = \"校正\"\nreplace = \"{replacement}\"\nmessage = \"置き換える\"\n"
        ));
        let preview = sandbox.run(&[
            "fix",
            "--encoding",
            "shift-jis",
            "--dry-run",
            "--show-diff",
            "t.txt",
        ]);
        assert_eq!(preview.status.code(), Some(2), "{preview:?}");
        assert!(preview.stdout.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        let output = sandbox.run(&["fix", "--encoding", "shift-jis", "t.txt"]);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("Shift_JISで保存できない"));
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
}

#[test]
fn fix_does_not_rewrite_unsupported_encodings() {
    let sandbox = Sandbox::new();
    let path = sandbox.path("t.txt");
    for original in [
        &b"\xa4\xcf\xa4\xa4"[..], // EUC-JPの「はい」
        b"A\0B\0",                // BOMのないUTF-16LEの「AB」
        b"\x1b$B$O$$\x1b(B",      // ISO-2022-JPの「はい」
    ] {
        std::fs::write(&path, original).unwrap();
        let output = sandbox.run(&["fix", "t.txt"]);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
}

#[test]
fn check_routes_japanese_morphology_by_paragraph_and_keeps_english_rules() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(
        "[builtin]\nmax-ten = { max = 0, strict = true, touten = \",\" }\ncommon-misspellings = true\n",
    );
    sandbox.write(
        "t.md",
        "Teh first sentence, with `日本語` in the code.\n\nCheck this, 結果をを確認します。\n\n操作操作\n",
    );
    let output = sandbox.run(&["check", "t.md"]);
    let findings = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        findings
            .lines()
            .any(|line| line.starts_with("t.md:1:")
                && finding_rule(line) == Some("common-misspellings")),
        "{findings}"
    );
    assert_eq!(
        findings
            .lines()
            .filter(|line| finding_rule(line) == Some("max-ten"))
            .count(),
        1,
        "{findings}"
    );
    assert!(
        findings
            .lines()
            .any(|line| line.starts_with("t.md:3:") && finding_rule(line) == Some("max-ten")),
        "{findings}"
    );
    assert!(
        findings.lines().any(|line| line.starts_with("t.md:5:")
            && finding_rule(line) == Some("ja-no-successive-word")),
        "{findings}"
    );
}

#[test]
fn fix_lists_changed_files_and_reports_counts_and_times() {
    let sandbox = Sandbox::new();
    let changed = sandbox.write("changed.md", "文（例）です。\n");
    let unchanged = sandbox.write("unchanged.md", "文(例)です。\n");
    let output = sandbox.run(&[
        "fix",
        "--list",
        "--summary",
        "--timing",
        "./changed.md",
        "unchanged.md",
    ]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(stdout(&output), "./changed.md\n");
    assert_eq!(read(&changed), "文(例)です。\n");
    assert_eq!(read(&unchanged), "文(例)です。\n");

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let lines: Vec<_> = stderr.lines().collect();
    assert_eq!(lines.len(), 3, "{stderr}");
    for (line, path) in lines.iter().zip(["./changed.md: ", "unchanged.md: "]) {
        let seconds: f64 = line
            .strip_prefix(path)
            .and_then(|time| time.strip_suffix(" s"))
            .unwrap_or_else(|| panic!("{line}"))
            .parse()
            .unwrap();
        assert!(seconds.is_finite() && seconds >= 0.0);
    }
    assert_eq!(lines[2], "修正: 1ファイル、変更なし: 1ファイル");
}

#[test]
fn fix_shows_the_saved_diff_without_a_trailing_newline() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "文（例）です。");
    let preview = sandbox.run(&["fix", "--dry-run", "--show-diff", "--summary", "t.md"]);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    assert_eq!(read(&path), "文（例）です。");
    assert_eq!(
        std::str::from_utf8(&preview.stderr).unwrap().trim_end(),
        "修正予定: 1ファイル、変更なし: 0ファイル"
    );
    let output = sandbox.run(&["fix", "--show-diff", "t.md"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(read(&path), "文(例)です。");
    assert_eq!(preview.stdout, output.stdout);
    assert_eq!(
        stdout(&output),
        "--- t.md\n+++ t.md\n@@ -1 +1 @@\n-文（例）です。\n\\ No newline at end of file\n+文(例)です。\n\\ No newline at end of file\n"
    );
    let unchanged = sandbox.run(&["fix", "--show-diff", "t.md"]);
    assert_eq!(unchanged.status.code(), Some(0), "{unchanged:?}");
    assert!(unchanged.stdout.is_empty());
    assert!(unchanged.stderr.is_empty());
}

#[test]
fn check_json_reports_findings_and_operational_errors_separately() {
    let sandbox = Sandbox::new();
    let original = "日本語 Rustです。\n結果をを確認します。\n";
    let path = sandbox.write("t.md", original);
    let run = |files: &[&str], expected_code| {
        let mut args = vec!["check", "--output-format", "json"];
        args.extend_from_slice(files);
        let output = sandbox.run(&args);
        assert_eq!(output.status.code(), Some(expected_code), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
    };
    let report = run(&["t.md", "missing.md"], 2);
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["checked_files"], 1);
    assert_eq!(
        report["findings"],
        serde_json::json!([
            {
                "path": "t.md", "line": 1, "column": 4,
                "rule_id": "space-between-ja-and-alnum", "message": "日本語と英数字の間の空白を削除する", "fixable": true
            },
            {
                "path": "t.md", "line": 2, "column": 4,
                "rule_id": "ja-no-successive-word", "message": "「を」が2回続いている", "fixable": false
            }
        ])
    );
    assert_eq!(report["errors"][0]["path"], "missing.md");
    assert_eq!(run(&["t.md"], 1)["errors"], serde_json::json!([]));
    assert_eq!(read(&path), original);
    sandbox.write("clean.md", "本文です。\n");
    assert_eq!(run(&["clean.md"], 0)["checked_files"], 1);
    let missing_target = run(&[], 2);
    assert_eq!(missing_target["checked_files"], 0);
    assert_eq!(missing_target["errors"][0]["path"], serde_json::Value::Null);
}

#[test]
fn fix_continues_after_an_unreadable_file_and_exits_2() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    let output = sandbox.run(&["fix", "--list", "missing.md", "t.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
    assert_eq!(stdout(&output), "t.md\n");
    assert_eq!(read(&path), "日本語Rust\n");
}

#[test]
fn check_prints_findings_without_changing_the_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rustです。");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:1:4: [space-between-ja-and-alnum] 日本語と英数字の間の空白を削除する\n"
    );
    assert_eq!(read(&path), "日本語 Rustです。");
}

#[test]
fn check_prints_each_rule_with_line_and_character_column() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "前の行。\n\n**強**の文 （補足）\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:3:8: [space-between-ja-and-paren] 日本語と括弧の間の空白を削除する\n\
         t.md:3:9: [fullwidth-paren] 全角括弧を半角にする\n\
         t.md:3:12: [fullwidth-paren] 全角括弧を半角にする\n"
    );
}

#[test]
fn check_prints_paths_as_given() {
    let sandbox = Sandbox::new();
    sandbox.write("sub/t.md", "日本語 Rust\n");
    let output = sandbox.run(&["check", "./sub/t.md"]);
    assert!(stdout(&output).starts_with("./sub/t.md:1:4: "));
}

#[test]
fn check_exits_0_with_empty_output_without_findings() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語Rustです。 Rustは(補足)です。\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_exits_2_when_a_file_cannot_be_read_even_with_findings() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "日本語 Rust\n");
    let output = sandbox.run(&["check", "t.md", "missing.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
    assert!(stdout(&output).starts_with("t.md:1:4: "));
}

/// `@textlint-rule/pattern` に、`patterns` の各要素を `message = "AIが多用する語"` と `pattern` で書いた設定で、
/// `content` の `t.md` に `jafix check t.md` を実行する
fn check_with_patterns(patterns: &[&str], content: &str) -> std::process::Output {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", content);
    let elements: String = patterns
        .iter()
        .map(|pattern| {
            format!(
                "[[builtin.\"@textlint-rule/pattern\".patterns]]\nmessage = \"AIが多用する語\"\npattern = '{pattern}'\n"
            )
        })
        .collect();
    sandbox.write_xdg_config(&elements);
    sandbox.run(&["check", "t.md"])
}

#[test]
fn check_reports_a_regex_pattern_element_with_the_matched_text() {
    let output = check_with_patterns(&["/[〜～]/"], "すごい〜です。\n");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:1:4: [@textlint-rule/pattern] 「〜」: AIが多用する語\n"
    );
}

#[test]
fn check_does_not_match_pattern_elements_across_an_emphasis_boundary() {
    let output = check_with_patterns(&["照", "合", "照合"], "照**合**\n");
    assert_eq!(
        stdout(&output),
        "t.md:1:1: [@textlint-rule/pattern] 「照」: AIが多用する語\n\
         t.md:1:4: [@textlint-rule/pattern] 「合」: AIが多用する語\n"
    );
}

#[test]
fn check_prints_fix_rule_findings_and_reports_in_position_order() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config(&format!(
        "[builtin]\nno-exclamation-question-mark = true\n{AI_WORDS_PATTERN}"
    ));
    sandbox.write("t.md", "照合 Rust！再び照合 Python。\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:1:1: [@textlint-rule/pattern] 「照合」: AIが多用する語\n\
         t.md:1:3: [space-between-ja-and-alnum] 日本語と英数字の間の空白を削除する\n\
         t.md:1:8: [no-exclamation-question-mark] 「！」を使わない\n\
         t.md:1:11: [@textlint-rule/pattern] 「照合」: AIが多用する語\n\
         t.md:1:13: [space-between-ja-and-alnum] 日本語と英数字の間の空白を削除する\n"
    );
}

#[test]
fn fix_leaves_builtin_reports_unchanged_and_prints_nothing() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nno-exclamation-question-mark = true\n");
    let path = sandbox.write("t.md", "すごい！\n");
    assert_eq!(sandbox.run(&["check", "t.md"]).status.code(), Some(1));
    let output = sandbox.run(&["fix", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(&path), "すごい！\n");
}

#[test]
fn fix_keeps_spaces_in_headings_but_replaces_fullwidth_parens() {
    assert_eq!(fix_one_file("# API の設計（案）\n"), "# API の設計(案)\n");
    let setext = "API の設計\n===\n\n本文 Rust\n";
    assert_eq!(fix_one_file(setext), "API の設計\n===\n\n本文Rust\n");
}

#[test]
fn check_reports_spaces_in_headings_with_a_message_about_links() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "# API の設計\n\n## 設定 (補足) の説明\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    let lines: Vec<&str> = stdout(&output).lines().collect();
    let expected = [
        "t.md:1:6: [space-between-ja-and-alnum] ",
        "t.md:3:6: [space-between-ja-and-paren] ",
        "t.md:3:11: [space-between-ja-and-paren] ",
    ];
    assert_eq!(lines.len(), expected.len(), "{lines:?}");
    for (line, prefix) in lines.into_iter().zip(expected) {
        let message = line
            .strip_prefix(prefix)
            .unwrap_or_else(|| panic!("{line}"));
        assert!(message.contains("リンク"), "{line}");
    }
}

#[test]
fn version_prints_name_and_version() {
    let output = Sandbox::new().run(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        concat!("jafix ", env!("CARGO_PKG_VERSION"), "\n")
    );
}

#[test]
fn check_without_config_reads_unusual_html_comments() {
    for content in [
        "`<!-- a -->`\n",
        "```\n<!-- a -->\n```\n",
        "\\<!-- a -->\n",
        "<!-- 閉じない",
        "段落 <!-- 閉じない",
        "<!-->",
        "<!-- a --> <!-- 閉じない",
        "> <!--\n> textlint-disable\n> -->\n",
    ] {
        let sandbox = Sandbox::new();
        sandbox.write("t.md", content);
        let output = sandbox.run(&["check", "t.md"]);
        // 箇所の有無で0か1になる。panicすると101で、標準エラーに理由が出る
        assert!(
            matches!(output.status.code(), Some(0 | 1)) && output.stderr.is_empty(),
            "{content:?}: {output:?}"
        );
    }
}
