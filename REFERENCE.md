# jafix リファレンス

[READMEに戻る](README.md)

jafixはtextlintから移植した日本語向けルールを中心に備えたLinterです。`jafix check` はルールに当たった箇所を表示し、`jafix fix` は修正ルールに当たった箇所を書き換えます。

UTF-8とShift_JIS (CP932を含む)のファイルを読み込めます。拡張子を問わずMarkdownの構文に従って本文を判定します。

辞書やパターンを設定すると、プロジェクト固有の表記に合わせて本文を校正できます。

## 組み込みのルール

組み込みのルールは102個です。
`fullwidth-paren`、`space-between-ja-and-alnum`、`space-between-ja-and-paren` の3つはjafix独自の修正ルールです。
`prh` と `@textlint-rule/pattern` は、利用者が書いた辞書やパターンに従って置き換えや指摘をします。
yomiyasuから移した3つの指摘ルールは、[ルール一覧](docs/rules.md#yomiyasu由来の指摘-yomiyasu)にあります。

textlintから移した94ルールの内訳は次のとおりです。

- `textlint-rule-preset-ja-technical-writing` の23ルール
- `@textlint-ja/textlint-rule-preset-ai-writing` の5ルール(`no-ai-list-formatting` から `ai-tech-writing-guideline` まで)
- `textlint-rule-preset-ai-words-ja` の2ルール(`no-ai-words` と `no-short-topic-comma`)
- `textlint-rule-preset-ja-spacing` の11ルール(`ja-nakaguro-or-halfwidth-space-between-katakana` から `ja-space-around-strong` まで)
- 日本翻訳連盟(Japan Translation Federation、JTF)のスタイルガイドに基づく37ルール(`textlint-rule-preset-jtf-style` の `1.1.1.本文` から `4.3.8.一重引用符` まで)
- `textlint-rule-no-mixed-zenkaku-and-hankaku-alphabet` と `textlint-rule-no-kangxi-radicals`
- ひらがなで書く語、さ入れ言葉、同義語を見る7ルール(`ja-hiragana-hojodoushi` から `@textlint-ja/no-synonyms` まで)
- 言語によらない3ルール(`no-todo`、`date-weekday-mismatch`、`no-dead-link`)
- 英語の文章を見る4ルール(`terminology`、`spelling`、`common-misspellings`、`write-good`)

textlint由来のルールでは、ルールIDとオプションの名前を揃えています。
設定ファイルの [`[builtin]`](#builtin) で、有効にするルールとオプションを選べます。

yomiyasu由来のルールは、次のように1つずつ有効にします。`yomiyasu-chat-response` は、上流の `meta_filler` のうちチャット応答の2つの型を対象にします。

```toml
[builtin]
yomiyasu-bold-not-rendered = true
yomiyasu-instruction-residue = true
yomiyasu-chat-response = true
```

設定ファイルがなくても、次の10ルールが有効です。

- 括弧と空白を整える `fullwidth-paren`、`space-between-ja-and-alnum`、`space-between-ja-and-paren`
- 康熙部首を漢字に置き換える `no-kangxi-radicals`
- 分離した濁点や半濁点、制御文字、ゼロ幅スペースを直す `no-nfd`、`no-invalid-control-character`、`no-zero-width-spaces`
- 括弧や引用符の閉じ忘れ、同じ語の連続、日付と曜日の食い違いを指摘する `no-unmatched-pair`、`ja-no-successive-word`、`date-weekday-mismatch`

ほかのルールは、使うルールのIDか[presetの名前](#presetの名前)を `[builtin]` に書いて有効にします。
`preset-ja-technical-writing = true` を指定すると、文の長さや文体を見るルールを含む23ルールが有効になります。
`prh` と `@textlint-rule/pattern` を使うには、[利用者の置き換え](#利用者の置き換え)のオプションに辞書かパターンを書きます。

`textlint-rule-preset-ja-technical-writing` のうち、`max-ten` から `ja-no-redundant-expression` までの11ルールは、形態素解析で文を語(トークン)に分けて判定します。
`no-ai-colon-continuation`、`no-ai-words`、`no-short-topic-comma` と、ひらがな、さ入れ、同義語の7ルールも形態素解析を使います。
形態素解析には [lindera](https://github.com/lindera/lindera) と、バイナリに埋め込んだIPADICの辞書を使います。

| ルールのID                                        | 種類                   | 内容                                                                                                                 | オプション(既定値)                                                                                                                                                                                    |
| ------------------------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fullwidth-paren`                                 | 修正                   | 全角括弧を半角の `(` と `)` に置き換える                                                                             | なし                                                                                                                                                                                                  |
| `space-between-ja-and-alnum`                      | 修正                   | 日本語の文字と半角英数字の間の空白を削除する                                                                         | なし                                                                                                                                                                                                  |
| `space-between-ja-and-paren`                      | 修正                   | 日本語の文字と開き括弧の間、閉じ括弧と日本語の文字の間の空白を削除する                                               | なし                                                                                                                                                                                                  |
| `sentence-length`                                 | 指摘                   | 1文の長さが `max` を超える                                                                                           | `max`(100)、`skipPatterns`([])、`skipUrlStringLink`(true)、`countBy`("codeunits")                                                                                                                     |
| `max-comma`                                       | 指摘                   | 1文の `,` の数が `max` を超える                                                                                      | `max`(3)                                                                                                                                                                                              |
| `max-kanji-continuous-len`                        | 指摘                   | 漢字が `max` 文字を超えて続く                                                                                        | `max`(6)、`allow`([])                                                                                                                                                                                 |
| `arabic-kanji-numbers`                            | 指摘                   | 算用数字と漢数字を使い分ける(JTFスタイルガイドの2.2.2)                                                               | なし                                                                                                                                                                                                  |
| `ja-no-mixed-period`                              | 指摘                   | 段落の末尾が句点で終わらない                                                                                         | `periodMark`("。")、`allowPeriodMarks`([])、`allowEmojiAtEnd`(false)                                                                                                                                  |
| `no-nfd`                                          | 修正と指摘             | 濁点・半濁点を前の文字と合成する。合成できない組み合わせは指摘する                                                   | なし                                                                                                                                                                                                  |
| `no-invalid-control-character`                    | 修正                   | 制御文字を削除する                                                                                                   | `allow`([])                                                                                                                                                                                           |
| `no-zero-width-spaces`                            | 修正                   | ゼロ幅スペース(U+200B)を削除する                                                                                     | なし                                                                                                                                                                                                  |
| `no-exclamation-question-mark`                    | 指摘                   | `!` `?` `！` `？` を使わない                                                                                         | `allow`([])、`allowHalfWidthExclamation`・`allowFullWidthExclamation`・`allowHalfWidthQuestion`・`allowFullWidthQuestion`(いずれもfalse)                                                              |
| `no-hankaku-kana`                                 | 修正                   | 半角カナを全角にする                                                                                                 | なし                                                                                                                                                                                                  |
| `ja-unnatural-alphabet`                           | 指摘                   | 日本語に挟まれた1文字のアルファベットを使わない                                                                      | `allow`(["a", "i", "u", "e", "o", "n", "/[A-Z]/"])、`allowCommonCase`(true)                                                                                                                           |
| `no-unmatched-pair`                               | 指摘                   | 対になる括弧や引用符が閉じていない                                                                                   | なし                                                                                                                                                                                                  |
| `max-ten`                                         | 指摘                   | 1文の読点の数が `max` を超える                                                                                       | `max`(3)、`strict`(false)、`touten`("、")、`kuten`("。")                                                                                                                                              |
| `no-doubled-joshi`                                | 指摘                   | 1文の中で、同じ助詞が近くに続く                                                                                      | `min_interval`(1)、`strict`(false)、`allow`([])、`separatorCharacters`([".", "．", "。", "?", "!", "？", "！"])、`commaCharacters`(["、", "，"])                                                      |
| `no-doubled-conjunctive-particle-ga`              | 指摘                   | 1文に逆接の接続助詞「が」が2回以上ある                                                                               | なし                                                                                                                                                                                                  |
| `no-doubled-conjunction`                          | 指摘                   | 段落の中で、文の最初の接続詞が、それより前で接続詞を持つ最後の文と同じ                                               | なし                                                                                                                                                                                                  |
| `no-mix-dearu-desumasu`                           | 指摘                   | 見出し・本文・箇条書きのそれぞれで、「である」調と「ですます」調を併用する                                           | `preferInHeader`("")、`preferInBody`("ですます")、`preferInList`("である")、`strict`(false)                                                                                                           |
| `no-double-negative-ja`                           | 指摘                   | 「ないこともない」などの二重否定を使う                                                                               | なし                                                                                                                                                                                                  |
| `no-dropping-the-ra`                              | 指摘                   | ら抜き言葉を使う                                                                                                     | なし                                                                                                                                                                                                  |
| `ja-no-weak-phrase`                               | 指摘                   | 「かも」「思う」などの弱い表現を使う                                                                                 | なし                                                                                                                                                                                                  |
| `ja-no-successive-word`                           | 指摘                   | 同じ語が2回続く                                                                                                      | `allowOnomatopee`(true)、`allow`([])                                                                                                                                                                  |
| `ja-no-abusage`                                   | 指摘                   | 「値を返却する」などの誤用を使う                                                                                     | なし                                                                                                                                                                                                  |
| `ja-no-redundant-expression`                      | 指摘                   | 「することができる」などの冗長な表現を使う                                                                           | `allowNodeTypes`(["BlockQuote", "Link", "ReferenceDef", "Code"])、`dictOptions`({})                                                                                                                   |
| `no-ai-list-formatting`                           | 指摘(既定で無効)       | リスト項目の先頭で太字とコロン・ハイフン・ダッシュを組み合わせるか、リスト項目で絵文字を使う                         | `allows`([])、`disableBoldListItems`・`disableEmojiListItems`(いずれもfalse)                                                                                                                          |
| `no-ai-hype-expressions`                          | 指摘(既定で無効)       | 誇張した表現(25語)を使う                                                                                             | `allows`([])、`disableAbsolutenessPatterns`・`disableAbstractPatterns`・`disabledPredictivePatterns`(いずれもfalse)                                                                                   |
| `no-ai-emphasis-patterns`                         | 指摘(既定で無効)       | 絵文字と太字を組み合わせるか、太字の情報プレフィックスか、見出しの中で太字を使う                                     | `allows`([])、`disableEmojiEmphasisPatterns`・`disableInfoPatterns`・`disableHeadingEmphasisPatterns`(いずれもfalse)                                                                                  |
| `no-ai-colon-continuation`                        | 指摘(既定で無効)       | 述語とコロンで終わる段落の後に、コードブロック・リスト・引用・表を続ける                                             | `allows`([])、`disableCodeBlock`・`disableList`・`disableQuote`・`disableTable`(いずれもfalse)                                                                                                        |
| `ai-tech-writing-guideline`                       | 指摘(既定で無効)       | 冗長な表現・受け身・具体性の無い語・表記の揺れ・箇条書きにできる列挙を使う                                           | `allows`([])、`disableRedundancyGuidance`・`disableVoiceGuidance`・`disableClarityGuidance`・`disableConsistencyGuidance`・`disableStructureGuidance`(いずれもfalse)、`enableDocumentAnalysis`(false) |
| `no-ai-words`                                     | 指摘(既定で無効)       | AIが多用する語を使う。語は品詞と原形で見分ける                                                                       | `allows`([])、`dictionaryPath`(なし)、`dictionaryMode`("append")                                                                                                                                      |
| `no-short-topic-comma`                            | 指摘(既定で無効)       | 短い主題を示す「は」の直後に読点を打つ                                                                               | `maxLength`(5)                                                                                                                                                                                        |
| `prh`                                             | 修正と指摘(既定で無効) | 利用者のprhの辞書に書いた置き換えで本文を直す                                                                        | `rulePaths`・`ruleContents`(いずれもなし。合わせて1つ以上)、`checkLink`・`checkBlockQuote`・`checkEmphasis`(いずれもfalse)、`checkHeader`・`checkParagraph`(いずれもtrue)                             |
| `@textlint-rule/pattern`                          | 修正と指摘(既定で無効) | 利用者のパターンの一致を、`replace` があれば直し、無ければ指摘する                                                   | `patterns`(なし。要素は `message`・`pattern`・`replace`・`allows`・`allowNodeTypes`)                                                                                                                  |
| `ja-nakaguro-or-halfwidth-space-between-katakana` | 指摘(既定で無効)       | カタカナ語の間を全角空白で区切る                                                                                     | なし                                                                                                                                                                                                  |
| `ja-no-space-around-parentheses`                  | 修正(既定で無効)       | 括弧(`[` `]` `（` `）` `［` `］` `「` `」` `『` `』`)の外側と内側の空白を削除する                                    | なし                                                                                                                                                                                                  |
| `ja-no-space-around-slash`                        | 修正(既定で無効)       | `/` の前後の空白を削除する                                                                                           | なし                                                                                                                                                                                                  |
| `ja-no-space-between-full-width`                  | 修正(既定で無効)       | 全角文字どうしの間の半角空白を削除する                                                                               | なし                                                                                                                                                                                                  |
| `ja-space-between-half-and-full-width`            | 修正と指摘(既定で無効) | 全角文字と半角英数字の間の空白を、`space` に合わせて削除するか、空白が無い箇所を指摘する                             | `space`("never")、`lintStyledNode`(false)、`allows`([])                                                                                                                                               |
| `ja-space-after-exclamation`                      | 指摘(既定で無効)       | 文末の `！` の後に別の文が続くのに、直後が半角空白である                                                             | なし                                                                                                                                                                                                  |
| `ja-space-after-question`                         | 指摘(既定で無効)       | 文末の `？` の後に別の文が続くのに、直後が半角空白である                                                             | なし                                                                                                                                                                                                  |
| `ja-space-around-code`                            | 修正と指摘(既定で無効) | インラインコードと日本語の文字の間の空白を、`before`・`after` に合わせて削除するか、空白が無い箇所を指摘する         | `before`・`after`(いずれもfalse)                                                                                                                                                                      |
| `ja-space-around-emphasis`                        | 修正と指摘(既定で無効) | 強調(`*x*`)と日本語の文字の間の空白を、`ja-space-around-code` と同じく削除するか指摘する                             | `before`・`after`(いずれもfalse)                                                                                                                                                                      |
| `ja-space-around-link`                            | 修正と指摘(既定で無効) | リンクと日本語の文字の間の空白を、`ja-space-around-code` と同じく削除するか指摘する                                  | `before`・`after`(いずれもfalse)                                                                                                                                                                      |
| `ja-space-around-strong`                          | 修正と指摘(既定で無効) | 太字(`**x**`)と日本語の文字の間の空白を、`ja-space-around-code` と同じく削除するか指摘する                           | `before`・`after`(いずれもfalse)                                                                                                                                                                      |
| `no-mixed-zenkaku-and-hankaku-alphabet`           | 修正(既定で無効)       | アルファベットを、`prefer` に合わせて半角か全角に揃える                                                              | `prefer`("半角")                                                                                                                                                                                      |
| `no-kangxi-radicals`                              | 修正              | 康熙部首(U+2F00からU+2FD5)を、同じ形の漢字にする                                                                     | なし                                                                                                                                                                                                  |
| `1.1.1.本文`                                      | 指摘(既定で無効)       | 本文で敬体と常体を併用する。文書の全体で数え、少ないほうを指摘する                                                   | なし                                                                                                                                                                                                  |
| `1.1.2.見出し`                                    | 指摘(既定で無効)       | 見出しの末尾に `。` を付ける                                                                                         | なし                                                                                                                                                                                                  |
| `1.1.3.箇条書き`                                  | 指摘(既定で無効)       | リストの項目で、末尾の `。` の有無か、敬体と常体が揃っていない                                                       | なし                                                                                                                                                                                                  |
| `1.1.5.図表のキャプション`                        | 指摘(既定で無効)       | 画像のaltで敬体と常体を併用する                                                                                      | なし                                                                                                                                                                                                  |
| `1.2.1.句点(。)と読点(、)`                        | 指摘(既定で無効)       | 和文の句読点に `,` `.` を使う                                                                                        | なし                                                                                                                                                                                                  |
| `1.2.2.ピリオド(.)とカンマ(,)`                    | 指摘(既定で無効)       | 全角の `，` `．` を使う                                                                                              | なし                                                                                                                                                                                                  |
| `2.1.2.漢字`                                      | 指摘(既定で無効)       | 常用漢字表にない漢字を使う                                                                                           | なし                                                                                                                                                                                                  |
| `2.1.5.カタカナ`                                  | 修正と指摘(既定で無効) | 半角カナを全角にする。カタカナ語の表記を辞書で指摘する                                                               | なし                                                                                                                                                                                                  |
| `2.1.6.カタカナの長音`                            | 指摘(既定で無効)       | カタカナ語の末尾の長音の表記を辞書で指摘する                                                                         | なし                                                                                                                                                                                                  |
| `2.1.8.算用数字`                                  | 修正(既定で無効)       | 全角の数字を半角にする                                                                                               | なし                                                                                                                                                                                                  |
| `2.1.9.アルファベット`                            | 修正(既定で無効)       | 全角のアルファベットを半角にする                                                                                     | なし                                                                                                                                                                                                  |
| `2.1.10.算用数字の位取りの表記`                   | 指摘(既定で無効)       | 小数点に `,` を使う                                                                                                  | なし                                                                                                                                                                                                  |
| `2.2.1.ひらがなと漢字の使い分け`                  | 指摘(既定で無効)       | ひらがなで書く語(`予め` など)を漢字で書く。語は辞書で見る                                                            | なし                                                                                                                                                                                                  |
| `2.2.2.算用数字と漢数字の使い分け`                | 指摘(既定で無効)       | 算用数字と漢数字を使い分ける。`arabic-kanji-numbers` と同じパターンで、すべての一致を指摘する                        | なし                                                                                                                                                                                                  |
| `2.2.3.一部の助数詞の表記`                        | 指摘(既定で無効)       | 数字に続く `ヶ月` などの `ヶ` を使う。語は辞書で見る                                                                 | なし                                                                                                                                                                                                  |
| `3.1.1.全角文字と半角文字の間`                    | 修正(既定で無効)       | 全角文字と半角英数字の間の半角空白を削除する                                                                         | なし                                                                                                                                                                                                  |
| `3.1.2.全角文字どうし`                            | 修正(既定で無効)       | 全角文字どうしの間の半角空白を削除する                                                                               | なし                                                                                                                                                                                                  |
| `3.2.カタカナ語間のスペースの有無`                | 指摘(既定で無効)       | カタカナ語の間を全角空白で区切る                                                                                     | なし                                                                                                                                                                                                  |
| `3.3.かっこ類と隣接する文字の間のスペースの有無`  | 修正と指摘(既定で無効) | 括弧の外側と内側の空白を削除する。`requireOutsideHalfParentheses` なら、半角の丸括弧の外側に空白が無い箇所を指摘する | `allowOutsideHalfParentheses`(true)、`requireOutsideHalfParentheses`(false)                                                                                                                           |
| `4.1.1.句点(。)`                                  | 指摘(既定で無効)       | 閉じ括弧の前に `。` を付ける                                                                                         | なし                                                                                                                                                                                                  |
| `4.1.3.ピリオド(.)、カンマ(,)`                    | 指摘(既定で無効)       | 和文の句点に `.` を使う                                                                                              | なし                                                                                                                                                                                                  |
| `4.2.1.感嘆符(！)`                                | 指摘(既定で無効)       | 和文に半角の `!` を使うか、文末の `！` の後に半角空白を続ける                                                        | なし                                                                                                                                                                                                  |
| `4.2.2.疑問符(？)`                                | 指摘(既定で無効)       | 和文に半角の `?` を使うか、文末の `？` の後に半角空白を続ける                                                        | なし                                                                                                                                                                                                  |
| `4.2.4.中黒(・)`                                  | 修正(既定で無効)       | 半角の中黒 `･` を全角の `・` にする                                                                                  | なし                                                                                                                                                                                                  |
| `4.2.5.波線(〜)`                                  | 指摘(既定で無効)       | 数値の範囲に `~` を使う                                                                                              | なし                                                                                                                                                                                                  |
| `4.2.6.ハイフン(-)`                               | 指摘(既定で無効)       | 和文にハイフン `-` を使う                                                                                            | なし                                                                                                                                                                                                  |
| `4.2.7.コロン(：)`                                | 指摘(既定で無効)       | 和文に半角のコロン `:` を使う                                                                                        | なし                                                                                                                                                                                                  |
| `4.2.8.セミコロン(;)`                             | 指摘(既定で無効)       | 和文にセミコロン `;` を使う                                                                                          | なし                                                                                                                                                                                                  |
| `4.2.9.ダッシュ(-)`                               | 指摘(既定で無効)       | 和文にダッシュ(U+2012からU+2015)を使う                                                                               | なし                                                                                                                                                                                                  |
| `4.3.1.丸かっこ（）`                              | 指摘(既定で無効)       | 和文に半角の丸括弧 `(` `)` を使う                                                                                    | なし                                                                                                                                                                                                  |
| `4.3.2.大かっこ［］`                              | 指摘(既定で無効)       | 和文に半角の大括弧 `[` `]` を使う                                                                                    | なし                                                                                                                                                                                                  |
| `4.3.3.かぎかっこ「」`                            | 指摘(既定で無効)       | 段落の中で `「` `」` が対になっていない                                                                              | なし                                                                                                                                                                                                  |
| `4.3.4.二重かぎかっこ『』`                        | 指摘(既定で無効)       | 段落の中で `『` `』` が対になっていない                                                                              | なし                                                                                                                                                                                                  |
| `4.3.5.二重引用符`                                | 指摘(既定で無効)       | 段落の中の `"` の数が奇数である                                                                                      | なし                                                                                                                                                                                                  |
| `4.3.6.中かっこ{ }`                               | 指摘(既定で無効)       | 段落の中で `{` `}` が対になっていない                                                                                | なし                                                                                                                                                                                                  |
| `4.3.7.山かっこ<>`                                | 指摘(既定で無効)       | 段落の中で `<` `>` が対になっていない                                                                                | なし                                                                                                                                                                                                  |
| `4.3.8.一重引用符`                                | なし(既定で無効)       | 何も指摘しない。textlintのpresetのキーを書けるようにIDだけを持つ                                                     | なし                                                                                                                                                                                                  |
| `ja-hiragana-hojodoushi`                          | 指摘(既定で無効)       | 補助動詞(`お願い致します` の `致し` など)を漢字で書く。語は品詞と読みで見分ける                                      | `rulePath`(なし)                                                                                                                                                                                      |
| `ja-hiragana-keishikimeishi`                      | 指摘(既定で無効)       | 形式名詞(`見た事がある` の `事` など)を漢字で書く。語は品詞と前の語で見分ける                                        | なし                                                                                                                                                                                                  |
| `ja-keishikimeishi`                               | 指摘(既定で無効)       | 形式名詞を漢字で書く。辞書と判定は形式名詞の別のルールと同じで、`ほう`・`うえ` の語をオプションで外せる              | `detection_hou_kata`・`detection_ue`(いずれもtrue)                                                                                                                                                    |
| `ja-hiragana-fukushi`                             | 指摘(既定で無効)       | 副詞(`既に`・`却って` など)を漢字で書く。語は品詞と読みで見分ける                                                    | `rulePath`(なし)                                                                                                                                                                                      |
| `ja-hiragana-daimeishi`                           | 指摘(既定で無効)       | 代名詞(`彼処`・`何処` など)を漢字で書く。語は品詞と読みで見分ける                                                    | なし                                                                                                                                                                                                  |
| `@textlint-ja/no-insert-dropping-sa`              | 指摘(既定で無効)       | さ入れ言葉(`辛さそう`)かさ抜き言葉(`良そう`)を使う                                                                   | なし                                                                                                                                                                                                  |
| `@textlint-ja/no-synonyms`                        | 指摘(既定で無効)       | 文書の中で、同義語の別の表記(`サーバ` と `サーバー` など)を使い分けずに混ぜる                                        | `allows`・`preferWords`(いずれも[])、`allowAlphabet`・`allowNumber`・`allowLexeme`(いずれもtrue)                                                                                                      |
| `no-todo`                                         | 指摘(既定で無効)       | `TODO:` か、リスト項目の未完了のタスクの記号 `[ ]` が残っている                                                      | なし                                                                                                                                                                                                  |
| `date-weekday-mismatch`                           | 指摘              | 日付(`2024年4月23日`・`2024-04-23`・`April 23, 2024` など)の直後の括弧の中の曜日が、日付と食い違う                   | `lang`(なし)、`useCurrentYearIfMissing`(false)、`currentYear`(実行した時点の年)                                                                                                                       |
| `no-dead-link`                                    | 指摘(既定で無効)       | リンク先のファイルが無いか、URLが届かないかリダイレクトする。外部のURLは `jafix check` だけが確かめる                | `checkRelative`(true)、`ignore`・`preferGET`(いずれも[])、`retry`(3)、`concurrency`(8)、`maxRetryTime`・`maxRetryAfterTime`(いずれも10)など                                                           |
| `terminology`                                     | 修正と指摘(既定で無効) | 英語の技術語・ブランド・略語の表記が辞書と違う。利用者の `terms` の項目の箇所は直し、組み込みの辞書の箇所は指摘する  | `terms`([])、`skip`(["BlockQuote"])、`defaultTerms`(true)、`exclude`([])                                                                                                                              |
| `spelling`                                        | 指摘(既定で無効)       | 英単語がen_USのhunspellの辞書に無い。辞書の候補を文面に示す                                                          | `language`("en")、`skipPatterns`([])、`wordDefinitionRegexp`(`/\b[\w']+\b/g`)、`suggestCorrections`(true)                                                                                             |
| `common-misspellings`                             | 指摘(既定で無効)       | 英語のよくある綴りの誤り(Wikipediaの一覧の4,310件)を使う。正しい語を文面に示す                                       | `ignore`([])                                                                                                                                                                                          |
| `write-good`                                      | 指摘(既定で無効)       | 英語の曖昧な語・受動態・冗長な句・決まり文句などを使う                                                               | `weasel`・`illusion`・`so`・`thereIs`・`passive`・`adverb`・`tooWordy`・`cliches`(いずれもtrue)、`eprime`(false)、`whitelist`([])                                                                     |

オプションの意味はtextlintのルールと同じです。`sentence-length` の `countBy` は、長さをUTF-16のコードユニットで数える `"codeunits"` か、文字で数える `"codepoints"` です。`sentence-length` の `skipPatterns`、`no-exclamation-question-mark` と `ja-unnatural-alphabet` の `allow` の要素は、「[パターン](#パターン)」の書き方で書きます。`max-kanji-continuous-len` の `allow` の要素は、漢字の連続の全体と一致する文字列で、`no-invalid-control-character` の `allow` の要素は、1文字の文字列です。

形態素解析を使う11ルールのオプションは、次のとおりです。

- `max-ten` の `max` は0以上の整数です。`touten` と `kuten` は空でない文字列で、`kuten` は読点の数を0に戻す記号を決めるだけです。文の区切り方は変えません
- `no-doubled-joshi` の `min_interval` は1以上の整数です。`allow` の要素は、指摘しない助詞の文字列です。`separatorCharacters` は数えるトークンを選ぶときだけ使い、文の区切り方は変えません
- `no-mix-dearu-desumasu` の `preferInHeader`・`preferInBody`・`preferInList` は、`""`(多いほうに合わせる)・`"ですます"`・`"である"` のどれかです
- `ja-no-successive-word` の `allow` の要素は、「[パターン](#パターン)」の書き方で書き、2つ目の語に当たれば指摘しません。品詞が記号のトークンの連続は、句点、読点、括弧開、括弧閉を指摘します。踊り字(`々`・`〻`・`ゝ`・`ゞ`・`ヽ`・`ヾ`)の連続も指摘し、伏せ字や罫線の記号(`○○`・`──`・`△△`・`……`)は指摘しません
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

### 空白と字種の13ルール

`textlint-rule-preset-ja-spacing` の11ルールと、`no-mixed-zenkaku-and-hankaku-alphabet`・`no-kangxi-radicals` は、空白を削除する箇所と字種を置き換える箇所を修正ルールとして直し、空白を入れる箇所と空白の種類を変える箇所を指摘ルールとして指摘します。hookは、対象行を確かめずに書き換えるので、自動で直すのは、文意を変えない置き換えだけです。空白を足すか、空白の種類を変えるかは、文書ごとの表記の方針に関わるので、agentに判断させます。同じルールでも、オプションの値と前後の側によって、修正と指摘に分かれます。

- `ja-space-between-half-and-full-width` は、`space` が空白を求めない設定(`"never"` か、`"alphabets"` と `"numbers"` のどちらも含まない配列)なら、間の空白を削除する修正ルールです。空白を求める設定(`"always"` か、`"alphabets"` か `"numbers"` を含む配列)なら、空白の無い箇所を指摘する指摘ルールです
- `ja-space-around-code`・`ja-space-around-link`・`ja-space-around-strong`・`ja-space-around-emphasis` は、`before` と `after` のうち `false` の側の空白を削除し、`true` の側で空白の無い箇所を指摘します
- `ja-space-after-exclamation` と `ja-space-after-question` は、直後の半角空白を全角空白にするよう指摘します。textlintは置き換えますが、空白の種類を変えるので直しません。`ja-nakaguro-or-halfwidth-space-between-katakana` は、textlintでも直しません
- `no-mixed-zenkaku-and-hankaku-alphabet` は `prefer` の向きに英字の字種を揃え、`no-kangxi-radicals` は康熙部首を同じ形の漢字に置き換えます。どちらも修正ルールです

オプションは、次のとおりです。

- `ja-space-between-half-and-full-width` の `space` は、`"never"`・`"always"` か、`"alphabets"`・`"numbers"`・`"punctuation"` からなる配列(空の配列を含む)です。`"always"` は `["alphabets", "numbers"]` と同じで、`"punctuation"` を含めると、`、` `。` と半角英数字の間も見ます。`lintStyledNode` を `true` にすると、段落の直下の文字列に加えて、見出し・表のセル・引用・リンクの文字列・太字・強調の中も見ます。`allows` の要素は「[パターン](#パターン)」の書き方で書きます。判定に使った文字の並び(空白を削除する側は前の文字・空白・後ろの文字、空白を求める側は隣り合う2文字)の先頭が、どれかの一致の範囲の中にあれば、その箇所を外します
- 4つの `ja-space-around-*` の `before` と `after` は真偽値です。`before` はnodeの前、`after` はnodeの後ろの空白を決めます
- `no-mixed-zenkaku-and-hankaku-alphabet` の `prefer` は、全角英字を半角にする `"半角"` か、半角英字を全角にする `"全角"` です。`"全角"` は、インラインコードとURLを除く本文の英字をすべて全角にします。元のソースの英字を置き換えるので、`&amp;` のような文字参照の英字も全角になり、文字参照として読まれなくなります
- textlintの `ja-space-between-half-and-full-width` の `exceptPunctuation` は使えません。同じ設定は `space` の配列で書けます

日本語の文字の範囲と、ルールが見る範囲は、ルールごとにtextlintと同じです。jafix独自の修正ルールの日本語の文字(「[修正ルール](#修正ルール)」)とは違います。下の漢字は、U+3400からU+4DBF、U+4E00からU+9FFF、U+F900からU+FAFF、U+20000からU+2FFFFの文字で、ひらがなは `ぁ` から `ん`、カタカナは `ァ` から `ヶ` です。

- `ja-space-between-half-and-full-width`: 日本語の文字は `、` `。`・漢字・ひらがな・カタカナです。段落の直下の文字列(引用の中を除く)を見て、`lintStyledNode = true` なら本文のすべてを見ます
- `ja-no-space-between-full-width`: 日本語の文字は `々` `〇` `〻`・漢字・ひらがな・カタカナで、前後がどちらもカタカナの空白は外します。段落の直下の文字列(引用の中を除く)を見ます
- `ja-nakaguro-or-halfwidth-space-between-katakana`: 前後がカタカナか `ー` の全角空白に当たります。段落の直下の文字列(引用の中を除く)を見ます
- `ja-no-space-around-parentheses`・`ja-no-space-around-slash`・`ja-space-after-exclamation`・`ja-space-after-question`: 日本語の文字かを見ません。段落の直下の文字列(引用の中を除く)を見ます
- 4つの `ja-space-around-*`: 日本語の文字は漢字・ひらがな・カタカナです。インラインコード・リンク・太字・強調の前後を見て、見出し・表のセル・リンクの文字列・太字の中のnodeも見ます。リンクは、インラインのリンクと自動リンク(`<…>`)で、裸のURLと参照形式のリンクを含めません
- `no-mixed-zenkaku-and-hankaku-alphabet`: 日本語の文字かを見ません。本文のうち、引用・リンク・強調の中を除くものを見ます。見出し・表のセル・太字・取り消し線の中は見ます
- `no-kangxi-radicals`: 日本語の文字かを見ません。本文のすべてを見ます

`ー`・`々`・`・` は `ja-space-between-half-and-full-width` の日本語の文字に入らないので、`コンピューター Hello` の空白はこのルールに当たりません(`space-between-ja-and-alnum` には当たります)。`ja-space-between-half-and-full-width` は間の空白が1文字のときだけ当たり、`ja-no-space-between-full-width` は半角空白だけに当たります。`ja-no-space-around-parentheses` と `ja-no-space-around-slash` は、括弧と `/` に隣り合って続く空白をまとめて削除し、行頭の字下げは削除しません。

見出しの中では、13ルールの修正の箇所を直さず、指摘にします(「[修正ルール](#修正ルール)」)。空白を求める側の `ja-space-between-half-and-full-width` と、4つの `ja-space-around-*` の `true` の側は、もともと指摘ルールなので、見出しの中でもアンカーの補足を付けずに指摘します。

### JTFスタイルガイドの37ルール

`textlint-rule-preset-jtf-style` の37ルールは、[JTF日本語標準スタイルガイド(翻訳用)](https://www.jtf.jp/tips/styleguide)に基づき、句読点・記号・括弧の使い方、文体の統一、表記などを確認します。IDは、textlintのpresetの設定に書くキー(`1.1.1.本文` など)と同じ文字列です。全角の括弧や空白を含むので、`[builtin]` には `"4.3.1.丸かっこ（）" = true` のように引用符で囲んで書きます。以下では、IDを先頭の番号(`4.3.1`)で略します。

37ルールは、空白と字種の13ルールと同じ考え方で、修正ルールと指摘ルールに分かれます。字種だけを置き換える箇所と、空白を削除する箇所は直します。記号の種類を変える箇所・記号を外す箇所・空白を足す箇所・辞書で語を置き換える箇所は指摘します。同じIDでも、`2.1.5` は半角カナの部分が修正で辞書の部分が指摘になり、`3.3` は空白を削除する側が修正で空白を求める側が指摘になります。

- 修正(字種の置き換え): `2.1.5` の半角カナの部分(全角カナへ)、`2.1.8`(全角の数字を半角へ)、`2.1.9`(全角の英字を半角へ)、`4.2.4`(`･` を `・` へ)
- 修正(空白の削除): `3.1.1`・`3.1.2` と、`3.3` の空白を削除する側
- 指摘(記号の種類を変える): `1.2.1`・`1.2.2`・`2.1.10`・`4.1.3`・`4.2.5`・`4.2.7`・`4.3.1`・`4.3.2` と、`4.2.1`・`4.2.2` の半角の `!` `?`
- 指摘(記号を外す): `1.1.2`・`4.1.1`
- 指摘(空白を足すか、空白の種類を変える): `3.3` の `requireOutsideHalfParentheses = true` の側と、`4.2.1`・`4.2.2` の文末の `！` `？` の後の半角空白
- 指摘(辞書と表で語を置き換える): `2.1.5` の辞書の部分と、`2.1.6`・`2.2.1`・`2.2.2`・`2.2.3`
- 指摘(textlintも直さない): `1.1.1`・`1.1.3`・`1.1.5`・`2.1.2`・`3.2`・`4.2.6`・`4.2.8`・`4.2.9`・`4.3.3`〜`4.3.7`

`1.2.2` は、全角の `，` `．` を半角にするだけの置き換えに見えますが、指摘にしています。和文の中の `，` は `、` にすべき箇所が多く、`,` と `、` のどちらにするかは文脈で決まるためです。`4.3.8` は、textlintと同じく何も指摘しません。textlintのpresetのキーをそのまま書けるように、IDだけを持ちます。

`3.3` のオプションは、`allowOutsideHalfParentheses`(真偽値、既定値 `true`)と `requireOutsideHalfParentheses`(真偽値、既定値 `false`)です。`allowOutsideHalfParentheses = true` は、半角の `(` の前と `)` の後ろの空白のうち、括弧に接する文字が半角空白のものを削除しません。`日本語 (補足) です` の空白は残り、`日本語　(補足)` の全角空白は削除します。`requireOutsideHalfParentheses = true` は、日本語の文字に接した半角の `(` と `)` を、外側に半角空白を入れるよう指摘します。2つを `allowOutsideHalfParentheses = false` と `requireOutsideHalfParentheses = true` の組で書くと、削除した空白を求め続けるので、設定の不正です。ほかの36ルールはオプションを持たず、値は `true`・`false`・空の表です。

37ルールが見る範囲は、ルールごとにtextlintと同じです。

- 段落の直下の文字列(引用の中を除く): 辞書の部分を除く `2.1.5`、`1.2.1`・`1.2.2`・`2.1.2`・`2.1.8`・`2.1.9`・`2.1.10`・`2.2.2`・`3.1.1`・`3.1.2`・`3.2`・`3.3`・`4.1.1`・`4.1.3`・`4.2.1`・`4.2.2`・`4.2.4`〜`4.2.9`・`4.3.1`・`4.3.2` です。見出し・表のセル・リンクの文字列・強調・太字・取り消し線の中は見ません。修正ルールになる7ルールはどれもここに入るので、見出しの中の修正の箇所はありません
- 見出し: `1.1.2` は、引用の外の見出しが `。` と空白だけで終わるときに、その `。` を指摘します。閉じの `#` を持つATX形式の見出しと、setext形式の見出しは当たりません
- 文書の全体: `1.1.1` は、リンク・引用・強調・リスト項目の中を除く本文(見出し・表のセル・太字・取り消し線・脚注を含む)を数えます。`1.1.5` は、文書のすべてのインラインの画像のaltを数えます。見出し・表のセル・リンク・引用の中の画像も数え、参照形式の画像は数えません。altの中の入れ子のリンクと画像は、文字列とaltを数え、宛先とtitleは数えません
- リスト: `1.1.3` は、リストごとに、そのリストの直下の項目を数えます。項目の文字列は、入れ子のリストを除いた、項目の直下の段落の本文です。入れ子のリストは、別のリストとして数えます
- 段落: `4.3.3`〜`4.3.7` は、引用の外の段落(リスト項目と脚注の定義の中を含む)ごとに、段落の中の本文の記号を数えます。インラインコードの中身と画像のaltは数えません。段落をまたぐ対は、閉じていないものとして指摘します。`4.3.7` は、矢印と比較の記号(`->`・`=>`・`<-`・`<=`・`>=`)の `<` `>` を数えません
- `prh` の既定と同じ本文: 辞書の4ルール(`2.1.5` の辞書の部分・`2.1.6`・`2.2.1`・`2.2.3`)は、リンク・引用・強調の中を除く本文(見出し・表のセル・太字を含む)に当てます。見出しの中の箇所も、文面を変えずに指摘します

`1.1.1`・`1.1.3`・`1.1.5` は、敬体の形(`です。` `ます。` `ですが` など)と常体の形(`だ。` `である。` `のだが` など)を数え、両方があれば少ないほうを指摘します。同数なら常体を指摘します。`1.1.3` は、末尾が `。` の項目と、そうでない項目の数も比べ、少ないほうの項目の先頭を指摘します。同数なら末尾が `。` でない項目を指摘します。hookは、文書・リスト・段落の全体を数えたうえで、対象行にある指摘だけを返します。

日本語の文字は、`3.1.1`・`4.1.3`・`4.2.1` では漢字・ひらがな・カタカナで、ほかのルールではそれに `々` `〇` `〻` を加えたものです。漢字は、空白と字種の13ルールと同じ範囲です。どちらも `ー` を含まないので、`サーバー Linux` の空白は `3.1.1` に当たりません(`space-between-ja-and-alnum` には当たります)。`4.2.7` は `例: これ` のコロンに、`1.2.1` と `4.1.3` は `日本語.md` の `.` に、`4.3.7` は `a < b` の `<` にも当たります。どれも判定の条件のとおりで、textlintも指摘します。使わない書き方のルールは、IDのキーで無効にします。

辞書の4ルールは、`textlint-rule-preset-jtf-style` 3.0.3の4つの辞書(`dict/2.1.5.yml`・`dict/2.1.6.yml`・`dict/2.2.1.yml`・`dict/2.2.3.yml`)を改変せずbinaryへ埋め込み、`prh` と同じ方法で読み込んで当てます。文面も `prh` と同じ `「<一致した文字列>」を「<置き換え後>」にする` です。辞書はJTF日本語標準スタイルガイド2.0を改変したもので、CC BY-SA(表示 - 継承)の条件で配られています。表示と条件は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。

37ルールの修正ルールの箇所が、ほかの組み込みの修正ルールの箇所と同じ範囲に当たるときは、重なりの規則(「[修正ルール](#修正ルール)」)で、ほかのルールの箇所が残ります。37ルールは「[組み込みのルール](#組み込みのルール)」の表の最後にあるためです。重なる組は、`3.1.1` と `space-between-ja-and-alnum`、`2.1.5` の半角カナの部分・`4.2.4` と `no-hankaku-kana`、`2.1.9` と `no-mixed-zenkaku-and-hankaku-alphabet`、`3.1.2` と `ja-no-space-between-full-width`、`3.3` と `space-between-ja-and-paren`・`ja-no-space-around-parentheses` です。たとえば `preset-jtf-style = true` と `fullwidth-paren = false` の設定では、`日本語 Rust` の空白に `space-between-ja-and-alnum` の箇所が出て、`3.1.1` の箇所は出ません。各ルールの判定はそれぞれのままです。

37ルールの指摘ルールには、ほかの指摘ルールと同じ箇所を指摘するものがあります。両方が有効なら、どちらの指摘も出ます。組は、`1.2.1` と `4.1.3`(日本語の直後の `.`)、`4.3.3`〜`4.3.6` と `no-unmatched-pair`、`1.1.1`・`1.1.3` と `no-mix-dearu-desumasu`、`4.2.1`・`4.2.2` と `no-exclamation-question-mark` です。設定を書かなければ37ルールは無効なので、重なるのはpresetかIDのキーで37ルールを有効にしたときだけです。片方だけを使うには、片方のIDのキーを `false` にします。`no-mix-dearu-desumasu` は `prefer` で、`1.1.1` と `1.1.3` は多数決で揃える文体を決めるので、2つの指摘が逆の文体を求めることがあります。見出し・本文・箇条書きのそれぞれを `prefer` の文体に揃えれば、両方を満たせます。

`2.2.2.算用数字と漢数字の使い分け` は `arabic-kanji-numbers` と同じパターンで判定するので、両方を有効にすると、同じ箇所に2つの指摘が出ます。設定を書かなければ `arabic-kanji-numbers` だけが有効なので、重なるのは `preset-jtf-style = true` かルールのIDのキーで `2.2.2` を有効にしたときだけです。指摘する一致の数も違い、`arabic-kanji-numbers` はパターンごとに最初の一致だけを、`2.2.2` はすべての一致を指摘します。片方だけを使うには、`arabic-kanji-numbers = false` か `"2.2.2.算用数字と漢数字の使い分け" = false` を書きます。

`2.2.1.ひらがなと漢字の使い分け` とひらがな系の3ルールが同じ語を指摘する組は、「[ひらがな・さ入れ・同義語の7ルール](#ひらがなさ入れ同義語の7ルール)」にあります。

利用者の置き換えの結果を、辞書の4ルールが別の語にするよう求める組は、設定の不正です(「[不正な設定](#不正な設定)」)。たとえば、`サーバー` を `サーバ` にする `prh` の辞書と `2.1.6` の組です。辞書の4ルールのほかの指摘ルールが置き換えの結果を指摘する組は、設定の不正にしません。`replace` で `:` や `(` を作る `@textlint-rule/pattern` の要素と `4.2.7`・`4.3.1` の組が、その例です。この組では、hookが置き換えた箇所をagentが指摘に従って戻し、次の編集でまた置き換わる、という往復が起きえます。どちらかのルールのIDのキーを `false` にするか、その置き換えを辞書やパターンから外してください。組み込みの修正ルールが置き換えを戻す組も、設定の不正です(「[修正の順と直さない箇所](#修正の順と直さない箇所)」)。

### ひらがな・さ入れ・同義語の7ルール

7ルールは、textlintで単独に配られている次の7つのpackageから移したものです。IDは、textlintが指摘に付けるruleIdと同じで、packageの名前から `textlint-rule-` を除いた名前です。

- `ja-hiragana-hojodoushi`: `textlint-rule-ja-hiragana-hojodoushi` 1.1.0
- `ja-hiragana-keishikimeishi`: `textlint-rule-ja-hiragana-keishikimeishi` 1.1.0
- `ja-keishikimeishi`: `textlint-rule-ja-keishikimeishi` 1.0.5
- `ja-hiragana-fukushi`: `textlint-rule-ja-hiragana-fukushi` 1.3.0
- `ja-hiragana-daimeishi`: `textlint-rule-ja-hiragana-daimeishi` 1.0.0
- `@textlint-ja/no-insert-dropping-sa`: `@textlint-ja/textlint-rule-no-insert-dropping-sa` 2.0.1
- `@textlint-ja/no-synonyms`: `@textlint-ja/textlint-rule-no-synonyms` 1.3.0

7つはどれも指摘ルールです。textlintのルールは組み込みの辞書の語に置き換えて直せますが、jafixは直さずに指摘します。`@textlint-ja/no-synonyms` の `preferWords` への置き換えも同じです。指摘の文面には置き換え後の語が入るので、agentはその場で直せます。設定を書かなければ7つとも無効で、7つを束ねるpresetの名前もありません。`[builtin]` にルールのIDのキーを書いて有効にします。`@textlint-ja/` で始まる2つのIDは、TOMLのキーとして引用符で囲みます。

```toml
[builtin]
ja-hiragana-hojodoushi = true
ja-keishikimeishi = { detection_ue = false }
"@textlint-ja/no-synonyms" = { preferWords = ["サーバー"] }
```

ひらがな系の5ルール(`ja-hiragana-hojodoushi`・`ja-hiragana-keishikimeishi`・`ja-keishikimeishi`・`ja-hiragana-fukushi`・`ja-hiragana-daimeishi`)は、漢字で書いた補助動詞・形式名詞・副詞・代名詞を、ひらがなで書くよう指摘します。語は、組み込みの辞書にあるトークンの並び(表層・品詞・活用形・読みなど)で見分けます。辞書の項目は、補助動詞が56、形式名詞が45、副詞が76、代名詞が15です。2つの形式名詞のルールは同じ45項目の辞書を使い、`ja-keishikimeishi` だけが `ほう` と `うえ` にする項目をオプションで外せます。文面は `補助動詞「致し」はひらがなの「いたし」にする` の形で、語の種類は `補助動詞`・`形式名詞`・`副詞`・`代名詞` のどれかです。`ja-hiragana-daimeishi` の辞書にある副詞の語も、textlintと同じく `代名詞` とします。

`@textlint-ja/no-insert-dropping-sa` は、さ入れ言葉とさ抜き言葉を指摘します。さ入れ言葉は、形容詞の後の `さそう`(`辛さそう`)と、動詞と `た` の後の `さそう`(`見たさそう`)です。さ抜き言葉は、`良い`・`無い` の後に `さ` の無い `そう`(`良そう`)です。文面は `さ入れ言葉「辛さそう」は「辛そう」にする` と `さ抜き言葉「良そう」は「良さそう」にする` です。

ひらがな系の5ルールとさ入れのルールは、本文の範囲のすべてに当てます。見出し・表のセル・引用・リンクの文字列・強調・太字の中も見ます。`@textlint-ja/no-synonyms` は、引用の中とリンクの文字列を除く本文の範囲を見ます。どれも、インラインコードの中身と本文でない箇所(裸のURL、自動リンク、定義を持つ省略形・collapsed形の参照リンクの文字列)を見ません。トークンの並びを当てる途中の状態は、本文の範囲をまたいで持ち越しません。

オプションは、次のとおりです。

- `ja-hiragana-hojodoushi` と `ja-hiragana-fukushi` の `rulePath` は、組み込みの辞書の代わりに使うYAMLの辞書のパスで、空でない文字列です。組み込みの辞書に足すのではなく、置き換えます。相対パスは設定ファイルのあるディレクトリから解決し、`~/` で始まればホームディレクトリに展開します
- `ja-keishikimeishi` の `detection_hou_kata` を `false` にすると `ほう` にする項目を、`detection_ue` を `false` にすると `うえ` にする項目を当てません。2つは独立で、片方だけを書いても他方は変わりません
- `@textlint-ja/no-synonyms` の `allows` と `preferWords` は文字列の配列で、要素はパターンではなく、見出しの全体と比べる文字列です。`allowAlphabet`・`allowNumber`・`allowLexeme` は真偽値です。意味は下の段落にあります
- オプションを持たない3つ(`ja-hiragana-keishikimeishi`・`ja-hiragana-daimeishi`・`@textlint-ja/no-insert-dropping-sa`)の値は、`true`・`false`・空の表です

`rulePath` の辞書は、textlintと同じ形のYAMLです。

```yaml
message: ひらがなで書く補助動詞
dict:
  - expected: いたし
    tokens:
      - surface_form: 致し
        pos: 動詞
        pos_detail_1: 非自立
```

`message` は空でない文字列で、指摘の文面の先頭に使います(`ひらがなで書く補助動詞: 「致し」を「いたし」にする`)。`dict` は空でない配列です。各項目は、置き換え後の語の `expected` (文字列)と、期待するトークンの並びの `tokens` (空でない配列)を持ちます。項目のほかのキー(`extensions` など)は読みません。`tokens` の要素は、`no-ai-words` の利用者の辞書と同じく読みます(「[組み込みのルール](#組み込みのルール)」)。

`@textlint-ja/no-synonyms` は、同じ同義語の集まりの別の表記が、1つの文書の中で2つ以上使われたら指摘します。辞書は、`sudachi-synonyms-dictionary` 20.0.0の `sudachi-synonyms-dictionary.json` から、判定に使う項目だけを取り出してbinaryに埋め込んだものです。このJSONは、SudachiDictの同義語辞書から作られています。

`@textlint-ja/no-synonyms` は、語をlinderaのトークンで区切ります。本文の範囲のトークンを先頭から見て、各トークンから、空白のトークンをまたがずに最大8個までつなげた並びを作ります。そのうち、辞書の見出しに一致する最も長いものを1語とし、その並びの次のトークンから続けます。一致が無ければ、次のトークンから続けます。そのため、`テンプレ` や `電子メール` のように複数のトークンに分かれる見出しにも当たります。一致はトークンの境目でだけ始まり、終わります。

同義語辞書のグループは、`allowLexeme` が `true` なら語彙番号ごとの集まりに分け、`false` ならグループの全体を1つの集まりにします。語彙番号の無い項目は、グループごとに1つの集まりにします。1つの見出しは、複数の集まりに入ることがあります。文書の全体で見出しを数えてから、集まりごとに、文書の中で使われた見出しを決めます。次の見出しは、使われた見出しに数えません。

- `allowAlphabet` が `true` なら、英字の表記(`server`)と英字の略語(`OS`)
- `allowNumber` が `true` なら、ASCIIの数字を含む見出し(`1位`)と、漢数字1字の見出し(`三`)
- `allows` の要素と同じ見出し

集まりの見出しに `preferWords` の要素があり、`allows` の要素がどれも無ければ、使われた見出しのうち `preferWords` の語でないものを指摘します。`preferWords` の要素が集まりに2つ以上あれば、配列で先にあるものを使います。この指摘は、使われた見出しが1つでも出ます。そうでなければ、使われた見出しが2つ以上ある集まりを指摘します。

指摘は、使われた見出しのそれぞれで、その見出しの文書の中の最後の出現の位置へ1件ずつ出します。文面は、`同義語の「サーバ」と「サーバー」が混ざっている` か、`「サーバ」は同義語の「サーバー」にする` です。前者の見出しの並びは、文書の中で最初に出た順です。hookは、文書の全体を数えてから、対象行にある指摘だけを返します。agentの書いた表記が文書の中でその1か所だけなら、既にある表記より前の行と後ろの行のどちらへ書いても、hookはその行の指摘を返します。

2つの形式名詞のルールを両方とも有効にすると、同じ形式名詞に2つのIDの指摘が1件ずつ出ます。辞書が同じなので、片方を使えば足ります。片方だけを使うには、`ja-hiragana-keishikimeishi = false` か `ja-keishikimeishi = false` を書きます。

`2.2.1.ひらがなと漢字の使い分け` は、ひらがな系の3ルール(`ja-hiragana-hojodoushi`・`ja-hiragana-fukushi`・`ja-hiragana-daimeishi`)と同じ語を、同じ向きに直すよう求めることがあります。両方を有効にすると、同じ箇所に2つのIDの指摘が出ます。組は、`下さい`(`ja-hiragana-hojodoushi`)、`更に`・`暫く`・`折角`(`ja-hiragana-fukushi`)、`何処`(`ja-hiragana-daimeishi`)などです。どちらも指摘ルールなので、設定の不正にはしません。片方だけを使うには、`"2.2.1.ひらがなと漢字の使い分け" = false` か、ひらがな系のルールのIDのキーを `false` にします。逆の向きの組もあり、`2.2.1` は `いたす` を `致す` に、`ja-hiragana-hojodoushi` は補助動詞の `致す` を `いたす` にするよう求めます。この組には両方を満たす書き方の無い場合があるので、片方を `false` にしてください。

利用者の置き換えの結果を、ひらがな系の5ルールか、`preferWords` を書いた `@textlint-ja/no-synonyms` が別の語にするよう求める組は、設定の不正です(「[不正な設定](#不正な設定)」)。たとえば、`お願いいたします` を `お願い致します` にする `prh` の辞書と `ja-hiragana-hojodoushi` の組です。ひらがな系の5ルールは品詞で語を見分けるので、置き換えの結果の文字列だけでは指摘の出ない組もあり、その組は設定の不正にしません。`いたします` を `致します` にする辞書では、`致し` を補助動詞とみなすかを前の語で決めるので、`致します` だけでは指摘が出ません。`preferWords` を書かない `@textlint-ja/no-synonyms` は、表記の揺れを指摘するだけなので、置き換えの結果を指摘しても設定の不正にしません。この2つの組では、hookが置き換えた箇所をagentが指摘に従って戻し、次の編集でまた置き換わる往復が起きえます。どちらかのルールのIDのキーを `false` にするか、その置き換えを辞書やパターンから外してください。

### 言語によらない3ルール

3ルールは、textlintで単独に配られている次の3つのpackageから移したものです。IDは、packageの名前から `textlint-rule-` を除いた名前です。

- `no-todo`: `textlint-rule-no-todo` 2.0.1
- `date-weekday-mismatch`: `textlint-rule-date-weekday-mismatch` 1.1.1。日付の書式は、このpackageが使う `chrono-node` 1.4.9から写しています
- `no-dead-link`: `textlint-rule-no-dead-link` 6.2.3

3つはどれも指摘ルールで、`jafix fix` とhookは書き換えません。textlintの `date-weekday-mismatch` は曜日を、`no-dead-link` はリダイレクト先のURLを置き換えて直せます。jafixは直さずに、置き換え後の曜日かURLを文面に入れて指摘します。初期状態で有効なのは `date-weekday-mismatch` です。
3つを束ねるpresetはありません。
`no-todo` と `no-dead-link` を使う場合は、`[builtin]` にルールのIDを書きます。

```toml
[builtin]
no-todo = true
date-weekday-mismatch = { useCurrentYearIfMissing = true }
no-dead-link = { ignore = ["https://example.com/**"], retry = 1 }
```

`no-todo` は、`TODO:` と未完了のタスクを指摘します。`TODO:` は大文字と小文字を区別せず、語の途中(`mastodo:`)にも当たり、本文の範囲の中のすべての一致を指摘します。全角の `ＴＯＤＯ：` とコロンの無い `TODO` には当たりません。見るのは、リンクの文字列と引用の中を除く本文の範囲で、見出し・表のセル・強調・太字の中も見ます。タスクは、リスト項目ごとに、項目の先頭のタスクの記号と項目の直下の段落の文字列から `[ ]` を探し、最初の1つを指摘します。項目の先頭の未完了のタスクの記号は、後ろに何があっても指摘し、本文の無い `- [ ]` も指摘します。段落の文字列の `[ ]` は、角括弧の間が1文字以上の空白で、後ろに空白が続くか、段落の末尾にあるもの(`- a [ ]`)です。引用の中の項目も見ます。`[x]` と `[]` には当たらず、入れ子の項目の `[ ]` はその項目でだけ数えます。文面は `TODOが残っている: 「<TODO: から本文の範囲の末尾まで>」` と `未完了のタスクが残っている` です。

`date-weekday-mismatch` は、日付の直後の括弧の中身を曜日として読み、日付から求めた曜日と食い違えば指摘します。括弧は `(` か `（` で開き、`)` か `）` で閉じるもので、日付との間に空白があってもかまいません。日付と括弧は、同じ本文の範囲の中になければならないので、`**2024年4月23日**(月)` は確かめません。見るのは本文の範囲のすべてで、リンクの文字列・引用・見出し・表のセルの中も見ます。読む日付の書式は次のとおりです。

- 日本語: `2024年4月23日`・`2024年 4月 23日`・`令和6年4月23日`・`令和元年5月1日`・`今年4月23日`・`本年4月23日`・`4月23日`。元号は `令和`・`平成`・`昭和` で、全角の数字も読みます
- 年が先の数字: `2024-04-23`・`2024/04/23`・`2024.04.23`・`2024/Apr/23`と、時刻の続く `2024-04-23T10:00`・`2024-04-23T10:00:00+09:00`
- 月が先の数字: `04/23/2024`・`4.23.2024`・`4/23/24`・`4/23`。月が12を超え、日が12以下なら入れ替えます(`23/04/2024`)。2桁の年は、50より大きければ1900年代、それ以外は2000年代とします。年の無い形は、`/` で区切ったときだけ読みます
- 英語の月名: `April 23, 2024`・`Apr. 23rd 2024`・`23 April 2024`・`23rd of April, 2024`・`twenty-third of April, 2024`・`April 23`。月名の大文字と小文字は区別しません。前に英語の曜日の語を置いた `Tuesday, April 23, 2024` も読みます

曜日は、括弧の中の前後の空白を除いた文字列が、日本語の `日`〜`土` と `日曜日`〜`土曜日`、英語の `Su`〜`Sa`・`Sun`〜`Sat`・`Sunday`〜`Saturday` のどれかと全体が一致したときだけ確かめます。大文字と小文字は区別するので、`mon` は確かめず、`月曜`・`祝` も確かめません。曜日の言語は括弧の中身で決めるので、`2024-04-23(火)` と `2024年4月23日(Tue)` のどちらも確かめます。文面は `「2024-04-23 (」の曜日は「Tue」で、「Mon」ではない` の形で、位置は括弧の中の曜日の先頭です。

オプションは、次のとおりです。

- `lang` は `"ja"` か `"en"` で、書くとその言語の曜日だけを確かめます。書かなければ両方を確かめます
- `useCurrentYearIfMissing` を `true` にすると、年の無い日付(`4月23日`・`4/23`・`April 23`)を `currentYear` の年として確かめます。`false` のときは確かめません
- `currentYear` は1以上の整数で、`今年`・`本年` の日付と、年を補う日付に使う年です。書かなければ、実行した時点のシステムの年(UTC)です

`同年` と元号の無い `元年` の日付は、年が決まらないので確かめません。存在しない日付(`2016年2月30日`)、範囲(`4月23日〜24日`・`April 23 - 25, 2024`)、`BE`・`AD`・`BC` の付く年の日付も確かめません。

`no-dead-link` は、リンクの宛先のファイルかURLがあるかを確かめます。宛先は、インラインのリンク(`[x](u)`)と自動リンク(`<…>`)、裸のURL、参照リンクの定義(`[id]: u`)の宛先です。見出し・表のセル・リスト項目・脚注の中も見ます。引用の中、画像、インラインコード・コードブロック・HTMLの中のURLは確かめません。宛先が `http:`・`https:`・`file:` 以外の方式(`mailto:` など)なら確かめず、メールアドレスの自動リンクも確かめません。裸のURLは、本文の範囲と同じく、対にならない `）` の手前で終えます。`公式（https://example.com）を参照。` の宛先は `https://example.com` です。

相対のリンク(`exists.md`・`./x`・`#anchor`)は、`baseURI` があればそれを、無ければMarkdownのファイルの場所を基準に解決します。解決したパスから `?` と `#` から後ろを除き、そのファイルかディレクトリがあるかを確かめます。ページの中のアンカーがあるかは確かめません。`/` で始まる絶対パスは、ローカルのファイルとして確かめます。文面は `リンク先のファイルが無い: <宛先>` です。

`no-dead-link` は、hookでは外部のURLに接続しません。`http:`・`https:` のURLに接続して確かめるのは `jafix check` だけで、hookと `jafix fix` はローカルのファイルだけを確かめます。hookは編集のたびに速く終わる必要があり、外部のサーバーへの接続はそれを満たせないためです。`jafix check` は `HEAD` で要求し、2xxでなければ `GET` で `retry` の回数まで、間を空けて再試行します。`preferGET` のoriginのURLは、最初から `GET` で要求します。1回の要求は10秒で打ち切ります。リダイレクトを辿った先が2xxなら、リダイレクト先のURLを文面に入れて指摘します。同じ `jafix check` の中の同じURLは1回だけ確かめ、1つのファイルの宛先を最大 `concurrency` 個ずつ同時に確かめます。文面は `リンク先に届かない: <URL> (404 Not Found)` と、`リンク先が <最後のURL> へリダイレクトする (301 Moved Permanently)。宛先を <最後のURL> にする` です。TLSの証明書は、binaryに入れた `webpki-roots` の一覧で検証します。外部のURLへの接続は、環境変数の `ALL_PROXY`・`HTTPS_PROXY`・`HTTP_PROXY`(小文字の名前も読みます)のうち最初に見つかったproxyを通り、`NO_PROXY` に当たるホストへは直に接続します。

オプションは、次のとおりです。

- `checkRelative` を `false` にすると、相対のリンクを確かめません。`/` で始まる絶対パスは確かめます
- `baseURI` は、相対のリンクを解決する基準で、空でない文字列です。`http://`・`https://`・`file://` で始まればURL、それ以外はパスです。パスは設定ファイルのあるディレクトリから解決し、`~/` で始まればホームディレクトリに展開します。末尾の `/` の有無によらず、ディレクトリとして扱います
- `ignore` は、確かめない宛先のglobの配列です。解決する前の宛先の文字列に当てます。`exclude` と同じく、`*` と `?` は `/` をまたがず、`**` はまたぎます
- `ignoreRedirects` を `true` にすると、リダイレクトする宛先を指摘しません
- `preferGET` は、最初から `GET` で要求するoriginのURLの配列です(`["https://example.com"]`)
- `retry` は再試行の回数の上限で、0以上の整数です。`concurrency` は同時に確かめる数で、1以上の整数です
- `userAgent` は要求の `User-Agent` で、空でない文字列です。書かなければ `jafix-no-dead-link/<jafixのversion>` です
- `maxRetryTime` と `maxRetryAfterTime` は、再試行の前に待つ時間の上限の秒数で、0以上の整数です。応答に `Retry-After` があればその秒数を `maxRetryAfterTime` まで待ちます。無ければ、`回数 ** 2 * 100` ミリ秒を `maxRetryTime` まで待ちます。上限を超えれば、待たずに再試行します
- `httpOnly` を `true` にすると、すべての宛先を `baseURI` を基準にURLとして解決します。`baseURI` が無ければ、相対のリンクと `/` で始まる絶対パスを確かめません

### 英語の4ルール

4ルールは、textlintで単独に配られている次の4つのpackageから移したものです。IDは、textlintの設定に書くルールの名前と同じです。

- `terminology`: `textlint-rule-terminology` 5.2.16
- `spelling`: `textlint-rule-spelling` 0.3.0。辞書は `dictionary-en` 4.0.0のen_USのhunspellの辞書です
- `common-misspellings`: `textlint-rule-common-misspellings` 1.0.1。誤綴りの一覧は、Wikipediaの一覧から作りました
- `write-good`: `textlint-rule-write-good` 2.0.0。判定の手順と単語リストは `write-good` 1.0.8と、それが使う6つのpackageから写しています

`jafix fix` とhookが直すのは、`terminology` の `terms` に利用者が書いた項目の箇所だけです。この箇所は、`prh` と同じ利用者の置き換えです(「[利用者の置き換え](#利用者の置き換え)」)。`terminology` の組み込みの辞書の箇所と、ほかの3つのルールは指摘ルールで、置き換え後の語・辞書の候補・正しい語を文面に入れます。textlintは、組み込みの辞書の置き換えと、候補か正しい語が1つの箇所を自動で直します。jafixは、確かめないまま語を書き換えることを避けて、指摘にします。設定を書かなければ4つとも無効で、4つを束ねるpresetの名前もありません。`[builtin]` にルールのIDのキーを書いて有効にします。

```toml
[builtin]
terminology = { terms = ["jafix", ["front[- ]end(\\w*)", "frontend$1"]] }
spelling = { skipPatterns = ["jafix", "npm", "lindera"] }
common-misspellings = true
write-good = { passive = false, whitelist = ["read-only"] }
```

4つが同じ箇所に当たれば、有効なルールがそれぞれ指摘を出します。たとえば `teh` には `spelling` と `common-misspellings` の指摘が、`pacakge` には `terminology` と `spelling` の指摘が出ます。どのルールも、本文でない箇所(裸のURL、自動リンクの文字列、定義を持つ省略形・collapsed形の参照リンクの文字列)を見ません。文字参照(`&nbsp;`・`&#160;`)の名前と数字も、語として見ません。`\` でエスケープした `&` (`\&nbsp;`)は文字参照ではなく、描画でも `&nbsp;` の文字のまま残るので、中の `nbsp` を語として見ます。

`terminology` は、技術語・ブランド・略語の表記を揃えます。`skip` の型の中にない本文の範囲へ、利用者の `terms` の項目と、組み込みの辞書の341項目を、この順に並べて当てます。大文字と小文字は区別しません。項目は、正しい表記を書いた文字列か、正規表現と置き換え後の文字列を並べた2つの文字列の配列です。文字列の項目の置き換え後の語は、小文字にして一致と等しくなる最初の項目です。配列の項目の置き換え後の文字列には、JavaScriptの `replace` と同じく `$1`・`$&` などを書けます。文頭の一致が大文字で始まるなら、置き換え後の語の先頭を大文字にします。置き換え後の語が一致と同じなら、何も出しません。先読みか後読みを持たない項目には、語の前後の条件を付けます。直前が `-`・英数字・`.` でなく、直後が空白・句読点・本文の範囲の末尾か、`’` を除くASCIIでない文字のときだけ当たります。文面は `prh` と同じ `「<一致した文字列>」を「<置き換え後>」にする` です。

- `terms` は、項目の配列か、項目の配列を書いたファイルのパスです。ファイルはJSONか、`//` と `/* */` のコメントを持つJSONで、末尾のカンマは書けません。相対パスは設定ファイルのあるディレクトリから解決し、`~/` で始まればホームディレクトリに展開します。項目の正規表現は、`i`・`m`・`u` を付けたJavaScriptの正規表現として読みます
- `skip` は、対象から外す型の名前の配列です。型の名前は `@textlint-rule/pattern` の `allowNodeTypes` と同じで、書くと既定の `["BlockQuote"]` を置き換えます
- `defaultTerms` を `false` にすると、組み込みの辞書を使いません
- `exclude` は、使わない項目を書いた文字列の配列です。文字列の項目は項目の全体と、配列の項目は1つ目の要素と比べます

`terms` のファイルは、次のように書きます。

```json
[
  // 文字列の項目は、正しい表記を書く
  "jafix",
  // 配列の項目は、正規表現と置き換え後の文字列を書く
  ["front[- ]end(\\w*)", "frontend$1"]
]
```

日本語の文書では、`terminology` の語の直後の条件で結果が変わります。jafixは、textlintの条件に、`’` を除くASCIIでない文字を足しました。`space-between-ja-and-alnum` が日本語と英数字の間の空白を消すので、直した後の文では語の直後がたいてい日本語の文字になるためです。そのため、`javascriptを使う` の `javascript` にも当たります。

`spelling` は、`dictionary-en` 4.0.0に含まれる米国英語(en_US)のHunspell辞書で英単語の綴りを確かめます。
辞書の `index.aff` と `index.dic` は改変せずバイナリに埋め込みます。
辞書を引く処理には `spellbook` 0.4.2を使います。
`spelling` が有効で、確かめる語があるときだけ辞書を1回読み込みます。

`wordDefinitionRegexp` で本文の範囲とインライン画像のalt(入れ子のリンクと画像の宛先とtitleを除く)から語を取り出し、辞書にない語を指摘します。
本文には見出し、引用、表のセル、リンクの文字列、強調や太字の中も含まれます。
指摘の範囲は語全体です。

`suggestCorrections` が `true` なら、`英語の辞書に無い語「recieve」を使っている。候補: receive、relieve、reverie` のように、文面に候補を入れます。
候補がなければ `候補は無い` と続けます。
同じ入力と設定なら、候補は毎回同じ順序です。
語の判定と候補は `spellbook` に従います。
そのため、textlintと異なる結果になることがあります([textlintとの違い](#textlintとの違い))。

`spelling` は、識別子として書かれた語を除外します。
ASCIIの数字を含む語(`50ms`)、`_` を含む語と `\_` に接する語、2文字目以降に大文字を含む語(`GitHub`)が対象です。
ファイル名とドメイン名は、ASCIIの英数字と `_`・`-`・`.`・`/` の並びのうち、英数字の続く `.` を含むものとして除外します。
文末の `.` (`recieve.`)と日本語の文字の前の `.` (`teh.の後`)、`...` の後ろの語は、この条件に当たりません。

`/` でつながった並びは、パスや名前と普通の語の列挙を次の条件で分けます。

- 文書内のGitHubリンク先にある `owner/repo` と一致する並びは、リポジトリ名として除外します
- `/` か `.` で始まるもの、`/` で終わるもの、先頭が `owner`・`src`・`lib`・`bin`・`etc`・`usr`・`var`・`tmp`・`home`・`docs`・`topics` のものは、パスか名前として除外します
- 数字、`_`、2文字目以降の大文字を含む要素があれば、並びを除外します。`fs` で終わるファイルシステム名や `stdin`・`stdout`・`stderr` を含む並びも除外します。辞書に無い要素がハイフンを含む場合、大文字で始まる場合、3文字以下で既知の誤綴りでもない場合も、名前や略語として除外します
- それ以外は、先頭の要素が辞書の語か `common-misspellings` の一覧の誤綴りなら、普通の語の列挙として調べます。3要素以上の並びは、どれかの要素が辞書にあれば調べます。`Demo/visualizaion`・`recieve/send`・`bicubic/lighweight/real-world` の誤綴りも対象です
- 先頭が辞書にも誤綴りの一覧にも無い2要素の並びは、名前を優先して除外します。`obra/superpowers` と `uid/gid` が該当します。形だけでは名前か誤綴りかを確定できない場合の扱いです

この除外は止められません。
プロジェクトで使う技術用語や固有名詞が指摘されたら、`skipPatterns` に書いて除外できます。
次の例は、`spelling` を有効にし、列挙した語を大文字と小文字の区別なく除外します。

```toml
[builtin.spelling]
skipPatterns = [
  '/\b(?:strikethrough|profiler|bicubic|grayscale|keypoints|monospaced|Linux)\b/i',
]
```

- `language` は `"en"` だけを書けます
- `skipPatterns` の要素は、「[パターン](#パターン)」の書き方で書きます。本文の範囲かaltに当てた一致と重なる語は、端が接するだけでも指摘しません
- `wordDefinitionRegexp` は、`/source/flags` の形の文字列です。書いたフラグ(`i`・`m`・`s`)のまま当て、`g` の有無は結果を変えません。空の一致は語にしません
- `suggestCorrections` を `false` にすると、候補を求めず、文面に入れません

よくある誤綴りに絞るには、`[builtin]` で `common-misspellings = true` を指定します。
`spelling` を有効にしている場合は、その設定を `spelling = false` に置き換えます。

`common-misspellings` は、引用の外の本文の範囲に、誤綴りの一覧の4,310件を、語の境界(ASCIIの `\b`)の間で大文字と小文字を区別せずに当てます。語の境界では、ASCIIの英数字と `_` に加えて、大文字と小文字を区別せずに当てると `s` と `k` に一致する `ſ` (U+017F)と `K` (U+212A)も語の文字とします。同じ位置では最も長い一致を取ります。`ignore` に書いた語は、大文字と小文字を区別せずに比べ、指摘しません。文面は `よくある綴りの誤り「Teh」を使っている。正しくは「The」` で、正しい語には一致の大文字を写します。正しい語が複数ある項目は、`正しくは「a lot」か「allot」` のように、すべてを入れます。一覧は、Wikipediaのページ「Wikipedia:Lists of common misspellings/For machines」のrevision 1199637275 (`oldid=1199637275`)から作り、CC BY-SA 3.0の条件でbinaryに埋め込んでいます。

`write-good` は、段落の直下にあり引用の中でない本文の範囲に、次の9つの判定を当てます。リスト項目の中の段落も見て、見出し・表のセル・リンクの文字列・強調・太字の中は見ません。`eprime` 以外の8つは、設定に書かなくても当てます。判定の名前のキーを `false` にすると、その判定を止めます。

- `weasel`: 曖昧な語(`very`・`clearly` など)
- `illusion`: 直前の語の繰り返し(`the the`)
- `so`: 文頭の `so`
- `thereIs`: 文頭の `there is`・`there are`
- `passive`: 受動態の形の句(`was done`)
- `adverb`: 意味を弱める語(`really`・`just` など)
- `tooWordy`: 冗長か不要な句
- `cliches`: 決まり文句
- `eprime`: be動詞の形

`so` と `thereIs` の文は、改行と `.`・`;`・`!`・`?` で区切ります。文字参照(`&amp;`)の終わりの `;` では区切りません。同じ範囲に複数の判定が当たれば1件にまとめ、`「clearly」は曖昧な語、意味を弱める語である(weasel・adverb)` のように説明をつなぎます。`whitelist` は空でない文字列の配列で、その文字列の出現と重なる指摘を出しません。大文字と小文字は区別します。

## 利用者の置き換え

設定に書いた辞書やパターンを使って本文を直す処理を「利用者の置き換え」と呼びます。次の設定が対象です。

- `prh` の辞書
- `@textlint-rule/pattern` の `replace` を指定した要素
- `terminology` の `terms` に追加した項目

`jafix fix` とhookは該当箇所を書き換え、`jafix check` は位置を表示します。`replace` を省いたパターンと、`terminology` の組み込み辞書に該当する箇所は指摘として扱います。

`prh` の辞書や `@textlint-rule/pattern` のパターンは、textlintの書式で設定できます。

### `prh`

辞書は `rulePaths` でファイルを参照するか、`ruleContents` に直接書きます。`rulePaths` は空でないパス文字列の配列、`ruleContents` は文字列の配列です。どちらかで辞書を1つ以上指定する必要があるため、`prh = true` は設定エラーになります。相対パスの基準は設定ファイルがあるディレクトリです。`~/` はホームディレクトリに展開します。

`checkLink`、`checkBlockQuote`、`checkEmphasis` を `true` にすると、それぞれリンクの文字列、引用、強調(`*x*`)の中も確認します。この3つの既定値は `false` です。裸のURLと自動リンクの文字列は対象外です。

`checkHeader` と `checkParagraph` の既定値は `true` です。`false` にすると、それぞれ見出しまたは段落が対象から外れます。textlint-rule-prhの `checkCodeComment` と `debug` には対応していません。

```toml
[builtin.prh]
rulePaths = ["dict/prh.yml", "~/dict/shared.yml"]
ruleContents = ["rules:\n  - expected: JavaScript\n"]
checkBlockQuote = true
```

辞書はprh 6.1.0のYAMLです。次の例のように、`expected` に置き換え後の文字列を、`pattern` に置き換え前の正規表現を書きます。

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

- `expected`: 置き換え後の文字列(必須)。`$1` から `$99` で、`pattern` の捕捉グループを参照します。`$` の後の2桁までの数字を1つの番号として読み(`$01` は1番目のグループ)、`$0` と捕捉グループの数を超える番号は書いたまま残します
- `pattern`: 置き換え前の正規表現か文字列そのもの。文字列の配列も書けます
- `patterns`: `pattern` と同じ。`pattern` が無いときだけ使います
- `regexpMustEmpty`: `$1` の形の文字列。その番号のグループが空でない一致は、直しません
- `options`: `wordBoundary`(真偽値)だけを読みます。`true` なら、正規表現の全体を `\b` で挟みます
- `specs`: `from` と `to` の組の配列。読み込みのときに、そのルールだけを `from` に当てた結果が `to` と等しいかを確かめます
- `prh`: 指摘の文面に添える補足

`version`・`meta`・`targets` と、知らないキーは使いません。

辞書は、`rulePaths` を書いた順に、次に `ruleContents` を書いた順に読み、1つにまとめます。各辞書は自分のルールを先に持ち、`imports` の辞書を書いた順に取り込みます。まとめるときは、`expected` が同じルールが2つの辞書(`imports` で取り込む辞書を含む)にあれば、先に読んだ辞書のルールだけを残すので、`rulePaths` の辞書のルールが `ruleContents` の同じ `expected` のルールに勝ちます。1つの辞書の中で `expected` が同じルールは、prh 6.1.0と同じく、どれも残します。`imports` の相対パスは、取り込む側の辞書のファイルのあるディレクトリから、`ruleContents` の中では設定ファイルのあるディレクトリから解決します。要素の `disableImports` が `true` なら、取り込む辞書の `imports` を読みません。`ignoreRules` は、取り込む辞書から、`expected` が等しいルールか、`pattern` の文字列が等しいルールを除きます。

一致した文字列と置き換え後の文字列が同じ箇所と、長さ0の一致は直しません。指摘と修正箇所の文面は `「<一致した文字列>」を「<置き換え後>」にする` です。辞書に `prh` の補足があれば `: <補足>` を続けます。

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

`prh` の `pattern`・`patterns` と、`@textlint-rule/pattern` の `pattern`・`allows`、組み込みのルールのオプションのパターン(「[パターン](#パターン)」)は、どれもJavaScriptの正規表現(`u` フラグ付き)の構文と意味で読みます。後読み・先読み・後方参照・名前付きグループ・`\p{Script=…}` を使え、`\b`・`\d`・`\w` はASCIIで判定します。

`/source/flags` の形の見分け方は、`prh` と `@textlint-rule/pattern` ではtextlintのそれぞれのルールと同じで、組み込みのルールのオプションのパターンは `@textlint-rule/pattern` と同じです。`prh` は `^/(.*)/([gimsuy]*)$` に当たれば正規表現にし、当たらなければ書いた文字列そのものに当てます。`g`・`m`・`u` は常に付き、`i` と `s` は書いたときだけ付き、`y` は無視します。`@textlint-rule/pattern` は `^/(.+)/([guimysd]*)$` に当たれば正規表現にし、`g`・`u` は常に付き、`i`・`m`・`s` は書いたときだけ付き、`d` は無視します。`y` は不正です。

### 修正の順と直さない箇所

`jafix fix` とhookは、まず利用者の置き換えを1回だけ適用します。
その結果を解析し直し、[組み込みの修正ルール](#修正ルール)を繰り返し適用します。
置き換えで生じた全角括弧や日本語と英数字の間の空白も同じ実行で直ります。

利用者の置き換えが重なった場合は、先に始まる箇所を選びます。
同じ位置で始まる場合は、長い箇所を選びます。
範囲も同じなら、`prh`、`@textlint-rule/pattern`、`terminology` の順で優先します。

同じルールの中では、`patterns` の前にある要素を優先します。
`terminology` は文字列の項目を優先し、配列の項目どうしでは `terms` の前にある項目を選びます。
選んだ箇所と重なる箇所は直しません。
`jafix check` には選んだ箇所だけを出します。

一致の周りの文字列がすでに置き換え後の文字列になっている箇所は、直さず、指摘もしません。置き換え後の文字列(`$1` などを展開した後)の中で一致した文字列が現れる位置に合わせて、元のソースの同じ長さの範囲が置き換え後の文字列と等しければ、すでに直っているとみなします。たとえば `サーバ` を `サーバー` にする辞書は、`サーバを` を `サーバーを` に直し、`サーバー` は変えません。`Node` を `Node.js` にする辞書も、`Node.js` を変えません。そのため、`jafix fix` を何回かけても、2回目からは文書が変わりません。

次の箇所は、直さずに、そのルールのIDで指摘にします。

- 見出しの中の箇所。見出しのアンカーが変わり、見出しへのリンクが切れるためです。文面の後に「見出しのアンカーが変わるので、見出しへのリンクも直す」を続けます。アンカーが変わらない置き換えを見分けることはしません
- 一致した文字列か置き換え後の文字列が、改行を含む箇所。hookは、直した後の行の番号が変わらないことを前提にするためです。文面の後に「改行を含むので直さない」を続けます

置き換えの固定の部分(`prh` の `expected` と `replace` から `$1` などを除いた部分と、`terminology` の `terms` の文字列の項目の全体、配列の項目の2つ目の要素から `$1`・`$&` などを除いた部分)を有効な組み込みの修正ルールが直すときは、設定ファイルが不正です。たとえば `expected: （$1）` は、`fullwidth-paren` が `（` を `(` に戻すので、置き換えを当てても次の段で戻されます。理由に置き換えと組み込みの修正ルールのIDを書くので、どちらかを無効にするか、置き換えを辞書から外してください。`imports` の `ignoreRules` で外したルールと、無効にした組み込みのルールは、この判定に入りません。空白と字種の13ルールと、JTFスタイルガイドの37ルールのうち有効な修正ルールは、この判定に入ります。`no-mixed-zenkaku-and-hankaku-alphabet` を有効にした設定では、`expected` に全角英字を含む辞書が不正になります。

置き換えの固定の部分を、辞書で語を別の書き方にするよう求める有効な組み込みの指摘ルールが指摘するときも、設定ファイルが不正です。たとえば `サーバー` を `サーバ` にする `prh` の辞書は、`2.1.6.カタカナの長音` を有効にした設定で不正になります。hookが `サーバ` に直すと、agentが `2.1.6` の指摘に従って `サーバー` に戻し、次の編集でhookがまた `サーバ` に直す、という往復が止まらないためです。判定に入るルールと、判定できない組の扱いは、「[不正な設定](#不正な設定)」にあります。

捕捉参照に隣接する端を、文書の先頭や末尾とはみなしません。たとえば `expected: サーバ$1` と `pattern: /サーバー(ラック)/` は、`2.1.6` が置き換え後の `サーバラック` を指摘しないので読み込めます。辞書の指摘と空白の修正が参照に接する場合は、参照の内容に依存するため、設定の不正にしません。固定の部分の内側で決まる衝突と、`（$1）` の全角括弧のように参照の内容によらない字種の修正は、不正にします。参照の内容次第で実行時に往復する場合は、どちらかのルールを無効にしてください。

## 修正ルール

修正ルールの対象はMarkdownの本文です。`jafix fix` とhookは当てはまる箇所を書き換え、`jafix check` は位置を表示します。

`space-between-ja-and-alnum` と `space-between-ja-and-paren` の日本語の文字は、漢字・ひらがな・カタカナと長音符 `ー` です。句読点(`。` `、`)は日本語の文字に含めないので、`です。 Rust` の空白は残ります。空白は半角空白と全角空白で、続けて並んだものはまとめて削除します。textlintから移したルールの日本語の文字は、ルールごとにtextlintと同じ範囲です(「[空白と字種の13ルール](#空白と字種の13ルール)」「[JTFスタイルガイドの37ルール](#jtfスタイルガイドの37ルール)」)。

`no-nfd` は、結合文字の濁点・半濁点や単独の `゛` `゜` を前の文字と合成して、1文字にします。合成して1文字にならない組み合わせ(`あ` と濁点など)は、直さずに指摘します。`no-invalid-control-character` が削除するのは、タブ・改行・復帰を除く制御文字と、U+202AからU+202Eの方向制御文字です。`no-nfd` は引用・強調・リンクの中の文字列を見ません。`no-hankaku-kana` は、段落の直下の文字列だけを直します。表のセル・強調・太字・リンク・引用・見出しの中の半角カナは、直さず指摘もしません。

本文は、段落・リスト・見出し・表のセル・強調・取り消し線・リンクの文字列・引用・脚注の本文と、インラインのHTMLのタグに挟まれた文字列です。コードブロック、インラインコード、画像、URL、front matter、HTMLのタグとHTMLブロックは直しません。`space-between-ja-and-alnum`・`space-between-ja-and-paren` と、本文の範囲に当てる13ルールの修正は、インラインコード・強調・リンク・HTMLのタグとの境目の空白(`` `code` の `` や `**強調** の`)を残します。インラインコード・リンク・太字・強調との境目の空白を削除するのは、`ja-space-around-code`・`ja-space-around-link`・`ja-space-around-strong`・`ja-space-around-emphasis` を有効にしたときだけです。HTMLのタグとの境目の空白は、どのルールも削除しません。

参照形式のリンクのうち、定義を持つ省略形(`[設定 API]`)とcollapsed形(`[設定 API][]`)の文字列は、直さず指摘もしません。この文字列は定義のラベルを兼ねるので、書き換えると定義との対応が切れるためです。full形(`[設定 API][api]`)の文字列は、他の本文と同じく直します。

見出しの本文では、アンカーが変わる修正ルール(空白を削除する2つ、`no-nfd`、`no-zero-width-spaces`、`no-invalid-control-character`、空白と字種の13ルールの修正)に当たる箇所を直さず、指摘にします(「[指摘ルール](#指摘ルール)」)。見出しの文字を変えると自動で付くアンカーが変わり、見出しへのリンクが切れるためです。たとえば `## API の設計` のアンカーは `api-の設計` で、空白を消すと `apiの設計` になります。全角括弧は置き換えてもアンカーが変わらないので、見出しでも直します。JTFスタイルガイドの37ルールの修正ルールは、段落の直下の文字列だけに当たるので、見出しの中の箇所はありません。

組み込みの修正ルールの箇所どうしが重なるときは、1件だけを残して直し、`jafix check` もその1件だけを出します。先に始まる箇所を残し、同じ位置で始まるなら長い箇所を、範囲も同じなら「[組み込みのルール](#組み込みのルール)」の表で前にあるルールの箇所を残します。たとえば `preset-ja-spacing = true` では、`日本語 Rust と語 （補足） です` の `Rust` の前後の空白が `space-between-ja-and-alnum` の箇所、`（` の前と `）` の後の空白が `space-between-ja-and-paren` の箇所になり、`ja-space-between-half-and-full-width` と `ja-no-space-around-parentheses` の箇所は出ません。各ルールの判定はそれぞれのままなので、jafix独自のルールが当たらない空白には、移したルールの箇所が出ます。`space = ["punctuation"]` の `ja-space-between-half-and-full-width` が当たる `です。 Rust` の空白がその例です。見出しの中で指摘にする箇所も、重なりを解いてから指摘にします。JTFスタイルガイドの37ルールは表の最後にあるので、ほかの修正ルールと同じ範囲に当たれば、ほかのルールの箇所が残ります(「[JTFスタイルガイドの37ルール](#jtfスタイルガイドの37ルール)」)。

`jafix fix` とhookは、組み込みの修正ルールを、対象行の内容が変わらなくなるまで繰り返し当てます。ある修正ルールの結果に別の修正ルールが当たる箇所も、1回の実行で直ります。たとえば `no-mixed-zenkaku-and-hankaku-alphabet` が `日本語 Ｒｕｓｔ` を `日本語 Rust` にすると、その空白を `space-between-ja-and-alnum` が削除して、`日本語Rust` になります。設定を書かない場合も、この繰り返しで修正の結果の変わる組があります。`日本語 Rust` の空白の後にゼロ幅スペースがある箇所は、ゼロ幅スペースを削除する `no-zero-width-spaces` の後に、空白を削除する `space-between-ja-and-alnum` も当たるので、1回で `日本語Rust` になります。`か` と結合文字の濁点の後に空白と英字が続く箇所も、`no-nfd` が濁点を合成した後の空白まで1回で削除します。JTFスタイルガイドの37ルールの修正ルールも、この繰り返しに入ります。`"2.1.9.アルファベット" = true` の設定では、`2.1.9` が `日本語 ＡＢＣ です` の英字を半角にした後の空白まで、1回で削除されて `日本語ABCです` になります。`jafix check` が出すのは、元の内容に当たる箇所だけで、繰り返しの2回目以降に当たる箇所は出しません。

## 指摘ルール

指摘ルールは、ファイルを直さずに指摘だけを出すルールです。組み込みの指摘ルール(`replace` の無い `@textlint-rule/pattern` の要素を含む)と、直さない修正ルールの箇所の2種類があります。直さない箇所は、見出しの中の修正ルールの箇所と、見出しの中や改行を含む利用者の置き換えの箇所です(「[修正の順と直さない箇所](#修正の順と直さない箇所)」)。`@textlint-rule/pattern` は、`allowNodeTypes` に書いた型の中を除き、引用・強調・見出しの中の本文にも当たります。見出しの中の指摘は、修正ルールのIDで出し、見出しへのリンクも直すことを文面で伝えます。

組み込みの指摘ルールは、ルールごとに当てる単位が違います。

- 本文の範囲ごとに当てるルール: `max-kanji-continuous-len`・`arabic-kanji-numbers`・`no-exclamation-question-mark`・`ja-unnatural-alphabet`・`no-double-negative-ja`・`no-dropping-the-ra`・`ja-no-weak-phrase`・`ja-no-successive-word`・`ja-no-abusage`・`no-ai-hype-expressions`・`no-ai-words`・`no-short-topic-comma`・`ja-nakaguro-or-halfwidth-space-between-katakana`・`ja-space-after-exclamation`・`ja-space-after-question`・空白を求める側の `ja-space-between-half-and-full-width`・ひらがな系の5ルール・`@textlint-ja/no-insert-dropping-sa`・`no-todo` の `TODO:`・`date-weekday-mismatch`・`@textlint-rule/pattern` です。形態素解析を使うルールは、本文の範囲を1つずつトークンへ分けます。本文の範囲は、インラインコード・強調・リンク・HTMLのタグで区切られるので、境目をまたいでは一致せず、`照**合**` は `照合` に当たりません。長さ0の一致は指摘にしません。組み込みのルールは、引用・強調・リンクの中、段落の直下でない範囲を、ルールごとにtextlintと同じ文脈で対象から外します。`@textlint-rule/pattern` は、`allowNodeTypes` に書いた型の中だけを対象から外します
- インラインコード・リンク・太字・強調の前後に当てるルール: 4つの `ja-space-around-*` の `true` の側です。nodeの直前か直後の文字が日本語の文字で、間に空白が無ければ指摘します。nodeの前か後ろに文字が無い側(文書の先頭か末尾)は見ません
- 段落と文の単位で当てるルール: `sentence-length`・`max-comma`・`ja-no-mixed-period`・`no-unmatched-pair`・`max-ten`・`no-doubled-joshi`・`no-doubled-conjunctive-particle-ga`・`no-doubled-conjunction`・`ja-no-redundant-expression` です。Markdownの解析結果のままの段落を見ます。リスト項目・引用・脚注の定義の中の段落も見ますが、`ja-no-mixed-period` はこの3つの中の段落を対象にせず、`sentence-length` は引用の中の段落と、子がリンク1つだけの段落を対象にしません。段落の文字列には、本文のほかにインラインコードの中身と画像のaltも入ります。文は、`。` `.` `?` `!` などで区切り、括弧と引用符が開いている間と、`Mr.` のような英語の略語の `.` では区切りません。ハード改行(行末の2つの空白か `\`)では、次の行が列挙の記号で始まる場合だけ、括弧が開いていても文を区切ります。記号は `– `・`— `・`• `・`・` とMarkdownの箇条書きの記号、数字に `.`・`．`・`)`・`、` を続けた番号、括弧で囲んだ数字か英字です。小数点と数字の続く番号は除きます。`api -> micro` のように、英数字の名前と空白を挟んだ矢印(`->`・`=>`・`→`)、空白の無い値だけからなる対応表の行でも区切ります。`**項目名**:` にインラインコードの値だけを続けたラベル行の末尾でも区切ります。コロンで終わる紹介文の次がURLだけの行である場合も、そこで区切ります。記号やラベルの無い行は、文の途中とみなします。独立した項目にも読める場合は、長い文の見逃しを避けるため、同じ扱いとします。ただし、`no-unmatched-pair` はハード改行で文を区切りません。ハード改行は書き手が行を分けただけなので、括弧と引用符はその行をまたいで閉じてよいからです。`sentence-length` は、継続行の行頭の空白とタブを長さに数えません。形態素解析を使うルールは、引用の中の段落を対象にしません(`ja-no-redundant-expression` は `allowNodeTypes` で決まります)。また、インラインコードを中身と同じ文字数の `ー` に置き換えてから文をトークンに分けるので、コードの中身を判定に入れません
- リスト項目・見出し・段落の元の文字列に当てるルール: `no-ai-list-formatting` はリスト項目ごとに、項目の先頭の太字の型と、項目の範囲にある段落(入れ子の項目の段落を含む)の最初の絵文字を見ます。`no-ai-emphasis-patterns` は段落と見出しごとに見ます。どちらも、インラインコードとHTMLのタグの中で始まるか終わる一致を捨てるので、コードの中の `**` や絵文字には当たりません。リスト項目の中のコードブロックも見ません
- リスト項目の直下の段落に当てるルール: `no-todo` のタスクです。項目の先頭のタスクの記号と、項目の直下の段落の文字列を見ます。インラインコード・リンク・強調の中と、入れ子の項目は見ません
- リンクの宛先に当てるルール: `no-dead-link` です。インラインのリンク・自動リンク・裸のURL・参照リンクの定義の宛先を、見出し・表のセル・リスト項目・脚注の中を含めて見ます
- 段落の全体の文字列に当てるルール: `ai-tech-writing-guideline` です。リスト項目・引用・脚注の定義の中の段落も見ます。段落の文字列は、段落の単位のルールと同じ作り方で、インラインコードだけを中身と同じ文字数の半角空白に置き換えます。コードを挟んだ語はつながりません
- 文書の直下の段落と、その次のブロックに当てるルール: `no-ai-colon-continuation` と、`ai-tech-writing-guideline` の構造化の一部です。リスト項目・引用・脚注の定義の中の段落は見ません。`no-ai-colon-continuation` は、次のブロックがコードブロック・リスト・引用・表の段落を、`ai-tech-writing-guideline` は、次のブロックがリストの段落を見ます
- JTFスタイルガイドの37ルールの指摘ルール: 段落の直下の文字列、見出し、リスト、段落、文書の全体のどれかに当てます。辞書の4ルールは、`prh` と同じく対象行と重なる本文の範囲だけに当てます。ルールごとの単位は「[JTFスタイルガイドの37ルール](#jtfスタイルガイドの37ルール)」にあります
- 見出しに当てるルール: `1.1.2.見出し` です。引用の外の見出しの末尾を見ます
- リストごとに当てるルール: `1.1.3.箇条書き` です。リストの直下の項目を数えてから、指摘する項目を決めます。hookでは、リストの全体を数えたうえで、対象行にある指摘だけを返します
- 段落ごとに記号の対を数えるルール: `4.3.3`〜`4.3.7` です。段落の中の本文の記号を数えてから、対にならない記号を決めます。段落と文の単位のルールと違い、インラインコードの中身と画像のaltを数えません
- 文書の全体で当てるルール: `no-mix-dearu-desumasu`、`ai-tech-writing-guideline` の件数の要約、`1.1.1.本文`、`1.1.5.図表のキャプション`、`@textlint-ja/no-synonyms` です。`@textlint-ja/no-synonyms` は、文書の全体で同義語の見出しを数えてから、指摘する箇所を決めます。`no-mix-dearu-desumasu` は、引用の外の見出し・箇条書き(リスト項目の中の段落)・本文(それ以外の段落)のそれぞれで、「である」と「ですます」を文書の全体で数えてから、指摘する箇所を決めます。hookでは、全体を数えたうえで、対象行にある指摘だけを返します

## 導入

### curlでインストール

macOS (Apple Silicon)とLinux (x86_64、ARM64)では、公開リリースのインストーラーを使えます。

```sh
curl -fsSL https://github.com/mjun0812/jafix/releases/latest/download/install.sh | sh
```

インストーラーは配布ファイルのSHA-256が `checksums.txt` と一致することを確認してから、`jafix` を `~/.local/bin` に置きます。`JAFIX_INSTALL_DIR` を設定すると配置先を変更できます。配置先が `PATH` に含まれていなければ追加してください。

### 配布バイナリを直接ダウンロード

[GitHub Releases](https://github.com/mjun0812/jafix/releases)から環境に合うファイルをダウンロードして展開します。配布対象はmacOS (Apple Silicon)、Linux (x86_64とARM64)、Windows (x64)です。展開した `jafix` または `jafix.exe` を `PATH` の通ったディレクトリに置きます。

環境ごとのファイル名は[クイックスタート](docs/quick-start.md)にあります。

### miseでインストール

[mise](https://mise.jdx.dev/)は環境に合う配布バイナリをGitHub Releasesから取得します。

```sh
mise use -g github:mjun0812/jafix
```

### ソースからビルド

リポジトリを取得したディレクトリで次のコマンドを実行します。

```sh
cargo install --path .
```

形態素解析用のIPADIC辞書を含むため、バイナリの大きさは約50MBです。初回のビルドでは `https://Lindera.dev/` から辞書を取得するため、ネットワーク接続が必要です。

## 使い方

`jafix check` はファイルを書き換えず、修正ルールに当たる箇所と指摘ルールの指摘を、ファイル内の位置の順に1件1行で出力します。
書式は `パス:行:列: [ルールID] 文面` です。列は文字単位で数え、先頭の列は1です。
`@textlint-rule/pattern` の行では、パターンに一致した文字列を `「」` で囲み、要素の `message` を続けます。
ほかのルールは1行の文面を出します。

```console
$ jafix check t.md
t.md:1:1: [@textlint-rule/pattern] 「照合」: AIが多用する語
t.md:1:3: [space-between-ja-and-alnum] 日本語と英数字の間の空白を削除する
t.md:2:1: [sentence-length] 1文が100文字を超えている (101文字)
```

`jafix fix` はファイルの全行を修正ルールの判定対象にし、修正した内容を書き戻します。指摘は `jafix check` で確認できます。

```sh
jafix fix t.md
```

どちらのコマンドにもテキストファイルを複数指定できます。jafixは拡張子を問わずMarkdownの構文に従って本文を判定します。

文字コードはファイルごとに判定します。UTF-8として読めなければ、Shift_JIS (CP932を含む)かどうかを推定します。Shift_JISの推定が成立し、再変換で元のバイト列を再現できる場合に読み込みます。文字コードが分かっている場合は、`--encoding utf-8` または `--encoding shift-jis` で指定できます。指定は引数の全ファイルに適用します。

`fix` は元の文字コードで保存します。Shift_JISでは変換結果を読み直し、修正後の文字列と一致するか確認します。一致しなければ、元のファイルを保ったまま終了コード2を返します。

NUL (U+0000)を含むファイルは、自動判定の対象外です。この文字を含むUTF-8ファイルを校正するときは、`--encoding utf-8` を指定します。

形態素解析を使う日本語向けルールは、段落や見出しの本文にある文字をもとに適用します。日本語の可能性がある箇所には、利用者が有効にしたルールを適用します。漢字だけの短文や日英の混在も対象です。英語向けルールと、`no-ai-words` に追加した利用者の辞書は、設定に従って適用します。

| 終了コード | `jafix check`                             | `jafix fix`                                    | `jafix rules`                        |
| ---------- | ----------------------------------------- | ---------------------------------------------- | ------------------------------------ |
| 0          | 修正ルールに当たる箇所も指摘も無い        | 成功した                                       | 一覧を出力した                       |
| 1          | 修正ルールに当たる箇所か指摘が1件以上ある | -                                              | -                                    |
| 2          | ファイルか標準入力を読めない、設定ファイルが不正、Gitの実行や差分の読み取りに失敗した | ファイルを読み書きできない、標準入力を読めない、設定ファイルが不正、Gitの実行や差分の読み取りに失敗した | パスが存在しない、設定ファイルが不正 |

どのコマンドも、引数が誤っているときは終了コード2を返します。

### JSON出力

`jafix check --output-format json` は結果をJSONオブジェクト1件として標準出力に書きます。たとえば `notes.md` の1行目に `日本語 Rustです。` と書いた場合は次の結果になります。

```json
{
  "schema_version": 1,
  "checked_files": 1,
  "findings": [
    {
      "path": "notes.md",
      "line": 1,
      "column": 4,
      "rule_id": "space-between-ja-and-alnum",
      "message": "日本語と英数字の間の空白を削除する",
      "fixable": true
    }
  ],
  "errors": []
}
```

`checked_files` は読み込んでチェックしたファイル数です。対象ファイルの読み込みに失敗しても、残りのファイルの結果を返します。エラーは `errors` に `path` と `message` で入ります。ファイルに紐づかないエラーは `path` が `null` です。実行に失敗した場合の終了コードは2です。

ファイル、`--diff`、`--staged` のいずれも指定しなかった場合は終了コード2を返します。`--diff` または `--staged` を指定して正常に処理でき、対象が見つからなかった場合は終了コード0で、`checked_files` が0になります。

### 修正結果の表示

ファイルを渡した `jafix fix` は、成功時に何も表示しません。保存した変更を確認するときは、次のオプションを使います。

| オプション    | 出力先     | 内容                                                   |
| ------------- | ---------- | ------------------------------------------------------ |
| `--list`      | 標準出力   | 修正したファイルのパスを1行ずつ表示                     |
| `--summary`   | 標準エラー | 修正したファイル数と変更のなかったファイル数             |
| `--timing`    | 標準エラー | 読み込みから修正、保存までにかかった時間をファイルごとに秒単位で表示 |
| `--show-diff` | 標準出力   | 保存した変更のunified diffをUTF-8で表示                 |

`--list` と `--show-diff` はどちらか一方を指定します。`--summary` と `--timing` はほかのオプションと併用できます。
たとえば `jafix fix --diff --show-diff` は、Gitの差分にある行を修正して保存し、その修正の差分を表示します。
ファイルを保存せずに確認するには `jafix fix --diff --dry-run --show-diff` を実行します。`--dry-run` は `--list`、`--summary`、`--timing` とも使えます。
標準入力を校正するときの出力は、[標準入力](#標準入力)で説明します。

### Gitの差分

`check --diff` と `fix --diff` は、`HEAD` と作業ツリーを比較します。ステージ済みと未ステージの変更の両方が対象です。`fix --diff` の修正先は作業ツリーです。

`check --staged` は、`HEAD` とインデックスを比較します。校正対象の本文はインデックスから読み、設定ファイルとローカルのリンク先は作業ツリーで探します。最初のコミットより前に実行すると、ステージ済みのファイル全体を対象にします。

各コマンドは、ファイルを指定すると対象を絞ります。未追跡のファイルと、削除した行は対象外です。対象ファイル全体を解析し、Gitの差分で追加または変更した行から始まる箇所を修正または報告します。Gitの実行や差分の読み取りに失敗した場合は終了コード2を返します。

### 標準入力

ファイルの代わりに `-` を1つだけ指定すると、標準入力の全体を1つの文書として校正します。ファイルに書く前のテキストを、パイプで渡して確かめられます。

```console
$ echo "本文（補足）です。" | jafix check -
<stdin>:1:3: [fullwidth-paren] 全角括弧を半角にする
<stdin>:1:6: [fullwidth-paren] 全角括弧を半角にする
$ echo "本文（補足）です。" | jafix fix -
本文(補足)です。
```

`jafix check -` は、指摘のパスを `<stdin>` として出力します。`--output-format json` では、`findings` の `path` が `<stdin>` になり、`checked_files` は1です。終了コードは、ファイルを渡したときと同じです。

`jafix fix -` は、修正後の本文を標準出力に書き、終了コード0を返します。直す箇所が無ければ、入力をそのまま書きます。ファイルは作らず、書き換えません。文字コードの判定と `--encoding` の指定は、ファイルを渡したときと同じに働きます。
標準出力をパイプやファイルにつないだときは、入力と同じ文字コードと改行で書きます。標準出力が端末のときは、文字コードによらず文字として表示するので、Shift_JISの入力の結果も端末で読めます。
`--dry-run` を付けると、標準出力に何も書きません。`--dry-run --show-diff` を付けると、本文の代わりに、ファイル名を `<stdin>` とした修正前後の差分を書きます。`--summary` と `--timing` は、ファイルを渡したときと同じく標準エラーに書きます。本文の書き込みに失敗して終了コード2を返すときは、`--summary` の件数を表示しません。

設定には、共通設定と、カレントディレクトリから探したプロジェクト設定を使います。ファイルを入力とする設定の `exclude` と `report-exclude`、リンク先を調べる `no-dead-link` は、標準入力の本文には適用しません。

次の指定は、標準入力を読まずに終了コード2を返します。

- `-` とほかのファイルを並べる(`-` を2つ以上並べる場合を含む)
- `-` と `--diff`、または `check` の `--staged` を一緒に指定する
- `fix` で `-` と `--list` を一緒に指定する
- `fix` で `-` と `--show-diff` を、`--dry-run` を付けずに指定する

`-` という名前のファイルを校正するときは、`./-` と書きます。

### ルールの一覧

`jafix rules` は、組み込みのルール102個を定義の順に、1行に1つ標準出力に書きます。各行は、タブで区切った3つの欄からなります。

| 欄     | 内容                                                                                       |
| ------ | ------------------------------------------------------------------------------------------ |
| 1つ目 | ルールID                                                                                   |
| 2つ目 | 共通設定とプロジェクト設定を当てた後に有効なら `有効`、無効なら `無効`                     |
| 3つ目 | そのルールを束ねるpresetの名前をカンマで区切ったもの。どのpresetにも属さなければ `-`       |

```console
$ jafix rules
fullwidth-paren	有効	-
space-between-ja-and-alnum	有効	-
space-between-ja-and-paren	有効	-
sentence-length	無効	preset-ja-technical-writing
```

3つ目の欄の `-` は、presetに属さないことを表します。[ルール一覧](docs/rules.md)の表の初期状態の `-` (既定で無効)とは意味が違います。

引数にはファイルかディレクトリを1つ渡せます。ディレクトリならそのディレクトリから、ファイルならそのファイルのあるディレクトリから、プロジェクト設定を探します。既定値はカレントディレクトリです。パスは設定を探す起点としてだけ使い、`exclude` と `report-exclude` によるファイルの除外は出力に反映しません。除外したファイルを渡しても、設定で有効なルールを `有効` と出します。

`--output-format json` を指定すると、同じ内容をJSONオブジェクト1件として1行で書きます。`presets` は、presetに属さないルールでは空の配列です。

```json
{
  "schema_version": 1,
  "rules": [
    { "id": "fullwidth-paren", "enabled": true, "presets": [] },
    { "id": "sentence-length", "enabled": false, "presets": ["preset-ja-technical-writing"] }
  ]
}
```

設定ファイルが不正なときと、パスが存在しないときは、`jafix: ` で始まる理由を標準エラーに書き、標準出力には何も書かずに終了コード2を返します。`--output-format json` でも同じです。

### コマンド体系の出力

`jafix --help` と各サブコマンドの `--help` は、末尾に実行例を `例:` の見出しで出します。サブコマンドの `--help` は、終了コードの意味も `終了コード:` の見出しで出します。

`jafix --usage` は、コマンド体系をJSONオブジェクト1件として1行で標準出力に書き、終了コード0を返します。agentやスクリプトが、ドキュメントを読まずに使えるコマンドと引数を調べるのに使います。サブコマンドやほかの引数と一緒に指定すると、終了コード2を返します。

| キー                       | 内容                                                                                                            |
| -------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `schema_version`           | 出力の形の版。今は1                                                                                             |
| `name`・`version`・`about` | コマンドの名前、版、説明                                                                                        |
| `args`                     | `jafix` そのものの引数                                                                                          |
| `examples`                 | 実行例。各要素は `command` と `description` を持つ                                                              |
| `commands`                 | サブコマンド。各要素は `name`・`about`・`args`・`examples`、終了コードの `exit_codes` (`code` と `meaning`)を持つ |

`args` の各要素は、次のキーを持ちます。`--help` と `--version` は含めません。

| キー              | 内容                                                                     |
| ----------------- | ------------------------------------------------------------------------ |
| `name`            | 引数の名前                                                               |
| `long`            | `--` を付けたオプションの名前。位置引数では `null`                       |
| `positional`      | 位置引数か                                                               |
| `takes_value`     | 値を取るか                                                               |
| `multiple`        | 複数の値を取れるか                                                       |
| `default`         | 既定値。無ければ `null`                                                  |
| `possible_values` | 取りうる値。選択肢の無い引数と、値を取らないフラグでは空の配列           |
| `help`            | 説明。無ければ `null`                                                    |

`examples` と `exit_codes` は、`--help` の `例:` と `終了コード:` と同じ内容です。

### JSON出力のスキーマ

`jafix check --schema` は[JSON出力](#json出力)の、`jafix rules --schema` は[ルールの一覧](#ルールの一覧)のJSON出力の、JSON Schema (draft 2020-12)を1行で標準出力に書き、終了コード0を返します。ほかの引数と一緒に指定すると、終了コード2を返します。

```sh
jafix check --schema
```

## hookへの登録

`jafix hook claude` と `jafix hook codex` は、PostToolUse hookの入力JSONを標準入力から読みます。
校正の対象は今回の編集で追加または変更した、UTF-8の `.md` ファイルの行です。
修正後のファイルを調べ、対象行から始まる指摘を標準エラー出力でagentに返します。標準出力は使いません。

Claude Codeでは `settings.json` の `PostToolUse` に登録します。
EditとWriteの結果にある差分の `+` 行が対象です。
新規作成したファイルは全行を直します。

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

Codexでは `hooks.json` の `PostToolUse` に登録し、matcherを `apply_patch` にします。
jafixはパッチの `+` 行に対応する位置を編集後のファイルで探し、対象行を決めます。
`tool_name` が `apply_patch` の入力を処理します。`Bash` のコマンドにパッチが含まれていても対象外です。

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

指摘があると、hookは標準エラー出力にメッセージを書き、終了コード2を返します。Claude CodeとCodexはいずれもこの出力をagentに届けます。Codexでは元のツール出力が置き換わるため、メッセージの1行目で、編集を適用済みであることと指摘への対応を伝えます。

2行目からは、`jafix check` と同じ書式で指摘を1件ずつ並べます。同じ実行で対象行の判定やファイルの読み書きに失敗した場合は、指摘の後に理由を表示します。

```text
jafix: 今回変えた行に指摘がある。編集は適用済み。指摘を直すか、誤検知なら直さずに進めてよい
/path/to/t.md:2:1: [@textlint-rule/pattern] 「照合」: AIが多用する語
```

同じ行をもう一度編集すると、その行が再び指摘の対象になります。終了コード1の場合は、Claude CodeとCodexのどちらでも利用者に失敗を表示し、agentは作業を続けます。

| 終了コード | 意味                                                                                                                                     |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 0          | 指摘が無く、直したか、直すものが無かった                                                                                                 |
| 1          | 指摘が無く、入力JSONを読めない、設定ファイルが不正、対象行を決められない、ファイルを読み書きできない、のどれかに当たる(理由は標準エラー) |
| 2          | 対象行に指摘が1件以上ある(指摘と、あれば対象行を決められない理由などを標準エラーに書く)                                                  |

対象行を決められないのは、Claude Codeの入力に差分が無いときと、編集後のファイルでCodexのpatchのchunkの位置を1か所に決められないときです。そのファイルは変えずに、他のファイルは直します。

jafixは編集直後のファイルを読み、校正の対象行を決めます。oxfmtなどが同じファイルを先に書き換えると、対象行の位置を誤って判定することがあります。
別のformatterと併用する場合は、1つのスクリプトでjafixを先に実行します。入力JSONは標準入力から渡します。

```zsh
#!/usr/bin/env zsh
input=$(cat)
print -r -- "$input" | jafix hook claude
# この後に他のformatterを呼ぶ
```

## MCP

`jafix mcp` はstdioで起動し、`check_text` と `fix_text` を提供します。どちらも `text` にMarkdownを受け取り、応答の `structuredContent.findings` に指摘を返します。指摘の項目は、[JSON出力](#json出力)の `findings` から `path` を省いた形です。`fix_text` は同じ応答の `text` に修正後の本文を入れます。指摘の位置はその本文を基準にします。

起動コマンドと設定の探し方は[クイックスタート](docs/quick-start.md#下書きをmcpで校正する-mcp)にあります。

## 設定ファイル

共通設定は `~/.config/jafix/config.toml` に書きます。
環境変数 `XDG_CONFIG_HOME` に空でない値を設定している場合は、`$XDG_CONFIG_HOME/jafix/config.toml` を使います。

ファイルを渡した `jafix check` と `jafix fix`、hookは、対象ファイルの実体があるディレクトリから親へたどり、最初に見つかった `.jafix.toml` をプロジェクト設定として読みます。
[標準入力](#標準入力)を校正するときは、カレントディレクトリから同じように探します。
途中に `.git` があれば、そのディレクトリまで探します。
一度の実行で複数のプロジェクトを扱う場合も、各ファイルに対応する設定を使います。

設定は[組み込みの既定値](#組み込みのルール)、共通設定、プロジェクト設定の順に適用します。
`[builtin]` と `[filters]` はルールやfilterごとに上書きします。
オプションの表を指定すると、該当するルールやfilterの設定全体が置き換わります。新しい表で省いたオプションは既定値に戻ります。
同じファイルの `[builtin]` では、presetを先に適用し、次に個別のルールを適用します。
プロジェクト設定のpresetは共通設定の個別のルールより優先されます。
辞書ファイルなどの相対パスは、その指定を書いた設定ファイルがあるディレクトリを基準にします。

`exclude` と `report-exclude` は、プロジェクト設定で指定した項目だけを一覧ごと置き換えます。
共通設定の一覧を空にするには `[]` を指定します。
設定ファイルがどちらも無ければ、ファイルの除外は行わず、2つのfilterも無効です。

```toml
exclude = ["~/.dotfiles/docs/bad-examples.md", "**/drafts/**"]
report-exclude = ["**/notes/**"]

[builtin]
no-unmatched-pair = false
sentence-length = { max = 120, countBy = "codepoints" }

[[builtin."@textlint-rule/pattern".patterns]]
message = "AIが多用する語"
pattern = "/照合|正本/"

[[builtin."@textlint-rule/pattern".patterns]]
message = "感嘆符を使わない"
pattern = "/[!！]/"
```

### 除外

`exclude` に当たるファイルは、修正と指摘の両方の対象から外れます。
`report-exclude` に当たるファイルでは修正ルールを適用し、指摘は出しません。
規則に反する文言をそのまま写す文書など、修正してよくても指摘が不要なファイルに使います。

見出しの中にある、指摘に回す修正箇所も `report-exclude` の対象です。
その箇所は直さず、指摘も出しません。

`exclude` と `report-exclude` の値はglobで、symlinkを解決した対象ファイルの実体の絶対パスと比べます。`*` と `?` は `/` をまたがず、`**` は0個以上のディレクトリに当たります。そのため `**/drafts/*.md` は `drafts/a.md` に当たり、`drafts/sub/a.md` には当たりません。

値のうち、globの特殊文字(`*`、`?`、`[`、`{`)を含む最初のパスの要素より前の部分は、`~/` で始まればホームディレクトリに展開し、そのパスが存在すればsymlinkを解決した実体のパスに置き換えます。特殊文字が無ければ値の全体が対象です。この部分に含まれる特殊文字は、文字そのものとして扱います。そのため、symlinkのパスを書いた値もその実体に当たり、意図して規則に反する例を含むファイルを、symlink経由で渡されても外せます。

共通設定で指定するglobは絶対パスで書くか、`~/` または `**/` で始めます。
Windowsの絶対パスは `C:/work/docs/**` のように書けます。
プロジェクト設定では `generated/**` のような相対パターンも使えます。
相対パターンは `.jafix.toml` のあるディレクトリを基準にします。

### `[builtin]`

`[builtin]` では、使うルールをIDで選べます。複数のルールをまとめて設定するときはpreset名を使います。
個別の指定やpresetが適用されないルールには、[組み込みのルール](#組み込みのルール)に記載した既定値を使います。

| 値                            | 意味                                                           |
| ----------------------------- | -------------------------------------------------------------- |
| `false`                       | そのルールを無効にする                                         |
| `true`                        | 既定のオプションで有効にする                                   |
| オプションの表(`{ max = 5 }`) | 書いたオプションを変えて有効にする。書かないオプションは既定値 |

```toml
[builtin]
no-unmatched-pair = false
sentence-length = { max = 120, countBy = "codepoints" }
max-kanji-continuous-len = { allow = ["情報処理推進機構"] }
ja-no-redundant-expression = { dictOptions = { dict5 = { disabled = true } } }
```

無効にした修正ルールは、`jafix fix` とhookが適用せず、`jafix check` では出しません。無効にした指摘ルールは、`jafix check` でもhookでも出しません。`space-between-ja-and-alnum` と `space-between-ja-and-paren` を無効にすると、見出しの中の空白の指摘は出ません。

#### presetの名前

`[builtin]` のキーには、ルールのIDのほかに、textlintのpresetの名前を書けます。値は `true` か `false` です。`@` と `/` を含む名前は、TOMLのキーとして引用符で囲みます。

| presetの名前                     | 束ねるルール                                                                                                                                                                                 |
| -------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `preset-ja-technical-writing`    | `sentence-length` から `ja-no-redundant-expression` までの23ルール(jafix独自の3つは含まない)                                                                                                 |
| `@textlint-ja/preset-ai-writing` | `no-ai-list-formatting` から `ai-tech-writing-guideline` までの5ルール                                                                                                                       |
| `preset-ai-words-ja`             | `no-ai-words` と `no-short-topic-comma`                                                                                                                                                      |
| `preset-ja-spacing`              | `ja-nakaguro-or-halfwidth-space-between-katakana` から `ja-space-around-strong` までの11ルール。presetが有効にするのは、`ja-space-around-emphasis` と `ja-space-around-strong` を除く9ルール |
| `preset-jtf-style`               | `1.1.1.本文` から `4.3.8.一重引用符` までの37ルール。presetが有効にするのは、`2.1.2.漢字`・`2.1.5.カタカナ`・`2.1.6.カタカナの長音`・`2.2.1.ひらがなと漢字の使い分け` を除く33ルール         |

`true` は、presetのルールを、presetの既定の有効・無効とオプションで有効にします。`preset-ai-words-ja` の既定の設定は `no-short-topic-comma` を無効にするので、`preset-ai-words-ja = true` で有効になるのは `no-ai-words` だけです。`preset-ja-spacing = true` は、`ja-space-around-emphasis` と `ja-space-around-strong` を除く9ルールを、ルールの既定のオプションで有効にします(`ja-space-between-half-and-full-width` は `space = "never"`)。2つは、`ja-space-around-strong = true` のようにルールのIDのキーを書くと有効になります。`preset-ja-spacing` は、`no-mixed-zenkaku-and-hankaku-alphabet` と `no-kangxi-radicals` を含みません。`false` は、presetのルールをすべて無効にします。

```toml
[builtin]
"@textlint-ja/preset-ai-writing" = true
preset-ai-words-ja = true
no-ai-hype-expressions = false
no-short-topic-comma = { maxLength = 3 }
```

presetの名前のキーと、そのpresetのルールのIDのキーを同じ表に両方書いたときは、書いた順によらずルールのIDのキーがpresetの名前のキーより優先します。上の例では、`no-ai-hype-expressions` を除く4つと `no-ai-words` が既定のオプションで、`no-short-topic-comma` が `maxLength = 3` で有効になります。presetの名前の値にオプションの表は書けません。オプションは、ルールのIDのキーで書きます。

`preset-jtf-style = true` は、37ルールのうち `2.1.2.漢字`・`2.1.5.カタカナ`・`2.1.6.カタカナの長音`・`2.2.1.ひらがなと漢字の使い分け` の4ルールを除く33ルールを、ルールの既定のオプションで有効にします。textlintのpresetも、この4ルールを有効にしません。4ルールは、`"2.1.2.漢字" = true` のようにルールのIDのキーを書くと有効になります。textlintでは、presetの設定の中で4ルールに `true` を書いても有効にならず、オプションの表を渡す必要がありますが、jafixでは `true` で有効になります。`preset-jtf-style` は、jafix独自の修正ルールと、`arabic-kanji-numbers` などほかのpresetのルールを含みません。

`preset-jtf-style = true` だけの設定は不正です。presetが有効にする `4.3.1.丸かっこ（）` は和文の丸括弧を全角にすることを求め、設定を書かなくても有効な `fullwidth-paren` は全角の丸括弧を半角にするためです(「[不正な設定](#不正な設定)」)。全角の丸括弧を使うなら `fullwidth-paren = false` を、半角の丸括弧を使うなら `"4.3.1.丸かっこ（）" = false` を、同じ表に書きます。

```toml
[builtin]
preset-jtf-style = true
fullwidth-paren = false
"2.1.6.カタカナの長音" = true
```

#### パターン

`sentence-length` の `skipPatterns`、`no-exclamation-question-mark`・`ja-unnatural-alphabet`・`ja-no-successive-word` の `allow`、`ja-no-redundant-expression` の `dictOptions` の `allows`、AIらしい書き方を指摘する6ルール(`no-short-topic-comma` 以外)と `ja-space-between-half-and-full-width` の `allows` は、パターンの配列です。各要素は、`/source/flags` の形なら正規表現、それ以外は書いた文字列そのものに当たります。正規表現は、`@textlint-rule/pattern` と同じくJavaScriptの正規表現(`u` フラグ付き)で読みます。`/source/flags` とみなすのは、`^/(.+)/([guimysd]*)$` に当たるときだけで、`/usr/bin` は文字列そのものです。`//` は、すべてに当たる空の正規表現ではなく、文字列そのものの `//` です。フラグは `i`・`m`・`s` を使えます。`g`・`u`・`d` は書いてよく、無視します。`y` は不正です。

JavaScriptの正規表現なので、後読み・先読み・後方参照を使え、`\p{sc=Han}` や `\p{Script=Han}` のようにscriptの名前を `sc=` か `Script=` 付きで書けます。`\d`・`\w`・`\b` はASCIIで判定するので、`/\d!/` は `1!` に当たり、`１!` に当たりません。行の区切りと空白の扱いもJavaScriptのとおりで、`.` は `\r`・U+2028・U+2029に当たらず、`m` 付きの `^`・`$` はこの3つも行の区切りとし、`\s` はU+FEFFを含みます。Rustの `regex` だけが読む書き方(`\p{Han}`・`(?i)`・`[[:alpha:]]`・`\z`)と、`u` 付きのJavaScriptが断る書き方(`\-` のような意味の無い退避、閉じていない `{`)は、設定の不正です。空の文字列も不正です。

### `[filters]`

`[filters]` では、文書内の範囲を修正と指摘の対象から外すfilterを設定します。
`comments` と `allowlist` は、それぞれtextlintの `textlint-filter-rule-comments` 1.3.0と `textlint-filter-rule-allowlist` 4.0.0から移したものです。

値は `false`(無効)、`true`(既定のオプションで有効)、またはオプションの表です。
指定を省くと、どちらも無効です。

```toml
[filters]
comments = true
allowlist = { allow = ["/TODO: 移行の後に消す/"], allowlistConfigPaths = ["allow.yml"] }
```

filterは、指定した範囲に収まる指摘と修正を除きます。修正には、組み込みの修正ルールと利用者の置き換えが含まれます。指摘は報告する範囲、修正は置き換える範囲で判定します。始点と終点がどちらも指定範囲の内側か境界にあれば除外します。

除外は、重なり合う修正箇所の優先順位を決める前に行います。そのため、修正箇所がfilterで除外されても、重なる別の候補は残ります。

`jafix fix` とhookは、利用者の置き換えの前と、組み込みの修正ルールを適用するたびに、ファイルのその時点の内容から除外範囲を求め直します。判定には文書全体を使います。hookの対象行の外にあるコメントやパターンの一致によって、対象行の指摘を除くこともできます。

`comments` は、HTMLコメントで囲んだ範囲を校正の対象から外します。開始位置は無効化コメントの `<!--`、終了位置は対応する有効化コメントの `<!--` です。有効化コメントを省くと、文書の末尾まで除外します。

コメントに `enablingComment` の値があれば有効化します。既定値は `textlint-enable` です。これに該当せず、`disablingComment` の値があれば無効化します。既定値は `textlint-disable` です。どちらも空でない文字列で、大文字と小文字を区別します。

引用、リスト項目、表のセル、見出しの中にあるコメントも読みます。コードブロックとインラインコードの中は対象外です。

```markdown
<!-- textlint-disable no-todo, sentence-length -- 移行の後に消す -->

TODO: この段落の指摘は出ません

<!-- textlint-enable -->
```

コメントの中身から指示の文字列を除き、`--` から後ろを捨てた残りを、カンマで区切ったものがルールのIDの一覧です。一覧が空なら、すべてのルールを外します。IDを書いた有効化は、そのIDの開いている無効化のうち最後のものを閉じ、IDの無い有効化は、開いている無効化をすべて閉じます。IDを書いた有効化は、IDを書かない無効化を閉じません。`textlint-disable-next-line` は、textlintと同じく、存在しないID `-next-line` の無効化になり、何も外しません。

IDは、jafixのルールのID(`sentence-length`)でも、textlintがpresetの下のルールに付けるID(`ja-technical-writing/sentence-length`)でも書けます。textlintと同じく、presetの部分からは前置きの `preset-`・`textlint-rule-preset-` を、ルールの部分からは `textlint-rule-` を除いて比べます。そのため、`preset-ja-technical-writing/sentence-length`・`textlint-rule-no-todo`・`@textlint-rule/textlint-rule-pattern` も当たります。presetの部分は、`ja-technical-writing`・`@textlint-ja/ai-writing`・`ai-words-ja`・`ja-spacing`・`jtf-style` のどれかで、そのpresetの束ねるルールに当たります。presetの名前で有効にしたかは問いません。どのルールにも当たらないIDは、何も外しません。半角のカンマを含む `1.2.2.ピリオド(.)とカンマ(,)` と `4.1.3.ピリオド(.)、カンマ(,)` は、一覧の区切りと衝突するので、どちらの書き方でも書けません。この2つのルールは、IDを書かない無効化でだけ止まります。

`allowlist` は、許可したパターンに一致した範囲を、すべてのルールについて外します。パターンは、`allow`(パターンの配列)と、`allowlistConfigPaths` のファイルの要素を、この順に合わせたものです。各要素は「[パターン](#パターン)」と同じく読み、空の文字列は不正です。パターンは、コード・URL・HTML・front matterを含む文書の元のソースの全体に当て、長さ0を含むすべての一致の範囲を外します。`^` は、`m` を付けなければ文書の先頭だけに当たります。`allowlistConfigPaths` は、文字列の配列を書いたJSONかYAMLのファイルのパスの配列で、空でない文字列です。拡張子は `.json`・`.yml`・`.yaml` のどれかです。相対パスは設定ファイルのあるディレクトリから解決し、`~/` で始まればホームディレクトリに展開します。

### 不正な設定

次のときは、設定ファイルが不正です。理由を標準エラーに書いて終え、終了コードは `jafix check`・`jafix fix`・`jafix rules` が2、hookが1です。`[builtin]` が不正なときは、理由にルールのIDかpresetの名前を含め、オプションの誤りでは、オプションの名前も含めます。

- TOMLとして読めないか、`exclude`・`report-exclude`・`builtin`・`filters` 以外のキーがある。`rules` の表は、知らないキーなので不正です。利用者のパターンは `[builtin]` の `@textlint-rule/pattern` に書きます
- `exclude` か `report-exclude` の値をglobとして読めない。または共通設定の値が絶対パスにも `~/`・`**/` から始まるパターンにも当たらない
- `[builtin]` のキーが組み込みのルールのIDでもpresetの名前でもない
- `[builtin]` のpresetの名前の値が、`true` か `false` でない
- `[builtin]` のルールのIDの値が、`true`・`false`・表のどれでもない
- `[builtin]` のルールの表に、そのルールの持たないオプションがある(オプションを持たないルールでは、どのキーも当たります)。textlintのオプションのうち `exclusionPatterns`・`forceAppendPeriod`・`checkFootnote`・`checkCode`・`checkImage`・`separatorChars` は、jafixが対応しないので、これに当たります
- オプションの値の型か値が誤っている。`sentence-length` と `max-ten` の `max` は0以上の整数、`max-comma` と `max-kanji-continuous-len` の `max` と `no-doubled-joshi` の `min_interval` は1以上の整数、`countBy` は `"codeunits"` か `"codepoints"`、`no-invalid-control-character` の `allow` の要素は1文字の文字列、パターンは「[パターン](#パターン)」のとおりに読めるもの
- 形態素解析を使うルールのオプションの値が誤っている。`max-ten` の `touten` と `kuten` は空でない文字列、`no-doubled-joshi` の `allow`・`separatorCharacters`・`commaCharacters` の要素は空でない文字列、`no-mix-dearu-desumasu` の `prefer` で始まる3つは `""`・`"ですます"`・`"である"` のどれか、`allowNodeTypes` の要素は「[組み込みのルール](#組み込みのルール)」に挙げた10の型のどれか、`dictOptions` のキーは `dict1` から `dict6` のどれかで、値は `disabled` と `allows` だけを持つ表
- AIらしい書き方を指摘する7ルールのオプションの値が誤っている。`disable` で始まるオプションと `enableDocumentAnalysis` は真偽値、`allows` はパターンの配列、`no-ai-words` の `dictionaryPath` は空でない文字列で `dictionaryMode` は `"append"` か `"override"`、`no-short-topic-comma` の `maxLength` は1以上の整数。textlintの `severity` は、jafixが対応しないので、知らないオプションに当たります
- `3.3.かっこ類と隣接する文字の間のスペースの有無` のオプションの値が誤っている。`allowOutsideHalfParentheses` と `requireOutsideHalfParentheses` は真偽値です。`allowOutsideHalfParentheses = false` と `requireOutsideHalfParentheses = true` の組も不正で、理由に2つのオプションの名前を含めます。前者が削除する半角の丸括弧の外側の空白を、後者が求めるためです。JTFスタイルガイドのほかの36ルールはオプションを持たないので、表にキーがあれば知らないオプションです
- 空白と字種の13ルールのオプションの値が誤っている。`ja-space-between-half-and-full-width` の `space` は `"never"`・`"always"` か、`"alphabets"`・`"numbers"`・`"punctuation"` からなる配列、`lintStyledNode` と4つの `ja-space-around-*` の `before`・`after` は真偽値、`allows` はパターンの配列、`no-mixed-zenkaku-and-hankaku-alphabet` の `prefer` は `"全角"` か `"半角"`。`exceptPunctuation` は、知らないオプションに当たります
- `no-ai-words` の `dictionaryPath` のファイルが読めないか、JSONとして読めないか、利用者の辞書の形でない。`tokens` の要素に、上に挙げたトークンの項目の名前でも `_` で始まる名前でもないキー(kuromojiの `word_id`・`word_type`・`word_position` を含む)があるときも、これに当たります。理由には `dictionaryPath` とファイルのパスを含めます
- `no-ai-words` の `dictionaryMode` が `"override"` なのに、`dictionaryPath` が無い
- ひらがな系の5ルール・`@textlint-ja/no-insert-dropping-sa`・`@textlint-ja/no-synonyms` のオプションの値が誤っている。`ja-hiragana-hojodoushi` と `ja-hiragana-fukushi` の `rulePath` は空でない文字列、`ja-keishikimeishi` の `detection_hou_kata`・`detection_ue` と、`@textlint-ja/no-synonyms` の `allowAlphabet`・`allowNumber`・`allowLexeme` は真偽値、`allows`・`preferWords` は文字列の配列です。`ja-hiragana-keishikimeishi`・`ja-hiragana-daimeishi`・`@textlint-ja/no-insert-dropping-sa` はオプションを持たないので、表にキーがあれば知らないオプションです
- `ja-hiragana-hojodoushi` か `ja-hiragana-fukushi` の `rulePath` のファイルが読めないか、YAMLとして読めないか、辞書の形でない。辞書は、空でない文字列の `message` と空でない配列の `dict` を持つ表で、`dict` の各項目は文字列の `expected` と空でない配列の `tokens` を持ちます。`tokens` の要素は `no-ai-words` の利用者の辞書と同じく読むので、トークンの項目の名前でも `_` で始まる名前でもないキー(`word_id`・`word_type`・`word_position` を含む)があれば、これに当たります。理由には `rulePath` とファイルのパスを含めます
- 言語によらない3ルールのオプションの値が誤っている。`date-weekday-mismatch` の `lang` は `"ja"` か `"en"`、`useCurrentYearIfMissing` は真偽値、`currentYear` は1以上の整数です。`no-dead-link` の `checkRelative`・`ignoreRedirects`・`httpOnly` は真偽値、`baseURI` と `userAgent` は空でない文字列、`ignore` はglobとして読める文字列の配列、`preferGET` はoriginを持つURLの配列です。`retry`・`maxRetryTime`・`maxRetryAfterTime` は0以上の整数、`concurrency` は1以上の整数です。textlintの `dotInIgnore`・`interval`・`intervalCap`・`linkMaxAge` は、jafixが対応しないので、知らないオプションに当たります。`no-todo` はオプションを持たないので、表にキーがあれば知らないオプションです
- 英語の4ルールのオプションの値が誤っている。`terminology` の `defaultTerms`、`spelling` の `suggestCorrections`、`write-good` の9つの判定の名前のキーは真偽値です。`terminology` の `exclude` と `common-misspellings` の `ignore` は文字列の配列、`write-good` の `whitelist` は空でない文字列の配列、`spelling` の `skipPatterns` はパターンの配列です。textlintの `write-good` の `checks` は、jafixが対応しないので、知らないオプションに当たります
- `terminology` の `terms` が、項目の配列か空でない文字列でない。項目が、空でない文字列か2つの文字列の配列でないか、`i`・`m`・`u` を付けたJavaScriptの正規表現として読めない。理由には `terms` と何番目の要素かを含めます。`skip` の要素が、`@textlint-rule/pattern` の `allowNodeTypes` と同じ13の型の名前のどれでもないときも不正です
- `terminology` の `terms` のファイルが読めないか、JSONかコメント付きのJSONとして読めないか、項目の配列でない。理由には `terms` とファイルのパスを含めます
- `spelling` の `language` が `"en"` でない。`wordDefinitionRegexp` が `/source/flags` の形でないか、正規表現として読めないか、`y` フラグを持つときも不正です
- `prh` のオプションが誤っている。`rulePaths` と `ruleContents` に辞書が1つも無い(`prh = true` を含む)、`check` で始まるオプションが真偽値でない、`checkCodeComment`・`debug`・知らないオプションがある。理由には `prh` とオプションの名前を含めます
- `prh` の `rulePaths` か `ruleContents` の辞書が誤っている。理由には `prh`、`rulePaths` か `ruleContents`、辞書のファイルのパス(`ruleContents` では何番目の要素か)を含めます。次のどれかに当たります
  - 辞書のファイルが読めないか、YAMLとして読めない。辞書の値の型が誤っている(`expected` が文字列でない、`rules` が配列でないなど)
  - ルールに `expected` が無いか、`pattern`・`patterns` の値か要素が空の文字列
  - `pattern`・`patterns` が正規表現として読めないか、配列の要素の `i` と `s` のフラグが食い違う。`regexpMustEmpty` が `$1` の形でない
  - `specs` の要素の `from` と `to` のどちらかが空、またはルールを `from` に当てた結果が `to` と違う。理由には、そのルールの `expected`、`from`、`to`、当てた結果を含めます
  - `imports` が循環する
- `@textlint-rule/pattern` のオプションが誤っている。`patterns` が無いか空(`"@textlint-rule/pattern" = true` を含む)、要素に `message` か `pattern` が無い、要素の知らないキー(`forceCode` を含む)がある、`allowNodeTypes` の型の名前が上に挙げたもの以外、`pattern` と `allows` が正規表現として読めない、`y` フラグがある。理由にはルールのID、オプションの名前、何番目の要素かを含めます
- 逆の書き方の設定。組み込みの修正ルールが直した書き方を、別のルールが元に戻すよう求める組を、両方とも有効にした場合です。hookが直した箇所への指摘が出続け、修正ルールどうしの組では修正の繰り返しが止まらないためです。理由には、2つのルールのIDと、逆の書き方を求める側の設定を含めます。理由の最後には、無効にする書き方を、設定にそのまま書けるキーと値で示します(`` `fullwidth-paren = false` か `"4.3.1.丸かっこ（）" = false` にする ``)。4つの `ja-space-around-*` の `true` の側は、どの修正ルールともnodeとの境目の空白を取り合わないので、組み合わせを問わず読み込めます。組は次のとおりです
  - 空白を求める設定(`space` が `"always"` か、`"alphabets"` か `"numbers"` を含む配列)の `ja-space-between-half-and-full-width` と、`space-between-ja-and-alnum`
  - 空白を求める設定の `ja-space-between-half-and-full-width` と、`3.1.1.全角文字と半角文字の間`
  - `4.3.1.丸かっこ（）` と `fullwidth-paren`。`fullwidth-paren` は設定を書かなくても有効なので、`preset-jtf-style = true` だけの設定はこの組に当たります
  - `requireOutsideHalfParentheses = true` の `3.3.かっこ類と隣接する文字の間のスペースの有無` と、`space-between-ja-and-paren`
  - `prefer = "全角"` の `no-mixed-zenkaku-and-hankaku-alphabet` と、`2.1.9.アルファベット`。どちらも修正ルールなので、修正を繰り返すたびに英字が全角と半角を行き来します
- 逆の置き換え。利用者の置き換えの固定の部分を、有効な組み込みの修正ルールが直す場合です。理由にはルールのID、その置き換え(`expected`・`replace`・`terms` の項目)、組み込みの修正ルールのIDを含めます。同じ箇所に当たる組み込みの修正ルールが2つ以上あれば、重なりを解く前のものをすべて挙げます
- 辞書の指摘ルールと逆の置き換え。利用者の置き換えの固定の部分を、捕捉参照に接する端を除いて、段落の直下にある本文の範囲1つだけの文書とみなし、辞書で語を別の書き方にするよう求める有効な組み込みの指摘ルールが指摘する場合です。ルールは、辞書の4ルール(`2.1.5` の辞書の部分・`2.1.6`・`2.2.1`・`2.2.3`)、ひらがな系の5ルール、`ja-no-abusage`、`preferWords` を書いた `@textlint-ja/no-synonyms`、`terminology` の組み込みの辞書、`common-misspellings` です。理由には、ルールのIDとその置き換え、指摘ルールのID、無効にする書き方を含めます。`terminology` の組み込みの辞書は、`defaultTerms = false` で無効にする書き方を示します。品詞や前後の語で決まる指摘は、固定の部分だけでは出ないことがあり、そのときは不正にしません。ほかの指摘ルール(`4.2.7` などの記号のルール、`spelling`、`preferWords` を書かない `@textlint-ja/no-synonyms` など)は、別の書き方で両方を満たせるので、判定に入りません
- `[filters]` のキーが、`comments`・`allowlist` のどちらでもない。値が `true`・`false`・表のどれでもない。`comments` に知らないオプションがあるか、`disablingComment`・`enablingComment` が空でない文字列でない。`allowlist` に知らないオプションがあるか、`allow` の要素が「[パターン](#パターン)」のとおりに読めない(空の文字列を含む)。理由には `filters`、filterのID、オプションの名前を含めます
- `allowlistConfigPaths` が、空でない文字列の配列でない。またはそのファイルが、読めない・拡張子が `.json`・`.yml`・`.yaml` のどれでもない・中身が文字列の配列でない・要素がパターンとして読めない(空の文字列を含む)のどれかに当たる。ファイルの誤りでは、理由にファイルのパスを含めます

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
- 文は、ハード改行(行末の2つの空白か `\`)の次の行が列挙の記号で始まる場合と、ラベル行の末尾で区切ります。`– 項目A` と `– 項目B` などを1文に数えません。記号の条件は、上の段落と文の単位の説明のとおりです。記号の無い行は文の継続として扱うので、折り返した長い文も `sentence-length` で指摘します。textlintはハード改行で文を区切りません。文の単位で当てるほかのルール(`max-comma` と、形態素解析を使う文の単位のルール)も、同じ区切りを使います。`no-unmatched-pair` は、textlintと同じくハード改行で文を区切らないので、ハード改行をまたいで閉じた括弧と引用符を指摘しません
- `sentence-length` は、リスト項目などの継続行の行頭の空白とタブを、文の長さに数えません。CommonMarkでは、この空白は段落の内容に入りません。textlintは数えるので、深く字下げした継続行のある文を長すぎると指摘します
- 指摘の文面は、textlintの文面ではなく、jafixの1行の日本語です

形態素解析を使う11ルールは、判定の条件(どのトークンの並びを指摘するか、例外、辞書)をtextlintから写していますが、textlintと同じ箇所を指摘するとは限りません。上の点のうち、列の数え方、パターン、文面は11ルールにも当てはまります。ほかに次の点が違います。

- トークンは、kuromojiではなくlinderaとIPADICで分けます。句読点の直後の語などで品詞がkuromojiと変わることがあり、その文では指摘の有無が変わります
- 位置は、指摘するトークンの元のソースでの位置です。textlintは、絵文字などの後ろ、同じ文字列の子が複数ある文、`no-doubled-conjunctive-particle-ga` の2文目以降などで、位置がずれます
- トークンの並びを当てる途中の状態を、本文の範囲・文・段落・ファイルをまたいで持ち越しません。textlintは、インラインコードや段落をまたいだ並びも指摘します
- 並びは、トークン列のすべての位置から当てます。一致の途中で外れたトークンも次の開始位置として試すので、textlintが指摘しない `ないないことはない` の「ないことはない」も指摘します
- 段落と文の単位のルールは、文から作った文字列を解析します。インラインコードの中身、リンクの宛先、リンクや強調の記号、HTMLのタグは判定に入りません。段落の直下の文字列は元のソースのまま解析するので、エスケープと文字参照は記号として残ります
- 文の区切り方は、ほかの組み込みのルールと同じです。textlintの `max-ten` と `no-doubled-joshi` はオプションで文の区切り文字を変えますが、jafixの `kuten` と `separatorCharacters` は文の区切り方を変えません
- `no-mix-dearu-desumasu` は、引用の中の文を数えず、入れ子のリスト項目の段落も1回ずつ数えます。textlintはリスト項目の原文を丸ごと解析するので、入れ子の項目を二重に数えます
- `no-mix-dearu-desumasu` は、文に区切ってから文末を見るので、`．` で終わる文の文末も数えます。textlintの `analyze-desumasu-dearu` 5.0.2は、文末を `。` `!` `?` `！` `？` の前でだけ判定するので、`，` `．` で書いた文書の文体の混在を見逃します
- `ja-no-successive-word` は、品詞が記号のトークンの連続を、句点、読点、括弧開、括弧閉のほかは指摘しません。踊り字の連続はトークンの境界によらず指摘します。`「「`・`」」`・`時々々々` の打ち間違いを見逃さないためです。伏せ字の `○○`、2倍のダッシュ `──`、2倍の三点リーダー `……` は正しい書き方だからです。textlintは、オプションを書かなければ指摘します
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
- 組み込みの修正ルールと逆の置き換えと、辞書で語を別の書き方にするよう求める組み込みの指摘ルールと逆の置き換えは、固定の部分だけで判定できる範囲で設定の不正にします。捕捉参照の端を文書の先頭や末尾とはみなさず、参照の内容で変わる指摘や空白の修正は判定から外します。textlintは、どちらの組も読み込みます
- 一致の周りがすでに置き換え後の文字列になっている箇所は、直さず、指摘もしません(「[修正の順と直さない箇所](#修正の順と直さない箇所)」)。textlintは直すので、`サーバ` を `サーバー` にする辞書では、すでに書いた `サーバー` が直すたびに `サーバーー`、`サーバーーー` と伸びます
- 位置は元のソースでの位置で、列は文字で数えます

空白と字種の13ルールも、判定の条件を `textlint-rule-preset-ja-spacing` 3.0.4・`textlint-rule-no-mixed-zenkaku-and-hankaku-alphabet` 1.0.1・`textlint-rule-no-kangxi-radicals` 0.2.2から写し、textlintの不具合に近い挙動は写しません。上の点のうち、列の数え方、パターン、文面は13ルールにも当てはまります。ほかに次の点が違います。

- 当たる箇所をすべて指摘します。textlintは一致を重ねずに探すので、`ja-no-space-between-full-width` の `日 本 語`、`ja-nakaguro-or-halfwidth-space-between-katakana` の `ア　イ　ウ`、`ja-space-after-exclamation` の `あ！ ！ これ` で、2件のうち1件だけを指摘します
- `ja-no-space-around-parentheses` は括弧に隣り合って続く空白を、`ja-no-space-around-slash` は `/` の前後に続く空白を、まとめて1件にして削除します。textlintは、括弧では1文字だけを、`/` では前後それぞれ10文字までを削除するので、1回の修正で消し切りません。行頭から続く空白(リストの継続行の字下げ)は、括弧と同じく `/` の前でも削除しません
- 4つの `ja-space-around-*` は、文字で日本語かを判定し、U+20000からU+2FFFFの漢字も日本語の文字に数えます。textlintはUTF-16のコードユニットで判定するので、サロゲートペアの漢字を日本語の文字とみなしません
- `ja-space-between-half-and-full-width` と `ja-no-space-between-full-width` は、U+20000からU+2FFFFの漢字の直後の空白を、その空白の位置で削除します。textlintは、一致の位置にUTF-16のコードユニット1つ分を足して空白の位置とするので、`𠮷 A` では漢字の後半を削除して文字を壊します
- `ja-no-space-around-slash` の位置は、削除する空白の並びの先頭です。`/` の前に空白が無ければ、`/` の後ろの空白の先頭です。textlintは一致の先頭に出すので、`/` の前に空白が無いと `/` に出します
- 4つの `ja-space-around-*` は、nodeの前か後ろに文字が無ければ、その側を判定しません。textlintの `before: true` は、文書の先頭のインラインコードにも指摘を出します。`after: false` の位置は、削除する空白です。textlintは空白の次の文字に出します
- `ja-space-around-link` は、裸のURLに当てません。textlintは裸のURLの前後の空白も削除するので、前の空白を消すとURLとして読まれなくなり、後ろの空白を消すと続く文字がURLに入ります。自動リンク(`<…>`)とインラインのリンクには当てます
- `ja-space-around-strong` と `ja-space-around-emphasis` は、空白を削除すると強調として読まれなくなる箇所(区切りが `_` のときや、区切りの内側の文字が句読点か記号のとき)を判定しません。textlintは削除して強調を壊します
- `ja-space-between-half-and-full-width` の `allows` は、判定で見た並びの先頭が、一致の範囲の中(終わりの位置を含まない)にあれば外します。textlintは終わりの位置も含めるので、許した文字列の直後の文字で始まる並びも外します。`allows` の要素は、「[パターン](#パターン)」の書き方で読みます
- `no-mixed-zenkaku-and-hankaku-alphabet` の `prefer` は、`"全角"` か `"半角"` だけを受け付けます。textlintは `"半角"` 以外のすべての値を `"全角"` とみなします
- 本文でない箇所(裸のURL、自動リンクの文字列、定義を持つ省略形とcollapsed形の参照リンクの文字列)を見ません。textlintでは、これらも文字列のnodeです。そのため `no-kangxi-radicals` と `lintStyledNode = true` の `ja-space-between-half-and-full-width` が当たり、`no-mixed-zenkaku-and-hankaku-alphabet` は参照リンクの文字列を直して定義との対応を切ります
- 見出しの中の修正の箇所は直さず、指摘として出します
- 組み込みの修正ルールの箇所の重なりを1件にし、修正を繰り返し当てます。textlintは、ルールごとに修正を当て、同じ空白に2つのルールの指摘を出します
- `ja-space-between-half-and-full-width` の `exceptPunctuation` は使えません

JTFスタイルガイドの37ルールも、判定の条件を `textlint-rule-preset-jtf-style` 3.0.3と、その依存の `analyze-desumasu-dearu` 2.1.5・`moji` 0.5.1・`textlint-rule-prh` 6.1.1から写し、textlintの不具合に近い挙動は写しません。上の点のうち、列の数え方と文面は37ルールにも当てはまり、辞書の4ルールは `prh` の違いも持ちます。ほかに次の点が違います。

- 記号の種類を変える箇所・記号を外す箇所・空白を足す箇所・辞書で語を置き換える箇所は、textlintと違って自動で直さず、指摘だけを出します(「[JTFスタイルガイドの37ルール](#jtfスタイルガイドの37ルール)」)
- `1.1.1`・`1.1.3`・`1.1.5` は、常体の文末の形の `ではない｜ないか` を、`ではない` と `ないか` の2つの形として数えます。textlintは全角の縦線を含む1つの文字列として探すので、どちらの形も常体に数えません
- `1.1.1`・`1.1.3`・`1.1.5` は、1行の中の文末の形をすべて数えます。textlintは1行に1件だけ数えます
- `1.1.3` は、リストごとに、そのリストの直下の項目だけを数えます。項目の文字列は、入れ子のリストを除いた、項目の直下の段落の本文で、インラインコードの中身、コードブロック、HTMLを入れません。textlintは入れ子の項目を親のリストでも数え直すので、子の項目に互いに逆の指摘を出します
- `1.2.2` は、本文の範囲のすべての `，` `．` を指摘します。textlintは最初の1件だけを指摘します
- `2.1.5` の辞書は、カタカナが前後に続く一致を指摘しません。`アンティーク` の `アンティ`、`バレーボール` の `バレー` のように、別の語の一部に当たる誤検知を避けます。`2.2.1` も、漢字だけの一致の前後に漢字が続くときは指摘せず、`何時間` の `何時` や `最早期` の `最早` を残します。辞書のファイルは変更しません。この判定では、辞書の語にカタカナや漢字をつないだ複合語も指摘から外れます
- `2.1.9` は、全角の大文字に加えて、全角の小文字(`ａ` から `ｚ`)も半角にします
- `2.2.2` はすべての一致を指摘し、無視パターンは一致と重なるときだけその一致を外します。textlintは、パターンごとに最初の一致だけを指摘し、無視パターンが文字列のどこかに当たれば、そのパターンを指摘しません。文面には変換後の数字を入れず、`arabic-kanji-numbers` と同じく一致した文字列を続けるので、textlintが例外で止まる `十十回` でも止まりません
- 前後の文字で判定する `3.1.2`・`3.2`・`4.2.4`、`4.2.1`・`4.2.2` の後ろの空白、`4.3.1` の前後がどちらも日本語の文字の括弧は、当たる箇所をすべて見ます。textlintは一致を重ねずに探すので、`日 本 語`、`ア　イ　ウ`、`あ！ ！ これ`、`ア･イ･ウ` の2つ目と、`日(日)日(日)日(日)日` の2つ目の `)` を指摘しません
- `3.3` は、括弧に隣り合って続く空白をまとめて1件にして削除し、行頭の字下げは削除しません。textlintは括弧に接する1文字だけを削除し、字下げも削除します
- `4.3.1` の位置は、正規表現が捕捉した括弧の位置です。textlintは、捕捉した文字を一致の先頭から探すので、`関数 foo() と bar() を呼ぶ` の後ろの括弧を、`bar()` の `)` ではなく `foo()` の `)` と報告します
- 同じルールの、位置と文面が同じ指摘は1件にします
- 本文でない箇所(裸のURL、定義を持つ省略形・collapsed形の参照リンクの文字列)を見ません。`4.3.3`〜`4.3.7` の段落の中の記号と、辞書の4ルールも同じです
- `4.3.7` は、矢印と比較の記号(`->`・`=>`・`<-`・`<=`・`>=`)の `<` `>` を山かっことして数えません。textlintは数えるので、`処理A -> 処理B` の `>` を対にならない山かっことして指摘します
- `1.1.5` は、altの元のソースに当て、元のソースでの位置を出します。textlintはエスケープを外したaltに当てるので、エスケープを含むaltで位置がずれます
- `2.1.2` の列は、ほかのルールと同じく文字で数えます。textlintは文字の添字を位置にするので、サロゲートペアの漢字の後ろで位置がずれます

ひらがな・さ入れ・同義語の7ルールも、判定の条件を7つのpackageから写し、textlintの不具合に近い挙動は写しません。上の点のうち、列の数え方と文面は7ルールにも当てはまります。ほかに次の点が違います。

- トークンは、kuromojiではなくlinderaとIPADICで分けます。品詞や読みがkuromojiと変わる文では、指摘の有無が変わります
- ひらがな系の5ルールとさ入れのルールは、辞書の並びを当てる途中の状態を、本文の範囲をまたいで持ち越しません。並びは、すべての位置から当てます。textlintは、前の本文の範囲のトークンから始まった一致を次の本文の範囲で完成させ、元の一致とは別の文字を置き換える指摘を出すことがあります
- `ja-hiragana-fukushi` の辞書の `却って` の項目は、キーの書き誤り(`po"`)を `pos` に直して写します。textlintでは、この項目は一度も当たりません。`薄々` の重複した項目は1つにします
- `ja-keishikimeishi` の `detection_hou_kata` と `detection_ue` は独立で、`false` にしたほうだけを止めます。textlintは、オプションの表にキーが1つでもあると、`true` と書かなかったほうも止めます
- `rulePath` の空文字列と、利用者の辞書のトークンの知らない項目名を、設定の不正にします。textlintは、空文字列ではルールの実行に失敗し、知らない項目名ではその項目が当たらなくなります
- `@textlint-ja/no-synonyms` は、語を `tiny-segmenter` ではなくlinderaのトークンの最長一致で区切ります。textlintが `極めて` を `極め` と `て` に割って出す指摘は出しません。textlintが1語にして当てない `カテゴリレベル` と `カテゴリーレベル` は、`カテゴリ` と `カテゴリー` の表記の揺れとして指摘します
- `@textlint-ja/no-synonyms` の集まりの指摘は、使われた表記ごとに、その表記が文書の中で最後に出た位置へ1件ずつ出します。textlintは集まりごとに1件だけを出し、その位置は、最初に出たのが最も遅い表記の最後の出現です
- 2つの形式名詞のルールを両方とも有効にすると、2つのIDの指摘を1件ずつ出します。textlintは、範囲と文面の同じ指摘を、ruleIdを問わず1件へまとめます。どちらのルールの指摘が残るかは、実行ごとに変わります
- 本文でない箇所(裸のURL、自動リンク、定義を持つ省略形・collapsed形の参照リンクの文字列)を見ません。textlintでは、ひらがな系の5ルールとさ入れのルールがこれらの文字列にも当たります。`@textlint-ja/no-synonyms` もリンクの中だけを外すので、参照リンクの文字列の語まで数えます
- `ja-hiragana-keishikimeishi`・`ja-keishikimeishi`・`@textlint-ja/no-insert-dropping-sa` は、絵文字や `𠮷` のようなサロゲートペアの文字の後でも、置き換える語の位置を指摘します。textlintの指摘の位置は、その文字の数だけ前へずれます
- 7つとも、textlintと違って自動で直さず、指摘だけを出します。`@textlint-ja/no-synonyms` の `preferWords` の置き換えも指摘にします

言語によらない3ルールも、判定の条件を `textlint-rule-no-todo` 2.0.1・`textlint-rule-date-weekday-mismatch` 1.1.1(`chrono-node` 1.4.9と `moment` 2.31.0)・`textlint-rule-no-dead-link` 6.2.3から写し、textlintの不具合に近い挙動は写しません。上の点のうち、列の数え方と文面は3ルールにも当てはまります。ほかに次の点が違います。

- `no-todo` は、本文の範囲ごとの最初の1件だけでなく、すべての `todo:` を指摘します
- `no-todo` のタスクは、項目の原文の全体ではなく、項目の直下の段落とタスクの記号だけを見ます。入れ子の子の `[ ]` を親の項目でも指摘せず、インラインコード・コードブロック・HTMLの中の `[ ]` には当てません
- `no-todo` は、項目の先頭の未完了のタスクの記号を後ろに何があっても指摘し、段落の末尾の `[ ]` も指摘します。textlintは項目の原文に `\[\s+\]\s` を当てるので、`[ ]` の後ろに空白か改行が続くときだけ指摘し、本文の無い `- [ ]` と `- a [ ]` を、行末の空白の有無で指摘したりしなかったりします
- `no-todo` は、本文でない箇所(裸のURL、定義を持つ省略形・collapsed形の参照リンクの文字列)を見ません
- `date-weekday-mismatch` は、曜日の言語を、日付を読んだparserの言語ではなく括弧の中身で決めます。textlintは、`lang` を書かなければ `2024-04-23(月)` を、`lang: "ja"` を書けば `2024年4月23日(Mon)` を確かめません
- `date-weekday-mismatch` は、日本語と英語以外の6言語(ドイツ語・オランダ語・ポルトガル語・スペイン語・フランス語・中国語)の日付と曜日を確かめません
- `date-weekday-mismatch` は、`同年` を現在の年とみなさず、元号の無い `元年` を西暦1年とみなしません。`今年`・`本年` には `currentYear` の年を使います。textlintは、`currentYear` によらずシステムの年を使います
- `date-weekday-mismatch` は、年を補うとき、日付の文字列に年を足して解析し直さず、読んだ月と日に年を直に当てます。textlintは `April 23 (Mon)` の年が決まらず、確かめません。元号の無い `元年` の後の月と日(`元年5月1日`)は、年の無い日付として補いません
- `date-weekday-mismatch` は、範囲の日付(`4月23日〜24日`・`April 23 - 25, 2024`)と、`BE`・`AD`・`BC` の付く年の日付を確かめません。textlintは、範囲の最初の日と、`AD` の付く年の日付を確かめます
- `date-weekday-mismatch` の位置は、括弧の中の前後の空白を除いた曜日の先頭です。textlintは、括弧の中に前の空白があると、その空白に出します
- `date-weekday-mismatch` は、http(s)のURLの中の日付を確かめません。裸のURLと、文字列がURLのリンクは、本文でない箇所だからです。textlintは、`[https://example.com/2019/07/08(Su)/x](https://e.com)` の日付も確かめます
- `date-weekday-mismatch` は、月が先の数字の日付(`4/23/2024`)の前に文字があっても、曜日を確かめます。textlintは、`chrono-node` 1.4.9の切り出しの誤りのため、行頭でない `会議は4/23/2024(Mon)です。` の日付を確かめません
- `no-dead-link` は、裸のURLを対にならない `）` の手前で終えます。GFMの裸のURLは空白まで続くので、textlintは `https://example.com）を参照。` へ接続します
- `no-dead-link` の相対のリンクの位置は、元のソースのリンク先の文字列です。textlintは、解決した後の `file:///...` の長さで範囲を計り、宛先の文字列をリンクの原文から探します。そのため、textlintの位置はずれることがあります
- `no-dead-link` は、1回の要求を10秒で打ち切ります。textlintは上限を持ちません。`User-Agent` の既定値は `jafix-no-dead-link/<jafixのversion>` です
- `no-dead-link` は、引用の中の参照リンクの定義の宛先も確かめません。textlintは、引用の中でも定義の宛先だけは確かめます
- `no-dead-link` は、パスで書いた `baseURI` を、末尾の `/` の有無によらずディレクトリとして扱います。textlintは、末尾に `/` が無いと1つ上のディレクトリで解決します
- `no-dead-link` は、`httpOnly` が `true` で `baseURI` が無いとき、相対のリンクと絶対パスを確かめません。textlintは、実行したディレクトリから見たファイルとして確かめます
- `no-dead-link` の `dotInIgnore`・`interval`・`intervalCap`・`linkMaxAge` は使えません。`ignore` のglobは `exclude` のglobと同じ読み方で、`*` と `?` は `/` をまたがず、`.` で始まる要素にも `*` が当たります
- `no-dead-link` は、リンクの宛先だけを確かめます。textlintは、文中の `//host.tld/...` の文字列も拾い、ローカルのファイルとして確かめます
- 3つとも、textlintと違って自動で直さず、指摘だけを出します。曜日とリダイレクト先のURLは文面に入れます

英語の4ルールも、判定の条件を `textlint-rule-terminology` 5.2.16・`textlint-rule-spelling` 0.3.0・`textlint-rule-common-misspellings` 1.0.1・`textlint-rule-write-good` 2.0.0 (`write-good` 1.0.8)から写し、textlintの不具合に近い挙動は写しません。`spelling` と `common-misspellings` のルールのコードはGPLなので、コードは写さず、判定の条件から書き直しました。上の点のうち、列の数え方と文面は4ルールにも当てはまります。ほかに次の点が違います。

- 4つとも、本文でない箇所(裸のURL、自動リンクの文字列、定義を持つ省略形・collapsed形の参照リンクの文字列)を見ません。textlintの `common-misspellings` は、`<https://example.com/teh>` の `teh` も指摘します
- `terminology` の語の直後の条件に、`’` (U+2019)を除くASCIIでない文字(日本語の文字、全角の記号、アクセント付きのラテン文字)を足します。textlintは、直後が空白・ASCIIの句読点・文字列の末尾のときだけ当てるので、`javascriptを使う`・`javascript、` の `javascript` に当てません。`’` を除くのは、`id’s` に、textlintと同じく `id['’]?s` の項目の1件だけを出すためです
- `terminology` の組み込みの辞書に、`\bHTTP(?!:)`・`\bHTTPS(?!:)` の2項目を入れません。textlintでは、JSONCの `"\b"` がバックスペースになり、2項目は一度も当たりません
- `terminology` の組み込みの辞書の `co[- ]locat(ion|ing|e|ed)` の項目は、置き換え後の語を `colocat$1` にします。textlintは `colocate$1` なので、`co-located` に `colocateed` を勧めます
- `terminology` の `terms` の項目も、`prh` と同じく、一致の周りがすでに置き換え後の文字列になっている箇所を直しません。`["JS", "JavaScript (JS)"]` は、`JavaScript (JS)` の `JS` を直しません
- `terminology` の置き換え後の語を決められない一致(文字列の項目の `.` が別の文字に当たったもの)は、修正と指摘のどちらにもせずに続けます。textlintは例外を投げて止まります
- `terminology` の `terms` のファイルのパスは、設定ファイルのあるディレクトリから解決し、npm moduleの名前は読みません。読んだ値が項目の配列でなければ、設定の不正にします。textlintは、配列でない値を空として扱います。項目の正規表現には、textlintと違って `u` を付けます
- `spelling` は、既定の除外に当たる語を辞書に当てません。textlintは、`123`・`foo_bar`・`README`・`JSON` と、`mytoken\_value` の全体、`README.md` の `md`、`src/rules/foo.rs` の `src`、`owner/zuikaku` の `zuikaku` も指摘します。スラッシュでつないだ普通の語は除外せず、`Demo/visualizaion` の `visualizaion` も調べます
- `spelling` の判定と候補は、textlintのnspellではなく `spellbook` に従います。既定の `wordDefinitionRegexp` で取り出し、既定の除外に当たらない語の判定は同じです。`-` か末尾の `.` を語に含める `wordDefinitionRegexp` では、`spellbook` は、ハイフンで分けた語がどれも辞書にある語(`well-known`・`up-to-date`)と、末尾の `.` を除けば辞書にある語(`Mr.`)を正しい語とします。nspellは辞書に無い語とします
- `spelling` の候補は、`spellbook` の候補をその順で出すので、並びと数がtextlintと違います。語を2つに分けた候補(`text lint`)も出ます。textlintが `textlint` に出す候補 `texting` は出ません
- `spelling` は、リンクと画像の `title` を見ません
- `spelling` の `wordDefinitionRegexp` は、READMEの書き方の `/source/flags` で読み、書いたフラグのまま当てます。textlintは、`/` を含む文字列をそのまま正規表現の元の文字列にするので、READMEの書き方では語を取り出せません。また、`g` 以外のフラグを落とします
- `spelling` は、`wordDefinitionRegexp` の空の一致を語にしません。textlintは、`/[a-z]*/g` のような正規表現で空の語を指摘します
- `spelling` の指摘の範囲は語の全体で、位置は元のソースでの位置です。textlintは、語の先頭の1文字を範囲にし、エスケープを解いた文字列の中の位置を出します。続く行の字下げ・引用の `> `・行末の空白を除いた文字列で数えるので、textlintの位置はその分もずれます
- `common-misspellings` の一覧は、textlintが使う `misspellings` 1.1.0の4,273件ではなく、Wikipediaの一覧のrevision 1199637275から作った4,310件です。4,266件は正しい語まで同じです。違いは、Wikipediaの一覧にだけある43件、`misspellings` にだけある6件、正しい語の違う1件です
- `common-misspellings` は、正しい語が複数ある項目(`alot`)でも、すべての正しい語を文面に入れます
- `write-good` の `whitelist` は、本文の範囲の先頭の出現を含む、すべての出現を見ます。textlintは、文字列が本文の範囲の先頭にあると、後ろの出現もすべて見ません
- `write-good` の `so` と `thereIs` は、終わりの記号(`.` `;` `!` `?`)の無い最後の文も見ます。位置は、文頭の空白をすべて飛ばした語の先頭です。textlintは終わりの記号の無い文を見ず、文頭の空白が2つ以上あると位置が空白の上にずれます
- `write-good` の `so` と `thereIs` は、文字参照(`&amp;`)の終わりの `;` で文を区切りません。textlintは原文の `;` で区切るので、`Read x &amp; so it works.` の `so` を文頭の語として指摘します
- `write-good` の `weasel` の `too many`・`too few` の例外は、一致の直前の4文字を `too ` と比べます。textlintは、一致が本文の範囲の先頭から4文字未満にあると、本文の範囲の末尾の4文字と比べます
- `write-good` の指摘の範囲は、当たった文字列の全体です。textlintは先頭の1文字を範囲にします
- `write-good` の `eprime` は、`‘` `’` を `'` とみなして当て、位置は元のソースの位置です
- `write-good` の `checks` (判定の一覧の置き換え)は使えません
- `terminology` の組み込みの辞書の箇所、`spelling`、`common-misspellings` は、textlintと違って自動で直さず、指摘だけを出します

2つのfilterも、外す範囲の求め方を `textlint-filter-rule-comments` 1.3.0と `textlint-filter-rule-allowlist` 4.0.0から写しています。ほかに次の点が違います。

- `comments` の外す範囲は、コメントを含むHTMLのnodeの先頭ではなく、そのコメントの `<!--` から始まります。textlintでは、1つのHTMLのブロックに複数のコメントがあると、どの範囲もブロックの先頭から始まります
- presetの下のルールにも、jafixのルールのIDで当たります。textlintでは、presetの下のルールに `sentence-length` とだけ書いても外れません
- `allowlist` のパターンは、`@textlint-rule/pattern` と同じくJavaScriptの正規表現(`u` フラグ付き)で読みます。textlintの4.0.0は `@textlint/regexp-string-matcher` 1.1.1で読むので、`u` を付けず、`s` と `d` をフラグとみなしません。空の文字列は設定の不正で、`//` は文字列そのものです。textlintの4.0.0は、`//` を書くと止まらなくなります
- 外すかどうかは、jafixの指摘と修正の箇所の範囲で判定します。textlintでは多くのルールが1文字の幅で報告するので、同じ設定でも外れる指摘は変わりえます。たとえば `allow = ["TODO"]` は、`TODO:` の5文字を範囲とする `no-todo` の指摘を外しません

## ドキュメントサイト

ドキュメントは `docs/` にある日本語版を基準にします。英語に翻訳したページは日本語版と同じファイル名で `docs/en/` に置きます。
日本語版のトップページは [docs/index.md](docs/index.md)、ルール一覧は [docs/rules.md](docs/rules.md) です。

ローカルでページを確認するには、Node.js 24以降を使い、リポジトリのルートで次のコマンドを実行します。

```sh
npm ci
npm run docs:dev
```

`npm run docs:build` は、公開用のHTMLを `docs/.vitepress/dist/` に生成します。
生成したページは `npm run docs:preview` で確認できます。

`main` に `docs/`、`package.json`、`package-lock.json`、`.github/workflows/docs.yml` のいずれかの変更をpushすると、GitHub Actionsの **Deploy documentation** がサイトをビルドして公開します。
変更が無いときに公開し直すには、**Actions → Deploy documentation** で `main` を選んで **Run workflow** を実行します。

GitHub Pagesの設定は、リポジトリの **Settings → Pages → Build and deployment** で、**Source** に **GitHub Actions** を選んであります。

## リリース

まず `Cargo.toml` と `Cargo.lock` のバージョンをそろえたコミットを `main` にpushします。
配布物を事前に確認するには、GitHub Actionsの **release** を手動で実行します。4環境向けの配布物は、対応するジョブのartifactから取得できます。

タグを作る前に、利用者向けの変更点を `release-notes.md` に書きます。
`Cargo.toml` のバージョンの先頭に `v` を付けてタグを作り、pushするとリリースが始まります。
バージョン `0.0.2` をリリースする場合のコマンドは次のとおりです。

```sh
git tag -a v0.0.2 --cleanup=whitespace -F release-notes.md
git push origin v0.0.2
```

タグに取り込んだ変更点は、リリースノートの冒頭に表示されます。その後ろにGitHubが変更履歴を追加します。
自動生成する変更履歴の分類は [`.github/release.yml`](.github/release.yml) で設定します。

テストと梱包が通ると、Actionsが `.tar.gz`、Windows向けのZIP、`install.sh`、`checksums.txt` をGitHub Releasesに登録します。
`v0.2.0-rc.1` のようなタグはプレリリースになります。

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。

組み込みのルールのうち、textlintと同じIDのものはtextlintのルールとライブラリの挙動に合わせて書き直し、正規表現や語の表、辞書の一部をそれらのソースから写しています。`prh` と `@textlint-rule/pattern` は `textlint-rule-prh`・`prh`・`@textlint-rule/textlint-rule-pattern`・`@textlint/regexp-string-matcher` の判定の条件を写しています。

binaryには、MIT Licenseのlinderaと、mecab-ipadicの条件で配られるIPADICの辞書と、CC BY-SAの条件で配られるJTF日本語標準スタイルガイドの4つの辞書(`textlint-rule-preset-jtf-style` の `dict/*.yml`)も入っています。Apache License 2.0で配られる `sudachi-synonyms-dictionary` 20.0.0とSudachiDictの同義語辞書から、`@textlint-ja/no-synonyms` の判定に使う項目だけを取り出したデータも入っています。`no-dead-link` が外部のURLに接続するときは、CDLA-Permissive-2.0で配られる `webpki-roots` 1.0.12の証明書一覧でTLSの証明書を検証します。この一覧もbinaryに含まれます。

`common-misspellings` が使う誤綴りの一覧は、Wikipediaの「Wikipedia:Lists of common misspellings/For machines」のrevision 1199637275 (`https://en.wikipedia.org/w/index.php?title=Wikipedia:Lists_of_common_misspellings/For_machines&oldid=1199637275`)から作ったもので、CC BY-SA 3.0の条件で入っています。`spelling` が使うhunspellの辞書は、`dictionary-en` 4.0.0のen_USの辞書を変えずに入れたもので、SCOWLとIspellの条件(`(MIT AND BSD)`)で配られています。辞書を引く `spellbook` 0.4.2はMPL-2.0で配られています。`terminology` の辞書と `write-good` の単語リストはMIT Licenseのpackageから写しています。`spelling` と `common-misspellings` のtextlintのルールはGPLなので、コードを写さず、判定の条件から書き直しました。

それぞれの名前・version・著作権表示と、MIT License・mecab-ipadic・CC BY-SA・Apache License 2.0・SCOWLとIspellの条件は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。`webpki-roots` と `spellbook` はcrateなので、CDLA-Permissive-2.0とMPL-2.0の全文は次の段落の `CRATE_LICENSES.txt` にあります。`THIRD_PARTY_NOTICES.md` には、その2つのcrateを含める理由と、MPL-2.0が求める `spellbook` のソースの入手先を記載しています。

GitHub Releasesの配布物には `CRATE_LICENSES.txt` が入っています。
バイナリにリンクするすべてのRustのcrateについて、名前、バージョン、ライセンスの種類と文面を記載したファイルです。
`scripts/package.sh` は梱包時に [cargo-about](https://github.com/EmbarkStudios/cargo-about) を使い、`Cargo.lock` と梱包対象の依存関係からこのファイルを生成します。
`CRATE_LICENSES.txt` はリポジトリには置いていません。
配布物にはバイナリとともに `LICENSE-MIT`、`LICENSE-APACHE`、`THIRD_PARTY_NOTICES.md`、`CRATE_LICENSES.txt` を入れています。

依存関係を変えたときは、`cargo about generate --frozen --fail about.hbs -o /dev/null` で、`about.toml` の `accepted` にないライセンスのcrateが入っていないかを確かめられます。
MPL-2.0とCDLA-Permissive-2.0を許可するのは、`THIRD_PARTY_NOTICES.md` に入手先や説明を書いたcrateだけです。
これらのライセンスのcrateが新しく入ると、このコマンドは失敗します。

コマンドが成功してもWARNが出る場合は、`about.toml` のclarifyのchecksumがcrateのファイルと一致していません。
`scripts/package.sh` はWARNが出ると、一覧を作らずに止まります。
