//! コマンドの実行例と終了コードの意味。`--help` の末尾と `--usage` のJSONの両方が、ここの同じ表を使う。

use std::fmt::Write;

use clap::{Arg, Command};
use serde::Serialize;
use serde_json::{Value, json};

/// 1つのコマンドの実行例と終了コード。`name` が `None` なら `jafix` そのものである
pub struct Guide {
    pub name: Option<&'static str>,
    pub examples: &'static [Example],
    pub exit_codes: &'static [ExitCodeMeaning],
}

#[derive(Serialize)]
pub struct Example {
    pub command: &'static str,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct ExitCodeMeaning {
    pub code: u8,
    pub meaning: &'static str,
}

/// `jafix` そのものと、すべてのサブコマンドの `Guide`。実行例は空白で区切って引数解析に通すので、空白を含む引数を書かない
pub const GUIDES: &[Guide] = &[
    Guide {
        name: None,
        examples: &[
            Example {
                command: "jafix check notes.md",
                description: "ファイルを書き換えずに、修正ルールに当たる箇所と指摘を出力する",
            },
            Example {
                command: "jafix fix notes.md",
                description: "修正ルールを適用してファイルに書き戻す",
            },
            Example {
                command: "jafix rules",
                description: "組み込みのルールと、設定で有効かどうかを一覧する",
            },
            Example {
                command: "jafix --usage",
                description: "コマンド、引数、終了コード、実行例をJSONで出力する",
            },
        ],
        exit_codes: &[],
    },
    Guide {
        name: Some("check"),
        examples: &[
            Example {
                command: "jafix check notes.md docs/guide.md",
                description: "2つのファイルの箇所と指摘を出力する",
            },
            Example {
                command: "jafix check -",
                description: "パイプで標準入力に渡したテキストの箇所と指摘を、パスを `<stdin>` として出力する",
            },
            Example {
                command: "jafix check --diff",
                description: "作業ツリーのGitの差分で追加・変更した行だけを確認する",
            },
            Example {
                command: "jafix check --staged",
                description: "ステージ済みの差分で追加・変更した行だけを確認する",
            },
            Example {
                command: "jafix check --output-format json notes.md",
                description: "結果をJSONオブジェクト1件で出力する",
            },
            Example {
                command: "jafix check --schema",
                description: "JSON出力のJSON Schemaを出力する",
            },
        ],
        exit_codes: &[
            ExitCodeMeaning {
                code: 0,
                meaning: "修正ルールに当たる箇所も指摘も無い",
            },
            ExitCodeMeaning {
                code: 1,
                meaning: "修正ルールに当たる箇所か指摘が1件以上ある",
            },
            ExitCodeMeaning {
                code: 2,
                meaning: "引数が誤っている、ファイルか標準入力を読めない、標準出力に書き込めない、設定ファイルが不正、Gitの実行や差分の読み取りに失敗した",
            },
        ],
    },
    Guide {
        name: Some("fix"),
        examples: &[
            Example {
                command: "jafix fix notes.md",
                description: "ファイルの全行に修正ルールを適用して書き戻す",
            },
            Example {
                command: "jafix fix -",
                description: "パイプで標準入力に渡したテキストに修正ルールを適用し、修正後の本文を標準出力に書く",
            },
            Example {
                command: "jafix fix --diff",
                description: "作業ツリーのGitの差分で追加・変更した行だけを直す",
            },
            Example {
                command: "jafix fix --diff --dry-run --show-diff",
                description: "ファイルを書き換えずに、直す内容の差分を表示する",
            },
            Example {
                command: "jafix fix --list docs/guide.md",
                description: "直して、修正したファイルのパスを表示する",
            },
        ],
        exit_codes: &[
            ExitCodeMeaning {
                code: 0,
                meaning: "成功した",
            },
            ExitCodeMeaning {
                code: 2,
                meaning: "引数が誤っている、ファイルを読み書きできない、標準入力を読めない、標準入力の結果を標準出力に書き込めない、`--summary` や `--timing` の出力を標準エラーに書き込めない、設定ファイルが不正、Gitの実行や差分の読み取りに失敗した",
            },
        ],
    },
    Guide {
        name: Some("rules"),
        examples: &[
            Example {
                command: "jafix rules",
                description: "カレントディレクトリのプロジェクト設定で、ルールが有効かどうかを一覧する",
            },
            Example {
                command: "jafix rules docs/guide.md",
                description: "そのファイルに使うプロジェクト設定で一覧する",
            },
            Example {
                command: "jafix rules --output-format json",
                description: "一覧をJSONオブジェクト1件で出力する",
            },
            Example {
                command: "jafix rules --schema",
                description: "JSON出力のJSON Schemaを出力する",
            },
        ],
        exit_codes: &[
            ExitCodeMeaning {
                code: 0,
                meaning: "一覧を出力した",
            },
            ExitCodeMeaning {
                code: 2,
                meaning: "引数が誤っている、パスが存在しない、設定ファイルが不正、JSONを標準出力に書き込めない",
            },
        ],
    },
    Guide {
        name: Some("hook"),
        examples: &[
            Example {
                command: "jafix hook claude",
                description: "Claude CodeのPostToolUse hookの入力を標準入力から読む",
            },
            Example {
                command: "jafix hook codex",
                description: "CodexのPostToolUse hookの入力を標準入力から読む",
            },
        ],
        exit_codes: &[
            ExitCodeMeaning {
                code: 0,
                meaning: "指摘が無く、直したか、直すものが無かった",
            },
            ExitCodeMeaning {
                code: 1,
                meaning: "指摘が無く、入力JSONを読めない、設定ファイルが不正、対象行を決められない、ファイルを読み書きできない、のどれかに当たる",
            },
            ExitCodeMeaning {
                code: 2,
                meaning: "引数が誤っている、対象行に指摘が1件以上ある",
            },
        ],
    },
    Guide {
        name: Some("mcp"),
        examples: &[
            Example {
                command: "jafix mcp",
                description: "カレントディレクトリのプロジェクト設定でMCPサーバーを起動する",
            },
            Example {
                command: "jafix mcp --project-dir docs",
                description: "`docs` から探したプロジェクト設定でMCPサーバーを起動する",
            },
        ],
        exit_codes: &[
            ExitCodeMeaning {
                code: 0,
                meaning: "標準入力が閉じてサーバーを終えた",
            },
            ExitCodeMeaning {
                code: 2,
                meaning: "引数が誤っている、プロジェクト設定を探すディレクトリが無い、サーバーが失敗した",
            },
        ],
    },
];

