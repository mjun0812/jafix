use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use super::{Target, Targets, is_markdown};
use crate::apply::TargetLines;
use crate::config::Config;

#[derive(Deserialize)]
struct Input {
    cwd: PathBuf,
    #[serde(default)]
    tool_input: Value,
}

/// patchのうち、jafixが直す対象になりうるファイルの変更。Delete Fileは含めない
#[derive(Debug, PartialEq)]
enum FileChange {
    Add(String),
    Update { path: String, chunks: Vec<Chunk> },
}

#[derive(Debug, Default, PartialEq)]
struct Chunk {
    /// `@@ <文脈>` の文脈
    context: Option<String>,
    lines: Vec<ChunkLine>,
    end_of_file: bool,
}

#[derive(Debug, PartialEq)]
enum ChunkLine {
    Context(String),
    Added(String),
    Removed,
}

impl Chunk {
    /// Codexは、文脈行も削除行も無いchunkを、chunkの順にファイルの末尾へ足す
    fn is_append_only(&self) -> bool {
        self.lines
            .iter()
            .all(|line| matches!(line, ChunkLine::Added(_)))
    }

    fn added_lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().filter_map(|line| match line {
            ChunkLine::Added(text) => Some(text.as_str()),
            _ => None,
        })
    }
}

/// CodexのPostToolUseの入力JSONの `tool_input.command` にあるpatch本文から、直すMarkdownファイルと対象行を決める。
/// Update Fileの対象行は、編集後のファイルを読んでchunkを位置づけて決める。`exclude` に当たるファイルは読まずに外す
pub fn targets(input: &str, config: &Config) -> Targets {
    let mut targets = Targets::default();
    let input: Input = match serde_json::from_str(input) {
        Ok(input) => input,
        Err(e) => {
            targets
                .errors
                .push(format!("hookの入力JSONを読めない: {e}"));
            return targets;
        }
    };
    let Some(body) = input
        .tool_input
        .get("command")
        .and_then(Value::as_str)
        .and_then(patch_body)
    else {
        return targets;
    };
    for change in parse_patch(body) {
        let (path, chunks) = match change {
            FileChange::Add(path) => (path, None),
            FileChange::Update { path, chunks } => (path, Some(chunks)),
        };
        let path = input.cwd.join(path);
        if !is_markdown(&path) || config.is_excluded(&path) {
            continue;
        }
        let Some(chunks) = chunks else {
            targets.files.push(Target {
                path,
                lines: TargetLines::All,
            });
            continue;
        };
        match locate(&path, &chunks) {
            Ok(lines) => targets.files.push(Target {
                path,
                lines: TargetLines::Lines(lines),
            }),
            Err(reason) => targets.errors.push(format!(
                "{}: 対象行を決められない: {reason}",
                path.display()
            )),
        }
    }
    targets
}

/// `*** Begin Patch` から `*** End Patch` までの間の行を返す。heredocの行など、markerの外は読まない
fn patch_body(command: &str) -> Option<&str> {
    let mut offset = 0;
    let mut begin = None;
    for line in command.split_inclusive('\n') {
        match (begin, line.trim()) {
            (None, "*** Begin Patch") => begin = Some(offset + line.len()),
            (Some(begin), "*** End Patch") => return Some(&command[begin..offset]),
            _ => {}
        }
        offset += line.len();
    }
    None
}

fn parse_patch(body: &str) -> Vec<FileChange> {
    let mut changes = Vec::new();
    // 直前のファイルの見出しがUpdate Fileか
    let mut in_update = false;
    // Update Fileの中で、行を足してよいchunkが開いているか。`*** End of File` で閉じる
    let mut chunk_open = false;
    for raw in body.lines() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let marker = line.trim();
        if let Some(path) = marker.strip_prefix("*** Add File:") {
            changes.push(FileChange::Add(path.trim().to_owned()));
            in_update = false;
            continue;
        }
        if marker.starts_with("*** Delete File:") {
            in_update = false;
            continue;
        }
        if let Some(path) = marker.strip_prefix("*** Update File:") {
            changes.push(FileChange::Update {
                path: path.trim().to_owned(),
                chunks: Vec::new(),
            });
            in_update = true;
            chunk_open = false;
            continue;
        }
        let Some(FileChange::Update { path, chunks }) = changes.last_mut().filter(|_| in_update)
        else {
            continue;
        };
        if let Some(dest) = marker.strip_prefix("*** Move to:") {
            *path = dest.trim().to_owned();
            continue;
        }
        if marker == "*** End of File" {
            if let Some(chunk) = chunks.last_mut() {
                chunk.end_of_file = true;
            }
            chunk_open = false;
            continue;
        }
        if let Some(context) = line.strip_prefix("@@") {
            let context = context.trim();
            chunks.push(Chunk {
                context: (!context.is_empty()).then(|| context.to_owned()),
                ..Chunk::default()
            });
            chunk_open = true;
            continue;
        }
        // Codexはchunkの間の空白だけの行を読み飛ばし、chunkの中の空行は空の文脈行として読む
        if !chunk_open && line.trim().is_empty() {
            continue;
        }
        let chunk_line = match line.chars().next() {
            Some('+') => ChunkLine::Added(line[1..].to_owned()),
            Some('-') => ChunkLine::Removed,
            Some(' ') => ChunkLine::Context(line[1..].to_owned()),
            None => ChunkLine::Context(String::new()),
            Some(_) => continue,
        };
        // Update Fileの最初の `@@` より前の変更行は、最初のchunkに入る
        if !chunk_open {
            chunks.push(Chunk::default());
            chunk_open = true;
        }
        chunks.last_mut().unwrap().lines.push(chunk_line);
    }
    changes
}

