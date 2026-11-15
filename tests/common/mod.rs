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

/// `preset-ja-spacing` と、presetに入らない `no-mixed-zenkaku-and-hankaku-alphabet`・`no-kangxi-radicals` を有効にする設定
pub const SPACING_RULES_CONFIG: &str = "[builtin]
preset-ja-spacing = true
no-mixed-zenkaku-and-hankaku-alphabet = true
no-kangxi-radicals = true
";

/// 見出し・本文・`！ ` の後に文が続く行・インラインコードの前後に空白のある行を持ち、
/// `SPACING_RULES_CONFIG` の修正ルールに当たる箇所が各行にある文書
pub const SPACING_RULES_INPUT: &str = "# Ｒｕｓｔの⼊門

本文の Ｒｕｓｔ と 日本 語 を読む。

すごい！ これは Ｇｏ で書く `code` を読む。

別の行も `code` を書く。
";

/// `SPACING_RULES_INPUT` の1行目と5行目を対象行にして直した文書。見出しの1行目は直さず、
/// 全角英字を半角にした後の空白も同じ実行で消える
pub const SPACING_RULES_FIXED: &str = "# Ｒｕｓｔの⼊門

本文の Ｒｕｓｔ と 日本 語 を読む。

すごい！ これはGoで書く`code`を読む。

別の行も `code` を書く。
";

/// `SPACING_RULES_INPUT` の1行目と5行目を対象行にしたhookが、`path` について返す13ルールの指摘の行
pub fn spacing_rule_reports(path: &Path) -> Vec<String> {
    let path = path.display();
    vec![
        format!(
            "{path}:1:3: 全角のアルファベット「Ｒｕｓｔ」を半角にする。見出しのアンカーが変わるので、見出しへのリンクも直す [no-mixed-zenkaku-and-hankaku-alphabet]"
        ),
        format!(
            "{path}:1:8: 康熙部首「⼊」を漢字「入」にする。見出しのアンカーが変わるので、見出しへのリンクも直す [no-kangxi-radicals]"
        ),
        format!(
            "{path}:5:5: 文末の「！」の後に別の文が続くので、直後の半角空白を全角空白にする [ja-space-after-exclamation]"
        ),
    ]
}

/// 標準エラーの行のうち、`preset-ja-spacing` の11ルールと `no-mixed-zenkaku-and-hankaku-alphabet`・
/// `no-kangxi-radicals` の行
pub fn spacing_rule_lines(stderr: &str) -> Vec<&str> {
    const IDS: [&str; 13] = [
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
    stderr
        .lines()
        .filter(|line| IDS.iter().any(|id| line.ends_with(&format!(" [{id}]"))))
        .collect()
}

/// `preset-jtf-style` と、presetの既定で無効な4ルールを有効にする設定。`preset-jtf-style` の `4.3.1.丸かっこ（）` と
/// 逆の `fullwidth-paren` は無効にする
pub const JTF_STYLE_RULES_CONFIG: &str = "[builtin]
preset-jtf-style = true
fullwidth-paren = false
\"2.1.2.漢字\" = true
\"2.1.5.カタカナ\" = true
\"2.1.6.カタカナの長音\" = true
\"2.2.1.ひらがなと漢字の使い分け\" = true
";

/// 見出し・本文・リスト・`例:` のある行・全角の数字のある行・辞書の語のある行を持つ文書。3行目と7行目を対象行にする。
/// 本文の敬体と常体、リストの項目の `。` の有無は、対象行の外の行を含めて数えると、対象行の側が少ない
pub const JTF_STYLE_RULES_HOOK_INPUT: &str = "# 見出し。

本文は１２３だ。例:これを読みます。

本文は４５６です。例:それを読みます。

- サーバは７８９です
- 項目はサーバを予め使います。
- 項目は３つです。

本文は１０個だ。
";

/// `JTF_STYLE_RULES_HOOK_INPUT` の3行目と7行目を対象行にして直した文書。対象行の全角の数字だけが半角になる
pub const JTF_STYLE_RULES_HOOK_FIXED: &str = "# 見出し。

本文は123だ。例:これを読みます。

本文は４５６です。例:それを読みます。

- サーバは789です
- 項目はサーバを予め使います。
- 項目は３つです。

本文は１０個だ。
";

/// `JTF_STYLE_RULES_HOOK_INPUT` の3行目と7行目を対象行にしたhookが、`path` について返す37ルールの指摘の行。
/// 1行目の `1.1.2`、5行目の `4.2.7`、8行目の辞書の2ルール、11行目の `1.1.1` は、対象行の外なので出ない
pub fn jtf_style_rule_reports(path: &Path) -> Vec<String> {
    let path = path.display();
    vec![
        format!("{path}:3:7: 本文を敬体(ですます調)に揃える: 「だ。」は常体 [1.1.1.本文]"),
        format!("{path}:3:10: コロンは全角の「：」にする [4.2.7.コロン(：)]"),
        format!(
            "{path}:7:1: 箇条書きの文末の「。」の有無を揃える: 「。」を付ける [1.1.3.箇条書き]"
        ),
        format!("{path}:7:3: 「サーバは7」を「サーバーは7」にする [2.1.6.カタカナの長音]"),
    ]
}

/// 標準エラーの行のうち、`preset-jtf-style` の37ルールの行。37ルールのIDだけが数字で始まる
pub fn jtf_style_rule_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|line| {
            line.rsplit_once(" [")
                .is_some_and(|(_, id)| id.starts_with(|c: char| c.is_ascii_digit()))
        })
        .collect()
}

