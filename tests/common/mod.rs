#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

/// 一時ディレクトリを作業ディレクトリにし、`HOME` と `XDG_CONFIG_HOME` もその中へ向けてjafixを起動する。
/// 実行した人の設定ファイルを読まないためである
pub struct Sandbox {
    dir: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("home")).unwrap();
        fs::create_dir(dir.path().join("xdg")).unwrap();
        Self { dir }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    pub fn home(&self) -> PathBuf {
        self.path("home")
    }

    pub fn write(&self, name: &str, content: &str) -> PathBuf {
        let path = self.path(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    pub fn write_xdg_config(&self, content: &str) -> PathBuf {
        self.write("xdg/jafix/config.toml", content)
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jafix"));
        command
            .current_dir(self.dir.path())
            .env("HOME", self.home())
            .env("XDG_CONFIG_HOME", self.path("xdg"));
        command
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    /// `jafix hook <agent>` を、`input` を標準入力に渡して起動する
    pub fn hook(&self, agent: &str, input: &str) -> Output {
        let mut child = self
            .command()
            .args(["hook", agent])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

pub fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).unwrap()
}

pub fn fixture(name: &str) -> String {
    read(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
}
