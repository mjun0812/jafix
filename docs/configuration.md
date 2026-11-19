# 設定

設定ファイルがなくても、[既定の10ルール](#defaults)で校正できます。
使うルールやオプションを変えるときは、TOML形式の設定ファイルを作ります。

## 設定ファイルの場所 {#files}

| 種類             | パス                          | 使いどころ                         |
| ---------------- | ----------------------------- | ---------------------------------- |
| 共通設定         | `~/.config/jafix/config.toml` | 複数のプロジェクトで設定を共用する |
| プロジェクト設定 | `.jafix.toml`                 | プロジェクト固有のルールを指定する |

`XDG_CONFIG_HOME` を設定している場合、共通設定は `$XDG_CONFIG_HOME/jafix/config.toml` です。

jafixは対象ファイルのディレクトリから親をたどり、最初に見つかった `.jafix.toml` を使います。
Gitリポジトリでは、`.git` のあるディレクトリまで探します。

## 優先順位 {#precedence}

設定は、既定値、共通設定、プロジェクト設定の順に適用します。
同じ項目を複数の設定で指定した場合は、あとから適用する値が優先されます。

| 項目                          | 上書きの単位                                                 |
| ----------------------------- | ------------------------------------------------------------ |
| `[builtin]` と `[filters]`    | ルールまたはfilterごと。指定がなければ前の設定を引き継ぐ     |
| ルールとfilterのオプション    | 表全体。あとから適用する表で省略したオプションは既定値を使う |
| `exclude` と `report-exclude` | 一覧ごと                                                     |

この例では、プロジェクト設定で `max` を省略しているため、`sentence-length` の上限は既定値の100です。
プロジェクト設定に項目がない `no-unmatched-pair` は、共通設定を引き継いで無効になります。

```toml
# ~/.config/jafix/config.toml
[builtin]
sentence-length = { max = 120 }
no-unmatched-pair = false
```

```toml
# .jafix.toml
[builtin]
sentence-length = { countBy = "codepoints" }
```

## 既定の10ルール {#defaults}

| ルールID                       | 種類       | 内容                                                             |
| ------------------------------ | ---------- | ---------------------------------------------------------------- |
| `fullwidth-paren`              | 修正       | 全角括弧を半角にする                                             |
| `space-between-ja-and-alnum`   | 修正       | 日本語と半角英数字の間の空白を削除する                           |
| `space-between-ja-and-paren`   | 修正       | 日本語と括弧の間の空白を削除する                                 |
| `no-kangxi-radicals`           | 修正       | 康熙部首を対応する漢字にする                                     |
| `no-nfd`                       | 修正と指摘 | 分離した濁点と半濁点を合成する。合成できない組み合わせは指摘する |
| `no-invalid-control-character` | 修正       | 制御文字を削除する                                               |
| `no-zero-width-spaces`         | 修正       | ゼロ幅スペースを削除する                                         |
| `no-unmatched-pair`            | 指摘       | 括弧や引用符の閉じ忘れ                                           |
| `ja-no-successive-word`        | 指摘       | 同じ語の連続                                                     |
| `date-weekday-mismatch`        | 指摘       | 日付と曜日の食い違い                                             |

「修正」は `jafix fix` とhookが書き換えるルール、「指摘」は報告だけをするルールです。

## ルールを個別にオン・オフする {#rules}

`[builtin]` に[ルールID](./rules.md)をキーとして書きます。

| 値              | 意味                           |
| --------------- | ------------------------------ |
| `true`          | 既定のオプションで有効にする   |
| `false`         | 無効にする                     |
| `{ max = 120 }` | オプションを指定して有効にする |

```toml
[builtin]
no-unmatched-pair = false # 既定のルールを無効にする
common-misspellings = true # ルールを追加する
sentence-length = { max = 120 } # オプションを変えて追加する
```

`@`、`/`、`.` を含むIDは引用符で囲みます。

```toml
[builtin]
"@textlint-ja/no-synonyms" = true
"1.1.2.見出し" = true
```

## presetでまとめて有効にする {#presets}

presetを使うと、関連するルールをまとめて指定できます。
`true` で有効にし、`false` で無効にします。

| preset                           | 有効になるルール                                           |
| -------------------------------- | ---------------------------------------------------------- |
| `preset-ja-technical-writing`    | [日本語の技術文書](./rules.md#technical-writing)の23ルール |
| `@textlint-ja/preset-ai-writing` | [AIが多用する書き方](./rules.md#ai-writing)の5ルール       |
| `preset-ai-words-ja`             | `no-ai-words`                                              |
| `preset-ja-spacing`              | [空白](./rules.md#spacing)の9ルール                        |
| `preset-jtf-style`               | [JTFスタイルガイド](./rules.md#jtf-style)の33ルール        |

同じファイルの中では、個別のルールの指定がpresetより優先されます。

```toml
[builtin]
preset-ja-technical-writing = true
ja-no-weak-phrase = false
```

プロジェクト設定のpresetは共通設定に書いた個別の指定も上書きします。

## ファイルを除外する {#exclude}

| キー             | 修正   | 指摘     |
| ---------------- | ------ | -------- |
| `exclude`        | しない | 出さない |
| `report-exclude` | する   | 出さない |

```toml
# .jafix.toml
exclude = ["generated/**"]
report-exclude = ["docs/bad-examples.md"]
```

値はglobです。
共通設定では絶対パスで書くか、`~/` または `**/` で始めます。
Windowsの絶対パスは `C:/work/docs/**` のように書けます。
プロジェクト設定の相対パターンは、`.jafix.toml` のあるディレクトリが基準です。

## 設定が不正なとき {#errors}

jafixは設定に誤りがあると、理由を表示して終了コード2を返します。
たとえば、定義されていないキーを書いたり、`fullwidth-paren` と `4.3.1.丸かっこ（）` のように異なる表記を求めるルールを両方有効にしたりすると、設定エラーになります。

```console
$ jafix check example.md
jafix: /path/to/.jafix.toml: [builtin]: `foo` は組み込みのルールのIDでもpresetの名前でもない
```

## さらに詳しく {#more}

- [ルールIDと内容を調べる](./rules.md)
- [オプションの既定値、`[filters]`、globの解決を確認する](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#設定ファイル)
