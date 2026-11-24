# jafix

jafixは日本語のテキストファイルを校正するCLIです。
全角括弧や余分な空白を直し、同じ語が続く箇所や括弧の閉じ忘れを指摘します。

## 導入

[GitHub Releases](https://github.com/mjun0812/jafix/releases)から環境に合うバイナリをダウンロードします。展開した `jafix` または `jafix.exe` を `PATH` の通ったディレクトリに置きます。

[mise](https://mise.jdx.dev/)を使う場合は、次のコマンドで配布バイナリをインストールできます。

```sh
mise use -g github:mjun0812/jafix
```

配布ファイルの選び方、非公開リポジトリの認証、ソースからのビルド手順は[導入の詳細](REFERENCE.md#導入)にあります。

## 使い方

校正するテキストファイルを指定します。UTF-8とShift_JIS (CP932を含む)を自動で判定します。

```sh
jafix check example.txt
jafix fix example.txt
```

`check` は修正できる箇所と指摘を表示します。ファイルの内容はそのままです。
`fix` は修正ルールを適用してファイルを書き換えます。
どちらもMarkdownの構文に従って本文を判定し、コードブロックなどを校正の対象から外します。
文字コードを指定する方法は[リファレンス](REFERENCE.md#使い方)を参照してください。
Gitの変更行だけを校正するには `--diff` を指定します。ステージ済みの変更を確認するには `check --staged` を使います。

## 設定ファイル

共通設定は `~/.config/jafix/config.toml` に書きます。
環境変数 `XDG_CONFIG_HOME` を設定している場合は、`$XDG_CONFIG_HOME/jafix/config.toml` を使います。
プロジェクト設定は `.jafix.toml` に書きます。
同じルールを両方で指定した場合は、対象ファイルに最も近い `.jafix.toml` の設定が優先されます。

使うルールは `[builtin]` で選べます。
ルールのIDに `true` を指定すると有効になり、`false` を指定すると無効になります。
文の長さの上限などはオプションで指定できます。

```toml
[builtin]
sentence-length = { max = 120 }
no-unmatched-pair = false
common-misspellings = true
```

設定ファイルがなくても、既定の10ルールで校正できます。
[初期設定のルール一覧](REFERENCE.md#組み込みのルール)

## hookへの登録

Claude CodeやCodexのPostToolUse hookにjafixを登録すると、`.md` ファイルを編集したあとに校正できます。
対象は、その編集で追加または変更した行です。指摘はagentに返します。

| 対象        | 設定ファイル    | matcher       | コマンド            |
| ----------- | --------------- | ------------- | ------------------- |
| Claude Code | `settings.json` | `Edit\|Write` | `jafix hook claude` |
| Codex       | `hooks.json`    | `apply_patch` | `jafix hook codex`  |

[登録用のJSONとhookの挙動](REFERENCE.md#hookへの登録)

## MCP

`jafix mcp` を登録すると、agentが下書きの文章を直接チェックし、自動修正した本文を受け取れます。[MCPの登録手順](docs/quick-start.md#mcp)

## ドキュメントサイト

- [ルール一覧](docs/rules.md) ([English](docs/en/rules.md))
- [設定と動作のリファレンス](REFERENCE.md)
- [textlintとの速度の比較](docs/benchmark-vs-textlint.md)
- [ドキュメントサイトの開発と公開](REFERENCE.md#ドキュメントサイト)
- [リリースの手順](REFERENCE.md#リリース)

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。
依存ライブラリや辞書の出典とライセンスは [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。
