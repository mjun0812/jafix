mod common;

use common::{Sandbox, fixture, read, stdout};

fn fix_one_file(content: &str) -> String {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", content);
    let output = sandbox.run(&["fix", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    read(&path)
}

#[test]
fn fix_removes_spaces_and_replaces_fullwidth_parens() {
    assert_eq!(
        fix_one_file("日本語 Rust で （補足） です\n"),
        "日本語Rustで(補足)です\n"
    );
}

#[test]
fn fix_removes_runs_of_halfwidth_and_fullwidth_spaces() {
    assert_eq!(
        fix_one_file("日本語\u{3000}Rust  です\n"),
        "日本語Rustです\n"
    );
}

#[test]
fn fix_treats_long_vowel_mark_as_japanese() {
    assert_eq!(fix_one_file("サーバー API を使う\n"), "サーバーAPIを使う\n");
}

#[test]
fn fix_keeps_space_after_punctuation() {
    assert_eq!(fix_one_file("です。 Rust は\n"), "です。 Rustは\n");
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
fn fix_accepts_files_with_any_extension() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.txt", "日本語 Rust\n");
    assert_eq!(sandbox.run(&["fix", "t.txt"]).status.code(), Some(0));
    assert_eq!(read(&path), "日本語Rust\n");
}

#[test]
fn fix_continues_after_an_unreadable_file_and_exits_2() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust\n");
    let output = sandbox.run(&["fix", "missing.md", "t.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
    assert_eq!(read(&path), "日本語Rust\n");
}

#[test]
fn check_prints_findings_without_changing_the_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("t.md", "日本語 Rust");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:1:4: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]\n"
    );
    assert_eq!(read(&path), "日本語 Rust");
}

#[test]
fn check_prints_each_rule_with_line_and_character_column() {
    let sandbox = Sandbox::new();
    sandbox.write("t.md", "前の行\n\n**強**の文 （補足）\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "t.md:3:8: 日本語と括弧の間の空白を削除する [space-between-ja-and-paren]\n\
         t.md:3:9: 全角括弧を半角にする [fullwidth-paren]\n\
         t.md:3:12: 全角括弧を半角にする [fullwidth-paren]\n"
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
    sandbox.write("t.md", "日本語Rustです。 Rustは(補足)です\n");
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

#[test]
fn version_prints_name_and_version() {
    let output = Sandbox::new().run(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "jafix 0.1.0\n");
}
