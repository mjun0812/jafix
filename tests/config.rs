mod common;

use common::{Sandbox, read, stdout};

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
