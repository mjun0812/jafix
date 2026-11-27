---
outline: 2
---

<!-- このファイルは scripts/rule-examples.mjs が docs/rule-examples/examples.json から生成する。直接編集しない -->

# ルールの例

このページは、ルールが文章をどう変えるかを、ルールごとの画像で示します。画像は、例文を実際の `jafix` に渡した結果から作っています。ルールの種類と初期状態は[ルール一覧](./rules.md)で確認できます。

## 画像の見方 {#legend}

画像は、上から順に、ルールIDと種類、例文、指摘の文面を示します。例文には、そのルールだけを有効にした設定を使っています。オプションを変えた例には、その設定を添えています。

- 種類が「修正」の画像は、修正前と修正後の2行です。赤い背景は修正で消える文字、緑の背景は修正で入る文字です
- 種類が「指摘」の画像は、例文だけです。赤い背景は、指摘の対象の語句です
- 底の開いた箱の形は、空白です。空白は見えないので、形に置き換えています
- `U+200B` のような枠は、表示されない文字です。枠の中は、その文字のコードポイントです
- `↵` は改行です
- 下の段は、`jafix check` が出す文面と、指摘の数です

## jafix独自の修正 {#jafix}

### `fullwidth-paren` {#fullwidth-paren}

![fullwidth-paren の例](./rule-examples/fullwidth-paren.svg)

### `space-between-ja-and-alnum` {#space-between-ja-and-alnum}

英単語の間の空白は残します。

![space-between-ja-and-alnum の例](./rule-examples/space-between-ja-and-alnum.svg)

### `space-between-ja-and-paren` {#space-between-ja-and-paren}

![space-between-ja-and-paren の例](./rule-examples/space-between-ja-and-paren.svg)

## 日本語の技術文書 {#technical-writing}

### `sentence-length` {#sentence-length}

![sentence-length の例](./rule-examples/sentence-length.svg)

この例の設定は、次のとおりです。

```toml
[builtin]
sentence-length = { max = 30 }
```

### `max-comma` {#max-comma}

![max-comma の例](./rule-examples/max-comma.svg)

### `max-kanji-continuous-len` {#max-kanji-continuous-len}

![max-kanji-continuous-len の例](./rule-examples/max-kanji-continuous-len.svg)

### `arabic-kanji-numbers` {#arabic-kanji-numbers}

![arabic-kanji-numbers の例](./rule-examples/arabic-kanji-numbers.svg)

### `ja-no-mixed-period` {#ja-no-mixed-period}

箇条書きの項目は、句点がなくても指摘しません。

![ja-no-mixed-period の例](./rule-examples/ja-no-mixed-period.svg)

### `no-nfd` {#no-nfd}

![no-nfd の例](./rule-examples/no-nfd.svg)

### `no-invalid-control-character` {#no-invalid-control-character}

![no-invalid-control-character の例](./rule-examples/no-invalid-control-character.svg)

### `no-zero-width-spaces` {#no-zero-width-spaces}

![no-zero-width-spaces の例](./rule-examples/no-zero-width-spaces.svg)

### `no-exclamation-question-mark` {#no-exclamation-question-mark}

URLの中の疑問符は指摘しません。

![no-exclamation-question-mark の例](./rule-examples/no-exclamation-question-mark.svg)

### `no-hankaku-kana` {#no-hankaku-kana}

![no-hankaku-kana の例](./rule-examples/no-hankaku-kana.svg)

### `ja-unnatural-alphabet` {#ja-unnatural-alphabet}

![ja-unnatural-alphabet の例](./rule-examples/ja-unnatural-alphabet.svg)

### `no-unmatched-pair` {#no-unmatched-pair}

![no-unmatched-pair の例](./rule-examples/no-unmatched-pair.svg)

### `max-ten` {#max-ten}

名詞を並べる読点は数えません。

![max-ten の例](./rule-examples/max-ten.svg)

### `no-doubled-joshi` {#no-doubled-joshi}

![no-doubled-joshi の例](./rule-examples/no-doubled-joshi.svg)

