mod common;

use std::fs;
use std::process::Command;

use common::{Sandbox, read, stdout};

fn git(sandbox: &Sandbox, args: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .current_dir(sandbox.path(""))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", sandbox.path("unused-gitconfig"))
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{args:?}: {output:?}");
    output.stdout
}

fn init(sandbox: &Sandbox) {
    git(sandbox, &["init", "-q"]);
    for (key, value) in [
        ("user.name", "Test"),
        ("user.email", "test@example.test"),
        ("commit.gpgsign", "false"),
        ("core.autocrlf", "false"),
    ] {
        git(sandbox, &["config", key, value]);
    }
}

fn commit(sandbox: &Sandbox) {
    git(sandbox, &["add", "--all"]);
    git(sandbox, &["commit", "-qm", "baseline"]);
}

#[test]
fn worktree_diff_limits_findings_and_fixes_to_added_lines_with_markdown_context() {
    let sandbox = Sandbox::new();
    init(&sandbox);
    let path = sandbox.write(
        "t.txt",
        concat!(
            "既存の（括弧）です。\n\n変更前です。\n\n```\n変更前のコード\n```\n\n",
            "残す（括弧）です。\n\nご質問ありがとうございます。\n\n最後の段落です。\n",
        ),
    );
    commit(&sandbox);
    sandbox.write_xdg_config("[builtin]\nyomiyasu-chat-response = true\n");
    fs::write(
        &path,
        concat!(
            "既存の（括弧）です。\n\n追加の（本文）と結果をを確認します。\n\n```\n追加の（コード）\n```\n\n",
            "残す（括弧）です。\n\nご質問ありがとうございます。\n\n必要なら次に設定例を紹介します。\n",
        ),
    )
    .unwrap();
    let untracked = sandbox.write("untracked.txt", "対象外の（括弧）です。\n");
    let output = sandbox.run(&["check", "--diff"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let lines: Vec<_> = stdout(&output).lines().collect();
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert!(lines[..3].iter().all(|line| line.starts_with("t.txt:3:")));
    assert!(
        lines[3].starts_with("t.txt:13:1: [yomiyasu-chat-response] "),
        "{lines:?}"
    );

    let output = sandbox.run(&["fix", "--diff", "--show-diff"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(stdout(&output).starts_with("--- t.txt\n+++ t.txt\n"));
    assert_eq!(
        stdout(&output)
            .lines()
            .filter(|line| line.starts_with(['-', '+']))
            .skip(2)
            .collect::<Vec<_>>(),
        [
            "-追加の（本文）と結果をを確認します。",
            "+追加の(本文)と結果をを確認します。"
        ]
    );
    assert_eq!(
        read(&path),
        concat!(
            "既存の（括弧）です。\n\n追加の(本文)と結果をを確認します。\n\n```\n追加の（コード）\n```\n\n",
            "残す（括弧）です。\n\nご質問ありがとうございます。\n\n必要なら次に設定例を紹介します。\n",
        )
    );
    assert_eq!(read(&untracked), "対象外の（括弧）です。\n");
}

#[test]
fn staged_check_reads_the_index_even_when_the_worktree_content_differs() {
    let sandbox = Sandbox::new();
    init(&sandbox);
    let path = sandbox.write(
        "sub/日本語 name.txt",
        "既存の（括弧）です。\n\n変更前です。\n",
    );
    commit(&sandbox);
    let staged = "既存の（括弧）です。\n\nステージした（本文）です。\n";
    fs::write(&path, staged).unwrap();
    git(&sandbox, &["add", "sub/日本語 name.txt"]);
    let worktree = "作業ツリーでは修正しました。\n";
    fs::write(&path, worktree).unwrap();

    let output = sandbox.run(&["check", "--staged"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(stdout(&output).lines().count(), 2, "{output:?}");
    assert!(
        stdout(&output)
            .lines()
            .all(|line| line.starts_with("sub/日本語 name.txt:3:"))
    );
    assert_eq!(read(&path), worktree);
    assert_eq!(
        git(&sandbox, &["show", ":sub/日本語 name.txt"]),
        staged.as_bytes()
    );

    fs::remove_file(&path).unwrap();
    sandbox.write("sub/.jafix.toml", "[builtin]\nfullwidth-paren = false\n");
    let output = sandbox.run(&["check", "--staged"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(stdout(&output).is_empty());
}

#[test]
fn worktree_fix_updates_both_staged_and_unstaged_lines_in_cp932_without_staging() {
    let sandbox = Sandbox::new();
    init(&sandbox);
    let path = sandbox.write("t.txt", "old\n\nkeep this line\n\nold\n");
    commit(&sandbox);
    let line = b"\x95\x5c\x8b\x4c\x82\xf0\x81\x69\x8a\x6d\x94\x46\x81\x6a\x82\xb5\x82\xdc\x82\xb7\x81\x42\n";
    let fixed = b"\x95\x5c\x8b\x4c\x82\xf0(\x8a\x6d\x94\x46)\x82\xb5\x82\xdc\x82\xb7\x81\x42\n";
    let unchanged: &[u8] = b"\nkeep this line\n\n";
    let staged = [line.as_slice(), unchanged, b"old\n"].concat();
    fs::write(&path, &staged).unwrap();
    git(&sandbox, &["add", "t.txt"]);
    fs::write(&path, [line.as_slice(), unchanged, line].concat()).unwrap();

    let output = sandbox.run(&["fix", "--diff"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        fs::read(&path).unwrap(),
        [fixed.as_slice(), unchanged, fixed].concat()
    );
    assert_eq!(git(&sandbox, &["show", ":t.txt"]), staged);
}

#[test]
fn diff_with_only_a_rename_or_a_deletion_has_no_target_lines() {
    let sandbox = Sandbox::new();
    init(&sandbox);
    let renamed = sandbox.write("old.txt", "既存の（括弧）です。\n");
    let edited = sandbox.write("edited.txt", "削除する行。\n\n既存の（括弧）です。\n");
    commit(&sandbox);
    git(&sandbox, &["mv", "old.txt", "new.txt"]);
    assert!(!renamed.exists());
    fs::write(&edited, "\n既存の（括弧）です。\n").unwrap();

    let output = sandbox.run(&["check", "--diff"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(stdout(&output).is_empty());
    let output = sandbox.run(&["check", "--staged"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(stdout(&output).is_empty());
}

#[test]
fn worktree_diff_accepts_a_literal_path_from_a_subdirectory() {
    let sandbox = Sandbox::new();
    init(&sandbox);
    let path = sandbox.write("sub/[entry].txt", "変更前です。\n");
    let other = sandbox.write("sub/other.txt", "変更前です。\n");
    commit(&sandbox);
    fs::write(path, "変更後の（括弧）です。\n").unwrap();
    fs::write(other, "変更後の（括弧）です。\n").unwrap();

    let output = sandbox
        .command()
        .current_dir(sandbox.path("sub"))
        .args(["check", "--diff", "[entry].txt"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(stdout(&output).lines().count(), 2, "{output:?}");
    assert!(
        stdout(&output)
            .lines()
            .all(|line| line.starts_with("sub/[entry].txt:1:"))
    );
}

#[test]
fn staged_check_accepts_files_before_the_first_commit() {
    let sandbox = Sandbox::new();
    init(&sandbox);
    sandbox.write("t.txt", "最初の（本文）です。\n");
    git(&sandbox, &["add", "t.txt"]);

    let output = sandbox.run(&["check", "--staged"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(stdout(&output).lines().count(), 2, "{output:?}");
}
