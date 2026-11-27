---
outline: 2
---

<!-- このファイルは scripts/rule-examples.mjs が docs/rule-examples/examples.json から生成する。直接編集しない -->

# Rule examples

This page shows how each rule changes text, with one image per rule. Every image is generated from the output of the real `jafix` binary for the example text. See [Rules](./rules.md) for each rule's kind and default state.

## How to read the images {#legend}

From top to bottom, an image shows the rule ID and kind, the example text, and what the rule does. Each example runs with only that rule enabled. When an example changes an option, the configuration follows the image.

- A `Fix` image has two rows, `Before` and `After`. A red background marks characters the fix removes, and a green background marks characters it inserts
- A `Report` image has the example text only. A red background marks the reported words
- An open box stands for a space, which would otherwise be invisible
- A frame such as `U+200B` stands for a character that is not displayed. It shows the code point of the character
- `↵` is a line break
- The bottom line describes the rule and gives the number of findings. `jafix check` itself prints its messages in Japanese

## jafix-specific fixes {#jafix}

### `fullwidth-paren` {#fullwidth-paren}

![Example of fullwidth-paren](./rule-examples/fullwidth-paren.svg)

### `space-between-ja-and-alnum` {#space-between-ja-and-alnum}

Spaces between English words stay.

![Example of space-between-ja-and-alnum](./rule-examples/space-between-ja-and-alnum.svg)

### `space-between-ja-and-paren` {#space-between-ja-and-paren}

![Example of space-between-ja-and-paren](./rule-examples/space-between-ja-and-paren.svg)

## Japanese technical writing {#technical-writing}

### `sentence-length` {#sentence-length}

![Example of sentence-length](./rule-examples/sentence-length.svg)

This example uses the following configuration.

```toml
[builtin]
sentence-length = { max = 30 }
```

### `max-comma` {#max-comma}

![Example of max-comma](./rule-examples/max-comma.svg)

### `max-kanji-continuous-len` {#max-kanji-continuous-len}

![Example of max-kanji-continuous-len](./rule-examples/max-kanji-continuous-len.svg)

### `arabic-kanji-numbers` {#arabic-kanji-numbers}

![Example of arabic-kanji-numbers](./rule-examples/arabic-kanji-numbers.svg)

### `ja-no-mixed-period` {#ja-no-mixed-period}

List items are not reported even without a period.

![Example of ja-no-mixed-period](./rule-examples/ja-no-mixed-period.svg)

### `no-nfd` {#no-nfd}

![Example of no-nfd](./rule-examples/no-nfd.svg)

### `no-invalid-control-character` {#no-invalid-control-character}

![Example of no-invalid-control-character](./rule-examples/no-invalid-control-character.svg)

### `no-zero-width-spaces` {#no-zero-width-spaces}

![Example of no-zero-width-spaces](./rule-examples/no-zero-width-spaces.svg)

### `no-exclamation-question-mark` {#no-exclamation-question-mark}

A question mark inside a URL is not reported.

![Example of no-exclamation-question-mark](./rule-examples/no-exclamation-question-mark.svg)

### `no-hankaku-kana` {#no-hankaku-kana}

![Example of no-hankaku-kana](./rule-examples/no-hankaku-kana.svg)

### `ja-unnatural-alphabet` {#ja-unnatural-alphabet}

![Example of ja-unnatural-alphabet](./rule-examples/ja-unnatural-alphabet.svg)

### `no-unmatched-pair` {#no-unmatched-pair}

![Example of no-unmatched-pair](./rule-examples/no-unmatched-pair.svg)

### `max-ten` {#max-ten}

Commas that separate a list of nouns are not counted.

![Example of max-ten](./rule-examples/max-ten.svg)

### `no-doubled-joshi` {#no-doubled-joshi}

![Example of no-doubled-joshi](./rule-examples/no-doubled-joshi.svg)

### `no-doubled-conjunctive-particle-ga` {#no-doubled-conjunctive-particle-ga}

![Example of no-doubled-conjunctive-particle-ga](./rule-examples/no-doubled-conjunctive-particle-ga.svg)

