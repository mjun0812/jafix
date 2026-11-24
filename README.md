# jafix

jafixはtextlintの日本語向けルールをRustで実装した高速なLinterです。
バイナリ1つで文章をチェックし、直せる箇所は自動で修正できます。

## 導入

macOS (Apple Silicon)とLinux (x86_64、ARM64)では、最新の公開リリースに `install.sh` がある場合に `curl` でインストールできます。

```sh
curl -fsSL https://github.com/mjun0812/jafix/releases/latest/download/install.sh | sh
```

[GitHub Releases](https://github.com/mjun0812/jafix/releases)から環境に合うバイナリをダウンロードします。展開した `jafix` または `jafix.exe` を `PATH` の通ったディレクトリに置きます。

[mise](https://mise.jdx.dev/)でも配布バイナリをインストールできます。

```sh
mise use -g github:mjun0812/jafix
```

配布ファイルの選び方、非公開リポジトリの認証、ソースからのビルド手順は[導入の詳細](REFERENCE.md#導入)にあります。

## 使い方

チェックするテキストファイルを指定します。

```sh
jafix check example.txt
jafix fix example.txt
```

`check` はファイルを書き換えず、ルールに当たった箇所を表示します。`fix` は修正ルールを適用してファイルに保存します。
どちらもMarkdownの構文に従って本文を判定し、コードブロックなどを対象から外します。

入力はUTF-8とShift_JIS (CP932を含む)に対応します。文字コードはファイルごとに自動で判定します。[文字コードを指定する方法](REFERENCE.md#使い方)
Gitの変更行だけを校正するには `--diff` を指定します。ステージ済みの変更を確認するには `check --staged` を使います。

## 設定ファイル

共通設定は `~/.config/jafix/config.toml` に書きます。
環境変数 `XDG_CONFIG_HOME` を設定している場合の共通設定は `$XDG_CONFIG_HOME/jafix/config.toml` です。
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

設定ファイルがなくても10ルールが有効です。
[初期設定のルール一覧](REFERENCE.md#組み込みのルール)

## hookへの登録

Claude CodeやCodexのPostToolUse hookに登録すると、jafixは `.md` ファイルの編集後に動きます。
編集で追加・変更した行を自動修正し、その行に残った指摘をagentに返します。

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
