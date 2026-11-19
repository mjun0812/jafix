mod apply;
mod config;
mod filter;
mod git_diff;
mod hook;
mod js_regex;
mod morph;
mod prose;
mod rules;
mod sentence;
mod text_file;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::apply::{LineIndex, TargetLines};
use crate::config::Configs;
use crate::filter::{Filters, Ignored};
use crate::git_diff::{FileTarget, Selection};
use crate::rules::builtin::Builtin;
use crate::rules::general::Links;
use crate::text_file::{InputEncoding, TextFile};

/// 日本語のテキストファイルを校正する
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 修正ルールに当たる箇所と指摘ルールの指摘を、ファイルを書き換えずに出力する
    Check(CheckArgs),
    /// ファイルの全行に修正ルールを適用して書き戻す
    Fix(Files),
    /// agentのPostToolUse hookの入力を標準入力から読み、今回の編集の行を直して、その行の指摘を標準エラーで返す
    Hook { agent: Agent },
}

#[derive(Args)]
struct CheckArgs {
    #[command(flatten)]
    files: Files,
    /// ステージ済みの内容を読み、HEADとの差分にある行だけを確認する
    #[arg(long, conflicts_with = "diff")]
    staged: bool,
}

#[derive(Args)]
struct Files {
    /// 入力の文字コード。既定ではUTF-8、Shift_JISの順に判定する
    #[arg(long, value_enum, default_value = "auto")]
    encoding: InputEncoding,
    /// 作業ツリーを読み、HEADとの差分にある行だけを対象にする
    #[arg(long)]
    diff: bool,
    files: Vec<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Agent {
    Claude,
    Codex,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Check(args) => run_check(&args.files, args.staged),
        Command::Fix(files) => run_fix(&files),
        Command::Hook { agent } => run_hook(agent),
    }
}

fn select_files(args: &Files, staged: bool) -> Option<Selection> {
    if args.diff || staged {
        return match git_diff::select(&args.files, staged) {
            Ok(selection) => Some(selection),
            Err(error) => {
                eprintln!("jafix: {error}");
                None
            }
        };
    }
    Some(Selection {
        root: PathBuf::new(),
        files: args
            .files
            .iter()
            .map(|path| FileTarget {
                path: path.clone(),
                lines: TargetLines::All,
                staged_blob: None,
            })
            .collect(),
    })
}

/// 箇所が1件以上あればexit 1、読み書きや設定の失敗があればexit 2
fn run_check(args: &Files, staged: bool) -> ExitCode {
    let Some(selection) = select_files(args, staged) else {
        return ExitCode::from(2);
    };
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
    for target in &selection.files {
        let path = selection.root.join(&target.path);
        let config = if staged {
            configs.for_staged_path(&path)
        } else {
            configs.for_path(&path)
        };
        let config = match config {
            Ok(config) => config,
            Err(e) => {
                eprintln!("jafix: {e}");
                failed = true;
                continue;
            }
        };
        let excluded = if staged {
            config.is_staged_path_excluded(&path)
        } else {
            config.is_excluded(&path)
        };
        if excluded {
            continue;
        }
        let file = match &target.staged_blob {
            Some(blob) => git_diff::read_blob(blob).and_then(|bytes| {
                TextFile::decode(bytes, args.encoding).map_err(|error| error.to_string())
            }),
            None => TextFile::read(&path, args.encoding).map_err(|error| error.to_string()),
        };
        let file = match file {
            Ok(file) => file,
            Err(e) => {
                eprintln!("jafix: {}: {e}", target.path.display());
                failed = true;
                continue;
            }
        };
        let src = &file.content;
        let document = prose::parse(src);
        let ignored = Ignored::new(src, &document, config.filters());
        let mut findings =
            rules::replacements(src, &document, config.builtin(), &target.lines, &ignored);
        let fixes = rules::fix_findings(src, &document, config.builtin(), &ignored);
        findings.extend(rules::fixable(&fixes));
        let report_excluded = if staged {
            config.is_staged_report_excluded(&path)
        } else {
            config.is_report_excluded(&path)
        };
        let reports = if report_excluded {
            Vec::new()
        } else {
            rules::report(
                src,
                &document,
                config.builtin(),
                &target.lines,
                &fixes,
                &ignored,
                &Links {
                    file: &path,
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
        let index = LineIndex::new(src);
        for &(start, message, rule) in &lines {
            let location = index.line_col(start);
            if !target.lines.contains(location.0) {
                continue;
            }
            let _ = writeln!(
                stdout,
                "{}",
                location_line(&target.path, location, message, rule)
            );
            found = true;
        }
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
fn run_fix(args: &Files) -> ExitCode {
    let Some(selection) = select_files(args, false) else {
        return ExitCode::from(2);
    };
    let mut configs = match Configs::load() {
        Ok(configs) => configs,
        Err(e) => {
            eprintln!("jafix: {e}");
            return ExitCode::from(2);
        }
    };
    let mut failed = false;
    for target in &selection.files {
        let path = selection.root.join(&target.path);
        let config = match configs.for_path(&path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("jafix: {e}");
                failed = true;
                continue;
            }
        };
        if config.is_excluded(&path) {
            continue;
        }
        if let Err(e) = fix_file(
            &path,
            args.encoding,
            &target.lines,
            config.builtin(),
            config.filters(),
        ) {
            eprintln!("jafix: {}: {e}", target.path.display());
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
            InputEncoding::Utf8,
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
    encoding: InputEncoding,
    lines: &TargetLines,
    builtin: &Builtin,
    filters: &Filters,
) -> std::io::Result<Fixed> {
    let file = TextFile::read(path, encoding)?;
    let src = &file.content;
    let document = prose::parse(src);
    let ignored = Ignored::new(src, &document, filters);
    let replacements = rules::replacements(src, &document, builtin, lines, &ignored);
    let mut fixed = apply::apply(src, &replacements, lines);
    // 利用者の置き換えと組み込みの修正ルールは改行を足し引きしないので、対象行は直した後の内容でも同じ行番号である
    let mut document = if fixed == *src {
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
            if fixed != *src {
                file.write(path, &fixed)?;
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
