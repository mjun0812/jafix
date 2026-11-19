# hookへの登録

PostToolUse hookに登録すると、agentがMarkdown (`.md`)を編集するたびにjafixが動きます。
修正するのは、その編集で追加または変更した行だけです。
その行の指摘はagentに返します。

設定ファイルは `jafix check` と[同じもの](./configuration.md)を読みます。

## Claude Code {#claude-code}

`~/.claude/settings.json` か、プロジェクトの `.claude/settings.json` に書きます。

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

`~/.codex/hooks.json` に書きます。
matcherは `apply_patch` だけにします。

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

## agentに返る内容 {#output}

指摘があると、標準エラーに次の形で書いて終了コード2で終わります。
2行目以降の書式は `jafix check` と同じです。

```text
jafix: 今回変えた行に指摘がある。編集は適用済み。指摘を直すか、誤検知なら直さずに進めてよい
/path/to/notes.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

| 終了コード | 意味                                                         |
| ---------- | ------------------------------------------------------------ |
| 0          | 指摘が無い                                                   |
| 1          | 指摘は無いが、入力や設定の誤り、ファイルの読み書きで失敗した |
| 2          | 編集した行に指摘がある                                       |

終了コード1はagentの作業を止めません。

## 他のformatterと併用する {#formatters}

jafixはagentが書いた直後のファイルから行の位置を決めます。
同じファイルを書き換えるformatterがある場合は、1つのscriptの中でjafixを先に呼びます。

```sh
#!/bin/sh
input=$(cat)
printf '%s' "$input" | jafix hook claude
# この後に他のformatterを呼ぶ
```

hookの細かい挙動は[リファレンス](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#hookへの登録)にあります。