### `no-doubled-conjunction` {#no-doubled-conjunction}

![Example of no-doubled-conjunction](./rule-examples/no-doubled-conjunction.svg)

### `no-mix-dearu-desumasu` {#no-mix-dearu-desumasu}

![Example of no-mix-dearu-desumasu](./rule-examples/no-mix-dearu-desumasu.svg)

### `no-double-negative-ja` {#no-double-negative-ja}

![Example of no-double-negative-ja](./rule-examples/no-double-negative-ja.svg)

### `no-dropping-the-ra` {#no-dropping-the-ra}

![Example of no-dropping-the-ra](./rule-examples/no-dropping-the-ra.svg)

### `ja-no-weak-phrase` {#ja-no-weak-phrase}

![Example of ja-no-weak-phrase](./rule-examples/ja-no-weak-phrase.svg)

### `ja-no-successive-word` {#ja-no-successive-word}

![Example of ja-no-successive-word](./rule-examples/ja-no-successive-word.svg)

### `ja-no-abusage` {#ja-no-abusage}

![Example of ja-no-abusage](./rule-examples/ja-no-abusage.svg)

### `ja-no-redundant-expression` {#ja-no-redundant-expression}

![Example of ja-no-redundant-expression](./rule-examples/ja-no-redundant-expression.svg)

## Patterns common in AI writing {#ai-writing}

### `no-ai-list-formatting` {#no-ai-list-formatting}

A bold item with no colon after it is not reported.

![Example of no-ai-list-formatting](./rule-examples/no-ai-list-formatting.svg)

### `no-ai-hype-expressions` {#no-ai-hype-expressions}

![Example of no-ai-hype-expressions](./rule-examples/no-ai-hype-expressions.svg)

### `no-ai-emphasis-patterns` {#no-ai-emphasis-patterns}

Bold text without an emoji is not reported.

![Example of no-ai-emphasis-patterns](./rule-examples/no-ai-emphasis-patterns.svg)

### `no-ai-colon-continuation` {#no-ai-colon-continuation}

![Example of no-ai-colon-continuation](./rule-examples/no-ai-colon-continuation.svg)

### `ai-tech-writing-guideline` {#ai-tech-writing-guideline}

![Example of ai-tech-writing-guideline](./rule-examples/ai-tech-writing-guideline.svg)

### `no-ai-words` {#no-ai-words}

![Example of no-ai-words](./rule-examples/no-ai-words.svg)

### `no-short-topic-comma` {#no-short-topic-comma}

A topic of six characters or more is not reported.

![Example of no-short-topic-comma](./rule-examples/no-short-topic-comma.svg)

## Rules adapted from yomiyasu {#yomiyasu}

### `yomiyasu-bold-not-rendered` {#yomiyasu-bold-not-rendered}

![Example of yomiyasu-bold-not-rendered](./rule-examples/yomiyasu-bold-not-rendered.svg)

### `yomiyasu-instruction-residue` {#yomiyasu-instruction-residue}

An exclusion followed by a link to where the topic is covered is not reported.

![Example of yomiyasu-instruction-residue](./rule-examples/yomiyasu-instruction-residue.svg)

### `yomiyasu-chat-response` {#yomiyasu-chat-response}

![Example of yomiyasu-chat-response](./rule-examples/yomiyasu-chat-response.svg)

### `yomiyasu-meta-filler` {#yomiyasu-meta-filler}

![Example of yomiyasu-meta-filler](./rule-examples/yomiyasu-meta-filler.svg)

### `yomiyasu-summary-restatement` {#yomiyasu-summary-restatement}

![Example of yomiyasu-summary-restatement](./rule-examples/yomiyasu-summary-restatement.svg)

### `yomiyasu-meta-intro` {#yomiyasu-meta-intro}

![Example of yomiyasu-meta-intro](./rule-examples/yomiyasu-meta-intro.svg)

### `yomiyasu-count-declaration` {#yomiyasu-count-declaration}

