# jafix

jafixは、AI agentが書いた日本語のMarkdownを、機械的に直せる範囲で直すCLIです。Markdownの本文にある全角括弧や半角カナなどを直し、日本語と英数字・括弧の間の空白を削除します。機械的に直せない書き方は、指摘ルールで指摘します。ルールは、全角括弧と空白を直す3つ、textlintの3つのpreset(`textlint-rule-preset-ja-technical-writing`・`@textlint-ja/textlint-rule-preset-ai-writing`・`textlint-rule-preset-ai-words-ja`)から移したもの、利用者がprhの辞書やパターンに書いた置き換えです。

## 組み込みのルール

組み込みのルールは35あります。`fullwidth-paren`・`space-between-ja-and-alnum`・`space-between-ja-and-paren` の3つはjafix独自の修正ルールで、`prh` と `@textlint-rule/pattern` の2つは、利用者が書いた辞書やパターンで置き換える・指摘するルールです。残りの30ルールはtextlintのpresetから移したもので、内訳は、`textlint-rule-preset-ja-technical-writing` の23ルール、`@textlint-ja/textlint-rule-preset-ai-writing` の5ルール(`no-ai-list-formatting` から `ai-tech-writing-guideline` まで)、`textlint-rule-preset-ai-words-ja` の2ルール(`no-ai-words` と `no-short-topic-comma`)で、IDとオプションの名前はtextlintと同じです。ルールの有効・無効とオプションは、設定ファイルの `[builtin]` で変えます(「[設定ファイル](#builtin)」)。

設定に何も書かなければ、jafix独自の3つと `textlint-rule-preset-ja-technical-writing` の23ルールが有効で、オプションの既定値はpresetと同じです。AIらしい書き方を指摘する残りの7ルールは無効です。7ルールには同じ書き方を重ねて見るものがあり、すべてを有効にすると同じ箇所に複数の指摘が出るためです。7ルールは、`[builtin]` にルールのIDかpresetの名前を書いて有効にします(「[presetの名前](#presetの名前)」)。`prh` と `@textlint-rule/pattern` も無効で、どのpresetにも入りません。辞書かパターンが無いと働かないので、`[builtin]` にオプションの表を書いたときだけ有効になります(「[利用者の置き換え](#利用者の置き換え)」)。

`textlint-rule-preset-ja-technical-writing` の23ルールのうち、`max-ten` から `ja-no-redundant-expression` までの11ルールは、文を形態素解析で語(トークン)に分けて判定します。7ルールのうち、`no-ai-colon-continuation`・`no-ai-words`・`no-short-topic-comma` の3つも形態素解析を使います。形態素解析には [lindera](https://github.com/lindera/lindera) と、binaryに埋め込んだIPADICの辞書を使います。

| ルールのID                           | 種類             | 内容                                                                                         | オプション(既定値)                                                                                                                                                                                    |
| ------------------------------------ | ---------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fullwidth-paren`                    | 修正             | 全角括弧を半角の `(` と `)` に置き換える                                                     | なし                                                                                                                                                                                                  |
| `space-between-ja-and-alnum`         | 修正             | 日本語の文字と半角英数字の間の空白を削除する                                                 | なし                                                                                                                                                                                                  |
| `space-between-ja-and-paren`         | 修正             | 日本語の文字と開き括弧の間、閉じ括弧と日本語の文字の間の空白を削除する                       | なし                                                                                                                                                                                                  |
| `sentence-length`                    | 指摘             | 1文の長さが `max` を超える                                                                   | `max`(100)、`skipPatterns`([])、`skipUrlStringLink`(true)、`countBy`("codeunits")                                                                                                                     |
| `max-comma`                          | 指摘             | 1文の `,` の数が `max` を超える                                                              | `max`(3)                                                                                                                                                                                              |
| `max-kanji-continuous-len`           | 指摘             | 漢字が `max` 文字を超えて続く                                                                | `max`(6)、`allow`([])                                                                                                                                                                                 |
| `arabic-kanji-numbers`               | 指摘             | 算用数字と漢数字を使い分ける(JTFスタイルガイドの2.2.2)                                       | なし                                                                                                                                                                                                  |
| `ja-no-mixed-period`                 | 指摘             | 段落の末尾が句点で終わらない                                                                 | `periodMark`("。")、`allowPeriodMarks`([])、`allowEmojiAtEnd`(false)                                                                                                                                  |
| `no-nfd`                             | 修正と指摘       | 濁点・半濁点を前の文字と合成する。合成できない組み合わせは指摘する                           | なし                                                                                                                                                                                                  |
| `no-invalid-control-character`       | 修正             | 制御文字を削除する                                                                           | `allow`([])                                                                                                                                                                                           |
| `no-zero-width-spaces`               | 修正             | ゼロ幅スペース(U+200B)を削除する                                                             | なし                                                                                                                                                                                                  |
| `no-exclamation-question-mark`       | 指摘             | `!` `?` `！` `？` を使わない                                                                 | `allow`([])、`allowHalfWidthExclamation`・`allowFullWidthExclamation`・`allowHalfWidthQuestion`・`allowFullWidthQuestion`(いずれもfalse)                                                              |
| `no-hankaku-kana`                    | 修正             | 半角カナを全角にする                                                                         | なし                                                                                                                                                                                                  |
| `ja-unnatural-alphabet`              | 指摘             | 日本語に挟まれた1文字のアルファベットを使わない                                              | `allow`(["a", "i", "u", "e", "o", "n", "/[A-Z]/"])、`allowCommonCase`(true)                                                                                                                           |
| `no-unmatched-pair`                  | 指摘             | 対になる括弧や引用符が閉じていない                                                           | なし                                                                                                                                                                                                  |
| `max-ten`                            | 指摘             | 1文の読点の数が `max` を超える                                                               | `max`(3)、`strict`(false)、`touten`("、")、`kuten`("。")                                                                                                                                              |
| `no-doubled-joshi`                   | 指摘             | 1文の中で、同じ助詞が近くに続く                                                              | `min_interval`(1)、`strict`(false)、`allow`([])、`separatorCharacters`([".", "．", "。", "?", "!", "？", "！"])、`commaCharacters`(["、", "，"])                                                      |
| `no-doubled-conjunctive-particle-ga` | 指摘             | 1文に逆接の接続助詞「が」が2回以上ある                                                       | なし                                                                                                                                                                                                  |
| `no-doubled-conjunction`             | 指摘             | 段落の中で、文の最初の接続詞が、それより前で接続詞を持つ最後の文と同じ                       | なし                                                                                                                                                                                                  |
| `no-mix-dearu-desumasu`              | 指摘             | 見出し・本文・箇条書きのそれぞれで、「である」調と「ですます」調を併用する                   | `preferInHeader`("")、`preferInBody`("ですます")、`preferInList`("である")、`strict`(false)                                                                                                           |
| `no-double-negative-ja`              | 指摘             | 「ないこともない」などの二重否定を使う                                                       | なし                                                                                                                                                                                                  |
| `no-dropping-the-ra`                 | 指摘             | ら抜き言葉を使う                                                                             | なし                                                                                                                                                                                                  |
| `ja-no-weak-phrase`                  | 指摘             | 「かも」「思う」などの弱い表現を使う                                                         | なし                                                                                                                                                                                                  |
| `ja-no-successive-word`              | 指摘             | 同じ語が2回続く                                                                              | `allowOnomatopee`(true)、`allow`([])                                                                                                                                                                  |
| `ja-no-abusage`                      | 指摘             | 「値を返却する」などの誤用を使う                                                             | なし                                                                                                                                                                                                  |
| `ja-no-redundant-expression`         | 指摘             | 「することができる」などの冗長な表現を使う                                                   | `allowNodeTypes`(["BlockQuote", "Link", "ReferenceDef", "Code"])、`dictOptions`({})                                                                                                                   |
| `no-ai-list-formatting`              | 指摘(既定で無効) | リスト項目の先頭で太字とコロン・ハイフン・ダッシュを組み合わせるか、リスト項目で絵文字を使う | `allows`([])、`disableBoldListItems`・`disableEmojiListItems`(いずれもfalse)                                                                                                                          |
| `no-ai-hype-expressions`             | 指摘(既定で無効) | 誇張した表現(25語)を使う                                                                     | `allows`([])、`disableAbsolutenessPatterns`・`disableAbstractPatterns`・`disabledPredictivePatterns`(いずれもfalse)                                                                                   |
| `no-ai-emphasis-patterns`            | 指摘(既定で無効) | 絵文字と太字を組み合わせるか、太字の情報プレフィックスか、見出しの中で太字を使う             | `allows`([])、`disableEmojiEmphasisPatterns`・`disableInfoPatterns`・`disableHeadingEmphasisPatterns`(いずれもfalse)                                                                                  |
| `no-ai-colon-continuation`           | 指摘(既定で無効) | 述語とコロンで終わる段落の後に、コードブロック・リスト・引用・表を続ける                     | `allows`([])、`disableCodeBlock`・`disableList`・`disableQuote`・`disableTable`(いずれもfalse)                                                                                                        |
| `ai-tech-writing-guideline`          | 指摘(既定で無効) | 冗長な表現・受け身・具体性の無い語・表記の揺れ・箇条書きにできる列挙を使う                   | `allows`([])、`disableRedundancyGuidance`・`disableVoiceGuidance`・`disableClarityGuidance`・`disableConsistencyGuidance`・`disableStructureGuidance`(いずれもfalse)、`enableDocumentAnalysis`(false) |
| `no-ai-words`                        | 指摘(既定で無効) | AIが多用する語を使う。語は品詞と原形で見分ける                                               | `allows`([])、`dictionaryPath`(なし)、`dictionaryMode`("append")                                                                                                                                      |
| `no-short-topic-comma`               | 指摘(既定で無効) | 短い主題を示す「は」の直後に読点を打つ                                                       | `maxLength`(5)                                                                                                                                                                                        |
| `prh`                                | 修正と指摘(既定で無効) | 利用者のprhの辞書に書いた置き換えで本文を直す | `rulePaths`・`ruleContents`(いずれもなし。合わせて1つ以上)、`checkLink`・`checkBlockQuote`・`checkEmphasis`(いずれもfalse)、`checkHeader`・`checkParagraph`(いずれもtrue) |
| `@textlint-rule/pattern`             | 修正と指摘(既定で無効) | 利用者のパターンの一致を、`replace` があれば直し、無ければ指摘する | `patterns`(なし。要素は `message`・`pattern`・`replace`・`allows`・`allowNodeTypes`) |

オプションの意味はtextlintのルールと同じです。`sentence-length` の `countBy` は、長さをUTF-16のコードユニットで数える `"codeunits"` か、文字で数える `"codepoints"` です。`sentence-length` の `skipPatterns`、`no-exclamation-question-mark` と `ja-unnatural-alphabet` の `allow` の要素は、「[パターン](#パターン)」の書き方で書きます。`max-kanji-continuous-len` の `allow` の要素は、漢字の連続の全体と一致する文字列で、`no-invalid-control-character` の `allow` の要素は、1文字の文字列です。

形態素解析を使う11ルールのオプションは、次のとおりです。

- `max-ten` の `max` は0以上の整数です。`touten` と `kuten` は空でない文字列で、`kuten` は読点の数を0に戻す記号を決めるだけです。文の区切り方は変えません
- `no-doubled-joshi` の `min_interval` は1以上の整数です。`allow` の要素は、指摘しない助詞の文字列です。`separatorCharacters` は数えるトークンを選ぶときだけ使い、文の区切り方は変えません
- `no-mix-dearu-desumasu` の `preferInHeader`・`preferInBody`・`preferInList` は、`""`(多いほうに合わせる)・`"ですます"`・`"である"` のどれかです
- `ja-no-successive-word` の `allow` の要素は、「[パターン](#パターン)」の書き方で書き、2つ目の語に当たれば指摘しません
- `ja-no-redundant-expression` の `allowNodeTypes` の要素は、`BlockQuote`・`ListItem`・`Link`・`LinkReference`・`Image`・`Emphasis`・`Strong`・`Delete`・`Code`・`ReferenceDef` のどれかです。`BlockQuote` と `ListItem` は、引用とリスト項目の中の段落を対象から外します。`Link` から `Delete` までは、一致の先頭がその要素の中にあれば指摘しません。`Code` と `ReferenceDef` は書けますが、何も変えません
- `ja-no-redundant-expression` の `dictOptions` は、キーが `dict1` から `dict6` の表です。値の表には、`disabled`(真偽値)と `allows`(パターンの配列)を書けます。`allows` を書くと、その項目の既定の `allows` を置き換えます

AIらしい書き方を指摘する7ルールのオプションは、次のとおりです。

- `no-ai-list-formatting`・`no-ai-hype-expressions`・`no-ai-emphasis-patterns`・`no-ai-colon-continuation`・`ai-tech-writing-guideline` の `allows` の要素は、「[パターン](#パターン)」の書き方で書きます。ルールが見る要素の文字列のどこかに当たれば、その要素からの指摘をすべて出しません。要素は、`no-ai-list-formatting` がリスト項目(入れ子の項目を含む)、`no-ai-hype-expressions` が本文の範囲、`no-ai-emphasis-patterns` が段落と見出し(リスト項目の中の段落では、その段落を含む最も内側のリスト項目)、`no-ai-colon-continuation` が段落、`ai-tech-writing-guideline` が段落の文字列です
- `disable` で始まるオプションは、`true` にすると、その型かカテゴリの指摘を止めます。`no-ai-hype-expressions` の `disabledPredictivePatterns` は、textlintの名前のとおり `disabled` で始まります。`ai-tech-writing-guideline` の5つのカテゴリは、`disableRedundancyGuidance` が簡潔性、`disableVoiceGuidance` が明確性、`disableClarityGuidance` が具体性、`disableConsistencyGuidance` が一貫性、`disableStructureGuidance` が構造化です
- `ai-tech-writing-guideline` の `enableDocumentAnalysis` を `true` にすると、このルールが文書の中で1件以上指摘したときに、文書の先頭(1行1列)へ、指摘の件数とカテゴリ別の内訳を示す指摘を1件足します。この指摘は箇所を持たず、hookでは1行目が対象行のときだけ出るので、textlintと違って `false` を既定値にしています
- `no-ai-words` の `allows` の要素はパターンで、一致した範囲の元の文字列に当たれば、その一致を指摘しません。`dictionaryPath` は、組み込みの54件の辞書に足す利用者の辞書のJSONファイルのパスで、空でない文字列です。相対パスは設定ファイルのあるディレクトリから解決し、`~/` で始まればホームディレクトリに展開します。`dictionaryMode` は、組み込みの辞書と利用者の辞書の両方を使う `"append"` か、利用者の辞書だけを使う `"override"` です
- `no-short-topic-comma` の `maxLength` は1以上の整数です。文頭から読点の手前までの文字数がこれ以下なら指摘します

利用者の辞書は、textlintの `textlint-rule-preset-ai-words-ja` と同じ形のJSONです。

```json
{
  "entries": [
    { "message": "「再考」は言い換える", "tokens": [{ "pos": "名詞", "basic_form": "再考" }] }
  ]
}
```

`entries` の各項目は、指摘の文面の `message` (空でない文字列)と、期待するトークンの並びの `tokens` (空でない配列)を持ちます。項目のほかのキーは読みません。`tokens` の要素は、トークンの項目の名前をキーにした表で、値は文字列か文字列の配列(どれかに一致)です。キーには `surface_form`・`pos`・`pos_detail_1`・`pos_detail_2`・`pos_detail_3`・`conjugated_type`・`conjugated_form`・`basic_form`・`reading`・`pronunciation` を書けます。`_` で始まるキーは一致を調べるときに使わず、`_skippable` (真偽値)が `true` のトークンは、一致しなければ飛ばして次のトークンを当てます。

## 利用者の置き換え

`prh` と `@textlint-rule/pattern` は、利用者が書いた辞書やパターンで本文を直し、指摘するルールです。辞書とパターンはtextlintで使っているものをそのまま書けるので、textlintから移るときに書き直す必要がありません。prhのルールの箇所と、`replace` を持つ `@textlint-rule/pattern` の要素の箇所は修正ルールで、`jafix fix` とhookが直し、`jafix check` が箇所として出します。この置き換えを、以下では利用者の置き換えと呼びます。`replace` を持たない要素の箇所は、利用者の置き換えではなく、指摘ルールの指摘です。

### `prh`

`prh` のオプションは、辞書の出どころの `rulePaths`・`ruleContents` と、見る範囲を決める `check` で始まる5つです。`rulePaths` は空でない文字列の配列で、`ruleContents` は文字列の配列です。2つを合わせて辞書が1つ以上要るので、`prh = true` は不正です。`rulePaths` の相対パスは設定ファイルのあるディレクトリから解決し、`~/` で始まればホームディレクトリに展開します。`checkLink`・`checkBlockQuote`・`checkEmphasis` の初期値はfalseで、リンク・引用・強調(`*x*`)の中を見ません。`true` にするとその中も見ます。ただし `checkLink` が見るのはリンクの文字列で、本文でない裸のURLと自動リンクの文字列は見ません。`checkHeader` と `checkParagraph` の初期値はtrueで、`false` にすると見出しと段落の中を見ません。textlint-rule-prhの `checkCodeComment` と `debug` は、jafixがコードを見ないので使えません。

```toml
[builtin.prh]
rulePaths = ["dict/prh.yml", "~/dict/shared.yml"]
ruleContents = ["rules:\n  - expected: JavaScript\n"]
checkBlockQuote = true
```

辞書は、prh 6.1.0のYAMLです。次の例のように、`expected` に置き換え後の文字列を、`pattern` に置き換え前の正規表現を書きます。

```yaml
imports:
  - path: ./common.yml
    ignoreRules:
      - expected: サーバ
rules:
  - expected: サーバー
    pattern: /サーバ(?!ー)/
    prh: 長音符を付ける
    specs:
      - from: サーバの設定
        to: サーバーの設定
  - expected: JavaScript
```

`rules` の要素は、文字列(その文字列を `expected` にしたルール)か、次のキーを持つ表です。`expected` だけのルールは、英数字の大文字・小文字と全角・半角を区別せずに当て、英数字以外の正規表現の記号は文字そのものに当てます。

- `expected`: 置き換え後の文字列(必須)。`$1` から `$99` で、`pattern` の捕捉グループを参照します
- `pattern`: 置き換え前の正規表現か文字列そのもの。文字列の配列も書けます
- `patterns`: `pattern` と同じ。`pattern` が無いときだけ使います
- `regexpMustEmpty`: `$1` の形の文字列。その番号のグループが空でない一致は、直しません
- `options`: `wordBoundary`(真偽値)だけを読みます。`true` なら、正規表現の全体を `\b` で挟みます
- `specs`: `from` と `to` の組の配列。読み込みのときに、そのルールだけを `from` に当てた結果が `to` と等しいかを確かめます
- `prh`: 指摘の文面に添える補足

`version`・`meta`・`targets` と、知らないキーは使いません。

辞書は、`rulePaths` を書いた順に、次に `ruleContents` を書いた順に読み、1つにまとめます。各辞書は自分のルールを先に持ち、`imports` の辞書を書いた順に取り込みます。まとめるときは、先にあるルールと `expected` が同じルールを捨てるので、`rulePaths` の辞書のルールが `ruleContents` の同じ `expected` のルールに勝ちます。`imports` の相対パスは、取り込む側の辞書のファイルのあるディレクトリから、`ruleContents` の中では設定ファイルのあるディレクトリから解決します。要素の `disableImports` が `true` なら、取り込む辞書の `imports` を読みません。`ignoreRules` は、取り込む辞書から、`expected` が等しいルールか、`pattern` の文字列が等しいルールを除きます。

一致した文字列と置き換え後の文字列が同じ箇所と、長さ0の一致は直しません。指摘と箇所の文面は、`「<一致した文字列>」を「<置き換え後>」にする` で、辞書に `prh` の補足があれば `: <補足>` を続けます。

### `@textlint-rule/pattern`

`@textlint-rule/pattern` のオプションは、空でない表の配列の `patterns` だけです。各要素は、`message`(空でない文字列、必須)、`pattern`(空でない文字列、必須)、`replace`(文字列)、`allows`(パターンの配列)、`allowNodeTypes`(型の名前の配列)を持ちます。`replace` を書くと、その要素の箇所を修正にします。`replace = ""` は、一致した文字列を消します。`replace` の `$1` から `$9` は捕捉グループで、参加しなかったグループと無い番号は、書いたまま残します。`allows` は、一致した文字列に当たれば、その一致を指摘しません。`allowNodeTypes` は、`Paragraph`・`Header`・`BlockQuote`・`List`・`ListItem`・`Table`・`TableRow`・`TableCell`・`Emphasis`・`Strong`・`Delete`・`Link`・`LinkReference` のどれかで、その型の中にある箇所を、この要素だけ対象から外します。

```toml
[builtin."@textlint-rule/pattern"]
patterns = [
  { message = "日付はYYYY-MM-DDで書く", pattern = '/(\d{4})/(\d{2})/(\d{2})/', replace = "$1-$2-$3", allows = ["/1000/01/01/"], allowNodeTypes = ["BlockQuote"] },
  { message = "TODOを残さない", pattern = "/TODO:/i" },
]
```

指摘の文面は、`「<一致した文字列>」: <message>` です。`@textlint-rule/pattern` は、`allowNodeTypes` に書いた型の中を除き、本文のすべてを見ます。

### 正規表現

`prh` の `pattern`・`patterns` と、`@textlint-rule/pattern` の `pattern`・`allows` は、JavaScriptの正規表現(`u` フラグ付き)の構文と意味で読みます。後読み・先読み・後方参照・名前付きグループ・`\p{Script=…}` を使え、`\b`・`\d`・`\w` はASCIIで判定します。組み込みのルールのオプションのパターン(「[パターン](#パターン)」)は、これまでどおりRustの `regex` crateで読みます。

`/source/flags` の形の見分け方は、textlintのそれぞれのルールと同じです。`prh` は `^/(.*)/([gimsuy]*)$` に当たれば正規表現にし、当たらなければ書いた文字列そのものに当てます。`g`・`m`・`u` は常に付き、`i` と `s` は書いたときだけ付き、`y` は無視します。`@textlint-rule/pattern` は `^/(.+)/([guimysd]*)$` に当たれば正規表現にし、`g`・`u` は常に付き、`i`・`m`・`s` は書いたときだけ付き、`d` は無視します。`y` は不正です。

### 修正の順と直さない箇所

`jafix fix` とhookは、利用者の置き換えを組み込みの修正ルールより先に当てます。置き換えた内容を解析し直してから、組み込みの修正ルールを当てるので、置き換えが生んだ全角括弧や日本語と英数字の間の空白は、同じ実行で直ります。

利用者の置き換えの箇所が重なるときは、先に始まる箇所を残します。同じ位置で始まるなら長い箇所を、範囲も同じなら `prh` の箇所を、それも同じなら `patterns` の前にある要素の箇所を残します。残した箇所と重なる箇所は直しません。`jafix check` は、重なりを解いた後の箇所だけを出します。

次の箇所は、直さずに、そのルールのIDで指摘にします。

- 見出しの中の箇所。見出しのアンカーが変わり、見出しへのリンクが切れるためです。文面の後に「見出しのアンカーが変わるので、見出しへのリンクも直す」を続けます。アンカーが変わらない置き換えを見分けることはしません
- 一致した文字列か置き換え後の文字列が、改行を含む箇所。hookは、直した後の行の番号が変わらないことを前提にするためです。文面の後に「改行を含むので直さない」を続けます

置き換えの固定の部分(`prh` の `expected` と `replace` から `$1` などを除いた部分)を有効な組み込みの修正ルールが直すときは、設定ファイルが不正です。たとえば `expected: （$1）` は、`fullwidth-paren` が `（` を `(` に戻すので、置き換えを当てても次の段で戻されます。理由に置き換えと組み込みの修正ルールのIDを書くので、どちらかを無効にするか、置き換えを辞書から外してください。`imports` の `ignoreRules` で外したルールと、無効にした組み込みのルールは、この判定に入りません。

## 修正ルール

修正ルールは、Markdownの本文にだけ適用します。`jafix fix` とhookが直し、`jafix check` は直す箇所を指摘として出します。

日本語の文字は、漢字・ひらがな・カタカナと長音符 `ー` です。句読点(`。` `、`)は日本語の文字に含めないので、`です。 Rust` の空白は残ります。空白は半角空白と全角空白で、続けて並んだものはまとめて削除します。

`no-nfd` は、結合文字の濁点・半濁点や単独の `゛` `゜` を前の文字と合成して、1文字にします。合成して1文字にならない組み合わせ(`あ` と濁点など)は、直さずに指摘します。`no-invalid-control-character` が削除するのは、タブ・改行・復帰を除く制御文字と、U+202AからU+202Eの方向制御文字です。`no-nfd` は引用・強調・リンクの中の文字列を見ません。`no-hankaku-kana` は、段落の直下の文字列だけを直します。表のセル・強調・太字・リンク・引用・見出しの中の半角カナは、直さず指摘もしません。

本文は、段落・リスト・見出し・表のセル・強調・取り消し線・リンクの文字列・引用・脚注の本文と、インラインのHTMLのタグに挟まれた文字列です。コードブロック、インラインコード、画像、URL、front matter、HTMLのタグとHTMLブロックは直しません。インラインコード・強調・リンク・HTMLのタグとの境目の空白(`` `code` の `` や `**強調** の`)も残します。

参照形式のリンクのうち、定義を持つ省略形(`[設定 API]`)とcollapsed形(`[設定 API][]`)の文字列は、直さず指摘もしません。この文字列は定義のラベルを兼ねるので、書き換えると定義との対応が切れるためです。full形(`[設定 API][api]`)の文字列は、他の本文と同じく直します。

見出しの本文では、アンカーが変わる修正ルール(空白を削除する2つ、`no-nfd`、`no-zero-width-spaces`、`no-invalid-control-character`)に当たる箇所を直さず、指摘にします(「[指摘ルール](#指摘ルール)」)。見出しの文字を変えると自動で付くアンカーが変わり、見出しへのリンクが切れるためです。たとえば `## API の設計` のアンカーは `api-の設計` で、空白を消すと `apiの設計` になります。全角括弧は置き換えてもアンカーが変わらないので、見出しでも直します。

## 指摘ルール

指摘ルールは、ファイルを直さずに指摘だけを出すルールです。組み込みの指摘ルール(`replace` の無い `@textlint-rule/pattern` の要素を含む)と、直さない修正ルールの箇所の2種類があります。直さない箇所は、見出しの中の修正ルールの箇所と、見出しの中や改行を含む利用者の置き換えの箇所です(「[修正の順と直さない箇所](#修正の順と直さない箇所)」)。`@textlint-rule/pattern` は、`allowNodeTypes` に書いた型の中を除き、引用・強調・見出しの中の本文にも当たります。見出しの中の指摘は、修正ルールのIDで出し、見出しへのリンクも直すことを文面で伝えます。

組み込みの指摘ルールは、ルールごとに当てる単位が違います。

- 本文の範囲ごとに当てるルール: `max-kanji-continuous-len`・`arabic-kanji-numbers`・`no-exclamation-question-mark`・`ja-unnatural-alphabet`・`no-double-negative-ja`・`no-dropping-the-ra`・`ja-no-weak-phrase`・`ja-no-successive-word`・`ja-no-abusage`・`no-ai-hype-expressions`・`no-ai-words`・`no-short-topic-comma`・`@textlint-rule/pattern` です。形態素解析を使うルールは、本文の範囲を1つずつトークンへ分けます。本文の範囲は、インラインコード・強調・リンク・HTMLのタグで区切られるので、境目をまたいでは一致せず、`照**合**` は `照合` に当たりません。長さ0の一致は指摘にしません。組み込みのルールは、引用・強調・リンクの中、段落の直下でない範囲を、ルールごとにtextlintと同じ文脈で対象から外します。`@textlint-rule/pattern` は、`allowNodeTypes` に書いた型の中だけを対象から外します
- 段落と文の単位で当てるルール: `sentence-length`・`max-comma`・`ja-no-mixed-period`・`no-unmatched-pair`・`max-ten`・`no-doubled-joshi`・`no-doubled-conjunctive-particle-ga`・`no-doubled-conjunction`・`ja-no-redundant-expression` です。Markdownの解析結果のままの段落を見ます。リスト項目・引用・脚注の定義の中の段落も見ますが、`ja-no-mixed-period` はこの3つの中の段落を対象にせず、`sentence-length` は引用の中の段落と、子がリンク1つだけの段落を対象にしません。段落の文字列には、本文のほかにインラインコードの中身と画像のaltも入ります。文は、`。` `.` `?` `!` などで区切り、括弧と引用符が開いている間と、`Mr.` のような英語の略語の `.` では区切りません。形態素解析を使うルールは、引用の中の段落を対象にしません(`ja-no-redundant-expression` は `allowNodeTypes` で決まります)。また、インラインコードを中身と同じ文字数の `ー` に置き換えてから文をトークンに分けるので、コードの中身を判定に入れません
- リスト項目・見出し・段落の元の文字列に当てるルール: `no-ai-list-formatting` はリスト項目ごとに、項目の先頭の太字の型と、項目の範囲にある段落(入れ子の項目の段落を含む)の最初の絵文字を見ます。`no-ai-emphasis-patterns` は段落と見出しごとに見ます。どちらも、インラインコードとHTMLのタグの中で始まるか終わる一致を捨てるので、コードの中の `**` や絵文字には当たりません。リスト項目の中のコードブロックも見ません
- 段落の全体の文字列に当てるルール: `ai-tech-writing-guideline` です。リスト項目・引用・脚注の定義の中の段落も見ます。段落の文字列は、段落の単位のルールと同じ作り方で、インラインコードだけを中身と同じ文字数の半角空白に置き換えます。コードを挟んだ語はつながりません
- 文書の直下の段落と、その次のブロックに当てるルール: `no-ai-colon-continuation` と、`ai-tech-writing-guideline` の構造化の一部です。リスト項目・引用・脚注の定義の中の段落は見ません。`no-ai-colon-continuation` は、次のブロックがコードブロック・リスト・引用・表の段落を、`ai-tech-writing-guideline` は、次のブロックがリストの段落を見ます
- 文書の全体で当てるルール: `no-mix-dearu-desumasu` と、`ai-tech-writing-guideline` の件数の要約です。`no-mix-dearu-desumasu` は、引用の外の見出し・箇条書き(リスト項目の中の段落)・本文(それ以外の段落)のそれぞれで、「である」と「ですます」を文書の全体で数えてから、指摘する箇所を決めます。hookでは、全体を数えたうえで、対象行にある指摘だけを返します

## 導入

GitHub Releasesに、macOS(Apple Silicon)とLinux(x86_64・aarch64)のバイナリを置いています。[mise](https://mise.jdx.dev/) では次のコマンドで入れられます。

```sh
mise use -g github:mjun0812/jafix
```

ソースからビルドする場合は、リポジトリを取得したディレクトリで次のコマンドを実行します。

```sh
cargo install --path .
```

binaryには形態素解析のIPADICの辞書が入るので、大きさは約50MBです。初めてビルドするときは、辞書のソースを `https://Lindera.dev/` から取得するので、ネットワークにつながっている必要があります。

## 使い方

`jafix check` は、ファイルを書き換えずに、修正ルールに当たる箇所と指摘ルールの指摘を、ファイルの中の位置の順に1件1行で出力します。列は1始まりで、文字の数で数えます。`@textlint-rule/pattern` の行は、当たった文字列を `「」` で囲み、要素の `message` を続けます。他のルールの行は、1行の文面だけを書きます。

```console
$ jafix check t.md
t.md:1:1: 「照合」: AIが多用する語 [@textlint-rule/pattern]
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
/path/to/t.md:2:1: 「照合」: AIが多用する語 [@textlint-rule/pattern]
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

設定ファイルは `$XDG_CONFIG_HOME/jafix/config.toml` です。`XDG_CONFIG_HOME` が未設定なら `~/.config/jafix/config.toml` を読みます。設定ファイルが無ければ、組み込みのルールを既定の設定にし(AIらしい書き方を指摘する7ルールと、`prh`・`@textlint-rule/pattern` は無効、ほかは有効)、何も除外しません。

```toml
exclude = ["~/.dotfiles/docs/bad-examples.md", "**/drafts/**"]
report-exclude = ["**/notes/**"]

[builtin]
no-exclamation-question-mark = false
sentence-length = { max = 120, countBy = "codepoints" }

[[builtin."@textlint-rule/pattern".patterns]]
message = "AIが多用する語"
pattern = "/照合|正本/"

[[builtin."@textlint-rule/pattern".patterns]]
message = "感嘆符を使わない"
pattern = "/[!！]/"
```

### 除外

`exclude` に当たるファイルは、修正ルールと指摘ルールのどちらの対象にもなりません。`report-exclude` に当たるファイルでは、修正ルールは適用し、指摘ルールの指摘だけを出しません。規則に反する文言をそのまま写す文書のように、直してよいが指摘は要らないファイルに使います。組み込みの指摘ルールの指摘も、指摘ルールの指摘なので出しません。見出しの中で直さない修正ルールの箇所も指摘なので、`report-exclude` に当たるファイルでは直らず、指摘も出ません。

`exclude` と `report-exclude` の値はglobで、symlinkを解決した対象ファイルの実体の絶対パスと比べます。`*` と `?` は `/` をまたがず、`**` は0個以上のディレクトリに当たります。そのため `**/drafts/*.md` は `drafts/a.md` に当たり、`drafts/sub/a.md` には当たりません。

値のうち、globの特殊文字(`*`、`?`、`[`、`{`)を含む最初のパスの要素より前の部分は、`~/` で始まればホームディレクトリに展開し、そのパスが存在すればsymlinkを解決した実体のパスに置き換えます。特殊文字が無ければ値の全体が対象です。この部分に含まれる特殊文字は、文字そのものとして扱います。そのため、symlinkのパスを書いた値もその実体に当たり、意図して規則に反する例を含むファイルを、symlink経由で渡されても外せます。

値は `/`、`~/`、`**/` のどれかで始めます。globは絶対パスと比べるうえ、hookを実行する作業ディレクトリは一定しないためです。

### `[builtin]`

`[builtin]` の表は、組み込みのルールを1つずつか、presetの名前でまとめて設定します。ルールのIDのキー(「[組み込みのルール](#組み込みのルール)」の表)の値は、次のどれかです。書かなかったルールは既定の設定のままで、AIらしい書き方を指摘する7ルールと、`prh`・`@textlint-rule/pattern` は無効、ほかのルールは有効です。

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
ja-no-redundant-expression = { dictOptions = { dict5 = { disabled = true } } }
```

無効にした修正ルールは、`jafix fix` とhookが適用せず、`jafix check` では出しません。無効にした指摘ルールは、`jafix check` でもhookでも出しません。`space-between-ja-and-alnum` と `space-between-ja-and-paren` を無効にすると、見出しの中の空白の指摘は出ません。

#### presetの名前

`[builtin]` のキーには、ルールのIDのほかに、textlintのpresetの名前を書けます。値は `true` か `false` です。`@` と `/` を含む名前は、TOMLのキーとして引用符で囲みます。

| presetの名前                     | 束ねるルール                                                                                 |
| -------------------------------- | -------------------------------------------------------------------------------------------- |
| `preset-ja-technical-writing`    | `sentence-length` から `ja-no-redundant-expression` までの23ルール(jafix独自の3つは含まない) |
| `@textlint-ja/preset-ai-writing` | `no-ai-list-formatting` から `ai-tech-writing-guideline` までの5ルール                       |
| `preset-ai-words-ja`             | `no-ai-words` と `no-short-topic-comma`                                                      |

`true` は、presetのルールを、presetの既定の有効・無効とオプションで有効にします。`preset-ai-words-ja` の既定の設定は `no-short-topic-comma` を無効にするので、`preset-ai-words-ja = true` で有効になるのは `no-ai-words` だけです。`false` は、presetのルールをすべて無効にします。

```toml
[builtin]
"@textlint-ja/preset-ai-writing" = true
preset-ai-words-ja = true
no-ai-hype-expressions = false
no-short-topic-comma = { maxLength = 3 }
```

presetの名前のキーと、そのpresetのルールのIDのキーを同じ表に両方書いたときは、書いた順によらずルールのIDのキーがpresetの名前のキーより優先します。上の例では、`no-ai-hype-expressions` を除く4つと `no-ai-words` が既定のオプションで、`no-short-topic-comma` が `maxLength = 3` で有効になります。presetの名前の値にオプションの表は書けません。オプションは、ルールのIDのキーで書きます。

#### パターン

`sentence-length` の `skipPatterns`、`no-exclamation-question-mark`・`ja-unnatural-alphabet`・`ja-no-successive-word` の `allow`、`ja-no-redundant-expression` の `dictOptions` の `allows`、AIらしい書き方を指摘する6ルール(`no-short-topic-comma` 以外)の `allows` は、パターンの配列です。各要素は、`/source/flags` の形なら正規表現、それ以外は書いた文字列そのものに当たります。`/source/flags` とみなすのは、最後の `/` の後ろがフラグの文字(`g`・`u`・`i`・`m`・`y`・`s`)だけのときで、`/usr/bin` は文字列そのものです。正規表現はRustの [`regex` crate](https://docs.rs/regex/latest/regex/#syntax) の構文で読み、フラグは `i`・`m`・`s` を使えます。`g` と `u` は書いてよく、無視します。`y` を含む他のフラグは不正です。空の文字列も不正です。

### 不正な設定

次のときは、設定ファイルが不正です。理由を標準エラーに書いて終え、終了コードは `jafix check` と `jafix fix` が2、hookが1です。`[builtin]` が不正なときは、理由にルールのIDかpresetの名前を含め、オプションの誤りでは、オプションの名前も含めます。

- TOMLとして読めないか、`exclude`・`report-exclude`・`builtin` 以外のキーがある。`rules` の表は、知らないキーなので不正です。利用者のパターンは `[builtin]` の `@textlint-rule/pattern` に書きます
- `exclude` か `report-exclude` の値が、`/`・`~/`・`**/` のどれでも始まらないか、globとして読めない
- `[builtin]` のキーが組み込みのルールのIDでもpresetの名前でもない
- `[builtin]` のpresetの名前の値が、`true` か `false` でない
- `[builtin]` のルールのIDの値が、`true`・`false`・表のどれでもない
- `[builtin]` のルールの表に、そのルールの持たないオプションがある(オプションを持たないルールでは、どのキーも当たります)。textlintのオプションのうち `exclusionPatterns`・`forceAppendPeriod`・`checkFootnote`・`checkCode`・`checkImage`・`separatorChars` は、jafixが対応しないので、これに当たります
- オプションの値の型か値が誤っている。`sentence-length` と `max-ten` の `max` は0以上の整数、`max-comma` と `max-kanji-continuous-len` の `max` と `no-doubled-joshi` の `min_interval` は1以上の整数、`countBy` は `"codeunits"` か `"codepoints"`、`no-invalid-control-character` の `allow` の要素は1文字の文字列、パターンは「[パターン](#パターン)」のとおりに読めるもの
- 形態素解析を使うルールのオプションの値が誤っている。`max-ten` の `touten` と `kuten` は空でない文字列、`no-doubled-joshi` の `allow`・`separatorCharacters`・`commaCharacters` の要素は空でない文字列、`no-mix-dearu-desumasu` の `prefer` で始まる3つは `""`・`"ですます"`・`"である"` のどれか、`allowNodeTypes` の要素は「[組み込みのルール](#組み込みのルール)」に挙げた10の型のどれか、`dictOptions` のキーは `dict1` から `dict6` のどれかで、値は `disabled` と `allows` だけを持つ表
- AIらしい書き方を指摘する7ルールのオプションの値が誤っている。`disable` で始まるオプションと `enableDocumentAnalysis` は真偽値、`allows` はパターンの配列、`no-ai-words` の `dictionaryPath` は空でない文字列で `dictionaryMode` は `"append"` か `"override"`、`no-short-topic-comma` の `maxLength` は1以上の整数。textlintの `severity` は、jafixが対応しないので、知らないオプションに当たります
- `no-ai-words` の `dictionaryPath` のファイルが読めないか、JSONとして読めないか、利用者の辞書の形でない。`tokens` の要素に、上に挙げたトークンの項目の名前でも `_` で始まる名前でもないキー(kuromojiの `word_id`・`word_type`・`word_position` を含む)があるときも、これに当たります。理由には `dictionaryPath` とファイルのパスを含めます
- `no-ai-words` の `dictionaryMode` が `"override"` なのに、`dictionaryPath` が無い
- `prh` のオプションが誤っている。`rulePaths` と `ruleContents` に辞書が1つも無い(`prh = true` を含む)、`check` で始まるオプションが真偽値でない、`checkCodeComment`・`debug`・知らないオプションがある。理由には `prh` とオプションの名前を含めます
- `prh` の `rulePaths` か `ruleContents` の辞書が誤っている。理由には `prh`、`rulePaths` か `ruleContents`、辞書のファイルのパス(`ruleContents` では何番目の要素か)を含めます。次のどれかに当たります
  - 辞書のファイルが読めないか、YAMLとして読めない。辞書の値の型が誤っている(`expected` が文字列でない、`rules` が配列でないなど)
  - ルールに `expected` が無いか、`pattern`・`patterns` の値か要素が空の文字列
  - `pattern`・`patterns` が正規表現として読めないか、配列の要素の `i` と `s` のフラグが食い違う。`regexpMustEmpty` が `$1` の形でない
  - `specs` の要素の `from` と `to` のどちらかが空、またはルールを `from` に当てた結果が `to` と違う。理由には、そのルールの `expected`、`from`、`to`、当てた結果を含めます
  - `imports` が循環する
- `@textlint-rule/pattern` のオプションが誤っている。`patterns` が無いか空(`"@textlint-rule/pattern" = true` を含む)、要素に `message` か `pattern` が無い、要素の知らないキー(`forceCode` を含む)がある、`allowNodeTypes` の型の名前が上に挙げたもの以外、`pattern` と `allows` が正規表現として読めない、`y` フラグがある。理由にはルールのID、オプションの名前、何番目の要素かを含めます
- 逆の置き換え。利用者の置き換えの固定の部分を、有効な組み込みの修正ルールが直す場合です。理由にはルールのID、その置き換え(`expected` か `replace`)、組み込みの修正ルールのIDを含めます

## textlintとの違い

形態素解析を使わない12ルールは、同じ入力に対して、textlint 15.8.0のpreset 12.0.2と同じ箇所を指摘します。textlintと違うのは次の点です。

- 本文ではない箇所を見ません。本文の範囲ごとに当てるルールは、裸のURLと自動リンクの文字列、定義を持つ省略形・collapsed形の参照リンクの文字列、画像、コード、HTMLのタグを見ません。textlintは、`no-zero-width-spaces`・`ja-unnatural-alphabet`・`no-invalid-control-character` でURLと参照リンクの文字列も見て、`no-invalid-control-character` では画像の原文も見ます。段落と文の単位で当てるルールは、textlintと同じく、インラインコードの中身と画像のaltも数えます
- 位置を直しました。textlintが誤った位置を出す3か所を、正しい位置にします。`sentence-length` は文の先頭、`max-comma` はその文の最後の `,`、`ja-no-mixed-period` は末尾の文字(末尾が空白なら空白の先頭)です
- 列は文字で数えます。textlintはUTF-16のコードユニットで数えます。長さの上限(`sentence-length`、`max-kanji-continuous-len`)は、textlintと同じくUTF-16のコードユニットで数えます
- `arabic-kanji-numbers` は、textlintが例外で全体を止める入力(`十十回` など)でも、止まらずに指摘します。textlintが文面に入れる変換後の文字列は入れず、「算用数字で書く」か「漢数字で書く」の後に、一致した文字列を続けます
- `no-nfd` は、合成できない組み合わせを直さずに指摘します。textlintは、文字列を変えないfixerを返します
- 見出しの中では、空白を削除する2つ、`no-nfd`、`no-zero-width-spaces`、`no-invalid-control-character` の箇所を直さず、指摘にします。アンカーが変わるためです
- `ja-no-mixed-period` と `arabic-kanji-numbers` は、textlintと違って自動で直さず、指摘だけを出します
- `exclusionPatterns`・`forceAppendPeriod`・`checkFootnote`・`checkCode`・`checkImage` のオプションは使えません
- `max-comma` と `max-kanji-continuous-len` の `max` に0は書けません。textlintは0をルール自身の既定値(4と5)に読み替えるので、同じ設定でも結果が変わります
- 組み込みのルールのオプションのパターンは、JavaScriptの正規表現ではなく、Rustの `regex` crateの構文で読みます。後読みと後方参照を使えず、フラグは `i`・`m`・`s` です。`prh` と `@textlint-rule/pattern` のパターンはJavaScriptの構文で読みます
- 指摘の文面は、textlintの文面ではなく、jafixの1行の日本語です

形態素解析を使う11ルールは、判定の条件(どのトークンの並びを指摘するか、例外、辞書)をtextlintから写していますが、textlintと同じ箇所を指摘するとは限りません。上の点のうち、列の数え方、パターン、文面は11ルールにも当てはまります。ほかに次の点が違います。

- トークンは、kuromojiではなくlinderaとIPADICで分けます。句読点の直後の語などで品詞がkuromojiと変わることがあり、その文では指摘の有無が変わります
- 位置は、指摘するトークンの元のソースでの位置です。textlintは、絵文字などの後ろ、同じ文字列の子が複数ある文、`no-doubled-conjunctive-particle-ga` の2文目以降などで、位置がずれます
- トークンの並びを当てる途中の状態を、本文の範囲・文・段落・ファイルをまたいで持ち越しません。textlintは、インラインコードや段落をまたいだ並びも指摘します
- 並びは、トークン列のすべての位置から当てます。一致の途中で外れたトークンも次の開始位置として試すので、textlintが指摘しない `ないないことはない` の「ないことはない」も指摘します
- 段落と文の単位のルールは、文から作った文字列を解析します。インラインコードの中身、リンクの宛先、リンクや強調の記号、HTMLのタグは判定に入りません。段落の直下の文字列は元のソースのまま解析するので、エスケープと文字参照は記号として残ります
- 文の区切り方は、ほかの組み込みのルールと同じです。textlintの `max-ten` と `no-doubled-joshi` はオプションで文の区切り文字を変えますが、jafixの `kuten` と `separatorCharacters` は文の区切り方を変えません
- `no-mix-dearu-desumasu` は、引用の中の文を数えず、入れ子のリスト項目の段落も1回ずつ数えます。textlintはリスト項目の原文を丸ごと解析するので、入れ子の項目を二重に数えます
- `no-doubled-conjunctive-particle-ga` の `separatorChars` は使えません。textlintのREADMEに無いオプションです
- `ja-no-abusage` は、`prh:disable` のコメントを扱いません
- `ja-no-abusage` と `ja-no-redundant-expression` は、textlintと違って自動で直さず、指摘だけを出します

AIらしい書き方を指摘する7ルールも、判定の条件(何を指摘するか、例外、語・絵文字・正規表現・辞書)を `@textlint-ja/textlint-rule-preset-ai-writing` 1.7.0と `textlint-rule-preset-ai-words-ja` 1.2.1から写し、textlintの不具合に近い挙動は写しません。上の点のうち、列の数え方、パターン、文面は7ルールにも当てはまります。形態素解析を使う3つ(`no-ai-colon-continuation`・`no-ai-words`・`no-short-topic-comma`)のトークンの分け方と位置も、11ルールと同じです。ほかに次の点が違います。

- インラインコードの中身とHTMLのタグを判定に入れません。`no-ai-list-formatting` と `no-ai-emphasis-patterns` は、インラインコードの中の絵文字や `**` に当てません。太字の中にインラインコードがあっても、太字には当てます。`ai-tech-writing-guideline` は、インラインコードを空にせず、同じ文字数の半角空白に置き換えてから当てるので、コードを挟んだ語はつながりません
- `no-ai-list-formatting` と `no-ai-emphasis-patterns` は、リスト項目の中の段落だけを見ます。textlintはリスト項目の原文の全体を見るので、項目の中のコードブロックやHTMLのブロックにも当てます
- `no-ai-colon-continuation` は、コロンの前の空白を除いてから最後のトークンを見ます。textlintは除かないので、`使用方法 :` を空白で終わる文として指摘します
- `no-ai-words` は、語の並びを当てる途中の状態を、本文の範囲をまたいで持ち越しません。並びは、すべての位置から当てます。textlintは、段落の境目の `腑に` と `落ちる` のように、本文の範囲をまたいだ並びも指摘します
- `no-ai-words` の利用者の辞書では、jafixのトークンに無い項目(`word_id`・`word_type`・`word_position`)と知らない項目の名前を、設定の不正にします。textlintでは、その項目を持つトークンが当たらなくなるだけです
- `no-short-topic-comma` は、`true` を書けば既定のオプションで有効になります。文頭から読点の手前までの長さは、UTF-16のコードユニットではなく文字で数えます
- `ai-tech-writing-guideline` の `enableDocumentAnalysis` の既定値は `false` です。件数の要約の文面は、件数とカテゴリ別の内訳だけの1行です。textlintの `severity` のオプションは使えません
- `no-ai-emphasis-patterns` は、見出しの中の太字を自動で直さず、指摘だけを出します

`prh` と `@textlint-rule/pattern` も、判定の条件(辞書の形式、一致の条件、`replace` の展開)を `textlint-rule-prh` 6.1.1・`prh` 6.1.0・`@textlint-rule/textlint-rule-pattern` 2.0.2・`@textlint/regexp-string-matcher` 2.0.2から写し、textlintの不具合に近い挙動は写しません。上の点のうち、列の数え方と文面は2つのルールにも当てはまります。ほかに次の点が違います。

- `prh` の `/source/flags` で、`s` と `u` のフラグを読みます。prh 6.1.0は `g`・`i`・`m`・`y` だけを読み、`/a/s` を文字列そのものとして扱います
- `prh` の `expected` だけのルールは、英数字以外の正規表現の記号をエスケープします。textlintでは `expected: C++` の辞書が読み込みで失敗します。そのため、記号を含む `expected` だけのルールは、`ignoreRules` の `pattern` にprhの `--rules-yaml` の出力を写しても外れません。`ignoreRules` の `expected` で外します
- `prh` の `options.wordBoundary` は、正規表現の全体を `\b` で挟みます。prhは括弧で囲まずにつなぐので、`/a|b/` の `\b` が片方ずつにしか掛かりません
- `prh` の箇所の重なりは、残した箇所と比べて解きます。prhは1つ前の箇所(捨てたものを含む)と比べるので、重なったまま残る箇所があります。`$1` を展開した結果が一致した文字列と同じ箇所は、他のルールの重なる箇所を捨てさせません
- `prh` の `prh:disable` のプラグマを扱いません。`version` の食い違いは不正にしません。`imports` の絶対パスはそのまま使い、`ruleContents` の中の `imports` は設定ファイルのあるディレクトリから解決します。`imports` の循環は設定の不正にします
- `prh` の `checkCodeComment` と `debug`、`@textlint-rule/pattern` の `forceCode` は使えません
- `@textlint-rule/pattern` の `allowNodeTypes` は、当たった要素の、そのnodeの中の箇所だけを除きます。textlintは、そのnodeの以降の箇所と以降の要素も調べません
- `@textlint-rule/pattern` の `replace = ""` は、一致した文字列を消す修正にします。textlintは指摘だけを出します
- `@textlint-rule/pattern` の `patterns` が空なら、設定の不正にします。textlintは文書の先頭に指摘を1件出します。`y` フラグも、先頭から続く一致だけになるので不正にします
- 利用者の置き換えの重なりは、元の内容に対して1回だけ解きます。textlintの `--fix` は、修正を持つルールを1つずつ当てます。ルールを1つ当てるごとに、直した文書を解析し直します。そのため、`@textlint-rule/pattern` の要素どうしが重なると、textlintは後ろで終わる箇所を、jafixは先に始まる箇所を残します。`prh` と `@textlint-rule/pattern` が重なると、textlintは片方で直した後の文書にもう片方を当てます。`jafix check` は、重なりを解いた後の箇所だけを出します。textlintの `@textlint-rule/pattern` は、重なる `replace` 付きの一致もすべて指摘します
- 裸のURL・自動リンク・定義を持つ省略形とcollapsed形の参照リンクの文字列を、`prh` と `@textlint-rule/pattern` は見ません。`prh` は `checkLink = true` のときも見ません。textlintでは、これらも `Str` なので、`@textlint-rule/pattern` は常に調べます。textlint-rule-prhは、参照リンクの文字列を常に、URLの文字列を `checkLink = true` のときに調べます
- 長さ0の一致は箇所にしません
- 見出しの中の箇所と、改行を含む箇所は、直さず指摘だけを出します
- 組み込みの修正ルールと逆の置き換えは、設定の不正にします
- 位置は元のソースでの位置で、列は文字で数えます

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。

組み込みのルールのうち、textlintと同じIDのものは、textlintのルールとライブラリの挙動に合わせて書き直したもので、正規表現や語の表、辞書の一部をそれらのソースから写しています。`prh` と `@textlint-rule/pattern` は、`textlint-rule-prh`・`prh`・`@textlint-rule/textlint-rule-pattern`・`@textlint/regexp-string-matcher` の判定の条件を写しています。binaryには、MIT Licenseのlinderaと、mecab-ipadicの条件で配られるIPADICの辞書も入っています。それぞれの名前・version・著作権表示と、MIT Licenseとmecab-ipadicの条件の文面は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。GitHub Releasesのtarballには、binaryと一緒に `LICENSE-MIT`・`LICENSE-APACHE`・`THIRD_PARTY_NOTICES.md` を入れています。
