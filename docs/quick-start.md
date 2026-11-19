# クイックスタート

## インストール {#install}

配布バイナリはmacOS (Apple Silicon)、Linux (x86_64、aarch64)、Windows (x64)向けです。
macOSとLinuxでは [mise](https://mise.jdx.dev/) で導入します。

```sh
mise use -g github:mjun0812/jafix
```

Windowsでは [GitHub Releases](https://github.com/mjun0812/jafix/releases) から `jafix-x86_64-pc-windows-msvc.zip` を取得して展開します。
`PATH` で指定済みのディレクトリへ `jafix.exe` を移します。

ソースからビルドする場合は、リポジトリを取得して次を実行します。
初回のビルドには形態素解析の辞書のダウンロードが必要です。

```sh
git clone https://github.com/mjun0812/jafix.git
cd jafix
cargo install --path .
```

```console
$ jafix --version
jafix 0.0.1
```

## 指摘を確認する {#check}

次の内容で `example.md` を作ります。

```markdown
# メモ

jafix は全角括弧（かっこ）を半角にします。
結果をを確認します。
```

`jafix check` は修正できる箇所と指摘を1件1行で出力します。ファイルの内容はそのままです。
各行には `パス:行:列: 文面 [ルールID]` の順で表示します。

```console
$ jafix check example.md
example.md:3:6: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]
example.md:3:12: 全角括弧を半角にする [fullwidth-paren]
example.md:3:16: 全角括弧を半角にする [fullwidth-paren]
example.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

| 終了コード | 意味                                   |
| ---------- | -------------------------------------- |
| 0          | 修正できる箇所も指摘も見つからなかった |
| 1          | 修正できる箇所または指摘が見つかった   |
| 2          | ファイルを読めない、または設定が不正   |

## 修正する {#fix}

`jafix fix` は修正ルールを適用して、ファイルを書き換えます。

```console
$ jafix fix example.md
$ cat example.md
# メモ

jafixは全角括弧(かっこ)を半角にします。
結果をを確認します。
```

この例では、`fix` で括弧と空白が直ります。もう一度 `check` を実行すると、残った指摘を確認できます。

```console
$ jafix check example.md
example.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

`check` と `fix` にはテキストファイルを複数指定できます。UTF-8とShift_JIS (CP932を含む)はファイルごとに自動で判定します。
どの拡張子でも、jafixはMarkdownの構文に従って本文を判定します。

`fix` は元の文字コードで保存します。自動で判定できないときは、`--encoding utf-8` または `--encoding shift-jis` で文字コードを指定します。

形態素解析を使う日本語向けルールは、日本語の可能性がある段落や見出しに適用します。日英混在の本文にも、それぞれ有効にしたルールを使えます。

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

どのコマンドも、ファイル名を渡すとそのファイルだけを対象にします。未追跡のファイルを差分に含めるには `git add` を実行します。

## 次に読むページ {#next}

- [使うルールを設定する](./configuration.md)
- [Claude CodeやCodexのhookに登録する](./hooks.md)
- [ルールIDと内容を調べる](./rules.md)
