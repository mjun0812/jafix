# hookへの登録

jafixをPostToolUse hookに登録すると、指定したツールで `.md` ファイルを編集したあとに校正できます。
対象は、その編集で追加または変更した行です。指摘はagentに返します。
hookはUTF-8のファイルを扱います。

校正には、`jafix check` と同じ[設定ファイル](./configuration.md)を使います。

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

指摘があると、jafixは標準エラー出力にメッセージを書き、終了コード2を返します。
2行目以降に、`jafix check` と同じ書式で指摘を並べます。

```text
jafix: 今回変えた行に指摘がある。編集は適用済み。指摘を直すか、誤検知なら直さずに進めてよい
/path/to/notes.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

| 終了コード | 意味                                           |
| ---------- | ---------------------------------------------- |
| 0          | 指摘が見つからなかった                         |
| 1          | 入力や設定の誤り、ファイルの読み書きで失敗した |
| 2          | 編集した行に指摘がある                         |

終了コード1はagentの作業を止めません。

## 他のformatterと併用する {#formatters}

jafixは編集直後のファイルを読み、校正の対象行を決めます。
同じファイルに別のformatterもかける場合は、1つのスクリプトでjafixを先に実行します。

```sh
#!/bin/sh
input=$(cat)
printf '%s' "$input" | jafix hook claude
# この後に他のformatterを呼ぶ
```

対象行の判定や終了コードの詳細は、[リファレンス](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#hookへの登録)を参照してください。
