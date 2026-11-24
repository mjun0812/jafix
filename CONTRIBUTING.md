# コントリビューションガイド

jafixへの報告と変更の提案を歓迎します。

## Issue

バグの報告と機能の提案は、[Issue](https://github.com/mjun0812/jafix/issues/new/choose)のテンプレートから作成します。
脆弱性は公開のIssueに書かず、[SECURITY.md](SECURITY.md)の手順で報告します。

## 開発環境

ツールの版は `mise.toml` で固定しています。[mise](https://mise.jdx.dev/)を入れてから、リポジトリのルートで次のコマンドを実行します。

```sh
mise install
prek install
```

`prek install` はpre-commit hookを登録します。hookはコミットの前に、formatを確認し、lintをかけ、ステージしたMarkdownを校正します。

## 変更を確かめる

PRを作る前に、次のコマンドがすべて成功することを確かめます。CIも同じコマンドを実行します。

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --no-fail-fast
```

ドキュメントサイトを変えたときは、`npm ci` と `npm run docs:build` も実行します。

## Pull Request

1. `main` からbranchを作ります。
2. commitメッセージは[Conventional Commits](https://www.conventionalcommits.org/)に従い、英語で書きます。
3. PRのテンプレートに沿って、変更の内容と確かめたことを書きます。
4. CIがすべて成功すると、mergeできます。

コードの構成と規約は[AGENTS.md](AGENTS.md)にあります。