![Example of yomiyasu-count-declaration](./rule-examples/yomiyasu-count-declaration.svg)

### `yomiyasu-ai-phrase` {#yomiyasu-ai-phrase}

![Example of yomiyasu-ai-phrase](./rule-examples/yomiyasu-ai-phrase.svg)

### `yomiyasu-placeholder-residue` {#yomiyasu-placeholder-residue}

![Example of yomiyasu-placeholder-residue](./rule-examples/yomiyasu-placeholder-residue.svg)

### `yomiyasu-short-mochiron` {#yomiyasu-short-mochiron}

![Example of yomiyasu-short-mochiron](./rule-examples/yomiyasu-short-mochiron.svg)

### `yomiyasu-say-hedge` {#yomiyasu-say-hedge}

![Example of yomiyasu-say-hedge](./rule-examples/yomiyasu-say-hedge.svg)

### `yomiyasu-fragment-run` {#yomiyasu-fragment-run}

![Example of yomiyasu-fragment-run](./rule-examples/yomiyasu-fragment-run.svg)

### `yomiyasu-cleft-list-ending` {#yomiyasu-cleft-list-ending}

![Example of yomiyasu-cleft-list-ending](./rule-examples/yomiyasu-cleft-list-ending.svg)

### `yomiyasu-copy-comma-noun` {#yomiyasu-copy-comma-noun}

![Example of yomiyasu-copy-comma-noun](./rule-examples/yomiyasu-copy-comma-noun.svg)

### `yomiyasu-bare-verb-chain` {#yomiyasu-bare-verb-chain}

![Example of yomiyasu-bare-verb-chain](./rule-examples/yomiyasu-bare-verb-chain.svg)

### `yomiyasu-dash-decoration` {#yomiyasu-dash-decoration}

![Example of yomiyasu-dash-decoration](./rule-examples/yomiyasu-dash-decoration.svg)

### `yomiyasu-dash-insertion` {#yomiyasu-dash-insertion}

![Example of yomiyasu-dash-insertion](./rule-examples/yomiyasu-dash-insertion.svg)

### `yomiyasu-dash-list-ending` {#yomiyasu-dash-list-ending}

![Example of yomiyasu-dash-list-ending](./rule-examples/yomiyasu-dash-list-ending.svg)

### `yomiyasu-emoji-prohibited` {#yomiyasu-emoji-prohibited}

![Example of yomiyasu-emoji-prohibited](./rule-examples/yomiyasu-emoji-prohibited.svg)

### `yomiyasu-renyo-noun` {#yomiyasu-renyo-noun}

![Example of yomiyasu-renyo-noun](./rule-examples/yomiyasu-renyo-noun.svg)

### `yomiyasu-slop-vocabulary` {#yomiyasu-slop-vocabulary}

![Example of yomiyasu-slop-vocabulary](./rule-examples/yomiyasu-slop-vocabulary.svg)

### `yomiyasu-katakana-term` {#yomiyasu-katakana-term}

![Example of yomiyasu-katakana-term](./rule-examples/yomiyasu-katakana-term.svg)

## Spacing and character forms {#spacing}

### `ja-nakaguro-or-halfwidth-space-between-katakana` {#ja-nakaguro-or-halfwidth-space-between-katakana}

![Example of ja-nakaguro-or-halfwidth-space-between-katakana](./rule-examples/ja-nakaguro-or-halfwidth-space-between-katakana.svg)

### `ja-no-space-around-parentheses` {#ja-no-space-around-parentheses}

Half-width round brackets are not checked.

![Example of ja-no-space-around-parentheses](./rule-examples/ja-no-space-around-parentheses.svg)

### `ja-no-space-around-slash` {#ja-no-space-around-slash}

![Example of ja-no-space-around-slash](./rule-examples/ja-no-space-around-slash.svg)

### `ja-no-space-between-full-width` {#ja-no-space-between-full-width}

![Example of ja-no-space-between-full-width](./rule-examples/ja-no-space-between-full-width.svg)

### `ja-space-between-half-and-full-width` {#ja-space-between-half-and-full-width}

