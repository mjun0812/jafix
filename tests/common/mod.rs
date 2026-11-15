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

/// `@textlint-ja/preset-ai-writing` の5ルールと `preset-ai-words-ja` の2ルールのそれぞれに当たる文書。
/// 段落を分けて、ルールどうしの指摘が重ならないようにしている
pub const AI_RULES_INPUT: &str = "- **項目**: 説明

これは革命的なです。

✅ **項目**です。

次のコマンドを実行します:

```sh
npm test
```

まず最初に確認します。

通信経路を確認する。

今日は、晴れです。
";

/// `AI_RULES_INPUT` の7ルールをすべて有効にする設定。2つのpresetと、presetの既定で無効な `no-short-topic-comma` を有効にする
pub const AI_RULES_CONFIG: &str = "[builtin]
\"@textlint-ja/preset-ai-writing\" = true
preset-ai-words-ja = true
no-short-topic-comma = true
";

/// `@textlint-rule/pattern` に、「照合」を指摘する `replace` の無い要素を1つ書いた設定
pub const AI_WORDS_PATTERN: &str = "[[builtin.\"@textlint-rule/pattern\".patterns]]
message = \"AIが多用する語\"
pattern = \"照合\"
";

/// `prh` に1ルールの辞書を、`@textlint-rule/pattern` に `replace` のある要素と無い要素を1つずつ書いた設定
pub const USER_REPLACEMENTS_CONFIG: &str = r#"[builtin.prh]
ruleContents = ["""
rules:
  - expected: Cookie
    pattern: クッキー
"""]

[[builtin."@textlint-rule/pattern".patterns]]
message = "TODOを残さない"
pattern = "TODO"
replace = "DONE"

[[builtin."@textlint-rule/pattern".patterns]]
message = "FIXMEを残さない"
pattern = "FIXME"
"#;

/// 見出しと本文の3行が、`USER_REPLACEMENTS_CONFIG` の3つの置き換えのすべてに当たる文書
pub const USER_REPLACEMENTS_INPUT: &str = "# クッキーとTODOの話

クッキーとTODOとFIXMEを書く。

クッキーとTODOとFIXMEを書く。

クッキーとTODOとFIXMEを書く。
";

/// `USER_REPLACEMENTS_INPUT` の1行目と5行目を対象行にして直した文書。見出しの1行目は直さない
pub const USER_REPLACEMENTS_FIXED: &str = "# クッキーとTODOの話

クッキーとTODOとFIXMEを書く。

CookieとDONEとFIXMEを書く。

クッキーとTODOとFIXMEを書く。
";

/// `USER_REPLACEMENTS_INPUT` の1行目と5行目を対象行にしたhookが、`path` について返す指摘の行
pub fn user_replacement_reports(path: &Path) -> Vec<String> {
    let path = path.display();
    vec![
        format!(
            "{path}:1:3: 「クッキー」を「Cookie」にする。見出しのアンカーが変わるので、見出しへのリンクも直す [prh]"
        ),
        format!(
            "{path}:1:8: 「TODO」: TODOを残さない。見出しのアンカーが変わるので、見出しへのリンクも直す [@textlint-rule/pattern]"
        ),
        format!("{path}:5:13: 「FIXME」: FIXMEを残さない [@textlint-rule/pattern]"),
    ]
}
