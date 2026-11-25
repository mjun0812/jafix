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
mod usage;

use std::fs;
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use schemars::JsonSchema;
use serde::Serialize;
use similar::TextDiff;

use crate::apply::{LineIndex, TargetLines};
use crate::config::{Config, Configs};
use crate::diagnostics::Diagnostic;
use crate::filter::Ignored;
use crate::git_diff::{FileTarget, Selection};
use crate::rules::RuleId;
use crate::rules::builtin::Preset;
use crate::rules::general::Links;
use crate::text_file::{InputEncoding, TextFile};

/// 日本語のテキストファイルを校正する
#[derive(Parser)]
#[command(
    version,
    arg_required_else_help = true,
    args_conflicts_with_subcommands = true,
    after_help = usage::after_help(None)
)]
struct Cli {
    /// コマンド、引数、終了コード、実行例の一覧をJSONで標準出力に書く
    #[arg(long, exclusive = true)]
    usage: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// 修正ルールに当たる箇所と指摘ルールの指摘を、ファイルを書き換えずに出力する
    #[command(after_help = usage::after_help(Some("check")))]
    Check(CheckArgs),
    /// 対象行に修正ルールを適用して書き戻す
    #[command(after_help = usage::after_help(Some("fix")))]
    Fix(FixArgs),
    /// 組み込みのルールのIDと、設定を当てた後に有効かどうかを出力する
    #[command(after_help = usage::after_help(Some("rules")))]
    Rules(RulesArgs),
    /// agentのPostToolUse hookの入力を標準入力から読み、今回の編集の行を直して、その行の指摘を標準エラーで返す
    #[command(after_help = usage::after_help(Some("hook")))]
    Hook { agent: Agent },
    /// 標準入出力を使うMCPサーバーを起動する
    #[command(after_help = usage::after_help(Some("mcp")))]
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
    /// `--output-format json` の出力のJSON Schemaを標準出力に書いて終える
    #[arg(long, exclusive = true)]
    schema: bool,
}

#[derive(Args)]
struct RulesArgs {
    /// 設定を探す起点のファイルかディレクトリ。既定値はカレントディレクトリ
    #[arg(default_value = ".")]
    path: PathBuf,
    /// 一覧の出力形式
    #[arg(long, value_enum, default_value = "text")]
    output_format: OutputFormat,
    /// `--output-format json` の出力のJSON Schemaを標準出力に書いて終える
    #[arg(long, exclusive = true)]
    schema: bool,
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
    let cli = Cli::parse();
    if cli.usage {
        return print_json(&usage::command_tree(&Cli::command()));
    }
    // `arg_required_else_help` が引数なしの実行をclapの中で終えるので、ここへ来るのは `--usage` かサブコマンドのどちらかである
    let Some(command) = cli.command else {
        return ExitCode::from(2);
    };
    match command {
        Command::Check(args) if args.schema => print_json(&schemars::schema_for!(CheckOutput)),
        Command::Check(args) => run_check(&args),
        Command::Fix(files) => run_fix(&files),
        Command::Rules(args) if args.schema => print_json(&schemars::schema_for!(RulesOutput)),
        Command::Rules(args) => run_rules(&args),
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

/// JSONを1行で標準出力に書く。書けなければexit 2
fn print_json(value: &impl Serialize) -> ExitCode {
    let mut stdout = std::io::stdout().lock();
    match serde_json::to_writer(&mut stdout, value)
        .map_err(std::io::Error::from)
        .and_then(|()| writeln!(stdout))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("jafix: {error}");
            ExitCode::from(2)
        }
    }
}

/// `jafix rules --output-format json` の出力
#[derive(Serialize, JsonSchema)]
struct RulesOutput {
    schema_version: u8,
    rules: Vec<RuleStatus>,
}

#[derive(Serialize, JsonSchema)]
struct RuleStatus {
    /// 設定の `[builtin]` のキーと、指摘の `[<rule-id>]` に使うID
    id: &'static str,
    /// 共通設定とプロジェクト設定を当てた後に有効か
    enabled: bool,
    /// このルールを束ねるpresetの名前
    presets: Vec<&'static str>,
}

