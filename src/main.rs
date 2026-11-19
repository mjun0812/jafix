mod apply;
mod config;
mod filter;
mod hook;
mod js_regex;
mod morph;
mod prose;
mod rules;
mod sentence;

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::apply::{LineIndex, TargetLines};
use crate::config::Configs;
use crate::filter::{Filters, Ignored};
use crate::rules::builtin::Builtin;
use crate::rules::general::Links;

/// AI agentが書いた日本語の文章を直す
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 修正ルールに当たる箇所と指摘ルールの指摘を、ファイルを書き換えずに出力する
    Check { files: Vec<PathBuf> },
    /// ファイルの全行に修正ルールを適用して書き戻す
    Fix { files: Vec<PathBuf> },
    /// agentのPostToolUse hookの入力を標準入力から読み、今回の編集の行を直して、その行の指摘を標準エラーで返す
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
    let mut configs = match Configs::load() {
        Ok(configs) => configs,
        Err(e) => {
            eprintln!("jafix: {e}");
            return ExitCode::from(2);
        }
    };
    let mut stdout = std::io::stdout().lock();
    let mut failed = false;
    let mut found = false;
    for path in files {
        let config = match configs.for_path(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("jafix: {e}");
                failed = true;
                continue;
            }
        };
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
        let document = prose::parse(&src);
        let ignored = Ignored::new(&src, &document, config.filters());
        let mut findings = rules::replacements(
            &src,
            &document,
            config.builtin(),
            &TargetLines::All,
            &ignored,
        );
        let fixes = rules::fix_findings(&src, &document, config.builtin(), &ignored);
        findings.extend(rules::fixable(&fixes));
        let reports = if config.is_report_excluded(path) {
            Vec::new()
        } else {
            rules::report(
                &src,
                &document,
                config.builtin(),
                &TargetLines::All,
                &fixes,
                &ignored,
                &Links {
                    file: path,
                    urls: config.url_checker(),
                },
            )
        };
        let mut lines: Vec<(usize, &str, &str)> = findings
            .iter()
            .map(|finding| {
                (
                    finding.position,
                    finding.message.as_ref(),
                    finding.rule.name(),
                )
            })
            .chain(
                reports
                    .iter()
                    .map(|report| (report.range.start, report.message.as_ref(), report.rule)),
            )
            .collect();
        lines.sort_by_key(|&(start, ..)| start);
        let index = LineIndex::new(&src);
        for &(start, message, rule) in &lines {
            let _ = writeln!(
                stdout,
                "{}",
                location_line(path, index.line_col(start), message, rule)
            );
        }
        found |= !lines.is_empty();
    }
    match (failed, found) {
        (true, _) => ExitCode::from(2),
        (false, true) => ExitCode::from(1),
        (false, false) => ExitCode::SUCCESS,
    }
}

/// `check` とhookが箇所と指摘を出す `<path>:<line>:<column>: <message> [<rule-id>]` の行
fn location_line(path: &Path, (line, column): (usize, usize), message: &str, rule: &str) -> String {
    format!("{}:{line}:{column}: {message} [{rule}]", path.display())
}

