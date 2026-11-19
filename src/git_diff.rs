use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ops::Range;
use std::path::PathBuf;
use std::process::Command;

use crate::apply::TargetLines;

pub struct FileTarget {
    pub path: PathBuf,
    pub lines: TargetLines,
    pub staged_blob: Option<String>,
}

pub struct Selection {
    pub root: PathBuf,
    pub files: Vec<FileTarget>,
}

pub fn select(paths: &[PathBuf], staged: bool) -> Result<Selection, String> {
    let root = git_output(Command::new("git").args(["rev-parse", "--show-toplevel"]))?;
    let root = git_path(
        root.strip_suffix(b"\n")
            .ok_or("Gitのルートを読み取れない")?,
    )?
    .canonicalize()
    .map_err(|error| format!("Gitのルートを開けない: {error}"))?;
    // rawはパスとblob IDをNUL区切りで返し、patchは同じ順に変更後の行番号を返す。
    let patch = git_output(
        Command::new("git")
            .args([
                "--literal-pathspecs",
                "diff",
                "--patch-with-raw",
                "--no-abbrev",
                "-z",
                "--unified=0",
                "--inter-hunk-context=0",
                "--no-color",
                "--no-ext-diff",
                "--no-textconv",
                "--no-relative",
                "--find-renames",
                "--ignore-submodules=all",
                "--diff-filter=AMR",
                if staged { "--cached" } else { "HEAD" },
                "--",
            ])
            .args(paths),
    )?;
    let files = parse(&patch, staged).ok_or("Git差分の対象行を読み取れない")?;
    Ok(Selection { root, files })
}

fn parse(patch: &[u8], staged: bool) -> Option<Vec<FileTarget>> {
    if patch.is_empty() {
        return Some(Vec::new());
    }
    let separator = patch.windows(2).position(|pair| pair == b"\0\0")?;
    let mut records = patch[..separator].split(|&byte| byte == 0);
    let mut patch_lines = patch[separator + 2..]
        .split(|&byte| byte == b'\n')
        .peekable();
    let mut files = Vec::new();
    while let Some(metadata) = records.next() {
        if !patch_lines.next()?.starts_with(b"diff --git ") {
            return None;
        }
        let mut metadata = std::str::from_utf8(metadata).ok()?.split(' ');
        metadata.next()?; // 変更前のモード
        let mode = metadata.next()?;
        metadata.next()?; // 変更前のblob ID
        let blob = metadata.next()?;
        let status = metadata.next()?;
        let path = records.next()?;
        let path = if status.starts_with('R') {
            records.next()?
        } else {
            path
        };

        let mut lines = BTreeSet::new();
        while let Some(line) = patch_lines.next_if(|line| !line.starts_with(b"diff --git ")) {
            if let Some(header) = line.strip_prefix(b"@@ -") {
                lines.extend(new_lines(header)?);
            }
        }
        if matches!(mode, "100644" | "100755") && !lines.is_empty() {
            files.push(FileTarget {
                path: git_path(path).ok()?,
                lines: TargetLines::Lines(lines),
                staged_blob: staged.then(|| blob.to_owned()),
            });
        }
    }
    if patch_lines.next().is_some() {
        return None;
    }
    Some(files)
}

fn new_lines(header: &[u8]) -> Option<Range<usize>> {
    let range = std::str::from_utf8(header.split(|&byte| byte == b' ').nth(1)?)
        .ok()?
        .strip_prefix('+')?;
    let (start, count) = range.split_once(',').unwrap_or((range, "1"));
    let start = start.parse::<usize>().ok()?;
    Some(start..start.checked_add(count.parse().ok()?)?)
}

#[cfg(unix)]
fn git_path(bytes: &[u8]) -> Result<PathBuf, String> {
    use std::os::unix::ffi::OsStrExt;
    Ok(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
fn git_path(bytes: &[u8]) -> Result<PathBuf, String> {
    let path = std::str::from_utf8(bytes).map_err(|_| "GitのパスをUTF-8として読めない")?;
    Ok(PathBuf::from(OsStr::new(path)))
}

pub fn read_blob(blob: &str) -> Result<Vec<u8>, String> {
    git_output(Command::new("git").args(["cat-file", "blob", blob]))
}

fn git_output(command: &mut Command) -> Result<Vec<u8>, String> {
    let output = command
        .output()
        .map_err(|error| format!("Gitを起動できない: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Git: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}