/// パスが存在しないか、設定が不正ならexit 2
fn run_rules(args: &RulesArgs) -> ExitCode {
    // `for_directory` は渡されたパスの親をそのままたどるので、相対パスの `.` を先に絶対パスへ直す
    let path = match args.path.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("jafix: {}: {error}", args.path.display());
            return ExitCode::from(2);
        }
    };
    let mut configs = match Configs::load() {
        Ok(configs) => configs,
        Err(error) => {
            eprintln!("jafix: {error}");
            return ExitCode::from(2);
        }
    };
    let config = if path.is_dir() {
        configs.for_directory(&path)
    } else {
        configs.for_path(&path)
    };
    let builtin = match config {
        Ok(config) => config.builtin(),
        Err(error) => {
            eprintln!("jafix: {error}");
            return ExitCode::from(2);
        }
    };
    let output = RulesOutput {
        schema_version: 1,
        rules: RuleId::ALL
            .into_iter()
            .map(|id| RuleStatus {
                id: id.name(),
                enabled: builtin.is_enabled(id),
                presets: Preset::ALL
                    .into_iter()
                    .filter(|preset| preset.rules().iter().any(|&(rule, _)| rule == id))
                    .map(Preset::name)
                    .collect(),
            })
            .collect(),
    };
    if let OutputFormat::Json = args.output_format {
        return print_json(&output);
    }
    let mut stdout = std::io::stdout().lock();
    for rule in &output.rules {
        let enabled = if rule.enabled { "有効" } else { "無効" };
        let presets = if rule.presets.is_empty() {
            "-".to_string()
        } else {
            rule.presets.join(",")
        };
        let _ = writeln!(stdout, "{}\t{enabled}\t{presets}", rule.id);
    }
    ExitCode::SUCCESS
}

/// 指摘のパスと差分の名前で、標準入力を表す名前
const STDIN_NAME: &str = "<stdin>";

/// ファイルの引数が `-` だけなら、標準入力を校正する。`-` をほかのファイルやGitの差分の選択と並べたら理由を返す
fn reads_stdin(args: &Files, staged: bool) -> Result<bool, String> {
    if !args.files.iter().any(|path| path.as_os_str() == "-") {
        return Ok(false);
    }
    if args.files.len() > 1 {
        return Err("標準入力を表す `-` は、ほかのファイルと一緒に指定できない".into());
    }
    if args.diff || staged {
        let option = if args.diff { "--diff" } else { "--staged" };
        return Err(format!("標準入力を表す `-` に {option} は指定できない"));
    }
    Ok(true)
}

/// 標準入力を最後まで読み、読んだバイト列と、文字コードを判定した内容を返す
fn read_stdin(encoding: InputEncoding) -> std::io::Result<(Vec<u8>, TextFile)> {
    let mut bytes = Vec::new();
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        // 端末からの入力は、Windowsのコンソールで文字コードを変換する `Stdin` で読む
        stdin.lock().read_to_end(&mut bytes)?;
    } else {
        duplicate(&stdin)?.read_to_end(&mut bytes)?;
    }
    let file = TextFile::decode(bytes.clone(), encoding)?;
    Ok((bytes, file))
}

/// 標準入力か標準出力のハンドルを複製した `File`。`std::io::Stdin` は読めないハンドルを入力の終わりとして、
/// `std::io::Stdout` は書けないハンドルへの書き込みを成功として扱うので、失敗を受け取るには複製を読み書きする
#[cfg(unix)]
fn duplicate(stream: &impl std::os::fd::AsFd) -> std::io::Result<fs::File> {
    Ok(fs::File::from(stream.as_fd().try_clone_to_owned()?))
}

#[cfg(windows)]
fn duplicate(stream: &impl std::os::windows::io::AsHandle) -> std::io::Result<fs::File> {
    Ok(fs::File::from(stream.as_handle().try_clone_to_owned()?))
}

/// 標準出力に書く。書けなければ失敗を返す
fn write_stdout(bytes: &[u8]) -> std::io::Result<()> {
    let stdout = std::io::stdout();
    // Windowsのコンソールは、文字として書く `Stdout` でないと日本語を表示できない
    #[cfg(windows)]
    if stdout.is_terminal() {
        let mut stdout = stdout.lock();
        return stdout.write_all(bytes).and_then(|()| stdout.flush());
    }
    duplicate(&stdout)?.write_all(bytes)
}