### `no-doubled-conjunctive-particle-ga` {#no-doubled-conjunctive-particle-ga}

![no-doubled-conjunctive-particle-ga の例](./rule-examples/no-doubled-conjunctive-particle-ga.svg)

### `no-doubled-conjunction` {#no-doubled-conjunction}

![no-doubled-conjunction の例](./rule-examples/no-doubled-conjunction.svg)

### `no-mix-dearu-desumasu` {#no-mix-dearu-desumasu}

![no-mix-dearu-desumasu の例](./rule-examples/no-mix-dearu-desumasu.svg)

### `no-double-negative-ja` {#no-double-negative-ja}

![no-double-negative-ja の例](./rule-examples/no-double-negative-ja.svg)

### `no-dropping-the-ra` {#no-dropping-the-ra}

![no-dropping-the-ra の例](./rule-examples/no-dropping-the-ra.svg)

### `ja-no-weak-phrase` {#ja-no-weak-phrase}

![ja-no-weak-phrase の例](./rule-examples/ja-no-weak-phrase.svg)

### `ja-no-successive-word` {#ja-no-successive-word}

![ja-no-successive-word の例](./rule-examples/ja-no-successive-word.svg)

### `ja-no-abusage` {#ja-no-abusage}

![ja-no-abusage の例](./rule-examples/ja-no-abusage.svg)

### `ja-no-redundant-expression` {#ja-no-redundant-expression}

![ja-no-redundant-expression の例](./rule-examples/ja-no-redundant-expression.svg)

## AIが多用する書き方 {#ai-writing}

### `no-ai-list-formatting` {#no-ai-list-formatting}

太字だけで、コロンの付かない項目は指摘しません。

![no-ai-list-formatting の例](./rule-examples/no-ai-list-formatting.svg)

### `no-ai-hype-expressions` {#no-ai-hype-expressions}

![no-ai-hype-expressions の例](./rule-examples/no-ai-hype-expressions.svg)

### `no-ai-emphasis-patterns` {#no-ai-emphasis-patterns}

絵文字の付かない太字は指摘しません。

![no-ai-emphasis-patterns の例](./rule-examples/no-ai-emphasis-patterns.svg)

### `no-ai-colon-continuation` {#no-ai-colon-continuation}

![no-ai-colon-continuation の例](./rule-examples/no-ai-colon-continuation.svg)

### `ai-tech-writing-guideline` {#ai-tech-writing-guideline}

![ai-tech-writing-guideline の例](./rule-examples/ai-tech-writing-guideline.svg)

### `no-ai-words` {#no-ai-words}

![no-ai-words の例](./rule-examples/no-ai-words.svg)

### `no-short-topic-comma` {#no-short-topic-comma}

主題が6文字以上のときは指摘しません。

![no-short-topic-comma の例](./rule-examples/no-short-topic-comma.svg)

## yomiyasu由来のルール {#yomiyasu}

### `yomiyasu-bold-not-rendered` {#yomiyasu-bold-not-rendered}

![yomiyasu-bold-not-rendered の例](./rule-examples/yomiyasu-bold-not-rendered.svg)

### `yomiyasu-instruction-residue` {#yomiyasu-instruction-residue}

参照先のリンクが続く除外は指摘しません。

![yomiyasu-instruction-residue の例](./rule-examples/yomiyasu-instruction-residue.svg)

### `yomiyasu-chat-response` {#yomiyasu-chat-response}

![yomiyasu-chat-response の例](./rule-examples/yomiyasu-chat-response.svg)

### `yomiyasu-meta-filler` {#yomiyasu-meta-filler}

![yomiyasu-meta-filler の例](./rule-examples/yomiyasu-meta-filler.svg)

### `yomiyasu-summary-restatement` {#yomiyasu-summary-restatement}

![yomiyasu-summary-restatement の例](./rule-examples/yomiyasu-summary-restatement.svg)

### `yomiyasu-meta-intro` {#yomiyasu-meta-intro}

![yomiyasu-meta-intro の例](./rule-examples/yomiyasu-meta-intro.svg)

### `yomiyasu-count-declaration` {#yomiyasu-count-declaration}

