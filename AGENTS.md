# AGENTS.md

jafixは、textlintの日本語向けルールをRustで実装したLinterである。バイナリ1つで文章をチェックし、直せる箇所を自動で修正する。

## 作業の前に読むもの

- [CONTEXT.md](CONTEXT.md): 用語集。コード、コメント、ドキュメントでは定義された語を使い、Avoidに挙げた語を使わない。
- [docs/adr/](docs/adr/): 設計判断の記録。採用済みのADRと矛盾する変更には、ADRの追加か更新を同じ変更に含める。
- [REFERENCE.md](REFERENCE.md): 設定と動作の仕様。

## セットアップ

ツールの版は `mise.toml` で固定している。次のコマンドでツールを入れ、pre-commit hookを登録する。

```sh
mise install
prek install
```

初回のビルドでは、形態素解析の辞書をダウンロードする。

## コマンド

| 目的               | コマンド                                             |
| ------------------ | ---------------------------------------------------- |
| ビルド             | `cargo build --locked`                               |
| すべてのテスト     | `cargo test --locked --no-fail-fast`                 |
| 1つのテストファイル | `cargo test --locked --test cli`                     |
| formatの確認       | `cargo fmt --check`                                  |
| lint               | `cargo clippy --locked --all-targets -- -D warnings` |
| ドキュメントのビルド | `npm ci && npm run docs:build`                       |

変更を終える前に、format、lint、すべてのテストを通す。CIは同じコマンドを実行する。

pre-commit hookは、Rustのファイルに `cargo fmt --check` と `cargo clippy` を、ステージしたMarkdownの対象行に `jafix check --staged` を実行する。

## 構成

- `src/main.rs`: CLIの定義。サブコマンドは `check`、`fix`、`hook`、`mcp` の4つである。
- `src/rules/`: 組み込みのルール。
- `src/hook/`: Claude CodeとCodexのhookの入力から対象行を決める処理。
- `tests/`: 実際のバイナリを起動する統合テスト。`tests/common` の `Sandbox` が `HOME` と `XDG_CONFIG_HOME` を一時ディレクトリへ向けるので、実行した人の設定ファイルを読まない。
- `docs/`: VitePressのドキュメントサイト。日本語版を基準にし、英語版は同じファイル名で `docs/en/` に置く。
- `skills/jafix/`: 配布するAgent Skill。

## 規約

- commitメッセージはConventional Commitsに従い、英語で書く。
- PRのタイトルと本文は日本語で書く。
- 利用者から見える振る舞いを変えたら、`README.md`、`REFERENCE.md`、`docs/` の日本語版と英語版を同じ変更で更新する。
- テストは実際のバイナリを起動して確かめる。新しいテストも `Sandbox` を使う。