/// 編集後のファイルでchunkを位置づけ、Codexが書いた `+` 行の行番号 (1始まり) を返す
fn locate(path: &Path, chunks: &[Chunk]) -> Result<BTreeSet<usize>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.lines().collect();
    let normalized: Vec<String> = lines.iter().map(|line| normalize(line)).collect();
    let mut targets = BTreeSet::new();

    let appended: Vec<&str> = chunks
        .iter()
        .filter(|chunk| chunk.is_append_only())
        .flat_map(Chunk::added_lines)
        .collect();
    let end = lines
        .len()
        .checked_sub(appended.len())
        .filter(|&end| lines[end..] == appended[..])
        .ok_or("末尾に足した行がファイルの末尾に無い")?;
    targets.extend(end + 1..=lines.len());

    let mut cursor = 0;
    for chunk in chunks.iter().filter(|chunk| !chunk.is_append_only()) {
        if chunk.added_lines().next().is_none() {
            continue;
        }
        if let Some(context) = &chunk.context {
            let context = normalize(context);
            let found = (cursor..end)
                .find(|&i| normalized[i] == context)
                .ok_or_else(|| format!("`@@ {context}` の行が見つからない"))?;
            cursor = found + 1;
        }
        let mut new_side: Vec<&ChunkLine> = chunk
            .lines
            .iter()
            .filter(|line| !matches!(line, ChunkLine::Removed))
            .collect();
        let mut starts = matching_starts(&lines, &normalized, &new_side, cursor, end, chunk);
        if starts.is_empty()
            && new_side
                .last()
                .is_some_and(|line| matches!(line, ChunkLine::Context(text) if text.is_empty()))
        {
            new_side.pop();
            starts = matching_starts(&lines, &normalized, &new_side, cursor, end, chunk);
        }
        let [start] = starts[..] else {
            let first = chunk.added_lines().next().unwrap_or_default();
            return Err(if starts.is_empty() {
                format!("`+{first}` を含むchunkの位置が見つからない")
            } else {
                format!("`+{first}` を含むchunkの位置が複数あり、1つに決まらない")
            });
        };
        for (offset, line) in new_side.iter().enumerate() {
            if matches!(line, ChunkLine::Added(_)) {
                targets.insert(start + offset + 1);
            }
        }
        cursor = start + new_side.len();
    }
    Ok(targets)
}

/// `cursor..end` の中で `new_side` が一致する位置 (0始まりの行番号) をすべて返す。
/// 追加行はpatchのとおりに書かれるので完全一致、文脈行は元のファイルの行が残るのでCodexと同じく正規化して比べる
fn matching_starts(
    lines: &[&str],
    normalized: &[String],
    new_side: &[&ChunkLine],
    cursor: usize,
    end: usize,
    chunk: &Chunk,
) -> Vec<usize> {
    let Some(last) = end
        .checked_sub(new_side.len())
        .filter(|&last| last >= cursor)
    else {
        return Vec::new();
    };
    let first = if chunk.end_of_file { last } else { cursor };
    (first..=last)
        .filter(|&start| {
            new_side
                .iter()
                .enumerate()
                .all(|(offset, line)| match line {
                    ChunkLine::Added(text) => lines[start + offset] == text.as_str(),
                    ChunkLine::Context(text) => normalized[start + offset] == normalize(text),
                    ChunkLine::Removed => unreachable!("new_side has no removed lines"),
                })
        })
        .collect()
}