![yomiyasu-count-declaration の例](./rule-examples/yomiyasu-count-declaration.svg)

### `yomiyasu-ai-phrase` {#yomiyasu-ai-phrase}

![yomiyasu-ai-phrase の例](./rule-examples/yomiyasu-ai-phrase.svg)

### `yomiyasu-placeholder-residue` {#yomiyasu-placeholder-residue}

![yomiyasu-placeholder-residue の例](./rule-examples/yomiyasu-placeholder-residue.svg)

### `yomiyasu-short-mochiron` {#yomiyasu-short-mochiron}

![yomiyasu-short-mochiron の例](./rule-examples/yomiyasu-short-mochiron.svg)

### `yomiyasu-say-hedge` {#yomiyasu-say-hedge}

![yomiyasu-say-hedge の例](./rule-examples/yomiyasu-say-hedge.svg)

### `yomiyasu-fragment-run` {#yomiyasu-fragment-run}

![yomiyasu-fragment-run の例](./rule-examples/yomiyasu-fragment-run.svg)

### `yomiyasu-cleft-list-ending` {#yomiyasu-cleft-list-ending}

![yomiyasu-cleft-list-ending の例](./rule-examples/yomiyasu-cleft-list-ending.svg)

### `yomiyasu-copy-comma-noun` {#yomiyasu-copy-comma-noun}

![yomiyasu-copy-comma-noun の例](./rule-examples/yomiyasu-copy-comma-noun.svg)

### `yomiyasu-bare-verb-chain` {#yomiyasu-bare-verb-chain}

![yomiyasu-bare-verb-chain の例](./rule-examples/yomiyasu-bare-verb-chain.svg)

### `yomiyasu-dash-decoration` {#yomiyasu-dash-decoration}

![yomiyasu-dash-decoration の例](./rule-examples/yomiyasu-dash-decoration.svg)

### `yomiyasu-dash-insertion` {#yomiyasu-dash-insertion}

![yomiyasu-dash-insertion の例](./rule-examples/yomiyasu-dash-insertion.svg)

### `yomiyasu-dash-list-ending` {#yomiyasu-dash-list-ending}

![yomiyasu-dash-list-ending の例](./rule-examples/yomiyasu-dash-list-ending.svg)

### `yomiyasu-emoji-prohibited` {#yomiyasu-emoji-prohibited}

![yomiyasu-emoji-prohibited の例](./rule-examples/yomiyasu-emoji-prohibited.svg)

### `yomiyasu-renyo-noun` {#yomiyasu-renyo-noun}

![yomiyasu-renyo-noun の例](./rule-examples/yomiyasu-renyo-noun.svg)

### `yomiyasu-slop-vocabulary` {#yomiyasu-slop-vocabulary}

![yomiyasu-slop-vocabulary の例](./rule-examples/yomiyasu-slop-vocabulary.svg)

### `yomiyasu-katakana-term` {#yomiyasu-katakana-term}

![yomiyasu-katakana-term の例](./rule-examples/yomiyasu-katakana-term.svg)

## 空白と字種 {#spacing}

### `ja-nakaguro-or-halfwidth-space-between-katakana` {#ja-nakaguro-or-halfwidth-space-between-katakana}

![ja-nakaguro-or-halfwidth-space-between-katakana の例](./rule-examples/ja-nakaguro-or-halfwidth-space-between-katakana.svg)

### `ja-no-space-around-parentheses` {#ja-no-space-around-parentheses}

半角の丸括弧は対象にしません。

![ja-no-space-around-parentheses の例](./rule-examples/ja-no-space-around-parentheses.svg)

### `ja-no-space-around-slash` {#ja-no-space-around-slash}

![ja-no-space-around-slash の例](./rule-examples/ja-no-space-around-slash.svg)

### `ja-no-space-between-full-width` {#ja-no-space-between-full-width}

![ja-no-space-between-full-width の例](./rule-examples/ja-no-space-between-full-width.svg)

### `ja-space-between-half-and-full-width` {#ja-space-between-half-and-full-width}

英単語の間の空白は残します。

