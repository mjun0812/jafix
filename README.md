# jafix

jafixは、AI agentが書いた日本語のMarkdownを、機械的に直せる範囲で直すCLIです。Markdownの本文にある全角括弧や半角カナなどを直し、日本語と英数字・括弧の間の空白を削除します。機械的に直せない書き方は、指摘ルールで指摘します。ルールは、全角括弧と空白を直す3つ、textlintの `textlint-rule-preset-ja-technical-writing` から移したもの、設定ファイルに書いたものです。

## 組み込みのルール

組み込みのルールは15あります。`fullwidth-paren`・`space-between-ja-and-alnum`・`space-between-ja-and-paren` 以外は、`textlint-rule-preset-ja-technical-writing` の23ルールのうち、形態素解析を使わない12ルールを移したもので、IDとオプションの名前はtextlintと同じです。設定に何も書かなければ、どのルールも有効で、オプションの既定値はpresetと同じです。ルールの有効・無効とオプションは、設定ファイルの `[builtin]` で変えます(「[設定ファイル](#builtin)」)。

| ルールのID                     | 種類       | 内容                                                                   | オプション(既定値)                                                                                                                       |
| ------------------------------ | ---------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `fullwidth-paren`              | 修正       | 全角括弧を半角の `(` と `)` に置き換える                               | なし                                                                                                                                     |
| `space-between-ja-and-alnum`   | 修正       | 日本語の文字と半角英数字の間の空白を削除する                           | なし                                                                                                                                     |
| `space-between-ja-and-paren`   | 修正       | 日本語の文字と開き括弧の間、閉じ括弧と日本語の文字の間の空白を削除する | なし                                                                                                                                     |
| `sentence-length`              | 指摘       | 1文の長さが `max` を超える                                             | `max`(100)、`skipPatterns`([])、`skipUrlStringLink`(true)、`countBy`("codeunits")                                                        |
| `max-comma`                    | 指摘       | 1文の `,` の数が `max` を超える                                        | `max`(3)                                                                                                                                 |
| `max-kanji-continuous-len`     | 指摘       | 漢字が `max` 文字を超えて続く                                          | `max`(6)、`allow`([])                                                                                                                    |
| `arabic-kanji-numbers`         | 指摘       | 算用数字と漢数字を使い分ける(JTFスタイルガイドの2.2.2)                 | なし                                                                                                                                     |
| `ja-no-mixed-period`           | 指摘       | 段落の末尾が句点で終わらない                                           | `periodMark`("。")、`allowPeriodMarks`([])、`allowEmojiAtEnd`(false)                                                                     |
| `no-nfd`                       | 修正と指摘 | 濁点・半濁点を前の文字と合成する。合成できない組み合わせは指摘する     | なし                                                                                                                                     |
| `no-invalid-control-character` | 修正       | 制御文字を削除する                                                     | `allow`([])                                                                                                                              |
| `no-zero-width-spaces`         | 修正       | ゼロ幅スペース(U+200B)を削除する                                       | なし                                                                                                                                     |
| `no-exclamation-question-mark` | 指摘       | `!` `?` `！` `？` を使わない                                           | `allow`([])、`allowHalfWidthExclamation`・`allowFullWidthExclamation`・`allowHalfWidthQuestion`・`allowFullWidthQuestion`(いずれもfalse) |
| `no-hankaku-kana`              | 修正       | 半角カナを全角にする                                                   | なし                                                                                                                                     |
| `ja-unnatural-alphabet`        | 指摘       | 日本語に挟まれた1文字のアルファベットを使わない                        | `allow`(["a", "i", "u", "e", "o", "n", "/[A-Z]/"])、`allowCommonCase`(true)                                                              |
| `no-unmatched-pair`            | 指摘       | 対になる括弧や引用符が閉じていない                                     | なし                                                                                                                                     |

オプションの意味はtextlintのルールと同じです。`sentence-length` の `countBy` は、長さをUTF-16のコードユニットで数える `"codeunits"` か、文字で数える `"codepoints"` です。`sentence-length` の `skipPatterns`、`no-exclamation-question-mark` と `ja-unnatural-alphabet` の `allow` の要素は、「[パターン](#パターン)」の書き方で書きます。`max-kanji-continuous-len` の `allow` の要素は、漢字の連続の全体と一致する文字列で、`no-invalid-control-character` の `allow` の要素は、1文字の文字列です。

## 修正ルール

修正ルールは、Markdownの本文にだけ適用します。`jafix fix` とhookが直し、`jafix check` は直す箇所を指摘として出します。

日本語の文字は、漢字・ひらがな・カタカナと長音符 `ー` です。句読点(`。` `、`)は日本語の文字に含めないので、`です。 Rust` の空白は残ります。空白は半角空白と全角空白で、続けて並んだものはまとめて削除します。

`no-nfd` は、結合文字の濁点・半濁点や単独の `゛` `゜` を前の文字と合成して、1文字にします。合成して1文字にならない組み合わせ(`あ` と濁点など)は、直さずに指摘します。`no-invalid-control-character` が削除するのは、タブ・改行・復帰を除く制御文字と、U+202AからU+202Eの方向制御文字です。`no-nfd` は引用・強調・リンクの中の文字列を見ません。`no-hankaku-kana` は、段落の直下の文字列だけを直します。表のセル・強調・太字・リンク・引用・見出しの中の半角カナは、直さず指摘もしません。

本文は、段落・リスト・見出し・表のセル・強調・取り消し線・リンクの文字列・引用・脚注の本文と、インラインのHTMLのタグに挟まれた文字列です。コードブロック、インラインコード、画像、URL、front matter、HTMLのタグとHTMLブロックは直しません。インラインコード・強調・リンク・HTMLのタグとの境目の空白(`` `code` の `` や `**強調** の`)も残します。

参照形式のリンクのうち、定義を持つ省略形(`[設定 API]`)とcollapsed形(`[設定 API][]`)の文字列は、直さず指摘もしません。この文字列は定義のラベルを兼ねるので、書き換えると定義との対応が切れるためです。full形(`[設定 API][api]`)の文字列は、他の本文と同じく直します。

見出しの本文では、アンカーが変わる修正ルール(空白を削除する2つ、`no-nfd`、`no-zero-width-spaces`、`no-invalid-control-character`)に当たる箇所を直さず、指摘にします(「[指摘ルール](#指摘ルール)」)。見出しの文字を変えると自動で付くアンカーが変わり、見出しへのリンクが切れるためです。たとえば `## API の設計` のアンカーは `api-の設計` で、空白を消すと `apiの設計` になります。全角括弧は置き換えてもアンカーが変わらないので、見出しでも直します。

## 指摘ルール

指摘ルールは、ファイルを直さずに指摘だけを出すルールです。組み込みの指摘ルール、設定ファイルの `[[rules]]` で足すもの(「[設定ファイル](#設定ファイル)」)、見出しの中で直さない修正ルールの3種類があります。文脈で対象を外すのは組み込みのルールだけで、設定の `[[rules]]` は、引用・強調・見出しの中の本文にも当たります。見出しの中の指摘は、修正ルールのIDで出し、見出しへのリンクも直すことを文面で伝えます。

組み込みの指摘ルールと `[[rules]]` は、当てる単位が違います。

- 本文の範囲ごとに当てるルール: `max-kanji-continuous-len`・`arabic-kanji-numbers`・`no-exclamation-question-mark`・`ja-unnatural-alphabet`と、設定の `[[rules]]` です。本文の範囲は、インラインコード・強調・リンク・HTMLのタグで区切られるので、境目をまたいでは一致せず、`照**合**` は `照合` に当たりません。長さ0の一致は指摘にしません。組み込みのルールは、引用・強調・リンクの中、段落の直下でない範囲を、ルールごとにtextlintと同じ文脈で対象から外します。設定の `[[rules]]` は、対象から外しません
- 段落と文の単位で当てるルール: `sentence-length`・`max-comma`・`ja-no-mixed-period`・`no-unmatched-pair` です。Markdownの解析結果のままの段落を見ます。リスト項目・引用・脚注の定義の中の段落も見ますが、`ja-no-mixed-period` はこの3つの中の段落を対象にせず、`sentence-length` は引用の中の段落と、子がリンク1つだけの段落を対象にしません。段落の文字列には、本文のほかにインラインコードの中身と画像のaltも入ります。文は、`。` `.` `?` `!` などで区切り、括弧と引用符が開いている間と、`Mr.` のような英語の略語の `.` では区切りません

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

`jafix check` は、ファイルを書き換えずに、修正ルールに当たる箇所と指摘ルールの指摘を、ファイルの中の位置の順に1件1行で出力します。列は1始まりで、文字の数で数えます。設定の指摘ルールの行は、当たった文字列を `「」` で囲み、設定の `message` を続けます。組み込みのルールの行は、1行の文面だけを書きます。

```console
$ jafix check t.md
t.md:1:1: 「照合」: AIが多用する語 [ai-words]
t.md:1:3: 日本語と英数字の間の空白を削除する [space-between-ja-and-alnum]
t.md:2:1: 1文が100文字を超えている (101文字) [sentence-length]
```

`jafix fix` は、ファイルの全行に修正ルールを適用して書き戻します。指摘ルールの指摘は出しません。

```sh
jafix fix t.md
```

どちらも、引数で渡されたファイルは拡張子を問わずMarkdownとして扱います。終了コードは次のとおりです。

| 終了コード | `jafix check`                             | `jafix fix`                                    |
| ---------- | ----------------------------------------- | ---------------------------------------------- |
| 0          | 修正ルールに当たる箇所も指摘も無い        | 成功した                                       |
| 1          | 修正ルールに当たる箇所か指摘が1件以上ある | -                                              |
| 2          | ファイルを読めない、設定ファイルが不正    | ファイルを読み書きできない、設定ファイルが不正 |

## hookへの登録

`jafix hook claude` と `jafix hook codex` は、PostToolUse hookの入力JSONを標準入力から読み、agentが今回の編集で追加・変更したMarkdownファイル(`.md`)の行だけを直します。直して書き戻した後の内容で、その行に始まりのある指摘を集め、標準エラーでagentへ返します。標準出力には何も書きません。

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

Codexでは、`hooks.json` の `PostToolUse` に、matcherを `apply_patch` だけにして登録します。対象行は、apply_patchのpatchの `+` 行を編集後のファイルで位置づけたものです。`tool_name` が `apply_patch` 以外の入力は、`Bash` のcommandにpatchの文字列があっても読みません。Codexはshellから呼んだapply_patchを自分で当ててPostToolUseを出さないので、`Bash` の入力にあるpatchは当てられていないためです。

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "apply_patch",
        "hooks": [{ "type": "command", "command": "jafix hook codex" }]
      }
    ]
  }
}
```

指摘が1件以上あると、hookは標準エラーに次の形で書いてexit 2で終えます。PostToolUseのexit 2の標準エラーは、Claude CodeとCodexのどちらでもagentへ届きます。1行目で、編集は適用済みであることと、指摘を直すか、誤検知なら直さずに進めてよいことを伝えます。Codexでは元のtoolの出力がこの文面に置き換わるためです。2行目からは、指摘を `jafix check` と同じ形で1件1行ずつ書きます。`<path>` は、hookが直したファイルのパスです。同じ実行で対象行を決められなかった理由や、ファイルを読み書きできなかった理由があれば、指摘の後に書きます。

```text
jafix: 今回変えた行に指摘がある。編集は適用済み。指摘を直すか、誤検知なら直さずに進めてよい
/path/to/t.md:2:1: 「照合」: AIが多用する語 [ai-words]
```

指摘は今回変えた行の分しか出ないので、同じ行を編集し直さない限り、同じ指摘は繰り返し出ません。hookの終了コードは次のとおりです。終了コード1はagentの作業を止めず(Claude CodeとCodexのどちらも同じ)、利用者に失敗として見えます。

| 終了コード | 意味                                                                                                                                     |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 0          | 指摘が無く、直したか、直すものが無かった                                                                                                 |
| 1          | 指摘が無く、入力JSONを読めない、設定ファイルが不正、対象行を決められない、ファイルを読み書きできない、のどれかに当たる(理由は標準エラー) |
| 2          | 今回変えた行に指摘が1件以上ある(指摘と、あれば対象行を決められない理由などを標準エラーに書く)                                            |

対象行を決められないのは、Claude Codeの入力に差分が無いときと、編集後のファイルでCodexのpatchのchunkの位置を1か所に決められないときです。そのファイルは変えずに、他のファイルは直します。

jafixの対象行は、agentが書いた直後のファイルを前提にします。oxfmtなど同じファイルを書き換える他のformatterと並列に登録すると、行の位置がずれることもあります。その場合は、1つのhookのscriptの中でjafixを先に呼び、hookの入力JSONを標準入力でjafixへ渡してから、他のformatterを呼びます。

```zsh
#!/usr/bin/env zsh
input=$(cat)
print -r -- "$input" | jafix hook claude
# この後に他のformatterを呼ぶ
```

## 設定ファイル

設定ファイルは `$XDG_CONFIG_HOME/jafix/config.toml` です。`XDG_CONFIG_HOME` が未設定なら `~/.config/jafix/config.toml` を読みます。設定ファイルが無ければ、組み込みのルールはすべて既定の設定で有効にし、何も除外せず、設定の指摘ルールも足しません。

```toml
exclude = ["~/.dotfiles/docs/bad-examples.md", "**/drafts/**"]
report-exclude = ["**/notes/**"]

[builtin]
no-exclamation-question-mark = false
sentence-length = { max = 120, countBy = "codepoints" }

[[rules]]
id = "ai-words"
message = "AIが多用する語"
words = ["照合", "正本"]

[[rules]]
id = "bang"
message = "感嘆符を使わない"
pattern = "[!！]"
```

### 除外

`exclude` に当たるファイルは、修正ルールと指摘ルールのどちらの対象にもなりません。`report-exclude` に当たるファイルでは、修正ルールは適用し、指摘ルールの指摘だけを出しません。規則に反する文言をそのまま写す文書のように、直してよいが指摘は要らないファイルに使います。組み込みの指摘ルールの指摘も、指摘ルールの指摘なので出しません。見出しの中で直さない修正ルールの箇所も指摘なので、`report-exclude` に当たるファイルでは直らず、指摘も出ません。

`exclude` と `report-exclude` の値はglobで、symlinkを解決した対象ファイルの実体の絶対パスと比べます。`*` と `?` は `/` をまたがず、`**` は0個以上のディレクトリに当たります。そのため `**/drafts/*.md` は `drafts/a.md` に当たり、`drafts/sub/a.md` には当たりません。

値のうち、globの特殊文字(`*`、`?`、`[`、`{`)を含む最初のパスの要素より前の部分は、`~/` で始まればホームディレクトリに展開し、そのパスが存在すればsymlinkを解決した実体のパスに置き換えます。特殊文字が無ければ値の全体が対象です。この部分に含まれる特殊文字は、文字そのものとして扱います。そのため、symlinkのパスを書いた値もその実体に当たり、意図して規則に反する例を含むファイルを、symlink経由で渡されても外せます。

値は `/`、`~/`、`**/` のどれかで始めます。globは絶対パスと比べるうえ、hookを実行する作業ディレクトリは一定しないためです。

### `[builtin]`

`[builtin]` の表は、組み込みのルールを1つずつ設定します。キーは組み込みのルールのID(「[組み込みのルール](#組み込みのルール)」の表)で、値は次のどれかです。書かなかったルールは、既定の設定で有効です。

| 値                            | 意味                                                           |
| ----------------------------- | -------------------------------------------------------------- |
| `false`                       | そのルールを無効にする                                         |
| `true`                        | 既定のオプションで有効にする                                   |
| オプションの表(`{ max = 5 }`) | 書いたオプションを変えて有効にする。書かないオプションは既定値 |

```toml
[builtin]
no-exclamation-question-mark = false
sentence-length = { max = 120, countBy = "codepoints" }
max-kanji-continuous-len = { allow = ["情報処理推進機構"] }
```

無効にした修正ルールは、`jafix fix` とhookが適用せず、`jafix check` では出しません。無効にした指摘ルールは、`jafix check` でもhookでも出しません。`space-between-ja-and-alnum` と `space-between-ja-and-paren` を無効にすると、見出しの中の空白の指摘は出ません。

#### パターン

`sentence-length` の `skipPatterns`、`no-exclamation-question-mark` と `ja-unnatural-alphabet` の `allow` は、パターンの配列です。各要素は、`/source/flags` の形なら正規表現、それ以外は書いた文字列そのものに当たります。`/source/flags` とみなすのは、最後の `/` の後ろがフラグの文字(`g`・`u`・`i`・`m`・`y`・`s`)だけのときで、`/usr/bin` は文字列そのものです。正規表現はRustの [`regex` crate](https://docs.rs/regex/latest/regex/#syntax) の構文で読み、フラグは `i`・`m`・`s` を使えます。`g` と `u` は書いてよく、無視します。`y` を含む他のフラグは不正です。空の文字列も不正です。

### `[[rules]]`

`[[rules]]` の表は、指摘ルールを1つ定めます。キーは次のとおりで、`pattern` と `words` はちょうど一方を書きます。置き換えは書けず、設定の指摘ルールは直さずに指摘だけを出します。

| キー      | 値                                                                                             |
| --------- | ---------------------------------------------------------------------------------------------- |
| `id`      | 指摘の行の `[<rule-id>]` に出す名前。他の `[[rules]]` や組み込みのルールのIDと同じにはできない |
| `message` | 指摘の行の、当たった文字列の後に出す文面                                                       |
| `pattern` | Rustの [`regex` crate](https://docs.rs/regex/latest/regex/#syntax) の構文の正規表現            |
| `words`   | そのまま探す文字列の配列。同じ位置で重なる語は、長い語の1件だけを指摘にする                    |

`pattern` の `regex` crateは、後読みと後方参照を使えません。漢字などの文字種は `\p{sc=Han}` のようにScriptで書きます。`\p{Han}` はScript_Extensionsで判定するので、`、` や `。` も含みます。

### 不正な設定

次のときは、設定ファイルが不正です。理由を標準エラーに書いて終え、終了コードは `jafix check` と `jafix fix` が2、hookが1です。`[[rules]]` の表が不正なときは、理由にその表の `id` を含めます。`[builtin]` が不正なときは、理由にルールのIDを含め、オプションの誤りでは、オプションの名前も含めます。

- TOMLとして読めないか、`exclude`・`report-exclude`・`builtin`・`rules` 以外のキーがある
- `exclude` か `report-exclude` の値が、`/`・`~/`・`**/` のどれでも始まらないか、globとして読めない
- `[builtin]` のキーが組み込みのルールのIDでない
- `[builtin]` の値が、`true`・`false`・表のどれでもない
- `[builtin]` のルールの表に、そのルールの持たないオプションがある(オプションを持たないルールでは、どのキーも当たります)。textlintのオプションのうち `exclusionPatterns`・`forceAppendPeriod`・`checkFootnote`・`checkCode`・`checkImage` は、jafixが対応しないので、これに当たります
- オプションの値の型か値が誤っている。`sentence-length` の `max` は0以上の整数、`max-comma` と `max-kanji-continuous-len` の `max` は1以上の整数、`countBy` は `"codeunits"` か `"codepoints"`、`no-invalid-control-character` の `allow` の要素は1文字の文字列、パターンは「[パターン](#パターン)」のとおりに読めるもの
- `[[rules]]` の表に未知のキーがある
- `[[rules]]` の表に `id` か `message` が無い
- `[[rules]]` の表に `pattern` と `words` の両方があるか、両方とも無い
- `pattern` が正規表現として読めないか、`words` に空の文字列がある
- `id` が他の `[[rules]]` か、15の組み込みのルールのIDのどれかと同じ

## textlintとの違い

組み込みの12ルールは、同じ入力に対して、textlint 15.8.0のpreset 12.0.2と同じ箇所を指摘します。textlintと違うのは次の点です。

- 本文ではない箇所を見ません。本文の範囲ごとに当てるルールは、裸のURLと自動リンクの文字列、定義を持つ省略形・collapsed形の参照リンクの文字列、画像、コード、HTMLのタグを見ません。textlintは、`no-zero-width-spaces`・`ja-unnatural-alphabet`・`no-invalid-control-character` でURLと参照リンクの文字列も見て、`no-invalid-control-character` では画像の原文も見ます。段落と文の単位で当てるルールは、textlintと同じく、インラインコードの中身と画像のaltも数えます
- 位置を直しました。textlintが誤った位置を出す3か所を、正しい位置にします。`sentence-length` は文の先頭、`max-comma` はその文の最後の `,`、`ja-no-mixed-period` は末尾の文字(末尾が空白なら空白の先頭)です
- 列は文字で数えます。textlintはUTF-16のコードユニットで数えます。長さの上限(`sentence-length`、`max-kanji-continuous-len`)は、textlintと同じくUTF-16のコードユニットで数えます
- `arabic-kanji-numbers` は、textlintが例外で全体を止める入力(`十十回` など)でも、止まらずに指摘します。textlintが文面に入れる変換後の文字列は入れず、「算用数字で書く」か「漢数字で書く」の後に、一致した文字列を続けます
- `no-nfd` は、合成できない組み合わせを直さずに指摘します。textlintは、文字列を変えないfixerを返します
- 見出しの中では、空白を削除する2つ、`no-nfd`、`no-zero-width-spaces`、`no-invalid-control-character` の箇所を直さず、指摘にします。アンカーが変わるためです
- `ja-no-mixed-period` と `arabic-kanji-numbers` は、textlintと違って自動で直さず、指摘だけを出します
- `exclusionPatterns`・`forceAppendPeriod`・`checkFootnote`・`checkCode`・`checkImage` のオプションは使えません
- `max-comma` と `max-kanji-continuous-len` の `max` に0は書けません。textlintは0をルール自身の既定値(4と5)に読み替えるので、同じ設定でも結果が変わります
- パターンは、JavaScriptの正規表現ではなく、Rustの `regex` crateの構文で読みます。後読みと後方参照を使えず、フラグは `i`・`m`・`s` です
- 指摘の文面は、textlintの文面ではなく、jafixの1行の日本語です

presetの23ルールのうち、形態素解析を使う11ルールは組み込んでいません。

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。

組み込みのルールのうち、textlintと同じIDのものは、textlintのルールとライブラリの挙動に合わせて書き直したもので、正規表現や語の表の一部をそれらのソースから写しています。写した元の名前・version・著作権表示とMIT Licenseの文面は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。