/// `--help` の末尾に足す、実行例と終了コードの文章。`name` に当たる `Guide` が無ければpanicする
pub fn after_help(name: Option<&'static str>) -> String {
    let guide = guide(name);
    let mut text = String::from("例:\n");
    for example in guide.examples {
        let _ = writeln!(text, "  {}\n      {}", example.command, example.description);
    }
    if !guide.exit_codes.is_empty() {
        text.push_str("\n終了コード:\n");
        for exit_code in guide.exit_codes {
            let _ = writeln!(text, "  {}  {}", exit_code.code, exit_code.meaning);
        }
    }
    text
}

fn guide(name: Option<&str>) -> &'static Guide {
    GUIDES
        .iter()
        .find(|guide| guide.name == name)
        .unwrap_or_else(|| panic!("`{name:?}` の実行例と終了コードが無い"))
}

/// `--usage` が出すJSON。`command` の定義から、サブコマンド、引数、既定値、取りうる値を読み、`GUIDES` の実行例と終了コードを足す
pub fn command_tree(command: &Command) -> Value {
    let root = guide(None);
    json!({
        "schema_version": 1,
        "name": command.get_name(),
        "version": command.get_version(),
        "about": command.get_about().map(ToString::to_string),
        "args": command.get_arguments().map(argument).collect::<Vec<_>>(),
        "examples": root.examples,
        "commands": command
            .get_subcommands()
            .map(|subcommand| {
                let guide = guide(Some(subcommand.get_name()));
                json!({
                    "name": subcommand.get_name(),
                    "about": subcommand.get_about().map(ToString::to_string),
                    "args": subcommand.get_arguments().map(argument).collect::<Vec<_>>(),
                    "examples": guide.examples,
                    "exit_codes": guide.exit_codes,
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn argument(arg: &Arg) -> Value {
    let takes_value = arg.get_action().takes_values();
    // clapは値を取らないフラグにも `true`・`false` を取りうる値として返すので、値を取る引数の選択肢だけを出す
    let possible_values: Vec<String> = if takes_value {
        arg.get_possible_values()
            .iter()
            .filter(|value| !value.is_hide_set())
            .map(|value| value.get_name().to_string())
            .collect()
    } else {
        Vec::new()
    };
    json!({
        "name": arg.get_id().as_str(),
        "long": arg.get_long().map(|long| format!("--{long}")),
        "positional": arg.is_positional(),
        "takes_value": takes_value,
        "multiple": arg.get_num_args().is_some_and(|range| range.max_values() > 1),
        "default": arg.get_default_values().first().map(|value| value.to_string_lossy()),
        "possible_values": possible_values,
        "help": arg.get_help().map(ToString::to_string),
    })
}