![ja-space-between-half-and-full-width の例](./rule-examples/ja-space-between-half-and-full-width.svg)

### `ja-space-after-exclamation` {#ja-space-after-exclamation}

![ja-space-after-exclamation の例](./rule-examples/ja-space-after-exclamation.svg)

### `ja-space-after-question` {#ja-space-after-question}

![ja-space-after-question の例](./rule-examples/ja-space-after-question.svg)

### `ja-space-around-code` {#ja-space-around-code}

![ja-space-around-code の例](./rule-examples/ja-space-around-code.svg)

### `ja-space-around-emphasis` {#ja-space-around-emphasis}

![ja-space-around-emphasis の例](./rule-examples/ja-space-around-emphasis.svg)

### `ja-space-around-link` {#ja-space-around-link}

![ja-space-around-link の例](./rule-examples/ja-space-around-link.svg)

### `ja-space-around-strong` {#ja-space-around-strong}

![ja-space-around-strong の例](./rule-examples/ja-space-around-strong.svg)

### `no-mixed-zenkaku-and-hankaku-alphabet` {#no-mixed-zenkaku-and-hankaku-alphabet}

![no-mixed-zenkaku-and-hankaku-alphabet の例](./rule-examples/no-mixed-zenkaku-and-hankaku-alphabet.svg)

### `no-kangxi-radicals` {#no-kangxi-radicals}

見た目は同じでも、文字のコードは別です。

![no-kangxi-radicals の例](./rule-examples/no-kangxi-radicals.svg)

## 日本語標準スタイルガイド: 文体と句読点 {#jtf-prose}

### `1.1.1.本文` {#1.1.1}

少ないほうの文体を指摘します。

![1.1.1.本文 の例](./rule-examples/1.1.1.svg)

### `1.1.2.見出し` {#1.1.2}

![1.1.2.見出し の例](./rule-examples/1.1.2.svg)

### `1.1.3.箇条書き` {#1.1.3}

![1.1.3.箇条書き の例](./rule-examples/1.1.3.svg)

### `1.1.5.図表のキャプション` {#1.1.5}

![1.1.5.図表のキャプション の例](./rule-examples/1.1.5.svg)

### `1.2.1.句点(。)と読点(、)` {#1.2.1}

![1.2.1.句点(。)と読点(、) の例](./rule-examples/1.2.1.svg)

### `1.2.2.ピリオド(.)とカンマ(,)` {#1.2.2}

![1.2.2.ピリオド(.)とカンマ(,) の例](./rule-examples/1.2.2.svg)

## 日本語標準スタイルガイド: 文字と用語の表記 {#jtf-characters}

### `2.1.2.漢字` {#2.1.2}

![2.1.2.漢字 の例](./rule-examples/2.1.2.svg)

### `2.1.5.カタカナ` {#2.1.5}

![2.1.5.カタカナ の例](./rule-examples/2.1.5.svg)

### `2.1.6.カタカナの長音` {#2.1.6}

![2.1.6.カタカナの長音 の例](./rule-examples/2.1.6.svg)

### `2.1.8.算用数字` {#2.1.8}

![2.1.8.算用数字 の例](./rule-examples/2.1.8.svg)

### `2.1.9.アルファベット` {#2.1.9}

![2.1.9.アルファベット の例](./rule-examples/2.1.9.svg)

### `2.1.10.算用数字の位取りの表記` {#2.1.10}

千の位の区切りのカンマは指摘しません。

![2.1.10.算用数字の位取りの表記 の例](./rule-examples/2.1.10.svg)

### `2.2.1.ひらがなと漢字の使い分け` {#2.2.1}

![2.2.1.ひらがなと漢字の使い分け の例](./rule-examples/2.2.1.svg)

### `2.2.2.算用数字と漢数字の使い分け` {#2.2.2}

![2.2.2.算用数字と漢数字の使い分け の例](./rule-examples/2.2.2.svg)

### `2.2.3.一部の助数詞の表記` {#2.2.3}

![2.2.3.一部の助数詞の表記 の例](./rule-examples/2.2.3.svg)

