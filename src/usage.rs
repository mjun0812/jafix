//! コマンドの実行例と終了コードの意味。`--help` の末尾と `--usage` のJSONの両方が、ここの同じ表を使う。

use clap::Command;
use serde::Serialize;

/// 1つのコマンドの実行例と終了コード。`name` が `None` なら `jafix` そのものである
#[expect(dead_code, reason = "skeleton: not implemented")]
pub struct Guide {
    pub name: Option<&'static str>,
    pub examples: &'static [Example],
    pub exit_codes: &'static [ExitCodeMeaning],
}

#[expect(dead_code, reason = "skeleton: not implemented")]
#[derive(Serialize)]
pub struct Example {
    pub command: &'static str,
    pub description: &'static str,
}

#[expect(dead_code, reason = "skeleton: not implemented")]
#[derive(Serialize)]
pub struct ExitCodeMeaning {
    pub code: u8,
    pub meaning: &'static str,
}

/// `jafix` そのものと、すべてのサブコマンドの `Guide`
#[expect(dead_code, reason = "skeleton: not implemented")]
pub const GUIDES: &[Guide] = &[];

/// `--help` の末尾に足す、実行例と終了コードの文章。`name` に当たる `Guide` が無ければpanicする
pub fn after_help(name: Option<&'static str>) -> String {
    let _ = name;
    String::new()
}

/// `--usage` が出すJSON。`command` の定義から、サブコマンド、引数、既定値、取りうる値を読み、`GUIDES` の実行例と終了コードを足す
pub fn command_tree(command: &Command) -> serde_json::Value {
    let _ = command;
    todo!("skeleton: not implemented")
}
