# Quick start

The [Japanese documentation](../quick-start.md) is the source of truth.

## Install {#install}

Prebuilt binaries support macOS (Apple Silicon), Linux (x86_64 and aarch64), and Windows (x64).
On macOS and Linux, install with [mise](https://mise.jdx.dev/).

```sh
mise use -g github:mjun0812/jafix
```

On Windows, extract `jafix-x86_64-pc-windows-msvc.zip` from [GitHub Releases](https://github.com/mjun0812/jafix/releases).
Place `jafix.exe` in a directory on your `PATH`.

To build from source, clone the repository and run the following.
The first build downloads the morphological dictionary.

```sh
git clone https://github.com/mjun0812/jafix.git
cd jafix
cargo install --path .
```

```console
$ jafix --version
jafix 0.0.2
```

## Check a file {#check}

Create `example.md` with this content.

```markdown
# メモ

jafix は全角括弧（かっこ）を半角にします。
結果をを確認します。
```

`jafix check` leaves the file unchanged and prints one line per fixable spot or finding.
The format is `path:line:column: message [rule-id]`. Messages are in Japanese.

```console
$ jafix check example.md
example.md:3:6: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]
example.md:3:12: 全角括弧を半角にする [fullwidth-paren]
example.md:3:16: 全角括弧を半角にする [fullwidth-paren]
example.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

| Exit code | Meaning                                                   |
| --------- | --------------------------------------------------------- |
| 0         | No output                                                 |
| 1         | One or more lines were printed                            |
| 2         | A file could not be read, or the configuration is invalid |

## Fix a file {#fix}

`jafix fix` applies the fix rules and rewrites the file.

```console
$ jafix fix example.md
$ cat example.md
# メモ

jafixは全角括弧(かっこ)を半角にします。
結果をを確認します。
```

Report rules never rewrite the file.
Run `check` again to see what is left to fix by hand.

```console
$ jafix check example.md
example.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

You can pass several files.
`check` and `fix` detect UTF-8 or Shift_JIS (including CP932) for each file.
Each file is read as Markdown regardless of its extension.

`fix` saves the file in its original encoding. If automatic detection fails, specify `--encoding utf-8` or `--encoding shift-jis`.

Rules that analyze Japanese words and grammar run on paragraphs and headings that may contain Japanese. You can enable both Japanese and English rules for mixed-language text.

## Proofread Git changes {#git-diff}

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

## Next steps {#next}

- [Configuration](./configuration.md): choose the rules to use
- [Hooks](./hooks.md): run jafix on every edit by Claude Code or Codex
- [Rules](./rules.md): look up rule IDs and behavior