## 日本語標準スタイルガイド: 空白 {#jtf-spacing}

### `3.1.1.全角文字と半角文字の間` {#3.1.1}

英単語の間の空白は残します。

![3.1.1.全角文字と半角文字の間 の例](./rule-examples/3.1.1.svg)

### `3.1.2.全角文字どうし` {#3.1.2}

![3.1.2.全角文字どうし の例](./rule-examples/3.1.2.svg)

### `3.2.カタカナ語間のスペースの有無` {#3.2}

![3.2.カタカナ語間のスペースの有無 の例](./rule-examples/3.2.svg)

### `3.3.かっこ類と隣接する文字の間のスペースの有無` {#3.3}

設定を変えなければ、半角の丸括弧の外側の空白は残します。

![3.3.かっこ類と隣接する文字の間のスペースの有無 の例](./rule-examples/3.3.svg)

## 日本語標準スタイルガイド: 記号と括弧 {#jtf-punctuation}

### `4.1.1.句点(。)` {#4.1.1}

括弧の外の句点は指摘しません。

![4.1.1.句点(。) の例](./rule-examples/4.1.1.svg)

### `4.1.3.ピリオド(.)、カンマ(,)` {#4.1.3}

![4.1.3.ピリオド(.)、カンマ(,) の例](./rule-examples/4.1.3.svg)

### `4.2.1.感嘆符(！)` {#4.2.1}

英語の直後の感嘆符は指摘しません。

![4.2.1.感嘆符(！) の例](./rule-examples/4.2.1.svg)

### `4.2.2.疑問符(？)` {#4.2.2}

英語の直後の疑問符は指摘しません。

![4.2.2.疑問符(？) の例](./rule-examples/4.2.2.svg)

### `4.2.4.中黒(・)` {#4.2.4}

![4.2.4.中黒(・) の例](./rule-examples/4.2.4.svg)

### `4.2.5.波線(〜)` {#4.2.5}

![4.2.5.波線(〜) の例](./rule-examples/4.2.5.svg)

### `4.2.6.ハイフン(-)` {#4.2.6}

英数字どうしをつなぐハイフンは指摘しません。

![4.2.6.ハイフン(-) の例](./rule-examples/4.2.6.svg)

### `4.2.7.コロン(：)` {#4.2.7}

URLの中のコロンは指摘しません。

![4.2.7.コロン(：) の例](./rule-examples/4.2.7.svg)

### `4.2.8.セミコロン(;)` {#4.2.8}

![4.2.8.セミコロン(;) の例](./rule-examples/4.2.8.svg)

### `4.2.9.ダッシュ(-)` {#4.2.9}

![4.2.9.ダッシュ(-) の例](./rule-examples/4.2.9.svg)

### `4.3.1.丸かっこ（）` {#4.3.1}

![4.3.1.丸かっこ（） の例](./rule-examples/4.3.1.svg)

### `4.3.2.大かっこ［］` {#4.3.2}

![4.3.2.大かっこ［］ の例](./rule-examples/4.3.2.svg)

### `4.3.3.かぎかっこ「」` {#4.3.3}

対になったかぎ括弧は指摘せず、閉じていないかぎ括弧だけを指摘します。

![4.3.3.かぎかっこ「」 の例](./rule-examples/4.3.3.svg)

### `4.3.4.二重かぎかっこ『』` {#4.3.4}

対になった二重かぎ括弧は指摘せず、閉じていない二重かぎ括弧だけを指摘します。

![4.3.4.二重かぎかっこ『』 の例](./rule-examples/4.3.4.svg)

### `4.3.5.二重引用符` {#4.3.5}

引用符の数が奇数のときだけ、対にならない1つを指摘します。

![4.3.5.二重引用符 の例](./rule-examples/4.3.5.svg)

### `4.3.6.中かっこ{ }` {#4.3.6}

対になった波括弧は指摘せず、閉じていない波括弧だけを指摘します。

![4.3.6.中かっこ{ } の例](./rule-examples/4.3.6.svg)

### `4.3.7.山かっこ<>` {#4.3.7}

