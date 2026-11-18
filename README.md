# jafix

jafixはAI agentが書いた日本語のMarkdownを校正するCLIです。
全角括弧や空白などの表記をそろえ、助詞の重複などを指摘します。

## 導入

リポジトリのルートで次のコマンドを実行します。
初回のビルドでは形態素解析の辞書をネット経由で取得します。

```sh
cargo install --path .
```

[配布バイナリとmiseでの導入](REFERENCE.md#導入)

## 使い方

```sh
jafix check example.md
jafix fix example.md
```

`check` はファイルを変えず、修正できる箇所と指摘を表示します。
`fix` は修正ルールを適用してファイルを書き換えます。

## 設定ファイル

設定は `~/.config/jafix/config.toml` に書きます。
`XDG_CONFIG_HOME` を指定すると、`$XDG_CONFIG_HOME/jafix/config.toml` を読みます。

`[builtin]` にルールのIDと値を書きます。
`true` で有効、`false` で無効です。
文の長さの上限などはオプションで指定します。

```toml
[builtin]
sentence-length = { max = 120 }
no-exclamation-question-mark = false
common-misspellings = true
```

初期状態ではjafix独自の修正ルール3つと、日本語の技術文書向けの23ルールが有効です。

## hookへの登録

Claude CodeやCodexのPostToolUse hookに登録すると、その回の編集で追加または変更したMarkdownの行を修正し、対象行の指摘をagentに返します。

| 対象        | 設定ファイル    | matcher       | コマンド            |
| ----------- | --------------- | ------------- | ------------------- |
| Claude Code | `settings.json` | `Edit\|Write` | `jafix hook claude` |
| Codex       | `hooks.json`    | `apply_patch` | `jafix hook codex`  |

[登録用のJSONとhookの挙動](REFERENCE.md#hookへの登録)

## ドキュメントサイト

- [ルール一覧](docs/textlint-rules.md) ([English](docs/en/textlint-rules.md))
- [設定と動作のリファレンス](REFERENCE.md)
- [textlintとの速度の比較](docs/benchmark-vs-textlint.md)
- [ドキュメントサイトの開発と公開](REFERENCE.md#ドキュメントサイト)

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。
依存ライブラリや辞書の出典とライセンスは [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。