/// 読み書きや設定の失敗があればexit 2
fn run_fix(files: &[PathBuf]) -> ExitCode {
    let mut configs = match Configs::load() {
        Ok(configs) => configs,
        Err(e) => {
            eprintln!("jafix: {e}");
            return ExitCode::from(2);
        }
    };
    let mut failed = false;
    for path in files {
        let config = match configs.for_path(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("jafix: {e}");
                failed = true;
                continue;
            }
        };
        if config.is_excluded(path) {
            continue;
        }
        if let Err(e) = fix_file(path, &TargetLines::All, config.builtin(), config.filters()) {
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

/// 標準出力には何も書かない。今回変えた行に指摘があれば、指摘と、決められなかった理由や失敗を標準エラーに書いてexit 2。
/// 指摘が無く、決められなかった理由や失敗があれば標準エラーに書いてexit 1
fn run_hook(agent: Agent) -> ExitCode {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("jafix: 標準入力を読めない: {e}");
        return ExitCode::FAILURE;
    }
    let mut configs = match Configs::load() {
        Ok(configs) => configs,
        Err(e) => {
            eprintln!("jafix: {e}");
            return ExitCode::FAILURE;
        }
    };
    let is_excluded = |path: &Path| {
        configs
            .for_path(path)
            .map(|config| config.is_excluded(path))
    };
    let targets = match agent {
        Agent::Claude => hook::claude::targets(&input, is_excluded),
        Agent::Codex => hook::codex::targets(&input, is_excluded),
    };
    let mut errors: Vec<String> = targets
        .errors
        .iter()
        .map(|error| format!("jafix: {error}"))
        .collect();
    let mut reports = Vec::new();
    for target in &targets.files {
        let config = match configs.for_path(&target.path) {
            Ok(config) => config,
            Err(e) => {
                errors.push(format!("jafix: {e}"));
                continue;
            }
        };
        let fixed = match fix_file(
            &target.path,
            &target.lines,
            config.builtin(),
            config.filters(),
        ) {
            Ok(fixed) => fixed,
            Err(e) => {
                errors.push(format!("jafix: {}: {e}", target.path.display()));
                continue;
            }
        };
        if config.is_report_excluded(&target.path) {
            continue;
        }
        // 修正ルールは改行を足し引きしないので、対象行は直した後の内容でも同じ行番号である
        let index = LineIndex::new(&fixed.content);
        let reports_in_file = rules::report(
            &fixed.content,
            &fixed.document,
            config.builtin(),
            &target.lines,
            &fixed.fixes,
            &fixed.ignored,
            &Links {
                file: &target.path,
                urls: None,
            },
        );
        for report in reports_in_file {
            let (line, column) = index.line_col(report.range.start);
            if !target.lines.contains(line) {
                continue;
            }
            reports.push(location_line(
                &target.path,
                (line, column),
                &report.message,
                report.rule,
            ));
        }
    }
    if reports.is_empty() {
        for error in &errors {
            eprintln!("{error}");
        }
        return if errors.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    // PostToolUseのexit 2の標準エラーはagentへ届く。Codexでは元のtoolの出力がこの文面に置き換わるので、
    // 編集が適用済みであることを先に伝える
    eprintln!(
        "jafix: 今回変えた行に指摘がある。編集は適用済み。指摘を直すか、誤検知なら直さずに進めてよい"
    );
    for line in reports.iter().chain(&errors) {
        eprintln!("{line}");
    }
    ExitCode::from(2)
}

/// `fix_file` が直した結果。`document`・`fixes`・`ignored` は `content` のものである
struct Fixed {
    content: String,
    document: prose::Document,
    fixes: Vec<rules::FixRuleFinding>,
    ignored: Ignored,
}

/// 対象行に、利用者の置き換えを1回当て、その結果に組み込みの修正ルールを内容が変わらなくなるまで繰り返し当てた
/// 内容を返す。`filters` の外す範囲は、その時点の内容から求め直す。変更が無ければ書き戻さない
fn fix_file(
    path: &Path,
    lines: &TargetLines,
    builtin: &Builtin,
    filters: &Filters,
) -> std::io::Result<Fixed> {
    let src = fs::read_to_string(path)?;
    let document = prose::parse(&src);
    let ignored = Ignored::new(&src, &document, filters);
    let replacements = rules::replacements(&src, &document, builtin, lines, &ignored);
    let mut fixed = apply::apply(&src, &replacements, lines);
    // 利用者の置き換えと組み込みの修正ルールは改行を足し引きしないので、対象行は直した後の内容でも同じ行番号である
    let mut document = if fixed == src {
        document
    } else {
        prose::parse(&fixed)
    };
    // ある修正ルールの結果に別の修正ルールが当たることがある (全角英字を半角にした後の `日本語 Rust` の空白)。
    // 組み込みの修正ルールは文字を消すか字種を一方向に変え、逆向きの組は設定の不正になるので、繰り返しは止まる
    loop {
        let ignored = Ignored::new(&fixed, &document, filters);
        let fixes = rules::fix_findings(&fixed, &document, builtin, &ignored);
        let next = apply::apply(&fixed, &rules::fixable(&fixes), lines);
        if next == fixed {
            if fixed != src {
                fs::write(path, &fixed)?;
            }
            return Ok(Fixed {
                content: fixed,
                document,
                fixes,
                ignored,
            });
        }
        document = prose::parse(&next);
        fixed = next;
    }
}
