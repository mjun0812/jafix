# jafix

jafixは日本語の文章を校正するCLIです。
全角括弧や空白などの表記をそろえ、同じ語が続く箇所などを指摘します。

## 導入

リポジトリのルートで次のコマンドを実行します。
初回のビルドでは形態素解析の辞書をネット経由で取得します。

```sh
cargo install --path .
```

[配布バイナリとmiseでの導入](REFERENCE.md#導入)

## 使い方

```sh
jafix check example.txt
jafix fix example.txt
```

`check` はファイルを変えず、修正できる箇所と指摘を表示します。
`fix` は修正ルールを適用してファイルを書き換えます。
どちらも拡張子を問わず、UTF-8のテキストファイルを受け付けます。
Markdownの構文に従って本文を見分け、コードブロックなどを校正の対象から外します。

## 設定ファイル

共通設定は `~/.config/jafix/config.toml` に書きます。
`XDG_CONFIG_HOME` を指定すると、`$XDG_CONFIG_HOME/jafix/config.toml` を読みます。
プロジェクトごとの指定は `.jafix.toml` に書きます。
対象ファイルに最も近い `.jafix.toml` の指定が共通設定より優先されます。

`[builtin]` にルールのIDと値を書きます。
`true` で有効、`false` で無効です。
文の長さの上限などはオプションで指定します。

```toml
[builtin]
sentence-length = { max = 120 }
no-unmatched-pair = false
common-misspellings = true
```

初期状態で有効なのは、括弧や空白を整え、文字の混入や語の重複などを見つける10ルールです。
[初期設定のルール一覧](REFERENCE.md#組み込みのルール)

## hookへの登録

Claude CodeやCodexのPostToolUse hookに登録すると、`.md` ファイルで今回追加または変更した行を修正し、指摘をagentに返します。

| 対象        | 設定ファイル    | matcher       | コマンド            |
| ----------- | --------------- | ------------- | ------------------- |
| Claude Code | `settings.json` | `Edit\|Write` | `jafix hook claude` |
| Codex       | `hooks.json`    | `apply_patch` | `jafix hook codex`  |

[登録用のJSONとhookの挙動](REFERENCE.md#hookへの登録)

## ドキュメントサイト

- [ルール一覧](docs/rules.md) ([English](docs/en/rules.md))
- [設定と動作のリファレンス](REFERENCE.md)
- [textlintとの速度の比較](docs/benchmark-vs-textlint.md)
- [ドキュメントサイトの開発と公開](REFERENCE.md#ドキュメントサイト)
- [リリースの手順](REFERENCE.md#リリース)

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。
依存ライブラリや辞書の出典とライセンスは [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。