/// 標準入力の本文に使う設定。ファイルが無いので、カレントディレクトリからプロジェクト設定を探す
fn current_dir_config(configs: &mut Configs) -> Result<&Config, String> {
    // `for_directory` は渡されたパスの親をそのままたどるので、相対パスの `.` を先に絶対パスへ直す
    let dir = Path::new(".")
        .canonicalize()
        .map_err(|error| format!(".: {error}"))?;
    configs.for_directory(&dir)
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

#[derive(Serialize, JsonSchema)]
struct CheckOutput {
    schema_version: u8,
    checked_files: usize,
    findings: Vec<FileDiagnostic>,
    errors: Vec<CheckError>,
}

#[derive(Serialize, JsonSchema)]
struct FileDiagnostic {
    path: String,
    #[serde(flatten)]
    diagnostic: Diagnostic,
}

#[derive(Serialize, JsonSchema)]
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
            let code = print_json(&output);
            if code != ExitCode::SUCCESS {
                return code;
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
    if reads_stdin(&args.files, args.staged)? {
        return check_stdin(args.files.encoding, output);
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

/// ファイルを入力とする `exclude`・`report-exclude` と、リンク先を調べる `no-dead-link` は当てない
fn check_stdin(encoding: InputEncoding, output: &mut CheckOutput) -> Result<(), String> {
    let mut configs = Configs::load()?;
    let config = current_dir_config(&mut configs)?;
    let file = match read_stdin(encoding) {
        Ok((_, file)) => file,
        Err(error) => {
            output.errors.push(CheckError {
                path: Some(STDIN_NAME.into()),
                message: error.to_string(),
            });
            return Ok(());
        }
    };
    let findings = diagnostics::check(&file.content, config, &TargetLines::All, None, false);
    output.checked_files = 1;
    output
        .findings
        .extend(findings.into_iter().map(|diagnostic| FileDiagnostic {
            path: STDIN_NAME.into(),
            diagnostic,
        }));
    Ok(())
}

/// `check` とhookが箇所と指摘を出す `<path>:<line>:<column>: [<rule-id>] <message>` の行
fn location_line(path: &str, (line, column): (usize, usize), message: &str, rule: &str) -> String {
    format!("{path}:{line}:{column}: [{rule}] {message}")
}

/// 読み書きや設定の失敗があればexit 2
fn run_fix(args: &FixArgs) -> ExitCode {
    match reads_stdin(&args.files, false) {
        Ok(false) => {}
        Ok(true) => {
            return match fix_stdin(args) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("jafix: {error}");
                    ExitCode::from(2)
                }
            };
        }
        Err(error) => {
            eprintln!("jafix: {error}");
            return ExitCode::from(2);
        }
    }
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
            print_elapsed(target.path.display(), started);
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
        print_summary(args.dry_run, changed, unchanged);
    }
    if failed {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_elapsed(name: impl std::fmt::Display, started: Instant) {
    eprintln!("{name}: {:.3} s", started.elapsed().as_secs_f64());
}

fn print_summary(dry_run: bool, changed: usize, unchanged: usize) {
    let label = if dry_run { "修正予定" } else { "修正" };
    eprintln!("{label}: {changed}ファイル、変更なし: {unchanged}ファイル");
}

/// 修正後の本文を標準出力に書く。`--dry-run` なら本文を書かず、`--show-diff` も付けば差分だけを書く。
/// 標準出力に書くものを本文か差分のどちらか1つに保つため、`--list` と、`--dry-run` の無い `--show-diff` は拒む
fn fix_stdin(args: &FixArgs) -> Result<(), String> {
    if args.list {
        return Err("標準入力を表す `-` に --list は指定できない".into());
    }
    if args.show_diff && !args.dry_run {
        return Err(
            "標準入力を表す `-` に --show-diff を指定するときは、--dry-run も指定する".into(),
        );
    }
    let mut configs = Configs::load()?;
    let config = current_dir_config(&mut configs)?;
    // ファイルを渡したときの「読み込みから保存まで」に合わせて、読み始める前から書き終えた後までを計る
    let started = args.timing.then(Instant::now);
    let (bytes, file) =
        read_stdin(args.files.encoding).map_err(|error| format!("{STDIN_NAME}: {error}"))?;
    let fixed = fix_content(&file.content, &TargetLines::All, config);
    let changed = fixed.content != file.content;
    let to_terminal = std::io::stdout().is_terminal();
    // パイプやファイルへは入力と同じ文字コードで書くので、その文字コードで表せない修正結果は `--dry-run` でも失敗にする。
    // 端末には文字として書く。Shift_JISのバイト列は、UTF-8の端末では読めず、Windowsのコンソールでは書き込みが拒まれる
    let encoded = if changed && !to_terminal {
        Some(
            file.encode(&fixed.content)
                .map_err(|error| format!("{STDIN_NAME}: {error}"))?,
        )
    } else {
        None
    };
    let written = if args.dry_run {
        if args.show_diff && changed {
            let diff = TextDiff::from_lines(file.content.as_str(), fixed.content.as_str())
                .unified_diff()
                .header(STDIN_NAME, STDIN_NAME)
                .to_string();
            write_stdout(diff.as_bytes())
        } else {
            Ok(())
        }
    } else if to_terminal {
        write_stdout(fixed.content.as_bytes())
    } else {
        // `--encoding shift-jis` で読んだ入力には符号化し直すと元と違うバイト列になるものがあるので、変わっていなければ読んだバイト列を書く
        write_stdout(encoded.as_deref().unwrap_or(&bytes))
    };
    match written {
        // `| head` のように読み手が先に閉じたときは、フィルターとして正常なので何も出さずに終える。
        // 全文を書き込めていないので、時間も修正した件数も表示しない
        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => return Ok(()),
        Err(error) => return Err(error.to_string()),
        Ok(()) => {}
    }
    if let Some(started) = started {
        print_elapsed(STDIN_NAME, started);
    }
    if args.summary {
        print_summary(args.dry_run, usize::from(changed), usize::from(!changed));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Cli;
    use crate::usage::GUIDES;

    #[test]
    fn every_example_parses() {
        for example in GUIDES.iter().flat_map(|guide| guide.examples) {
            let args = example.command.split(' ');
            if let Err(error) = Cli::try_parse_from(args) {
                panic!("`{}` の引数解析に失敗した: {error}", example.command);
            }
        }
    }
}
