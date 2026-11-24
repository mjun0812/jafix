# クイックスタート

## インストール {#install}

### curlでインストール {#curl}

macOS (Apple Silicon)とLinux (x86_64、ARM64)では、最新の公開リリースに `install.sh` がある場合にインストーラーを使えます。

```sh
curl -fsSL https://github.com/mjun0812/jafix/releases/latest/download/install.sh | sh
```

インストーラーは配布ファイルのSHA-256を検証し、`jafix` を `~/.local/bin` に置きます。このディレクトリが `PATH` に含まれていなければ追加してください。

### 配布バイナリを直接ダウンロード {#download}

[GitHub Releases](https://github.com/mjun0812/jafix/releases)から環境に合うファイルをダウンロードして展開します。リポジトリが非公開の場合は、アクセス権のあるアカウントでGitHubにログインします。

| 環境                  | 配布ファイル                            |
| --------------------- | --------------------------------------- |
| macOS (Apple Silicon) | `jafix-aarch64-apple-darwin.tar.gz`       |
| Linux (x86_64)        | `jafix-x86_64-unknown-linux-musl.tar.gz`  |
| Linux (ARM64)         | `jafix-aarch64-unknown-linux-musl.tar.gz` |
| Windows (x64)         | `jafix-x86_64-pc-windows-msvc.zip`        |

展開した `jafix` または `jafix.exe` を `PATH` の通ったディレクトリに置きます。

### miseでインストール {#mise}

[mise](https://mise.jdx.dev/)は環境に合う配布バイナリをGitHub Releasesから取得します。リポジトリが非公開の場合は、読み取り権限のあるトークンを環境変数 `MISE_GITHUB_TOKEN` に設定してから実行します。[miseの認証手順](https://mise.jdx.dev/dev-tools/github-tokens.html)

```sh
mise use -g github:mjun0812/jafix
```

### ソースからビルド {#source}

リポジトリを取得して次を実行します。
初回のビルドには形態素解析の辞書のダウンロードが必要です。

```sh
git clone https://github.com/mjun0812/jafix.git
cd jafix
cargo install --path .
```

### インストールを確認する {#verify-install}

`jafix --version` を実行します。miseを有効にしていないシェルでは `mise exec -- jafix --version` を使えます。

```console
$ jafix --version
jafix 0.0.2
```

## 文章をチェックする {#check}

次の内容で `example.md` を作ります。

```markdown
# メモ

jafix は全角括弧（かっこ）を半角にします。
結果をを確認します。
```

`jafix check` は有効なルールを使って本文を調べ、該当箇所を1件1行で出力します。ファイルの内容はそのままです。
各行には `パス:行:列: [ルールID] 文面` の順で表示します。

```console
$ jafix check example.md
example.md:3:6: [space-between-ja-and-alnum] 日本語と英数字の間の空白を削除する
example.md:3:12: [fullwidth-paren] 全角括弧を半角にする
example.md:3:16: [fullwidth-paren] 全角括弧を半角にする
example.md:4:4: [ja-no-successive-word] 「を」が2回続いている
```

| 終了コード | 意味                                   |
| ---------- | -------------------------------------- |
| 0          | 修正できる箇所も指摘も見つからなかった |
| 1          | 修正できる箇所または指摘が見つかった   |
| 2          | 対象の指定、ファイルの読み込み、設定のいずれかに問題がある |

機械で結果を処理するには `jafix check --output-format json example.md` を使います。JSONにはチェックしたファイル数、指摘、処理中に発生したエラーが入ります。[JSONの形式](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#json出力)

## 自動修正する {#fix}

`jafix fix` は修正ルールに当たる箇所を自動で直し、ファイルに保存します。出力オプションを省くと、成功時は何も表示しません。

```console
$ jafix fix example.md
$ cat example.md
# メモ

jafixは全角括弧(かっこ)を半角にします。
結果をを確認します。
```

保存した変更を確認するときは、目的に合うオプションを指定します。

- `--list` は修正したファイルのパスだけを表示します。
- `--summary` は修正したファイル数と変更のなかったファイル数を表示します。
- `--timing` はファイルごとの処理時間を秒単位で表示します。
- `--show-diff` はファイルを保存したあと、修正前後の差分を表示します。

ファイルを保存せずに差分を確認するには `jafix fix --dry-run --show-diff example.md` を実行します。

この例では `fix` で括弧と空白が直ります。もう一度 `check` を実行すると、残った指摘を確認できます。

```console
$ jafix check example.md
example.md:4:4: [ja-no-successive-word] 「を」が2回続いている
```

`check` と `fix` にはテキストファイルを複数指定できます。文字コードはUTF-8とShift_JIS (CP932を含む)に対応し、ファイルごとに自動で判定します。
jafixは拡張子を問わずMarkdownの構文に従って本文を判定します。

`fix` は元の文字コードで保存します。自動で判定できないときは、`--encoding utf-8` または `--encoding shift-jis` で文字コードを指定します。

形態素解析を使う日本語向けルールは日本語の可能性がある段落や見出しに適用します。日本語と英語が混在する本文にも、それぞれ有効にしたルールを使えます。

## Gitの差分を校正する {#git-diff}

Gitリポジトリで `--diff` を指定すると、`HEAD` から追加または変更した行だけを校正します。本文を判定するときはファイル全体を読みます。

```sh
jafix check --diff
jafix fix --diff
```

`fix --diff` は作業ツリーを書き換えます。修正をコミットに含めるには `git add` を実行します。

ステージ済みの変更を確認するには、`check --staged` を使います。校正対象の本文はGitのインデックスから読みます。

```sh
jafix check --staged
```

どのコマンドもファイル名を渡すと、そのファイルだけを対象にします。未追跡のファイルを差分に含めるには `git add` を実行します。

## 下書きをMCPで校正する {#mcp}

MCPクライアントにstdioの起動コマンド `jafix mcp` を登録します。Claude CodeまたはCodexでは次のコマンドを使えます。

```sh
claude mcp add jafix -- jafix mcp
codex mcp add jafix -- jafix mcp
```

`check_text` と `fix_text` は `text` 引数にMarkdownの本文を受け取ります。`check_text` は指摘を返します。`fix_text` は自動修正後の本文と、その本文に残った指摘を返します。どちらもファイルを書き換えません。

サーバーは共通設定と、起動ディレクトリから探したプロジェクト設定を使います。別の場所から設定を探すには `jafix mcp --project-dir /path/to/repo` を起動コマンドにします。ファイルを入力とする設定の `exclude` と `report-exclude`、リンク先を調べる `no-dead-link` はMCPの本文には適用しません。

## Agent Skillを導入する {#agent-skill}

Agent Skillを導入すると、Claude CodeやCodexが日本語の文書を編集した後に、今回変えた行と新規ファイルをjafixで校正します。hookを登録していない環境でも、agentが自動修正と手修正を使い分け、指摘が無くなるまで確かめます。

```sh
npx skills add mjun0812/jafix
```

手で導入する場合は、リポジトリの[`skills/jafix`](https://github.com/mjun0812/jafix/tree/main/skills/jafix)ディレクトリを次の場所にコピーします。

| 対象        | 置き場所                 |
| ----------- | ------------------------ |
| Claude Code | `~/.claude/skills/jafix` |
| Codex       | `~/.agents/skills/jafix` |

skillは `jafix` コマンドを使うので、先にjafixをインストールします。

## 次に読むページ {#next}

- [使うルールを設定する](./configuration.md)
- [Claude CodeやCodexのhookに登録する](./hooks.md)
- [ルールIDと内容を調べる](./rules.md)