Spaces between English words stay.

![Example of ja-space-between-half-and-full-width](./rule-examples/ja-space-between-half-and-full-width.svg)

### `ja-space-after-exclamation` {#ja-space-after-exclamation}

![Example of ja-space-after-exclamation](./rule-examples/ja-space-after-exclamation.svg)

### `ja-space-after-question` {#ja-space-after-question}

![Example of ja-space-after-question](./rule-examples/ja-space-after-question.svg)

### `ja-space-around-code` {#ja-space-around-code}

![Example of ja-space-around-code](./rule-examples/ja-space-around-code.svg)

### `ja-space-around-emphasis` {#ja-space-around-emphasis}

![Example of ja-space-around-emphasis](./rule-examples/ja-space-around-emphasis.svg)

### `ja-space-around-link` {#ja-space-around-link}

![Example of ja-space-around-link](./rule-examples/ja-space-around-link.svg)

### `ja-space-around-strong` {#ja-space-around-strong}

![Example of ja-space-around-strong](./rule-examples/ja-space-around-strong.svg)

### `no-mixed-zenkaku-and-hankaku-alphabet` {#no-mixed-zenkaku-and-hankaku-alphabet}

![Example of no-mixed-zenkaku-and-hankaku-alphabet](./rule-examples/no-mixed-zenkaku-and-hankaku-alphabet.svg)

### `no-kangxi-radicals` {#no-kangxi-radicals}

The two characters look the same but have different code points.

![Example of no-kangxi-radicals](./rule-examples/no-kangxi-radicals.svg)

## Japanese translation style guide: prose style and punctuation {#jtf-prose}

### `1.1.1.本文` {#1.1.1}

The less common style is the one reported.

![Example of 1.1.1.本文](./rule-examples/1.1.1.svg)

### `1.1.2.見出し` {#1.1.2}

![Example of 1.1.2.見出し](./rule-examples/1.1.2.svg)

### `1.1.3.箇条書き` {#1.1.3}

![Example of 1.1.3.箇条書き](./rule-examples/1.1.3.svg)

### `1.1.5.図表のキャプション` {#1.1.5}

![Example of 1.1.5.図表のキャプション](./rule-examples/1.1.5.svg)

### `1.2.1.句点(。)と読点(、)` {#1.2.1}

![Example of 1.2.1.句点(。)と読点(、)](./rule-examples/1.2.1.svg)

### `1.2.2.ピリオド(.)とカンマ(,)` {#1.2.2}

![Example of 1.2.2.ピリオド(.)とカンマ(,)](./rule-examples/1.2.2.svg)

## Japanese translation style guide: characters and terms {#jtf-characters}

### `2.1.2.漢字` {#2.1.2}

![Example of 2.1.2.漢字](./rule-examples/2.1.2.svg)

### `2.1.5.カタカナ` {#2.1.5}

![Example of 2.1.5.カタカナ](./rule-examples/2.1.5.svg)

### `2.1.6.カタカナの長音` {#2.1.6}

![Example of 2.1.6.カタカナの長音](./rule-examples/2.1.6.svg)

### `2.1.8.算用数字` {#2.1.8}

![Example of 2.1.8.算用数字](./rule-examples/2.1.8.svg)

### `2.1.9.アルファベット` {#2.1.9}

![Example of 2.1.9.アルファベット](./rule-examples/2.1.9.svg)

### `2.1.10.算用数字の位取りの表記` {#2.1.10}

A comma that separates thousands is not reported.

![Example of 2.1.10.算用数字の位取りの表記](./rule-examples/2.1.10.svg)

### `2.2.1.ひらがなと漢字の使い分け` {#2.2.1}

![Example of 2.2.1.ひらがなと漢字の使い分け](./rule-examples/2.2.1.svg)

### `2.2.2.算用数字と漢数字の使い分け` {#2.2.2}

![Example of 2.2.2.算用数字と漢数字の使い分け](./rule-examples/2.2.2.svg)

### `2.2.3.一部の助数詞の表記` {#2.2.3}