/// ひらがな系の5ルール、`@textlint-ja/no-insert-dropping-sa`、`@textlint-ja/no-synonyms`。どれも既定で無効である
pub const HIRAGANA_AND_SYNONYMS_RULES: [&str; 7] = [
    "ja-hiragana-hojodoushi",
    "ja-hiragana-keishikimeishi",
    "ja-keishikimeishi",
    "ja-hiragana-fukushi",
    "ja-hiragana-daimeishi",
    "@textlint-ja/no-insert-dropping-sa",
    "@textlint-ja/no-synonyms",
];

/// `HIRAGANA_AND_SYNONYMS_RULES` をすべて有効にする設定ファイル
pub fn hiragana_and_synonyms_rules_config() -> String {
    let enabled: String = HIRAGANA_AND_SYNONYMS_RULES
        .iter()
        .map(|rule| format!("\"{rule}\" = true\n"))
        .collect();
    format!("[builtin]\n{enabled}")
}

/// `HIRAGANA_AND_SYNONYMS_RULES` の7つが3行目と5行目に当たり、対象行の外の1行目と7行目にも当たる文書。3行目と5行目を
/// 対象行にする。5行目の `サーバ` と、対象行の外の9行目の `サーバー` は、同じ同義語の集まりの別の表記である
pub const HIRAGANA_AND_SYNONYMS_HOOK_INPUT: &str = "お願い致します。

見た事がある。既に終わった。彼処に行く。

このカレーは辛さそうだ。サーバを使う。お願い致します。

見た事がある。

サーバーを使う。
";

/// `HIRAGANA_AND_SYNONYMS_HOOK_INPUT` の3行目と5行目を対象行にしたhookが、`path` について返す7ルールの指摘の行。
/// 1行目と7行目の指摘と、9行目の `サーバー` の指摘は、対象行の外なので出ない
pub fn hiragana_and_synonyms_rule_reports(path: &Path) -> Vec<String> {
    let path = path.display();
    vec![
        format!(
            "{path}:3:3: 形式名詞「事」はひらがなの「こと」にする [ja-hiragana-keishikimeishi]"
        ),
        format!("{path}:3:3: 形式名詞「事」はひらがなの「こと」にする [ja-keishikimeishi]"),
        format!("{path}:3:8: 副詞「既に」はひらがなの「すでに」にする [ja-hiragana-fukushi]"),
        format!("{path}:3:15: 代名詞「彼処」はひらがなの「あそこ」にする [ja-hiragana-daimeishi]"),
        format!(
            "{path}:5:8: さ入れ言葉「辛さそう」は「辛そう」にする [@textlint-ja/no-insert-dropping-sa]"
        ),
        format!(
            "{path}:5:13: 同義語の「サーバ」と「サーバー」が混ざっている [@textlint-ja/no-synonyms]"
        ),
        format!(
            "{path}:5:23: 補助動詞「致し」はひらがなの「いたし」にする [ja-hiragana-hojodoushi]"
        ),
    ]
}

/// 標準エラーの行のうち、`HIRAGANA_AND_SYNONYMS_RULES` の7ルールの行
pub fn hiragana_and_synonyms_rule_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|line| {
            HIRAGANA_AND_SYNONYMS_RULES
                .iter()
                .any(|id| line.ends_with(&format!(" [{id}]")))
        })
        .collect()
}

/// `no-todo`・`date-weekday-mismatch`・`no-dead-link` と、`comments`・`allowlist` を有効にする設定。`allowlist` は
/// `allowed.md` を許す
pub const GENERAL_RULES_AND_FILTERS_CONFIG: &str = "[builtin]
no-todo = true
date-weekday-mismatch = true
no-dead-link = true

[filters]
comments = true
allowlist = { allow = [\"allowed.md\"] }
";

/// `<!-- textlint-disable -->` の前の1行目と後の5行目が、3つのルールに当たる文書。1行目と5行目を対象行にする。
/// 5行目は、全角括弧の修正ルールにも当たる
pub const GENERAL_RULES_HOOK_INPUT: &str =
    "TODO: 2024年4月23日(月)に[設定](missing.md)と[別](allowed.md)を見る。

<!-- textlint-disable -->

TODO: 2024年4月23日(月)に[設定](missing.md)と[別](allowed.md)を見る（補足）。
";

/// `GENERAL_RULES_HOOK_INPUT` の1行目と5行目を対象行にしたhookが、`path` について返す3つのルールの指摘の行。
/// 5行目は無効化の後なので出ず、`allowlist` が許す `allowed.md` のリンクも出ない
pub fn general_rule_reports(path: &Path) -> Vec<String> {
    let path = path.display();
    vec![
        format!("{path}:1:1: TODOが残っている: 「TODO: 2024年4月23日(月)に」 [no-todo]"),
        format!(
            "{path}:1:18: 「2024年4月23日(」の曜日は「火」で、「月」ではない [date-weekday-mismatch]"
        ),
        format!("{path}:1:26: リンク先のファイルが無い: missing.md [no-dead-link]"),
    ]
}

/// 標準エラーの行のうち、`no-todo`・`date-weekday-mismatch`・`no-dead-link` の行
pub fn general_rule_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|line| {
            ["no-todo", "date-weekday-mismatch", "no-dead-link"]
                .iter()
                .any(|id| line.ends_with(&format!(" [{id}]")))
        })
        .collect()
}
