# jafix

jafixは、AI agentが書いた日本語のMarkdownを、機械的に直せる範囲で直すCLIです。Markdownの本文にある全角括弧や半角カナなどを直し、日本語と英数字・括弧の間の空白を削除します。機械的に直せない書き方は、指摘ルールで指摘します。ルールは、全角括弧と空白を直す3つ、textlintの3つのpreset(`textlint-rule-preset-ja-technical-writing`・`@textlint-ja/textlint-rule-preset-ai-writing`・`textlint-rule-preset-ai-words-ja`)から移したもの、設定ファイルに書いたものです。

## 組み込みのルール

組み込みのルールは33あります。`fullwidth-paren`・`space-between-ja-and-alnum`・`space-between-ja-and-paren` の3つはjafix独自の修正ルールで、残りの30ルールはtextlintのpresetから移したものです。内訳は、`textlint-rule-preset-ja-technical-writing` の23ルール、`@textlint-ja/textlint-rule-preset-ai-writing` の5ルール(`no-ai-list-formatting` から `ai-tech-writing-guideline` まで)、`textlint-rule-preset-ai-words-ja` の2ルール(`no-ai-words` と `no-short-topic-comma`)で、IDとオプションの名前はtextlintと同じです。ルールの有効・無効とオプションは、設定ファイルの `[builtin]` で変えます(「[設定ファイル](#builtin)」)。

設定に何も書かなければ、jafix独自の3つと `textlint-rule-preset-ja-technical-writing` の23ルールが有効で、オプションの既定値はpresetと同じです。AIらしい書き方を指摘する残りの7ルールは無効です。7ルールには同じ書き方を重ねて見るものがあり、すべてを有効にすると同じ箇所に複数の指摘が出るためです。7ルールは、`[builtin]` にルールのIDかpresetの名前を書いて有効にします(「[presetの名前](#presetの名前)」)。

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

## 修正ルール

修正ルールは、Markdownの本文にだけ適用します。`jafix fix` とhookが直し、`jafix check` は直す箇所を指摘として出します。

日本語の文字は、漢字・ひらがな・カタカナと長音符 `ー` です。句読点(`。` `、`)は日本語の文字に含めないので、`です。 Rust` の空白は残ります。空白は半角空白と全角空白で、続けて並んだものはまとめて削除します。

`no-nfd` は、結合文字の濁点・半濁点や単独の `゛` `゜` を前の文字と合成して、1文字にします。合成して1文字にならない組み合わせ(`あ` と濁点など)は、直さずに指摘します。`no-invalid-control-character` が削除するのは、タブ・改行・復帰を除く制御文字と、U+202AからU+202Eの方向制御文字です。`no-nfd` は引用・強調・リンクの中の文字列を見ません。`no-hankaku-kana` は、段落の直下の文字列だけを直します。表のセル・強調・太字・リンク・引用・見出しの中の半角カナは、直さず指摘もしません。

本文は、段落・リスト・見出し・表のセル・強調・取り消し線・リンクの文字列・引用・脚注の本文と、インラインのHTMLのタグに挟まれた文字列です。コードブロック、インラインコード、画像、URL、front matter、HTMLのタグとHTMLブロックは直しません。インラインコード・強調・リンク・HTMLのタグとの境目の空白(`` `code` の `` や `**強調** の`)も残します。

参照形式のリンクのうち、定義を持つ省略形(`[設定 API]`)とcollapsed形(`[設定 API][]`)の文字列は、直さず指摘もしません。この文字列は定義のラベルを兼ねるので、書き換えると定義との対応が切れるためです。full形(`[設定 API][api]`)の文字列は、他の本文と同じく直します。

見出しの本文では、アンカーが変わる修正ルール(空白を削除する2つ、`no-nfd`、`no-zero-width-spaces`、`no-invalid-control-character`)に当たる箇所を直さず、指摘にします(「[指摘ルール](#指摘ルール)」)。見出しの文字を変えると自動で付くアンカーが変わり、見出しへのリンクが切れるためです。たとえば `## API の設計` のアンカーは `api-の設計` で、空白を消すと `apiの設計` になります。全角括弧は置き換えてもアンカーが変わらないので、見出しでも直します。

## 指摘ルール

指摘ルールは、ファイルを直さずに指摘だけを出すルールです。組み込みの指摘ルール(`replace` の無い `@textlint-rule/pattern` の要素を含む)と、見出しの中で直さない修正ルールの2種類があります。`@textlint-rule/pattern` は、`allowNodeTypes` に書いた型の中を除き、引用・強調・見出しの中の本文にも当たります。見出しの中の指摘は、修正ルールのIDで出し、見出しへのリンクも直すことを文面で伝えます。

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

設定ファイルは `$XDG_CONFIG_HOME/jafix/config.toml` です。`XDG_CONFIG_HOME` が未設定なら `~/.config/jafix/config.toml` を読みます。設定ファイルが無ければ、組み込みのルールを既定の設定にし(AIらしい書き方を指摘する7ルールは無効、ほかは有効)、何も除外しません。

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

`[builtin]` の表は、組み込みのルールを1つずつか、presetの名前でまとめて設定します。ルールのIDのキー(「[組み込みのルール](#組み込みのルール)」の表)の値は、次のどれかです。書かなかったルールは既定の設定のままで、AIらしい書き方を指摘する7ルールは無効、ほかのルールは有効です。

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

- TOMLとして読めないか、`exclude`・`report-exclude`・`builtin` 以外のキーがある。利用者のパターンは `rules` の表ではなく、`[builtin]` の `@textlint-rule/pattern` に書きます
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
- パターンは、JavaScriptの正規表現ではなく、Rustの `regex` crateの構文で読みます。後読みと後方参照を使えず、フラグは `i`・`m`・`s` です
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

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。

組み込みのルールのうち、textlintと同じIDのものは、textlintのルールとライブラリの挙動に合わせて書き直したもので、正規表現や語の表、辞書の一部をそれらのソースから写しています。binaryには、MIT Licenseのlinderaと、mecab-ipadicの条件で配られるIPADICの辞書も入っています。それぞれの名前・version・著作権表示と、MIT Licenseとmecab-ipadicの条件の文面は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。GitHub Releasesのtarballには、binaryと一緒に `LICENSE-MIT`・`LICENSE-APACHE`・`THIRD_PARTY_NOTICES.md` を入れています。