/// Codexの `seek_sequence` の最も緩い照合と同じく、前後の空白を除き、似た記号と空白を寄せる
fn normalize(line: &str) -> String {
    line.trim()
        .chars()
        .map(|c| match c {
            '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
            '\u{2018}'..='\u{201B}' => '\'',
            '\u{201C}'..='\u{201F}' => '"',
            '\u{00A0}' | '\u{2002}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => ' ',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn update_lines(file: &str, patch_chunks: &str) -> Result<BTreeSet<usize>, String> {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("t.md");
        std::fs::write(&path, file).unwrap();
        let body = format!("*** Update File: t.md\n{patch_chunks}");
        let [FileChange::Update { chunks, .. }] = &parse_patch(&body)[..] else {
            panic!("not a single Update File");
        };
        locate(&path, chunks)
    }

    fn set(lines: &[usize]) -> BTreeSet<usize> {
        lines.iter().copied().collect()
    }

    #[test]
    fn extracts_the_patch_from_a_heredoc() {
        let command = "apply_patch <<'EOF'\n  *** Begin Patch  \n*** Add File: a.md\n+x\n*** End Patch\nEOF\n";
        assert_eq!(patch_body(command), Some("*** Add File: a.md\n+x\n"));
        assert_eq!(patch_body("ls -la"), None);
    }

    #[test]
    fn parses_files_move_to_and_chunks() {
        let body = "\
*** Add File: a.md
+new
*** Delete File: b.md
*** Update File: c.md
*** Move to: d.md
-before first header
+added
@@ fn main
 ctx

-old
*** End of File
";
        assert_eq!(
            parse_patch(body),
            [
                FileChange::Add("a.md".into()),
                FileChange::Update {
                    path: "d.md".into(),
                    chunks: vec![
                        Chunk {
                            context: None,
                            lines: vec![ChunkLine::Removed, ChunkLine::Added("added".into())],
                            end_of_file: false,
                        },
                        Chunk {
                            context: Some("fn main".into()),
                            lines: vec![
                                ChunkLine::Context("ctx".into()),
                                ChunkLine::Context(String::new()),
                                ChunkLine::Removed,
                            ],
                            end_of_file: true,
                        },
                    ],
                },
            ]
        );
    }

    #[test]
    fn locates_a_chunk_after_an_earlier_identical_line() {
        let file = "X\na\nb\nX\nc\n";
        assert_eq!(update_lines(file, "@@\n b\n-old\n+X\n c\n"), Ok(set(&[4])));
    }

    #[test]
    fn context_lines_match_after_normalization() {
        let file = "a \u{2014} b\nnew\n";
        assert_eq!(
            update_lines(file, "@@\n a - b\n-old\n+new\n"),
            Ok(set(&[2]))
        );
    }

    #[test]
    fn added_lines_must_match_exactly() {
        let file = "a\n new \n";
        assert!(update_lines(file, "@@\n a\n-old\n+new\n").is_err());
    }

    #[test]
    fn append_only_chunks_target_the_end_of_the_file() {
        let file = "a\nB\nc\nx\ny\n";
        let patch = "@@\n+x\n@@\n a\n-b\n+B\n@@\n+y\n";
        assert_eq!(update_lines(file, patch), Ok(set(&[2, 4, 5])));
    }

    #[test]
    fn append_only_lines_missing_at_the_end_is_an_error() {
        assert!(update_lines("x\na\n", "@@\n+x\n").is_err());
    }

    #[test]
    fn end_of_file_chunk_matches_only_at_the_end() {
        let file = "a\nB\na\nB\n";
        assert_eq!(
            update_lines(file, "@@\n a\n-b\n+B\n*** End of File\n"),
            Ok(set(&[4]))
        );
    }

    #[test]
    fn ambiguous_chunk_is_an_error() {
        let file = "B\nc\nB\n";
        assert!(update_lines(file, "@@\n-b\n+B\n").is_err());
    }

    #[test]
    fn context_header_moves_the_search_past_it() {
        let file = "B\n## h\nB\n";
        assert_eq!(update_lines(file, "@@ ## h\n-b\n+B\n"), Ok(set(&[3])));
    }

    #[test]
    fn trailing_empty_context_line_is_dropped_when_nothing_matches() {
        let file = "a\nB";
        assert_eq!(update_lines(file, "@@\n a\n-b\n+B\n\n"), Ok(set(&[2])));
    }

    #[test]
    fn delete_only_chunk_is_skipped() {
        let file = "a\nB\n";
        assert_eq!(
            update_lines(file, "@@\n-gone\n@@\n a\n-b\n+B\n"),
            Ok(set(&[2]))
        );
    }
}
