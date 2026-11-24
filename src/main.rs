mod apply;
mod config;
mod diagnostics;
mod filter;
mod git_diff;
mod hook;
mod js_regex;
mod mcp;
mod morph;
mod prose;
mod rules;
mod sentence;
mod text_file;

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use similar::TextDiff;

use crate::apply::{LineIndex, TargetLines};
use crate::config::{Config, Configs};
use crate::diagnostics::Diagnostic;
use crate::filter::Ignored;
use crate::git_diff::{FileTarget, Selection};
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
    /// 対象行に修正ルールを適用して書き戻す
    Fix(FixArgs),
    /// agentのPostToolUse hookの入力を標準入力から読み、今回の編集の行を直して、その行の指摘を標準エラーで返す
    Hook { agent: Agent },
    /// 標準入出力を使うMCPサーバーを起動する
    Mcp {
        /// プロジェクト設定を探すディレクトリ。既定値はカレントディレクトリ
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
}

#[derive(Args)]
struct CheckArgs {
    #[command(flatten)]
    files: Files,
    /// ステージ済みの内容を読み、HEADとの差分にある行だけを確認する
    #[arg(long, conflicts_with = "diff")]
    staged: bool,
    /// 指摘の出力形式
    #[arg(long, value_enum, default_value = "text")]
    output_format: OutputFormat,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Args)]
struct FixArgs {
    #[command(flatten)]
    files: Files,
    /// 修正したファイルのパスを標準出力に1行ずつ表示する
    #[arg(long, conflicts_with = "show_diff")]
    list: bool,
    /// 修正したファイル数と変更の無かったファイル数を標準エラーに表示する
    #[arg(long)]
    summary: bool,
    /// 各ファイルの処理時間を秒単位で標準エラーに表示する
    #[arg(long)]
    timing: bool,
    /// 修正前後の差分を標準出力に表示する
    #[arg(long)]
    show_diff: bool,
    /// 修正結果を計算するだけで、ファイルには書き込まない
    #[arg(long)]
    dry_run: bool,
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
        Command::Check(args) => run_check(&args),
        Command::Fix(files) => run_fix(&files),
        Command::Hook { agent } => run_hook(agent),
        Command::Mcp { project_dir } => match mcp::serve(&project_dir) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("jafix: {error}");
                ExitCode::from(2)
            }
        },
    }
}

