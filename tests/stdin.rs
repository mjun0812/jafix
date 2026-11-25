mod common;

use std::fs;
use std::io::Write;
use std::process::{Output, Stdio};

use common::{Sandbox, commit, git, init, read, stdout};

/// 全角括弧の修正ルールに当たる1行の本文
const PAREN_TEXT: &str = "本文（補足）です。\n";
const PAREN_FIXED: &str = "本文(補足)です。\n";

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

fn entries(sandbox: &Sandbox) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(sandbox.path(""))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

/// 拒んだ実行が、標準出力に何も書かず、`jafix: ` で始まる理由を標準エラーに書いて終了コード2を返したことを確かめる
fn assert_rejected(output: &Output) {
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(stdout(output), "", "{output:?}");
    assert!(!stderr(output).is_empty(), "{output:?}");
    assert!(
        stderr(output)
            .lines()
            .all(|line| line.starts_with("jafix: ")),
        "{output:?}"
    );
}

#[test]
fn check_reads_stdin_and_labels_findings() {
    let sandbox = Sandbox::new();
    let output = sandbox.run_with_stdin(&["check", "-"], PAREN_TEXT.as_bytes());
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let lines: Vec<&str> = stdout(&output).lines().collect();
    assert!(
        lines[0].starts_with("<stdin>:1:3: [fullwidth-paren] "),
        "{lines:?}"
    );
    assert!(lines.iter().all(|line| line.starts_with("<stdin>:")));
}

#[test]
fn check_stdin_exits_0_without_findings() {
    let sandbox = Sandbox::new();
    let output = sandbox.run_with_stdin(&["check", "-"], "本文です。\n".as_bytes());
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_json_reports_stdin_path() {
    let sandbox = Sandbox::new();
    let output = sandbox.run_with_stdin(
        &["check", "--output-format", "json", "-"],
        PAREN_TEXT.as_bytes(),
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["checked_files"], 1);
    let findings = report["findings"].as_array().unwrap();
    assert!(!findings.is_empty(), "{report}");
    assert!(
        findings.iter().all(|finding| finding["path"] == "<stdin>"),
        "{report}"
    );
    assert_eq!(report["errors"], serde_json::json!([]));
}

#[test]
fn fix_writes_fixed_text_to_stdout() {
    let sandbox = Sandbox::new();
    let before = entries(&sandbox);
    let output = sandbox.run_with_stdin(&["fix", "-"], PAREN_TEXT.as_bytes());
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(stdout(&output), PAREN_FIXED);
    assert_eq!(entries(&sandbox), before);
}

#[test]
fn fix_stdin_passes_unchanged_text_through() {
    let sandbox = Sandbox::new();
    // 末尾に改行の無い本文で、出力が改行を足さないことも確かめる
    let input = "本文です。\n\n二つ目の段落です。";
    let output = sandbox.run_with_stdin(&["fix", "-"], input.as_bytes());
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, input.as_bytes());
}

#[test]
fn fix_stdin_preserves_encoding_and_line_endings() {
    let sandbox = Sandbox::new();
    // Shift_JISの「表記を（確認）します。」と、括弧を半角にしたもの
    let line: &[u8] = b"\x95\x5c\x8b\x4c\x82\xf0\x81\x69\x8a\x6d\x94\x46\x81\x6a\x82\xb5\x82\xdc\x82\xb7\x81\x42\r\n";
    let fixed: &[u8] =
        b"\x95\x5c\x8b\x4c\x82\xf0(\x8a\x6d\x94\x46)\x82\xb5\x82\xdc\x82\xb7\x81\x42\r\n";
    let input = [line, b"\r\n", line].concat();

    let output = sandbox.run_with_stdin(&["fix", "-"], &input);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, [fixed, b"\r\n", fixed].concat());

    let output = sandbox.run_with_stdin(&["fix", "--encoding", "shift-jis", "-"], &input);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, [fixed, b"\r\n", fixed].concat());
}