![Example of 2.2.3.一部の助数詞の表記](./rule-examples/2.2.3.svg)

## Japanese translation style guide: spacing {#jtf-spacing}

### `3.1.1.全角文字と半角文字の間` {#3.1.1}

Spaces between English words stay.

![Example of 3.1.1.全角文字と半角文字の間](./rule-examples/3.1.1.svg)

### `3.1.2.全角文字どうし` {#3.1.2}

![Example of 3.1.2.全角文字どうし](./rule-examples/3.1.2.svg)

### `3.2.カタカナ語間のスペースの有無` {#3.2}

![Example of 3.2.カタカナ語間のスペースの有無](./rule-examples/3.2.svg)

### `3.3.かっこ類と隣接する文字の間のスペースの有無` {#3.3}

Unless the setting is changed, spaces outside half-width round brackets stay.

![Example of 3.3.かっこ類と隣接する文字の間のスペースの有無](./rule-examples/3.3.svg)

## Japanese translation style guide: symbols and brackets {#jtf-punctuation}

### `4.1.1.句点(。)` {#4.1.1}

A period outside the brackets is not reported.

![Example of 4.1.1.句点(。)](./rule-examples/4.1.1.svg)

### `4.1.3.ピリオド(.)、カンマ(,)` {#4.1.3}

![Example of 4.1.3.ピリオド(.)、カンマ(,)](./rule-examples/4.1.3.svg)

### `4.2.1.感嘆符(！)` {#4.2.1}

An exclamation mark right after English is not reported.

![Example of 4.2.1.感嘆符(！)](./rule-examples/4.2.1.svg)

### `4.2.2.疑問符(？)` {#4.2.2}

A question mark right after English is not reported.

![Example of 4.2.2.疑問符(？)](./rule-examples/4.2.2.svg)

### `4.2.4.中黒(・)` {#4.2.4}

![Example of 4.2.4.中黒(・)](./rule-examples/4.2.4.svg)

### `4.2.5.波線(〜)` {#4.2.5}

![Example of 4.2.5.波線(〜)](./rule-examples/4.2.5.svg)

### `4.2.6.ハイフン(-)` {#4.2.6}

A hyphen that joins alphanumeric characters is not reported.

![Example of 4.2.6.ハイフン(-)](./rule-examples/4.2.6.svg)

### `4.2.7.コロン(：)` {#4.2.7}

A colon inside a URL is not reported.

![Example of 4.2.7.コロン(：)](./rule-examples/4.2.7.svg)

### `4.2.8.セミコロン(;)` {#4.2.8}

![Example of 4.2.8.セミコロン(;)](./rule-examples/4.2.8.svg)

### `4.2.9.ダッシュ(-)` {#4.2.9}

![Example of 4.2.9.ダッシュ(-)](./rule-examples/4.2.9.svg)

### `4.3.1.丸かっこ（）` {#4.3.1}

![Example of 4.3.1.丸かっこ（）](./rule-examples/4.3.1.svg)

### `4.3.2.大かっこ［］` {#4.3.2}

![Example of 4.3.2.大かっこ［］](./rule-examples/4.3.2.svg)

### `4.3.3.かぎかっこ「」` {#4.3.3}

Paired brackets are not reported, only the opening bracket that is never closed.

![Example of 4.3.3.かぎかっこ「」](./rule-examples/4.3.3.svg)

### `4.3.4.二重かぎかっこ『』` {#4.3.4}

Paired brackets are not reported, only the opening bracket that is never closed.

![Example of 4.3.4.二重かぎかっこ『』](./rule-examples/4.3.4.svg)

### `4.3.5.二重引用符` {#4.3.5}

Only when the number of quotation marks is odd, the unpaired one is reported.

![Example of 4.3.5.二重引用符](./rule-examples/4.3.5.svg)

### `4.3.6.中かっこ{ }` {#4.3.6}

Paired braces are not reported, only the opening brace that is never closed.

![Example of 4.3.6.中かっこ{ }](./rule-examples/4.3.6.svg)