fn select_files(args: &Files, staged: bool) -> Result<Selection, String> {
    if args.diff || staged {
        return git_diff::select(&args.files, staged);
    }
    Ok(Selection {
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

#[derive(Serialize)]
struct CheckOutput {
    schema_version: u8,
    checked_files: usize,
    findings: Vec<FileDiagnostic>,
    errors: Vec<CheckError>,
}

#[derive(Serialize)]
struct FileDiagnostic {
    path: String,
    #[serde(flatten)]
    diagnostic: Diagnostic,
}

#[derive(Serialize)]
struct CheckError {
    path: Option<String>,
    message: String,
}

/// 箇所が1件以上あればexit 1、引数・読み書き・設定の失敗があればexit 2
fn run_check(args: &CheckArgs) -> ExitCode {
    let mut output = CheckOutput {
        schema_version: 1,
        checked_files: 0,
        findings: Vec::new(),
        errors: Vec::new(),
    };
    if let Err(message) = check_files(args, &mut output) {
        output.errors.push(CheckError {
            path: None,
            message,
        });
    }
    let mut stdout = std::io::stdout().lock();
    match args.output_format {
        OutputFormat::Text => {
            for FileDiagnostic {
                path,
                diagnostic: d,
            } in &output.findings
            {
                let _ = writeln!(
                    stdout,
                    "{}",
                    location_line(path, (d.line, d.column), &d.message, d.rule_id)
                );
            }
            for error in &output.errors {
                if let Some(path) = &error.path {
                    eprintln!("jafix: {path}: {}", error.message);
                } else {
                    eprintln!("jafix: {}", error.message);
                }
            }
        }
        OutputFormat::Json => {
            if let Err(error) = serde_json::to_writer(&mut stdout, &output)
                .map_err(std::io::Error::from)
                .and_then(|()| writeln!(stdout))
            {
                eprintln!("jafix: {error}");
                return ExitCode::from(2);
            }
        }
    }
    match (output.errors.is_empty(), output.findings.is_empty()) {
        (false, _) => ExitCode::from(2),
        (true, false) => ExitCode::from(1),
        (true, true) => ExitCode::SUCCESS,
    }
}

fn check_files(args: &CheckArgs, output: &mut CheckOutput) -> Result<(), String> {
    if args.files.files.is_empty() && !args.files.diff && !args.staged {
        return Err("チェックするファイル、--diff、または --staged を指定してください".into());
    }
    let selection = select_files(&args.files, args.staged)?;
    let mut configs = Configs::load()?;
    for target in &selection.files {
        let path = selection.root.join(&target.path);
        let display_path = target.path.display().to_string();
        let config = if args.staged {
            configs.for_staged_path(&path)
        } else {
            configs.for_path(&path)
        };
        let config = match config {
            Ok(config) => config,
            Err(message) => {
                output.errors.push(CheckError {
                    path: None,
                    message,
                });
                continue;
            }
        };
        let excluded = if args.staged {
            config.is_staged_path_excluded(&path)
        } else {
            config.is_excluded(&path)
        };
        if excluded {
            continue;
        }
        let file = match &target.staged_blob {
            Some(blob) => git_diff::read_blob(blob).and_then(|bytes| {
                TextFile::decode(bytes, args.files.encoding).map_err(|error| error.to_string())
            }),
            None => TextFile::read(&path, args.files.encoding).map_err(|error| error.to_string()),
        };
        let file = match file {
            Ok(file) => file,
            Err(message) => {
                output.errors.push(CheckError {
                    path: Some(display_path),
                    message,
                });
                continue;
            }
        };
        let report_excluded = if args.staged {
            config.is_staged_report_excluded(&path)
        } else {
            config.is_report_excluded(&path)
        };
        let links = Links {
            file: &path,
            urls: if report_excluded {
                None
            } else {
                config.url_checker()
            },
        };
        let findings = diagnostics::check(
            &file.content,
            config,
            &target.lines,
            Some(&links),
            report_excluded,
        );
        output.checked_files += 1;
        output
            .findings
            .extend(findings.into_iter().map(|diagnostic| FileDiagnostic {
                path: display_path.clone(),
                diagnostic,
            }));
    }
    Ok(())
}

/// `check` とhookが箇所と指摘を出す `<path>:<line>:<column>: [<rule-id>] <message>` の行
fn location_line(path: &str, (line, column): (usize, usize), message: &str, rule: &str) -> String {
    format!("{path}:{line}:{column}: [{rule}] {message}")
}

/// 読み書きや設定の失敗があればexit 2
fn run_fix(args: &FixArgs) -> ExitCode {
    let selection = match select_files(&args.files, false) {
        Ok(selection) => selection,
        Err(error) => {
            eprintln!("jafix: {error}");
            return ExitCode::from(2);
        }
    };
    let mut configs = match Configs::load() {
        Ok(configs) => configs,
        Err(e) => {
            eprintln!("jafix: {e}");
            return ExitCode::from(2);
        }
    };
    let mut failed = false;
    let mut changed = 0;
    let mut unchanged = 0;
    let mut stdout = std::io::stdout().lock();
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
        let started = args.timing.then(Instant::now);
        let (fixed, original) = match fix_file(
            &path,
            args.files.encoding,
            &target.lines,
            config,
            args.dry_run,
        ) {
            Ok(fixed) => fixed,
            Err(e) => {
                eprintln!("jafix: {}: {e}", target.path.display());
                failed = true;
                continue;
            }
        };
        if let Some(started) = started {
            eprintln!(
                "{}: {:.3} s",
                target.path.display(),
                started.elapsed().as_secs_f64()
            );
        }
        let Some(original) = original else {
            unchanged += 1;
            continue;
        };
        changed += 1;
        if args.show_diff {
            let name = target.path.display().to_string();
            let diff = TextDiff::from_lines(original.as_str(), fixed.content.as_str());
            let _ = diff
                .unified_diff()
                .header(&name, &name)
                .to_writer(&mut stdout);
        } else if args.list {
            let _ = writeln!(stdout, "{}", target.path.display());
        }
    }
    if args.summary {
        let label = if args.dry_run {
            "修正予定"
        } else {
            "修正"
        };
        eprintln!("{label}: {changed}ファイル、変更なし: {unchanged}ファイル");
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
        let (fixed, _) = match fix_file(
            &target.path,
            InputEncoding::Utf8,
            &target.lines,
            config,
            false,
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
            Some(&Links {
                file: &target.path,
                urls: None,
            }),
        );
        for report in reports_in_file {
            let (line, column) = index.line_col(report.range.start);
            if !target.lines.contains(line) {
                continue;
            }
            reports.push(location_line(
                &target.path.to_string_lossy(),
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

/// `document`・`fixes`・`ignored` は修正後の `content` のものである
struct Fixed {
    content: String,
    document: prose::Document,
    fixes: Vec<rules::FixRuleFinding>,
    ignored: Ignored,
}

fn fix_file(
    path: &Path,
    encoding: InputEncoding,
    lines: &TargetLines,
    config: &Config,
    dry_run: bool,
) -> std::io::Result<(Fixed, Option<String>)> {
    let file = TextFile::read(path, encoding)?;
    let fixed = fix_content(&file.content, lines, config);
    if fixed.content == file.content {
        return Ok((fixed, None));
    }
    let encoded = file.encode(&fixed.content)?;
    if !dry_run {
        fs::write(path, encoded)?;
    }
    Ok((fixed, Some(file.content)))
}

/// 利用者の置き換えを1回当て、組み込みの修正ルールは内容が変わらなくなるまで繰り返し当てる。
/// filterの外す範囲はその時点の内容から求め直す。
fn fix_content(src: &str, lines: &TargetLines, config: &Config) -> Fixed {
    let builtin = config.builtin();
    let filters = config.filters();
    let document = prose::parse(src);
    let ignored = Ignored::new(src, &document, filters);
    let replacements = rules::replacements(src, &document, builtin, lines, &ignored);
    let mut fixed = apply::apply(src, &replacements, lines);
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
            return Fixed {
                content: fixed,
                document,
                fixes,
                ignored,
            };
        }
        document = prose::parse(&next);
        fixed = next;
    }
}