矢印や比較の記号は、山かっことして数えません。

![4.3.7.山かっこ<> の例](./rule-examples/4.3.7.svg)

## ひらがな、さ入れ言葉、同義語 {#japanese-wording}

### `ja-hiragana-hojodoushi` {#ja-hiragana-hojodoushi}

補助動詞ではない「頂きました」は指摘しません。

![ja-hiragana-hojodoushi の例](./rule-examples/ja-hiragana-hojodoushi.svg)

### `ja-hiragana-keishikimeishi` {#ja-hiragana-keishikimeishi}

![ja-hiragana-keishikimeishi の例](./rule-examples/ja-hiragana-keishikimeishi.svg)

### `ja-keishikimeishi` {#ja-keishikimeishi}

「うえ」の指摘は設定で外しているので、「上」は指摘しません。

![ja-keishikimeishi の例](./rule-examples/ja-keishikimeishi.svg)

この例の設定は、次のとおりです。

```toml
[builtin]
ja-keishikimeishi = { detection_ue = false }
```

### `ja-hiragana-fukushi` {#ja-hiragana-fukushi}

![ja-hiragana-fukushi の例](./rule-examples/ja-hiragana-fukushi.svg)

### `ja-hiragana-daimeishi` {#ja-hiragana-daimeishi}

![ja-hiragana-daimeishi の例](./rule-examples/ja-hiragana-daimeishi.svg)

### `@textlint-ja/no-insert-dropping-sa` {#textlint-ja-no-insert-dropping-sa}

![@textlint-ja/no-insert-dropping-sa の例](./rule-examples/textlint-ja-no-insert-dropping-sa.svg)

### `@textlint-ja/no-synonyms` {#textlint-ja-no-synonyms}

![@textlint-ja/no-synonyms の例](./rule-examples/textlint-ja-no-synonyms.svg)

## 言語を問わない確認 {#general}

### `no-todo` {#no-todo}

完了したタスクは指摘しません。

![no-todo の例](./rule-examples/no-todo.svg)

### `date-weekday-mismatch` {#date-weekday-mismatch}

曜日が合う日付は指摘しません。

![date-weekday-mismatch の例](./rule-examples/date-weekday-mismatch.svg)

## 英語の綴りと表現 {#english}

### `terminology` {#terminology}

組み込みの辞書だけに当たる語は、直さずに指摘だけを出します。

![terminology の例](./rule-examples/terminology.svg)

この例の設定は、次のとおりです。

```toml
[builtin]
terminology = { terms = [['front[- ]end(\w*)', 'frontend$1']] }
```

### `spelling` {#spelling}

![spelling の例](./rule-examples/spelling.svg)

### `common-misspellings` {#common-misspellings}

![common-misspellings の例](./rule-examples/common-misspellings.svg)

### `write-good` {#write-good}

![write-good の例](./rule-examples/write-good.svg)

## 利用者の置き換えとfilter {#custom-rules-and-filters}

### `prh` {#prh}

![prh の例](./rule-examples/prh.svg)

この例の設定は、次のとおりです。

```toml
[builtin]
prh = { ruleContents = ['''
rules:
  - expected: 利用者
    pattern: /ユーザー?/
'''] }
```

### `@textlint-rule/pattern` {#textlint-rule-pattern}

年の無い日付は、パターンに当たらないので直しません。

![@textlint-rule/pattern の例](./rule-examples/textlint-rule-pattern.svg)

この例の設定は、次のとおりです。

```toml
[builtin]
"@textlint-rule/pattern" = { patterns = [{ message = "日付はYYYY-MM-DDで書く", pattern = '/(\d{4})/(\d{2})/(\d{2})/', replace = "$1-$2-$3" }] }
```

## 例の無いルール {#skipped}

次のルールは、1つの例文では示せないので、画像がありません。

- `4.3.8.一重引用符`: presetのキーを設定に書けるようにIDだけを持つルールで、どの例文にも指摘を出しません。
- `no-dead-link`: リンク先を調べるルールは標準入力の本文には適用されず、例文は標準入力から渡すので、指摘を出せません。