#[test]
fn fix_dry_run_stdin_prints_only_the_diff() {
    let sandbox = Sandbox::new();
    let output = sandbox.run_with_stdin(&["fix", "--dry-run", "-"], PAREN_TEXT.as_bytes());
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(stdout(&output), "");

    let output = sandbox.run_with_stdin(
        &["fix", "--dry-run", "--show-diff", "-"],
        PAREN_TEXT.as_bytes(),
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let lines: Vec<&str> = stdout(&output).lines().collect();
    assert!(lines.contains(&"--- <stdin>"), "{lines:?}");
    assert!(lines.contains(&"+++ <stdin>"), "{lines:?}");
    assert!(lines.contains(&"-本文（補足）です。"), "{lines:?}");
    assert!(lines.contains(&"+本文(補足)です。"), "{lines:?}");
}

#[test]
fn stdin_rejects_other_files() {
    let sandbox = Sandbox::new();
    let dash = sandbox.write("-", PAREN_TEXT);
    let other = sandbox.write("other.md", PAREN_TEXT);
    for args in [
        ["check", "-", "other.md"],
        ["fix", "-", "other.md"],
        ["check", "-", "-"],
        ["fix", "-", "-"],
    ] {
        let output = sandbox.run(&args);
        assert_rejected(&output);
        assert_eq!(read(&dash), PAREN_TEXT, "{args:?}");
        assert_eq!(read(&other), PAREN_TEXT, "{args:?}");
    }
}

#[test]
fn check_json_reports_rejected_stdin_as_an_error() {
    let sandbox = Sandbox::new();
    sandbox.write("-", PAREN_TEXT);
    sandbox.write("other.md", PAREN_TEXT);
    let output = sandbox.run(&["check", "--output-format", "json", "-", "other.md"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!report["errors"].as_array().unwrap().is_empty(), "{report}");
    assert_eq!(report["findings"], serde_json::json!([]));
}

#[test]
fn stdin_rejects_git_selection() {
    let sandbox = Sandbox::new();
    init(&sandbox);
    let dash = sandbox.write("-", "本文です。\n");
    commit(&sandbox);
    let changed = format!("本文です。\n\n{PAREN_TEXT}");
    fs::write(&dash, &changed).unwrap();
    git(&sandbox, &["add", "--all"]);
    for args in [
        ["check", "--diff", "-"],
        ["check", "--staged", "-"],
        ["fix", "--diff", "-"],
    ] {
        let output = sandbox.run(&args);
        assert_rejected(&output);
        assert_eq!(read(&dash), changed, "{args:?}");
    }
}

#[test]
fn fix_stdin_rejects_list_and_plain_show_diff() {
    let sandbox = Sandbox::new();
    let dash = sandbox.write("-", PAREN_TEXT);
    for args in [["fix", "--list", "-"], ["fix", "--show-diff", "-"]] {
        let output = sandbox.run(&args);
        assert_rejected(&output);
        assert_eq!(read(&dash), PAREN_TEXT, "{args:?}");
    }
}

// 書き込みが必ず容量不足で失敗する `/dev/full` はLinuxにしか無い。読み取り専用のハンドルを標準出力にする方法は、
// Rustの標準出力が無効なハンドルへの書き込みを成功として扱うので、失敗を起こせない
#[cfg(target_os = "linux")]
#[test]
fn fix_stdin_does_not_count_text_it_could_not_write() {
    let sandbox = Sandbox::new();
    let full = fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let mut child = sandbox
        .command()
        .args(["fix", "--summary", "-"])
        .stdin(Stdio::piped())
        .stdout(full)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(PAREN_TEXT.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(stderr(&output).starts_with("jafix: "), "{output:?}");
    assert!(!stderr(&output).contains("修正: "), "{output:?}");
}

#[test]
fn stdin_uses_project_config_from_current_directory() {
    let sandbox = Sandbox::new();
    let output = sandbox.run_with_stdin(&["check", "-"], PAREN_TEXT.as_bytes());
    assert_eq!(output.status.code(), Some(1), "{output:?}");

    sandbox.write(".jafix.toml", "[builtin]\nfullwidth-paren = false\n");
    let output = sandbox.run_with_stdin(&["check", "-"], PAREN_TEXT.as_bytes());
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(stdout(&output), "");

    // カレントディレクトリの親にあるプロジェクト設定も見つける
    fs::create_dir(sandbox.path("sub")).unwrap();
    let mut child = sandbox
        .command()
        .current_dir(sandbox.path("sub"))
        .args(["check", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(PAREN_TEXT.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
}

#[test]
fn stdin_ignores_exclude() {
    let sandbox = Sandbox::new();
    sandbox.write(".jafix.toml", "exclude = [\"**\"]\n");
    sandbox.write("t.md", PAREN_TEXT);
    let file = sandbox.run(&["check", "t.md"]);
    assert_eq!(file.status.code(), Some(0), "{file:?}");

    let output = sandbox.run_with_stdin(&["check", "-"], PAREN_TEXT.as_bytes());
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stdout(&output).starts_with("<stdin>:1:3: [fullwidth-paren] "));
}

#[test]
fn fix_stdin_exits_2_for_undecodable_input() {
    let sandbox = Sandbox::new();
    let output = sandbox.run_with_stdin(&["fix", "-"], "本文（補足）\0です。\n".as_bytes());
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(
        stderr(&output).starts_with("jafix: <stdin>: "),
        "{output:?}"
    );
}

#[test]
fn dash_file_is_readable_with_a_directory_prefix() {
    let sandbox = Sandbox::new();
    sandbox.write("-", PAREN_TEXT);
    let output = sandbox.run(&["check", "./-"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        stdout(&output).starts_with("./-:1:3: [fullwidth-paren] "),
        "{output:?}"
    );
}

#[test]
fn help_shows_stdin_examples() {
    let sandbox = Sandbox::new();
    for command in ["check", "fix"] {
        let output = sandbox.run(&[command, "--help"]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let (_, examples) = stdout(&output).split_once("例:\n").unwrap();
        assert!(
            examples.lines().any(|line| {
                let mut args = line.split_whitespace();
                args.next() == Some("jafix")
                    && args.next() == Some(command)
                    && args.any(|arg| arg == "-")
            }),
            "{examples}"
        );
    }
}
