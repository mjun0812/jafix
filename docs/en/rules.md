# Rules

jafix has 121 built-in rules.
Switch them by rule ID or preset name under `[builtin]` in the [configuration](./configuration.md#rules).

| Column  | Meaning                                                                                                               |
| ------- | --------------------------------------------------------------------------------------------------------------------- |
| Action  | **Fix** rules are applied by `jafix fix` and the hooks. **Report** rules only flag findings. `jafix check` lists both |
| Default | **On** marks the 10 rules that run without configuration. **-** marks rules you add in the configuration              |

Options and their defaults are in the [reference](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#組み込みのルール) (Japanese). [Rule examples](./rule-examples.md) shows how each rule changes text, with one image per rule. Each rule ID in the tables links to its example.

Run `jafix rules` to see which rules your configuration enables.

```sh
jafix rules docs/guide.md
```

It prints one rule per line as three tab-separated fields. The first field is the rule ID. The second is `有効` (enabled) or `無効` (disabled) after applying the global and project settings. The third lists the presets that include the rule, or `-` if no preset does. This `-` differs from the `-` in the Default column above. When you pass a file or directory, jafix answers with the project settings found from that path. File exclusions (`exclude` and `report-exclude`) are not applied, so a rule enabled by the settings shows `有効` even for an excluded file.

## jafix-specific fixes {#jafix}

| Rule ID                                                                       | Action | Default | What it does                                                            |
| ----------------------------------------------------------------------------- | ------ | ------- | ----------------------------------------------------------------------- |
| [`fullwidth-paren`](./rule-examples.md#fullwidth-paren)                       | Fix    | On      | Converts full-width parentheses to half-width                           |
| [`space-between-ja-and-alnum`](./rule-examples.md#space-between-ja-and-alnum) | Fix    | On      | Removes spaces between Japanese characters and half-width alphanumerics |
| [`space-between-ja-and-paren`](./rule-examples.md#space-between-ja-and-paren) | Fix    | On      | Removes spaces between Japanese characters and parentheses              |

## Japanese technical writing {#technical-writing}

Set `preset-ja-technical-writing = true` to enable all 23 rules below.
Set `sentence-length = true` to add the sentence-length check to the default rules.

| Rule ID                                                                                       | Action         | Default | What it does                                                                                                          |
| --------------------------------------------------------------------------------------------- | -------------- | ------- | --------------------------------------------------------------------------------------------------------------------- |
| [`sentence-length`](./rule-examples.md#sentence-length)                                       | Report         | -       | Flags sentences that exceed the length limit                                                                          |
| [`max-comma`](./rule-examples.md#max-comma)                                                   | Report         | -       | Flags sentences with too many ASCII commas                                                                            |
| [`max-kanji-continuous-len`](./rule-examples.md#max-kanji-continuous-len)                     | Report         | -       | Flags runs of consecutive kanji that exceed the limit                                                                 |
| [`arabic-kanji-numbers`](./rule-examples.md#arabic-kanji-numbers)                             | Report         | -       | Checks where to use Arabic numerals or kanji numerals                                                                 |
| [`ja-no-mixed-period`](./rule-examples.md#ja-no-mixed-period)                                 | Report         | -       | Flags paragraph-ending punctuation, including missing Japanese full stops where required                              |
| [`no-nfd`](./rule-examples.md#no-nfd)                                                         | Fix and report | On      | Combines separate dakuten or handakuten marks with the preceding character and reports combinations it cannot compose |
| [`no-invalid-control-character`](./rule-examples.md#no-invalid-control-character)             | Fix            | On      | Removes disallowed control characters                                                                                 |
| [`no-zero-width-spaces`](./rule-examples.md#no-zero-width-spaces)                             | Fix            | On      | Removes zero-width spaces                                                                                             |
| [`no-exclamation-question-mark`](./rule-examples.md#no-exclamation-question-mark)             | Report         | -       | Flags exclamation and question marks                                                                                  |
| [`no-hankaku-kana`](./rule-examples.md#no-hankaku-kana)                                       | Fix            | -       | Converts half-width kana to full-width kana                                                                           |
| [`ja-unnatural-alphabet`](./rule-examples.md#ja-unnatural-alphabet)                           | Report         | -       | Flags a single Latin letter between Japanese characters                                                               |
| [`no-unmatched-pair`](./rule-examples.md#no-unmatched-pair)                                   | Report         | On      | Flags unmatched brackets and quotation marks                                                                          |
| [`max-ten`](./rule-examples.md#max-ten)                                                       | Report         | -       | Flags sentences with too many Japanese commas                                                                         |
| [`no-doubled-joshi`](./rule-examples.md#no-doubled-joshi)                                     | Report         | -       | Flags repeated particles close together in a sentence                                                                 |
| [`no-doubled-conjunctive-particle-ga`](./rule-examples.md#no-doubled-conjunctive-particle-ga) | Report         | -       | Flags sentences with two or more uses of the adversative particle `が`                                                |
| [`no-doubled-conjunction`](./rule-examples.md#no-doubled-conjunction)                         | Report         | -       | Flags repetition of the first conjunction in sentences that contain one within the same paragraph                     |
| [`no-mix-dearu-desumasu`](./rule-examples.md#no-mix-dearu-desumasu)                           | Report         | -       | Flags mixtures of polite and plain styles separately in headings, prose, and lists                                    |
| [`no-double-negative-ja`](./rule-examples.md#no-double-negative-ja)                           | Report         | -       | Flags double negatives such as `ないこともない`                                                                       |
| [`no-dropping-the-ra`](./rule-examples.md#no-dropping-the-ra)                                 | Report         | -       | Flags forms that drop `ら`, such as `見れる`                                                                          |
| [`ja-no-weak-phrase`](./rule-examples.md#ja-no-weak-phrase)                                   | Report         | -       | Flags hedges such as `かも` and `思う`                                                                                |
| [`ja-no-successive-word`](./rule-examples.md#ja-no-successive-word)                           | Report         | On      | Flags repeated words and certain repeated marks, including brackets                                                   |
| [`ja-no-abusage`](./rule-examples.md#ja-no-abusage)                                           | Report         | -       | Flags listed usage errors, such as `値を返却する`                                                                     |
| [`ja-no-redundant-expression`](./rule-examples.md#ja-no-redundant-expression)                 | Report         | -       | Flags redundant expressions such as `することができる`                                                                |

## Patterns common in AI writing {#ai-writing}

Set `"@textlint-ja/preset-ai-writing" = true` to enable five rules covering list formatting and wording in technical prose.
The `preset-ai-words-ja = true` setting enables `no-ai-words`.
Set `no-short-topic-comma = true` separately to check commas after short topics.

| Rule ID                                                                     | Action | Default | What it does                                                                                              |
| --------------------------------------------------------------------------- | ------ | ------- | --------------------------------------------------------------------------------------------------------- |
| [`no-ai-list-formatting`](./rule-examples.md#no-ai-list-formatting)         | Report | -       | Flags patterns such as bold text followed by a colon at the start of a list item, and emoji in list items |
| [`no-ai-hype-expressions`](./rule-examples.md#no-ai-hype-expressions)       | Report | -       | Flags listed hype expressions such as `革命的な`, meaning “revolutionary”                                 |
| [`no-ai-emphasis-patterns`](./rule-examples.md#no-ai-emphasis-patterns)     | Report | -       | Flags patterns such as combining emoji with bold text or using bold text inside headings                  |
| [`no-ai-colon-continuation`](./rule-examples.md#no-ai-colon-continuation)   | Report | -       | Flags paragraphs ending in a predicate and a colon before a code block, list, or similar block            |
| [`ai-tech-writing-guideline`](./rule-examples.md#ai-tech-writing-guideline) | Report | -       | Flags patterns such as redundancy, passive voice, vague wording, and inconsistent spelling                |
| [`no-ai-words`](./rule-examples.md#no-ai-words)                             | Report | -       | Flags words common in AI writing by matching their base form and part of speech                           |
| [`no-short-topic-comma`](./rule-examples.md#no-short-topic-comma)           | Report | -       | Flags a Japanese comma immediately after `は` when it marks a short topic                                 |

### Rules adapted from yomiyasu {#yomiyasu}

Enable each of these rules separately. All 22 are off by default. `yomiyasu-bold-not-rendered` fixes the pairs it can confirm a repair for and flags the rest. The other 21 rules only report.

| Rule ID                                                                           | Action         | Default | What it does                                                                                                    |
| --------------------------------------------------------------------------------- | -------------- | ------- | --------------------------------------------------------------------------------------------------------------- |
| [`yomiyasu-bold-not-rendered`](./rule-examples.md#yomiyasu-bold-not-rendered)     | Fix and report | -       | Fixes pairs of `**` that appear as literal text instead of rendering as bold, and flags the pairs it cannot fix |
| [`yomiyasu-instruction-residue`](./rule-examples.md#yomiyasu-instruction-residue) | Report         | -       | Flags writing instructions left in the prose and scope exclusions without a reference                           |
| [`yomiyasu-chat-response`](./rule-examples.md#yomiyasu-chat-response)             | Report         | -       | Flags stock chat replies left in prose, such as `ご質問ありがとうございます`                                    |
| [`yomiyasu-meta-filler`](./rule-examples.md#yomiyasu-meta-filler)                 | Report         | -       | Flags stock openers and closers left in prose, such as `重要なのは` and `いかがでしたでしょうか`                |
| [`yomiyasu-summary-restatement`](./rule-examples.md#yomiyasu-summary-restatement) | Report         | -       | Flags sentences that restate a point starting with `まとめると、` or `総じて、`                                 |
| [`yomiyasu-meta-intro`](./rule-examples.md#yomiyasu-meta-intro)                   | Report         | -       | Flags previews that start with `本記事では`                                                                     |
| [`yomiyasu-count-declaration`](./rule-examples.md#yomiyasu-count-declaration)     | Report         | -       | Flags count announcements such as `ポイントは3つです`                                                           |
| [`yomiyasu-ai-phrase`](./rule-examples.md#yomiyasu-ai-phrase)                     | Report         | -       | Flags phrases common in AI writing, such as `多岐にわたる` and `が求められている`                               |
| [`yomiyasu-placeholder-residue`](./rule-examples.md#yomiyasu-placeholder-residue) | Report         | -       | Flags placeholders left in prose: `〇〇`, `XXX`, `[ここに…]`, and `【要確認】`                                  |
| [`yomiyasu-short-mochiron`](./rule-examples.md#yomiyasu-short-mochiron)           | Report         | -       | Flags short sentences in which only a few words follow a leading `もちろん、`                                   |
| [`yomiyasu-say-hedge`](./rule-examples.md#yomiyasu-say-hedge)                     | Report         | -       | Flags sentences ending in `と言えるでしょう`                                                                    |
| [`yomiyasu-fragment-run`](./rule-examples.md#yomiyasu-fragment-run)               | Report         | -       | Flags runs of three or more very short sentences, such as `速い。軽い。安い。`                                  |
| [`yomiyasu-cleft-list-ending`](./rule-examples.md#yomiyasu-cleft-list-ending)     | Report         | -       | Flags sentences that follow `〜のは、` with a list of nouns and no predicate                                    |
| [`yomiyasu-copy-comma-noun`](./rule-examples.md#yomiyasu-copy-comma-noun)         | Report         | -       | Flags sentences that put a comma between a modifier and a short final noun, such as `軽い、美しい仕様。`        |
| [`yomiyasu-bare-verb-chain`](./rule-examples.md#yomiyasu-bare-verb-chain)         | Report         | -       | Flags sentences that list verbs with commas and no subject or object, such as `書く、試す、直す。`              |
| [`yomiyasu-dash-decoration`](./rule-examples.md#yomiyasu-dash-decoration)         | Report         | -       | Flags dashes such as `——` used for asides or a trailing effect                                                  |
| [`yomiyasu-dash-insertion`](./rule-examples.md#yomiyasu-dash-insertion)           | Report         | -       | Flags asides set off between dashes, such as `今日——正確には昨日——`                                             |
| [`yomiyasu-dash-list-ending`](./rule-examples.md#yomiyasu-dash-list-ending)       | Report         | -       | Flags sentences that end a list of nouns with a dash, such as `速さ、軽さ、静けさ──。`                          |
| [`yomiyasu-emoji-prohibited`](./rule-examples.md#yomiyasu-emoji-prohibited)       | Report         | -       | Flags emoji in prose                                                                                            |
| [`yomiyasu-renyo-noun`](./rule-examples.md#yomiyasu-renyo-noun)                   | Report         | -       | Flags verb stems used as nouns after `の`, such as `結果の確かめ`                                               |
| [`yomiyasu-slop-vocabulary`](./rule-examples.md#yomiyasu-slop-vocabulary)         | Report         | -       | Flags figurative words common in AI writing, such as `解像度`, `手触り`, and `羅針盤`                           |
| [`yomiyasu-katakana-term`](./rule-examples.md#yomiyasu-katakana-term)             | Report         | -       | Flags katakana loanwords that tend to lose meaning, such as `シナジー` and `ロードマップ`                       |

## Spacing and character forms {#spacing}

Set `preset-ja-spacing = true` to enable nine rules.
Enable spacing around emphasis and bold text by each rule's ID.
The two character-form rules are outside this preset. Of those two, `no-kangxi-radicals` is enabled by default.

| Rule ID                                                                                                                 | Action         | Default | What it does                                                                                                                       |
| ----------------------------------------------------------------------------------------------------------------------- | -------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| [`ja-nakaguro-or-halfwidth-space-between-katakana`](./rule-examples.md#ja-nakaguro-or-halfwidth-space-between-katakana) | Report         | -       | Flags full-width spaces between katakana words                                                                                     |
| [`ja-no-space-around-parentheses`](./rule-examples.md#ja-no-space-around-parentheses)                                   | Fix            | -       | Removes spaces inside and outside the brackets covered by the rule                                                                 |
| [`ja-no-space-around-slash`](./rule-examples.md#ja-no-space-around-slash)                                               | Fix            | -       | Removes spaces around slashes                                                                                                      |
| [`ja-no-space-between-full-width`](./rule-examples.md#ja-no-space-between-full-width)                                   | Fix            | -       | Removes half-width spaces between full-width characters                                                                            |
| [`ja-space-between-half-and-full-width`](./rule-examples.md#ja-space-between-half-and-full-width)                       | Fix and report | -       | Removes spaces between full-width characters and half-width letters or digits, or reports missing spaces, depending on the setting |
| [`ja-space-after-exclamation`](./rule-examples.md#ja-space-after-exclamation)                                           | Report         | -       | Flags a half-width space after a full-width exclamation mark when another sentence follows                                         |
| [`ja-space-after-question`](./rule-examples.md#ja-space-after-question)                                                 | Report         | -       | Flags a half-width space after a full-width question mark when another sentence follows                                            |
| [`ja-space-around-code`](./rule-examples.md#ja-space-around-code)                                                       | Fix and report | -       | Checks spaces before and after inline code next to Japanese characters, using separate settings for each side                      |
| [`ja-space-around-emphasis`](./rule-examples.md#ja-space-around-emphasis)                                               | Fix and report | -       | Checks spaces before and after emphasis next to Japanese characters, using separate settings for each side                         |
| [`ja-space-around-link`](./rule-examples.md#ja-space-around-link)                                                       | Fix and report | -       | Checks spaces before and after links next to Japanese characters, using separate settings for each side                            |
| [`ja-space-around-strong`](./rule-examples.md#ja-space-around-strong)                                                   | Fix and report | -       | Checks spaces before and after bold text next to Japanese characters, using separate settings for each side                        |
| [`no-mixed-zenkaku-and-hankaku-alphabet`](./rule-examples.md#no-mixed-zenkaku-and-hankaku-alphabet)                     | Fix            | -       | Standardizes Latin letters to the configured half-width or full-width form                                                         |
| [`no-kangxi-radicals`](./rule-examples.md#no-kangxi-radicals)                                                           | Fix            | On      | Replaces Kangxi radical characters with their corresponding kanji                                                                  |

## Japanese translation style guide {#jtf-style}

The Japan Translation Federation (JTF) publishes a [style guide for translators working into Japanese](https://www.jtf.jp/tips/styleguide) to help them use consistent spelling and punctuation.

jafix includes 37 rules based on the guide. Set `preset-jtf-style = true` to enable 33 of them. Enable the remaining four individually (`2.1.2.漢字`, `2.1.5.カタカナ`, `2.1.6.カタカナの長音`, and `2.2.1.ひらがなと漢字の使い分け`).
Quote JTF rule IDs when using them as TOML keys.

The JTF parentheses rule calls for full-width parentheses. jafix's `fullwidth-paren` rule, enabled by default, converts them to half-width.
When enabling the preset, disable one of these rules according to the style you want.
This example selects full-width parentheses for Japanese prose.

```toml
[builtin]
preset-jtf-style = true
fullwidth-paren = false
```

### Prose style and punctuation {#jtf-prose}

| Rule ID                                                    | Action | Default | What it does                                                                                  |
| ---------------------------------------------------------- | ------ | ------- | --------------------------------------------------------------------------------------------- |
| [`1.1.1.本文`](./rule-examples.md#1.1.1)                   | Report | -       | Counts polite and plain forms in prose and flags the less frequent style when they are mixed  |
| [`1.1.2.見出し`](./rule-examples.md#1.1.2)                 | Report | -       | Flags Japanese full stops at the end of headings                                              |
| [`1.1.3.箇条書き`](./rule-examples.md#1.1.3)               | Report | -       | Flags inconsistent use of final Japanese full stops or mixed writing styles across list items |
| [`1.1.5.図表のキャプション`](./rule-examples.md#1.1.5)     | Report | -       | Flags a mixture of polite and plain styles in image alt text                                  |
| [`1.2.1.句点(。)と読点(、)`](./rule-examples.md#1.2.1)     | Report | -       | Flags half-width commas and periods used as Japanese punctuation                              |
| [`1.2.2.ピリオド(.)とカンマ(,)`](./rule-examples.md#1.2.2) | Report | -       | Flags full-width commas and periods                                                           |

### Characters and terms {#jtf-characters}

| Rule ID                                                        | Action         | Default | What it does                                                                 |
| -------------------------------------------------------------- | -------------- | ------- | ---------------------------------------------------------------------------- |
| [`2.1.2.漢字`](./rule-examples.md#2.1.2)                       | Report         | -       | Flags kanji outside the jōyō kanji list                                      |
| [`2.1.5.カタカナ`](./rule-examples.md#2.1.5)                   | Fix and report | -       | Converts half-width kana to full-width and flags listed katakana spellings   |
| [`2.1.6.カタカナの長音`](./rule-examples.md#2.1.6)             | Report         | -       | Checks the use of a final long vowel mark in listed katakana words           |
| [`2.1.8.算用数字`](./rule-examples.md#2.1.8)                   | Fix            | -       | Converts full-width Arabic numerals to half-width                            |
| [`2.1.9.アルファベット`](./rule-examples.md#2.1.9)             | Fix            | -       | Converts full-width Latin letters to half-width                              |
| [`2.1.10.算用数字の位取りの表記`](./rule-examples.md#2.1.10)   | Report         | -       | Flags commas used as decimal separators                                      |
| [`2.2.1.ひらがなと漢字の使い分け`](./rule-examples.md#2.2.1)   | Report         | -       | Flags kanji spellings such as `予め` where the dictionary calls for hiragana |
| [`2.2.2.算用数字と漢数字の使い分け`](./rule-examples.md#2.2.2) | Report         | -       | Checks where to use Arabic numerals or kanji numerals                        |
| [`2.2.3.一部の助数詞の表記`](./rule-examples.md#2.2.3)         | Report         | -       | Flags listed counter spellings, such as `ヶ月` after a number                |

### Spacing {#jtf-spacing}

| Rule ID                                                                    | Action         | Default | What it does                                                                                            |
| -------------------------------------------------------------------------- | -------------- | ------- | ------------------------------------------------------------------------------------------------------- |
| [`3.1.1.全角文字と半角文字の間`](./rule-examples.md#3.1.1)                 | Fix            | -       | Removes half-width spaces between Japanese characters and half-width letters or digits                  |
| [`3.1.2.全角文字どうし`](./rule-examples.md#3.1.2)                         | Fix            | -       | Removes half-width spaces between full-width characters                                                 |
| [`3.2.カタカナ語間のスペースの有無`](./rule-examples.md#3.2)               | Report         | -       | Flags full-width spaces between katakana words                                                          |
| [`3.3.かっこ類と隣接する文字の間のスペースの有無`](./rule-examples.md#3.3) | Fix and report | -       | Removes spaces inside and outside brackets. An option can require spaces outside half-width parentheses |

### Symbols and brackets {#jtf-punctuation}

| Rule ID                                                    | Action | Default | What it does                                                                                                                  |
| ---------------------------------------------------------- | ------ | ------- | ----------------------------------------------------------------------------------------------------------------------------- |
| [`4.1.1.句点(。)`](./rule-examples.md#4.1.1)               | Report | -       | Flags Japanese full stops just before closing brackets                                                                        |
| [`4.1.3.ピリオド(.)、カンマ(,)`](./rule-examples.md#4.1.3) | Report | -       | Flags half-width periods used as full stops in Japanese prose                                                                 |
| [`4.2.1.感嘆符(！)`](./rule-examples.md#4.2.1)             | Report | -       | Flags half-width exclamation marks in Japanese prose and half-width spaces after sentence-ending full-width exclamation marks |
| [`4.2.2.疑問符(？)`](./rule-examples.md#4.2.2)             | Report | -       | Flags half-width question marks in Japanese prose and half-width spaces after sentence-ending full-width question marks       |
| [`4.2.4.中黒(・)`](./rule-examples.md#4.2.4)               | Fix    | -       | Converts half-width middle dots to full-width                                                                                 |
| [`4.2.5.波線(〜)`](./rule-examples.md#4.2.5)               | Report | -       | Flags half-width tildes in numeric ranges                                                                                     |
| [`4.2.6.ハイフン(-)`](./rule-examples.md#4.2.6)            | Report | -       | Flags hyphens in Japanese prose                                                                                               |
| [`4.2.7.コロン(：)`](./rule-examples.md#4.2.7)             | Report | -       | Flags half-width colons in Japanese prose                                                                                     |
| [`4.2.8.セミコロン(;)`](./rule-examples.md#4.2.8)          | Report | -       | Flags semicolons in Japanese prose                                                                                            |
| [`4.2.9.ダッシュ(-)`](./rule-examples.md#4.2.9)            | Report | -       | Flags dashes from U+2012 through U+2015 in Japanese prose                                                                     |
| [`4.3.1.丸かっこ（）`](./rule-examples.md#4.3.1)           | Report | -       | Flags half-width parentheses in Japanese prose                                                                                |
| [`4.3.2.大かっこ［］`](./rule-examples.md#4.3.2)           | Report | -       | Flags half-width square brackets in Japanese prose                                                                            |
| [`4.3.3.かぎかっこ「」`](./rule-examples.md#4.3.3)         | Report | -       | Flags unmatched Japanese corner brackets within a paragraph                                                                   |
| [`4.3.4.二重かぎかっこ『』`](./rule-examples.md#4.3.4)     | Report | -       | Flags unmatched double corner brackets within a paragraph                                                                     |
| [`4.3.5.二重引用符`](./rule-examples.md#4.3.5)             | Report | -       | Flags an odd number of ASCII double quotation marks in a paragraph                                                            |
| [`4.3.6.中かっこ{ }`](./rule-examples.md#4.3.6)            | Report | -       | Flags unmatched braces within a paragraph                                                                                     |
| [`4.3.7.山かっこ<>`](./rule-examples.md#4.3.7)             | Report | -       | Flags unmatched angle brackets within a paragraph, excluding arrows and operators such as `<=`                                |
| [`4.3.8.一重引用符`](./rule-examples.md#skipped)           | None   | -       | Accepts the preset's configuration key and produces no findings                                                               |

## Hiragana, forms with さ, and synonyms {#japanese-wording}

Enable each of these seven rules by its ID.
The hiragana rules use part of speech and nearby words to check how a word is used.

| Rule ID                                                                                      | Action | Default | What it does                                                                              |
| -------------------------------------------------------------------------------------------- | ------ | ------- | ----------------------------------------------------------------------------------------- |
| [`ja-hiragana-hojodoushi`](./rule-examples.md#ja-hiragana-hojodoushi)                        | Report | -       | Flags auxiliary verbs written in kanji, such as `致し` in `お願い致します`                |
| [`ja-hiragana-keishikimeishi`](./rule-examples.md#ja-hiragana-keishikimeishi)                | Report | -       | Flags formal nouns written in kanji, such as `事` in `見た事がある`                       |
| [`ja-keishikimeishi`](./rule-examples.md#ja-keishikimeishi)                                  | Report | -       | Flags formal nouns written in kanji, with options to exclude `ほう` and `うえ`            |
| [`ja-hiragana-fukushi`](./rule-examples.md#ja-hiragana-fukushi)                              | Report | -       | Flags kanji spellings of adverbs normally written in hiragana, such as `既に`             |
| [`ja-hiragana-daimeishi`](./rule-examples.md#ja-hiragana-daimeishi)                          | Report | -       | Flags kanji spellings of pronouns normally written in hiragana, such as `何処`            |
| [`@textlint-ja/no-insert-dropping-sa`](./rule-examples.md#textlint-ja-no-insert-dropping-sa) | Report | -       | Flags an extra `さ`, as in `辛さそう`, or a missing `さ`, as in `良そう`                  |
| [`@textlint-ja/no-synonyms`](./rule-examples.md#textlint-ja-no-synonyms)                     | Report | -       | Flags inconsistent forms for the same term in a document, such as `サーバ` and `サーバー` |

## Language-independent checks {#general}

| Rule ID                                                             | Action | Default | What it does                                                                                                          |
| ------------------------------------------------------------------- | ------ | ------- | --------------------------------------------------------------------------------------------------------------------- |
| [`no-todo`](./rule-examples.md#no-todo)                             | Report | -       | Flags `TODO:` and unchecked task items                                                                                |
| [`date-weekday-mismatch`](./rule-examples.md#date-weekday-mismatch) | Report | On      | Flags dates followed by a weekday in parentheses that falls on a different day of the week                            |
| [`no-dead-link`](./rule-examples.md#skipped)                        | Report | -       | Flags missing local link targets, failed URL requests, and redirects. External URLs are checked only by `jafix check` |

## English spelling and style {#english}

Enable `common-misspellings` on its own to check for frequently misspelled words.
If you already use `spelling`, set it to `false` to limit the check to the list.
When using the dictionary-based `spelling` rule, use `skipPatterns` for names and technical terms used in your project.

| Rule ID                                                         | Action         | Default | What it does                                                                                                                                                                          |
| --------------------------------------------------------------- | -------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`terminology`](./rule-examples.md#terminology)                 | Fix and report | -       | Checks the spelling and capitalization of technical terms, brands, and abbreviations. Applies replacements from your `terms` setting and reports matches from the built-in dictionary |
| [`spelling`](./rule-examples.md#spelling)                       | Report         | -       | Flags words outside the US English Hunspell dictionary and provides suggestions when configured                                                                                       |
| [`common-misspellings`](./rule-examples.md#common-misspellings) | Report         | -       | Shows listed misspellings and their correct forms                                                                                                                                     |
| [`write-good`](./rule-examples.md#write-good)                   | Report         | -       | Flags patterns such as vague words, passive voice, and redundant phrases                                                                                                              |

## Custom replacements and filters {#custom-rules-and-filters}

Use dictionaries and patterns under `[builtin]` to specify project terminology or expressions to flag.

| Rule ID                                                              | Action         | Default | What it does                                                                                       |
| -------------------------------------------------------------------- | -------------- | ------- | -------------------------------------------------------------------------------------------------- |
| [`prh`](./rule-examples.md#prh)                                      | Fix and report | -       | Applies replacements from the prh dictionaries specified in `rulePaths`                            |
| [`@textlint-rule/pattern`](./rule-examples.md#textlint-rule-pattern) | Fix and report | -       | Applies the `replace` value to a match in `patterns` when provided and reports the match otherwise |

Under `[filters]`, `comments` excludes ranges marked by HTML comments, and `allowlist` excludes ranges that match specified patterns. These exclusions apply to fixes and findings.
