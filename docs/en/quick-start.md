# Quick start

The [Japanese documentation](../quick-start.md) is the source of truth.

## Install {#install}

### Install with curl {#curl}

On macOS (Apple Silicon) or Linux (x86_64 or ARM64), install jafix with this command.

```sh
curl -fsSL https://github.com/mjun0812/jafix/releases/latest/download/install.sh | sh
```

The installer verifies the archive's SHA-256 checksum and installs `jafix` in `~/.local/bin`. Add this directory to your `PATH` if needed.

### Download a prebuilt binary {#download}

Download the archive for your platform from [GitHub Releases](https://github.com/mjun0812/jafix/releases) and extract it.

| Platform              | Release asset                           |
| --------------------- | --------------------------------------- |
| macOS (Apple Silicon) | `jafix-aarch64-apple-darwin.tar.gz`       |
| Linux (x86_64)        | `jafix-x86_64-unknown-linux-musl.tar.gz`  |
| Linux (ARM64)         | `jafix-aarch64-unknown-linux-musl.tar.gz` |
| Windows (x64)         | `jafix-x86_64-pc-windows-msvc.zip`        |

Place the extracted `jafix` or `jafix.exe` in a directory on your `PATH`.

### Install with mise {#mise}

[mise](https://mise.jdx.dev/) downloads the prebuilt binary for your platform from GitHub Releases.

```sh
mise use -g github:mjun0812/jafix
```

### Build from source {#source}

Clone the repository and run the following.
The first build downloads the morphological dictionary.

```sh
git clone https://github.com/mjun0812/jafix.git
cd jafix
cargo install --path .
```

### Verify the installation {#verify-install}

Run `jafix --version`. If mise is not activated in your shell, use `mise exec -- jafix --version`.

```console
$ jafix --version
jafix 0.0.3
```

## Lint a file {#check}

Create `example.md` with this content.

```markdown
# メモ

jafix は全角括弧（かっこ）を半角にします。
結果をを確認します。
```

`jafix check` checks the prose against the enabled rules and prints one line per finding without changing the file.
The format is `path:line:column: [rule-id] message`. Messages are in Japanese.

```console
$ jafix check example.md
example.md:3:6: [space-between-ja-and-alnum] 日本語と英数字の間の空白を削除する
example.md:3:12: [fullwidth-paren] 全角括弧を半角にする
example.md:3:16: [fullwidth-paren] 全角括弧を半角にする
example.md:4:4: [ja-no-successive-word] 「を」が2回続いている
```

| Exit code | Meaning                                                   |
| --------- | --------------------------------------------------------- |
| 0         | No output                                                 |
| 1         | One or more lines were printed                            |
| 2         | The target selection, file read, or configuration failed |

Use `jafix check --output-format json example.md` to get machine-readable results, including the number of files checked, findings, and execution errors. [JSON format](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#json出力)

## Apply automatic fixes {#fix}

`jafix fix` applies the fix rules and rewrites the file. When you pass files, it is silent on success by default.

```console
$ jafix fix example.md
$ cat example.md
# メモ

jafixは全角括弧(かっこ)を半角にします。
結果をを確認します。
```

Use these options to see the results:

- `--list` prints only the paths of changed files.
- `--summary` prints the number of changed and unchanged files.
- `--timing` prints the processing time for each file in seconds.
- `--show-diff` saves the files and prints a unified diff of the changes.

Run `jafix fix --dry-run --show-diff example.md` to preview the diff without writing the file.

Report rules never rewrite the file.
Run `check` again to see what is left to fix by hand.

```console
$ jafix check example.md
example.md:4:4: [ja-no-successive-word] 「を」が2回続いている
```

You can pass several files.
`check` and `fix` detect UTF-8 or Shift_JIS (including CP932) for each file.
Each file is read as Markdown regardless of its extension.

`fix` saves the file in its original encoding. If automatic detection fails, specify `--encoding utf-8` or `--encoding shift-jis`.

Rules that analyze Japanese words and grammar run on paragraphs and headings that may contain Japanese. You can enable both Japanese and English rules for mixed-language text.

## Lint and fix Git changes {#git-diff}

In a Git repository, `--diff` checks or fixes lines added or changed since `HEAD`. jafix reads each whole file for Markdown context.

```sh
jafix check --diff
jafix fix --diff
```

`fix --diff` writes to the working tree. Run `git add` to include the fixes in your commit.

Use `check --staged` to check the staged changes. It reads the text to check from the Git index.

```sh
jafix check --staged
```

Pass file paths to limit the scope of any of these commands. Run `git add` to include untracked files in the diff.

## Proofread text from standard input {#stdin}

To proofread text before you write it to a file, pass `-` instead of a path and pipe the text to jafix.

```sh
echo "本文（補足）です。" | jafix check -
echo "本文（補足）です。" | jafix fix -
```

`check -` reports findings with the path `<stdin>`. `fix -` prints the corrected text to standard output and does not write any file. jafix uses the global settings and the project settings found from the current directory. To pass a file named `-`, write `./-`. [Details](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#標準入力)

## Proofread drafts with MCP {#mcp}

Register `jafix mcp` as a stdio server in your MCP client. For Claude Code or Codex, run one of these commands.

```sh
claude mcp add jafix -- jafix mcp
codex mcp add jafix -- jafix mcp
```

Both `check_text` and `fix_text` accept Markdown in a `text` argument. `check_text` returns findings. `fix_text` returns the corrected text and any remaining findings. The tools do not write files.

The server uses global settings and looks for project settings from its working directory. To use a different directory, launch it with `jafix mcp --project-dir /path/to/repo`. File exclusions (`exclude` and `report-exclude`) and `no-dead-link` do not apply to MCP text input.

## Install the Agent Skill {#agent-skill}

With the Agent Skill installed, Claude Code and Codex proofread the lines they changed and the files they created after editing Japanese documents. Even without hooks, the agent runs the automatic fixes, rewrites the remaining findings by hand, and rechecks until none remain.

```sh
npx skills add mjun0812/jafix
```

To install it manually, copy the [`skills/jafix`](https://github.com/mjun0812/jafix/tree/main/skills/jafix) directory from the repository to one of these locations.

| Agent       | Location                 |
| ----------- | ------------------------ |
| Claude Code | `~/.claude/skills/jafix` |
| Codex       | `~/.agents/skills/jafix` |

The skill runs the `jafix` command, so install jafix first.

## Ask jafix how to use it {#usage}

The `--help` of each command shows examples and the meaning of each exit code after the option descriptions.

```sh
jafix check --help
```

`jafix rules` shows whether each built-in rule is enabled by your current settings. It prints one rule per line as three tab-separated fields: the rule ID, `有効` (enabled) or `無効` (disabled), and the presets that include the rule (`-` if none). When you pass a file, jafix answers with the project settings used for that file. File exclusions (`exclude` and `report-exclude`) are not applied.

```sh
jafix rules docs/guide.md
```

Agents and scripts can run `jafix --usage` to get the commands, arguments, exit codes, and examples as JSON, and `jafix check --schema` or `jafix rules --schema` to get the JSON Schema of the `--output-format json` output. The [reference](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#コマンド体系の出力) (Japanese) describes the output.

## Next steps {#next}

- [Configuration](./configuration.md): choose the rules to use
- [Hooks](./hooks.md): run jafix on every edit by Claude Code or Codex
- [Rules](./rules.md): look up rule IDs and behavior
