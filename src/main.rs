mod apply;
mod config;
mod hook;
mod prose;
mod rules;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::apply::{LineIndex, TargetLines};

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
    let config = match config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("jafix: {e}");
            return ExitCode::from(2);
        }
    };
    let mut stdout = std::io::stdout().lock();
    let mut failed = false;
    let mut found = false;
    for path in files {
        if config.is_excluded(path) {
            continue;
        }
        let src = match fs::read_to_string(path) {
            Ok(src) => src,
            Err(e) => {
                eprintln!("jafix: {}: {e}", path.display());
                failed = true;
                continue;
            }
        };
        let findings = rules::find(&src, &prose::prose_ranges(&src));
        let index = LineIndex::new(&src);
        for finding in &findings {
            let (line, column) = index.line_col(finding.range.start);
            let _ = writeln!(
                stdout,
                "{}:{line}:{column}: {} [{}]",
                path.display(),
                finding.rule.message(),
                finding.rule.name()
            );
        }
        found |= !findings.is_empty();
    }
    match (failed, found) {
        (true, _) => ExitCode::from(2),
        (false, true) => ExitCode::from(1),
        (false, false) => ExitCode::SUCCESS,
    }
}

/// 読み書きや設定の失敗があればexit 2
fn run_fix(files: &[PathBuf]) -> ExitCode {
    let config = match config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("jafix: {e}");
            return ExitCode::from(2);
        }
    };
    let mut failed = false;
    for path in files {
        if config.is_excluded(path) {
            continue;
        }
        let result = fs::read_to_string(path).and_then(|src| {
            let findings = rules::find(&src, &prose::prose_ranges(&src));
            let fixed = apply::apply(&src, &findings, &TargetLines::All);
            if fixed == src {
                return Ok(());
            }
            fs::write(path, fixed)
        });
        if let Err(e) = result {
            eprintln!("jafix: {}: {e}", path.display());
            failed = true;
        }
    }
    if failed {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

/// 標準出力には何も書かない。決められなかった理由や失敗があれば標準エラーに書いてexit 1
fn run_hook(agent: Agent) -> ExitCode {
    let _ = agent;
    todo!("skeleton: not implemented")
}
