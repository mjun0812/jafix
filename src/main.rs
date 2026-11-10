mod apply;
mod config;
mod hook;
mod prose;
mod rules;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

/// AI agentが書いた日本語の文章を直す
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 修正ルールに当たる箇所を、ファイルを書き換えずに出力する
    Check { files: Vec<PathBuf> },
    /// ファイルの全行に修正ルールを適用して書き戻す
    Fix { files: Vec<PathBuf> },
    /// agentのPostToolUse hookの入力を標準入力から読み、今回の編集の行を直す
    Hook { agent: Agent },
}

#[derive(Clone, Copy, ValueEnum)]
enum Agent {
    Claude,
    Codex,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Check { files } => run_check(&files),
        Command::Fix { files } => run_fix(&files),
        Command::Hook { agent } => run_hook(agent),
    }
}

/// 箇所が1件以上あればexit 1、読み書きや設定の失敗があればexit 2
fn run_check(files: &[PathBuf]) -> ExitCode {
    let _ = files;
    todo!("skeleton: not implemented")
}

/// 読み書きや設定の失敗があればexit 2
fn run_fix(files: &[PathBuf]) -> ExitCode {
    let _ = files;
    todo!("skeleton: not implemented")
}

/// 標準出力には何も書かない。決められなかった理由や失敗があれば標準エラーに書いてexit 1
fn run_hook(agent: Agent) -> ExitCode {
    let _ = agent;
    todo!("skeleton: not implemented")
}