### `4.3.7.山かっこ<>` {#4.3.7}

Arrows and comparison operators are not counted as angle brackets.

![Example of 4.3.7.山かっこ<>](./rule-examples/4.3.7.svg)

## Hiragana, forms with `さ`, and synonyms {#japanese-wording}

### `ja-hiragana-hojodoushi` {#ja-hiragana-hojodoushi}

A main verb such as `頂きました` is not reported.

![Example of ja-hiragana-hojodoushi](./rule-examples/ja-hiragana-hojodoushi.svg)

### `ja-hiragana-keishikimeishi` {#ja-hiragana-keishikimeishi}

![Example of ja-hiragana-keishikimeishi](./rule-examples/ja-hiragana-keishikimeishi.svg)

### `ja-keishikimeishi` {#ja-keishikimeishi}

The report for `うえ` is turned off by the setting, so `上` is not reported.

![Example of ja-keishikimeishi](./rule-examples/ja-keishikimeishi.svg)

This example uses the following configuration.

```toml
[builtin]
ja-keishikimeishi = { detection_ue = false }
```

### `ja-hiragana-fukushi` {#ja-hiragana-fukushi}

![Example of ja-hiragana-fukushi](./rule-examples/ja-hiragana-fukushi.svg)

### `ja-hiragana-daimeishi` {#ja-hiragana-daimeishi}

![Example of ja-hiragana-daimeishi](./rule-examples/ja-hiragana-daimeishi.svg)

### `@textlint-ja/no-insert-dropping-sa` {#textlint-ja-no-insert-dropping-sa}

![Example of @textlint-ja/no-insert-dropping-sa](./rule-examples/textlint-ja-no-insert-dropping-sa.svg)

### `@textlint-ja/no-synonyms` {#textlint-ja-no-synonyms}

![Example of @textlint-ja/no-synonyms](./rule-examples/textlint-ja-no-synonyms.svg)

## Language-independent checks {#general}

### `no-todo` {#no-todo}

A completed task is not reported.

![Example of no-todo](./rule-examples/no-todo.svg)

### `date-weekday-mismatch` {#date-weekday-mismatch}

A date whose weekday is correct is not reported.

![Example of date-weekday-mismatch](./rule-examples/date-weekday-mismatch.svg)

## English spelling and style {#english}

### `terminology` {#terminology}

A word that only the built-in dictionary matches is reported and left as it is.

![Example of terminology](./rule-examples/terminology.svg)

This example uses the following configuration.

```toml
[builtin]
terminology = { terms = [['front[- ]end(\w*)', 'frontend$1']] }
```

### `spelling` {#spelling}

![Example of spelling](./rule-examples/spelling.svg)

### `common-misspellings` {#common-misspellings}

![Example of common-misspellings](./rule-examples/common-misspellings.svg)

### `write-good` {#write-good}

![Example of write-good](./rule-examples/write-good.svg)

## Custom replacements and filters {#custom-rules-and-filters}

### `prh` {#prh}

![Example of prh](./rule-examples/prh.svg)

This example uses the following configuration.

```toml
[builtin]
prh = { ruleContents = ['''
rules:
  - expected: 利用者
    pattern: /ユーザー?/
'''] }
```

### `@textlint-rule/pattern` {#textlint-rule-pattern}

A date without a year does not match the pattern, so it stays.

![Example of @textlint-rule/pattern](./rule-examples/textlint-rule-pattern.svg)

This example uses the following configuration.

```toml
[builtin]
"@textlint-rule/pattern" = { patterns = [{ message = "日付はYYYY-MM-DDで書く", pattern = '/(\d{4})/(\d{2})/(\d{2})/', replace = "$1-$2-$3" }] }
```

## Rules without an example {#skipped}

The following rules have no image because a single example text cannot show them.

- `4.3.8.一重引用符`: This rule only exists so that the preset key can be written in a configuration, and it reports nothing for any text.
- `no-dead-link`: The rule that checks link targets does not apply to text from standard input, and the example text is passed through standard input, so it can report nothing.
