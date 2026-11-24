# Hooks

Registered as a PostToolUse hook, jafix runs each time the agent edits a Markdown file (`.md`).
It applies automatic fixes to lines the edit added or changed, then reports any remaining findings on those lines to the agent.
Hooks process UTF-8 files.

The hooks read the [same configuration files](./configuration.md) as `jafix check`.

## Claude Code {#claude-code}

Add this to `~/.claude/settings.json` or to `.claude/settings.json` in the project.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [{ "type": "command", "command": "jafix hook claude" }]
      }
    ]
  }
}
```

## Codex {#codex}

Add this to `~/.codex/hooks.json`.
Set the matcher to `apply_patch` only.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "apply_patch",
        "hooks": [{ "type": "command", "command": "jafix hook codex" }]
      }
    ]
  }
}
```

## What the agent receives {#output}

When there are findings, jafix writes them to standard error in this form and exits with code 2.
From the second line on, the format matches `jafix check`.

```text
jafix: 今回変えた行に指摘がある。編集は適用済み。指摘を直すか、誤検知なら直さずに進めてよい
/path/to/notes.md:4:4: [ja-no-successive-word] 「を」が2回続いている
```

The first line tells the agent that the edit has been applied, and that it should fix the findings or move on if they are false positives.

| Exit code | Meaning                                                                     |
| --------- | --------------------------------------------------------------------------- |
| 0         | No findings                                                                 |
| 1         | No findings, but the input or configuration was invalid, or file I/O failed |
| 2         | Findings on the edited lines                                                |

Exit code 1 does not stop the agent.

## Use with other formatters {#formatters}

jafix locates the lines in the file as the agent just wrote it.
If another formatter rewrites the same file, call jafix first within one script.

```sh
#!/bin/sh
input=$(cat)
printf '%s' "$input" | jafix hook claude
# call other formatters after this
```

Details of hook behavior are in the [reference](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#hookへの登録) (Japanese).
