# jafix

jafixは、AI agentが書いた日本語のMarkdownを、機械的に直せる範囲で直すCLIです。Markdownの本文にある全角括弧を半角にし、日本語と英数字・括弧の間の空白を削除します。

## 修正ルール

jafixの修正ルールは3つで、Markdownの本文にだけ適用します。

| ルールのID                   | 直す内容                                                               |
| ---------------------------- | ---------------------------------------------------------------------- |
| `fullwidth-paren`            | 全角括弧を半角の `(` と `)` に置き換える                               |
| `space-between-ja-and-alnum` | 日本語の文字と半角英数字の間の空白を削除する                           |
| `space-between-ja-and-paren` | 日本語の文字と開き括弧の間、閉じ括弧と日本語の文字の間の空白を削除する |

日本語の文字は、漢字・ひらがな・カタカナと長音符 `ー` です。句読点(`。` `、`)は日本語の文字に含めないので、`です。 Rust` の空白は残ります。空白は半角空白と全角空白で、続けて並んだものはまとめて削除します。

本文は、段落・リスト・見出し・表のセル・強調・取り消し線・リンクの文字列・引用・脚注の本文と、インラインのHTMLのタグに挟まれた文字列です。コードブロック、インラインコード、URL、front matter、HTMLのタグとHTMLブロックは直しません。インラインコード・強調・リンク・HTMLのタグとの境目の空白(`` `code` の `` や `**強調** の`)も残します。

## 導入

GitHub Releasesに、macOS(Apple Silicon)とLinux(x86_64・aarch64)のバイナリを置いています。[mise](https://mise.jdx.dev/) では次のコマンドで入れられます。

```sh
mise use -g github:mjun0812/jafix
```

ソースからビルドする場合は、リポジトリを取得したディレクトリで次のコマンドを実行します。

```sh
cargo install --path .
```

## 使い方

`jafix check` は、ファイルを書き換えずに、修正ルールに当たる箇所を1件1行で出力します。列は1始まりで、文字の数で数えます。

```console
$ jafix check t.md
t.md:1:4: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]
```

`jafix fix` は、ファイルの全行に修正ルールを適用して書き戻します。

```sh
jafix fix t.md
```

どちらも、引数で渡されたファイルは拡張子を問わずMarkdownとして扱います。終了コードは次のとおりです。

| 終了コード | `jafix check`                          | `jafix fix`                                    |
| ---------- | -------------------------------------- | ---------------------------------------------- |
| 0          | 修正ルールに当たる箇所が無い           | 成功した                                       |
| 1          | 修正ルールに当たる箇所が1件以上ある    | -                                              |
| 2          | ファイルを読めない、設定ファイルが不正 | ファイルを読み書きできない、設定ファイルが不正 |

## hookへの登録

`jafix hook claude` と `jafix hook codex` は、PostToolUse hookの入力JSONを標準入力から読み、agentが今回の編集で追加・変更したMarkdownファイル(`.md`)の行だけを直します。標準出力には何も書きません。

Claude Codeでは、`settings.json` の `PostToolUse` に登録します。対象行は、Edit・Writeの結果にある差分の `+` 行です。新規作成のファイルは全行を直します。

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

Codexでは、`hooks.json` の `PostToolUse` に登録します。対象行は、apply_patchのpatchの `+` 行を編集後のファイルで位置づけたものです。`Bash` で呼んだapply_patchのpatchも読みます。

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "^(apply_patch|Bash)$",
        "hooks": [{ "type": "command", "command": "jafix hook codex" }]
      }
    ]
  }
}
```

hookの終了コードは次のとおりです。終了コード1はagentの作業を止めず(Claude CodeとCodexのどちらも同じ)、利用者に失敗として見えます。

| 終了コード | 意味                                                                                                        |
| ---------- | ----------------------------------------------------------------------------------------------------------- |
| 0          | 直した、または直すものが無かった                                                                            |
| 1          | 入力JSONを読めない、設定ファイルが不正、対象行を決められない、ファイルを読み書きできない(理由は標準エラー) |

対象行を決められないのは、Claude Codeの入力に差分が無いときと、編集後のファイルでCodexのpatchのchunkの位置を1か所に決められないときです。そのファイルは変えずに、他のファイルは直します。

jafixの対象行は、agentが書いた直後のファイルを前提にします。oxfmtなど同じファイルを書き換える他のformatterと並列に登録すると、行の位置がずれることもあります。その場合は、1つのhookのscriptの中でjafixを先に呼び、hookの入力JSONを標準入力でjafixへ渡してから、他のformatterを呼びます。

```zsh
#!/usr/bin/env zsh
input=$(cat)
print -r -- "$input" | jafix hook claude
# この後に他のformatterを呼ぶ
```

## 設定ファイル

設定ファイルは `$XDG_CONFIG_HOME/jafix/config.toml` です。`XDG_CONFIG_HOME` が未設定なら `~/.config/jafix/config.toml` を読みます。設定ファイルが無ければ、何も除外しません。

`exclude` に書いたファイルは、jafixの対象から外れます。`~/` で始まる値はホームディレクトリに展開し、symlinkを解決した実体のパスどうしで照らし合わせます。意図して規則に反する例を含むファイルを、symlink経由で渡されても外せます。

```toml
exclude = ["~/.dotfiles/docs/bad-examples.md"]
```

`exclude` の値には、絶対パスか `~/` で始まるパスを書きます。hookを実行する作業ディレクトリは一定しないためです。

`exclude` 以外のキーがあるとき、またはTOMLとして読めないときは、理由を標準エラーに書いて終えます。終了コードは、`jafix check` と `jafix fix` が2、hookが1です。

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。
