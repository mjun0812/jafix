# クイックスタート

## インストール {#install}

配布バイナリはmacOS (Apple Silicon)、Linux (x86_64、aarch64)、Windows (x64)向けです。
macOSとLinuxでは [mise](https://mise.jdx.dev/) で導入します。

```sh
mise use -g github:mjun0812/jafix
```

Windowsでは [GitHub Releases](https://github.com/mjun0812/jafix/releases) から `jafix-x86_64-pc-windows-msvc.zip` を取得して展開します。
`jafix.exe` を `PATH` が通ったディレクトリに置きます。

ソースからビルドする場合は、リポジトリを取得して次を実行します。
初回のビルドでは形態素解析の辞書をネットワークから取得します。

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

`jafix check` はファイルを変えず、修正できる箇所と指摘を1件1行で出力します。
書式は `パス:行:列: 文面 [ルールID]` です。

```console
$ jafix check example.md
example.md:3:6: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]
example.md:3:12: 全角括弧を半角にする [fullwidth-paren]
example.md:3:16: 全角括弧を半角にする [fullwidth-paren]
example.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

| 終了コード | 意味                                 |
| ---------- | ------------------------------------ |
| 0          | 出力が無い                           |
| 1          | 1件以上を出力した                    |
| 2          | ファイルを読めない、または設定が不正 |

## 修正する {#fix}

`jafix fix` は修正ルールを適用して、ファイルを書き換えます。

```console
$ jafix fix example.md
$ cat example.md
# メモ

jafixは全角括弧(かっこ)を半角にします。
結果をを確認します。
```

指摘ルールの箇所は書き換えません。
もう一度 `check` を実行すると、手で直す箇所だけが出力されます。

```console
$ jafix check example.md
example.md:4:4: 「を」が2回続いている [ja-no-successive-word]
```

ファイルは複数渡せます。
拡張子を問わずMarkdownとして読みます。

## 次に読むページ {#next}

- [設定](./configuration.md): 使うルールを選ぶ
- [hookへの登録](./hooks.md): Claude CodeやCodexの編集のたびに実行する
- [ルール一覧](./rules.md): 99ルールのIDと内容を調べる
