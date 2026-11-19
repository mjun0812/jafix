# 設定

設定ファイルが無くても、[既定の10ルール](#defaults)で動きます。
ルールを変える場合はTOMLを書きます。

## 設定ファイルの場所 {#files}

| 種類             | パス                          | 使いどころ                         |
| ---------------- | ----------------------------- | ---------------------------------- |
| 共通設定         | `~/.config/jafix/config.toml` | 自分が書く文書すべてに適用する     |
| プロジェクト設定 | `.jafix.toml`                 | リポジトリに置いてチームで共有する |

`XDG_CONFIG_HOME` を設定している場合、共通設定は `$XDG_CONFIG_HOME/jafix/config.toml` です。

プロジェクト設定は対象ファイルのディレクトリから親へたどって最初に見つかった `.jafix.toml` です。
`.git` のあるディレクトリより上は探しません。

## 優先順位 {#precedence}

既定値、共通設定、プロジェクト設定の順に適用します。
同じ項目は後の設定が上書きします。

| 項目                          | 上書きの単位                               |
| ----------------------------- | ------------------------------------------ |
| `[builtin]` と `[filters]`    | ルールごと。書いていないルールは変わらない |
| ルールのオプション            | 表ごと。省略したオプションは既定値に戻る   |
| `exclude` と `report-exclude` | 一覧ごと                                   |

次の例では、`sentence-length` の上限は100 (既定値)になり、`no-unmatched-pair` は無効のままです。

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

presetは複数のルールを束ねたキーです。
値は `true` か `false` です。

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

定義されていないキーがあると、理由を出力して終了コード2で終わります。
`fullwidth-paren` と `4.3.1.丸かっこ（）` のように、逆の表記を求めるルールを両方有効にしたときも同じです。

```console
$ jafix check example.md
jafix: /path/to/.jafix.toml: [builtin]: `foo` は組み込みのルールのIDでもpresetの名前でもない
```

## さらに詳しく {#more}

- [ルール一覧](./rules.md): ルールIDと内容
- [リファレンス](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#設定ファイル): オプションの既定値、`[filters]`、globの解決
