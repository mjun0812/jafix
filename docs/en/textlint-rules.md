# Rules from textlint

## Choosing rules {#configuration}

Select built-in rules under `[builtin]` in `~/.config/jafix/config.toml`.
If `XDG_CONFIG_HOME` is set, use `$XDG_CONFIG_HOME/jafix/config.toml`.
Use the rule ID as the key. Set it to `true` to enable the rule or `false` to disable it.
Use a TOML table such as `{ max = 120 }` as the value to set the rule's options.

This example sets the sentence length limit to 120, disables the rule for exclamation and question marks, and checks for common English misspellings.

```toml
[builtin]
sentence-length = { max = 120 }
no-exclamation-question-mark = false
common-misspellings = true
```

The 23 rules for Japanese technical writing are enabled by default.
Enable the other rules on this page as needed.
A **preset** configures a group of rules at once.
A setting for an individual rule takes precedence over its preset.

In the tables below, **Fix** means `jafix fix` and the hooks can apply a change. **Report** means the rule flags a finding for review.
`jafix check` lists both kinds of findings.

## Japanese technical writing {#technical-writing}

These 23 rules belong to `preset-ja-technical-writing`.
To use just the sentence length rule from this preset, set `preset-ja-technical-writing = false` and `sentence-length = true`.

| Rule ID                              | Action         | What it does                                                                                                          |
| ------------------------------------ | -------------- | --------------------------------------------------------------------------------------------------------------------- |
| `sentence-length`                    | Report         | Flags sentences that exceed the length limit                                                                          |
| `max-comma`                          | Report         | Flags sentences with too many ASCII commas                                                                            |
| `max-kanji-continuous-len`           | Report         | Flags runs of consecutive kanji that exceed the limit                                                                 |
| `arabic-kanji-numbers`               | Report         | Checks where to use Arabic numerals or kanji numerals                                                                 |
| `ja-no-mixed-period`                 | Report         | Flags paragraph-ending punctuation, including missing Japanese full stops where required                              |
| `no-nfd`                             | Fix and report | Combines separate dakuten or handakuten marks with the preceding character and reports combinations it cannot compose |
| `no-invalid-control-character`       | Fix            | Removes disallowed control characters                                                                                 |
| `no-zero-width-spaces`               | Fix            | Removes zero-width spaces                                                                                             |
| `no-exclamation-question-mark`       | Report         | Flags exclamation and question marks                                                                                  |
| `no-hankaku-kana`                    | Fix            | Converts half-width kana to full-width kana                                                                           |
| `ja-unnatural-alphabet`              | Report         | Flags a single Latin letter between Japanese characters                                                               |
| `no-unmatched-pair`                  | Report         | Flags unmatched brackets and quotation marks                                                                          |
| `max-ten`                            | Report         | Flags sentences with too many Japanese commas                                                                         |
| `no-doubled-joshi`                   | Report         | Flags repeated particles close together in a sentence                                                                 |
| `no-doubled-conjunctive-particle-ga` | Report         | Flags sentences with two or more uses of the adversative particle が                                                  |
| `no-doubled-conjunction`             | Report         | Flags repetition of the first conjunction in sentences that contain one within the same paragraph                     |
| `no-mix-dearu-desumasu`              | Report         | Flags mixtures of polite and plain styles separately in headings, prose, and lists                                    |
| `no-double-negative-ja`              | Report         | Flags double negatives such as `ないこともない`                                                                         |
| `no-dropping-the-ra`                 | Report         | Flags forms that drop `ら`, such as `見れる`                                                                              |
| `ja-no-weak-phrase`                  | Report         | Flags hedges such as `かも` and `思う`                                                                                    |
| `ja-no-successive-word`              | Report         | Flags repeated words and certain repeated marks, including brackets                                                   |
| `ja-no-abusage`                      | Report         | Flags listed usage errors, such as `値を返却する`                                                                       |
| `ja-no-redundant-expression`         | Report         | Flags redundant expressions such as することができる                                                                  |

## Patterns common in AI writing {#ai-writing}

Set `"@textlint-ja/preset-ai-writing" = true` to enable five rules covering list formatting and wording in technical prose.
The `preset-ai-words-ja = true` setting enables `no-ai-words`.
Set `no-short-topic-comma = true` separately to check commas after short topics.

| Rule ID                     | Action | What it does                                                                                              |
| --------------------------- | ------ | --------------------------------------------------------------------------------------------------------- |
| `no-ai-list-formatting`     | Report | Flags patterns such as bold text followed by a colon at the start of a list item, and emoji in list items |
| `no-ai-hype-expressions`    | Report | Flags listed hype expressions such as `革命的な`, meaning “revolutionary”                                          |
| `no-ai-emphasis-patterns`   | Report | Flags patterns such as combining emoji with bold text or using bold text inside headings                  |
| `no-ai-colon-continuation`  | Report | Flags paragraphs ending in a predicate and a colon before a code block, list, or similar block            |
| `ai-tech-writing-guideline` | Report | Flags patterns such as redundancy, passive voice, vague wording, and inconsistent spelling                |
| `no-ai-words`               | Report | Flags words common in AI writing by matching their base form and part of speech                           |
| `no-short-topic-comma`      | Report | Flags a Japanese comma immediately after は when it marks a short topic                                   |

## Spacing and character forms {#spacing}

Set `preset-ja-spacing = true` to enable nine rules.
Enable the rules for spacing around emphasis and bold text, and the two character-form rules at the end of the table, by their individual IDs.

| Rule ID                                           | Action         | What it does                                                                                                                       |
| ------------------------------------------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `ja-nakaguro-or-halfwidth-space-between-katakana` | Report         | Flags full-width spaces between katakana words                                                                                     |
| `ja-no-space-around-parentheses`                  | Fix            | Removes spaces inside and outside the brackets covered by the rule                                                                 |
| `ja-no-space-around-slash`                        | Fix            | Removes spaces around slashes                                                                                                      |
| `ja-no-space-between-full-width`                  | Fix            | Removes half-width spaces between full-width characters                                                                            |
| `ja-space-between-half-and-full-width`            | Fix and report | Removes spaces between full-width characters and half-width letters or digits, or reports missing spaces, depending on the setting |
| `ja-space-after-exclamation`                      | Report         | Flags a half-width space after a full-width exclamation mark when another sentence follows                                         |
| `ja-space-after-question`                         | Report         | Flags a half-width space after a full-width question mark when another sentence follows                                            |
| `ja-space-around-code`                            | Fix and report | Checks spaces before and after inline code next to Japanese characters, using separate settings for each side                      |
| `ja-space-around-emphasis`                        | Fix and report | Checks spaces before and after emphasis next to Japanese characters, using separate settings for each side                         |
| `ja-space-around-link`                            | Fix and report | Checks spaces before and after links next to Japanese characters, using separate settings for each side                            |
| `ja-space-around-strong`                          | Fix and report | Checks spaces before and after bold text next to Japanese characters, using separate settings for each side                        |
| `no-mixed-zenkaku-and-hankaku-alphabet`           | Fix            | Standardizes Latin letters to the configured half-width or full-width form                                                         |
| `no-kangxi-radicals`                              | Fix            | Replaces Kangxi radical characters with their corresponding kanji                                                                  |

## JTF style guide {#jtf-style}

The `preset-jtf-style` group has 37 IDs. Enable these four individually: `2.1.2.漢字`, `2.1.5.カタカナ`, `2.1.6.カタカナの長音`, and `2.2.1.ひらがなと漢字の使い分け`.
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

| Rule ID                        | Action | What it does                                                                                  |
| ------------------------------ | ------ | --------------------------------------------------------------------------------------------- |
| `1.1.1.本文`                   | Report | Counts polite and plain forms in prose and flags the less frequent style when they are mixed  |
| `1.1.2.見出し`                 | Report | Flags Japanese full stops at the end of headings                                              |
| `1.1.3.箇条書き`               | Report | Flags inconsistent use of final Japanese full stops or mixed writing styles across list items |
| `1.1.5.図表のキャプション`     | Report | Flags a mixture of polite and plain styles in image alt text                                  |
| `1.2.1.句点(。)と読点(、)`     | Report | Flags half-width commas and periods used as Japanese punctuation                              |
| `1.2.2.ピリオド(.)とカンマ(,)` | Report | Flags full-width commas and periods                                                           |

### Characters and terms {#jtf-characters}

| Rule ID                            | Action         | What it does                                                               |
| ---------------------------------- | -------------- | -------------------------------------------------------------------------- |
| `2.1.2.漢字`                       | Report         | Flags kanji outside the jōyō kanji list                                    |
| `2.1.5.カタカナ`                   | Fix and report | Converts half-width kana to full-width and flags listed katakana spellings |
| `2.1.6.カタカナの長音`             | Report         | Checks the use of a final long vowel mark in listed katakana words         |
| `2.1.8.算用数字`                   | Fix            | Converts full-width Arabic numerals to half-width                          |
| `2.1.9.アルファベット`             | Fix            | Converts full-width Latin letters to half-width                            |
| `2.1.10.算用数字の位取りの表記`    | Report         | Flags commas used as decimal separators                                    |
| `2.2.1.ひらがなと漢字の使い分け`   | Report         | Flags kanji spellings such as 予め where the dictionary calls for hiragana |
| `2.2.2.算用数字と漢数字の使い分け` | Report         | Checks where to use Arabic numerals or kanji numerals                      |
| `2.2.3.一部の助数詞の表記`         | Report         | Flags listed counter spellings, such as ヶ月 after a number                |

### Spacing {#jtf-spacing}

| Rule ID                                          | Action         | What it does                                                                                            |
| ------------------------------------------------ | -------------- | ------------------------------------------------------------------------------------------------------- |
| `3.1.1.全角文字と半角文字の間`                   | Fix            | Removes half-width spaces between Japanese characters and half-width letters or digits                  |
| `3.1.2.全角文字どうし`                           | Fix            | Removes half-width spaces between full-width characters                                                 |
| `3.2.カタカナ語間のスペースの有無`               | Report         | Flags full-width spaces between katakana words                                                          |
| `3.3.かっこ類と隣接する文字の間のスペースの有無` | Fix and report | Removes spaces inside and outside brackets. An option can require spaces outside half-width parentheses |

### Symbols and brackets {#jtf-punctuation}

| Rule ID                        | Action | What it does                                                                                                                  |
| ------------------------------ | ------ | ----------------------------------------------------------------------------------------------------------------------------- |
| `4.1.1.句点(。)`               | Report | Flags Japanese full stops just before closing brackets                                                                        |
| `4.1.3.ピリオド(.)、カンマ(,)` | Report | Flags half-width periods used as full stops in Japanese prose                                                                 |
| `4.2.1.感嘆符(！)`             | Report | Flags half-width exclamation marks in Japanese prose and half-width spaces after sentence-ending full-width exclamation marks |
| `4.2.2.疑問符(？)`             | Report | Flags half-width question marks in Japanese prose and half-width spaces after sentence-ending full-width question marks       |
| `4.2.4.中黒(・)`               | Fix    | Converts half-width middle dots to full-width                                                                                 |
| `4.2.5.波線(〜)`               | Report | Flags half-width tildes in numeric ranges                                                                                     |
| `4.2.6.ハイフン(-)`            | Report | Flags hyphens in Japanese prose                                                                                               |
| `4.2.7.コロン(：)`             | Report | Flags half-width colons in Japanese prose                                                                                     |
| `4.2.8.セミコロン(;)`          | Report | Flags semicolons in Japanese prose                                                                                            |
| `4.2.9.ダッシュ(-)`            | Report | Flags dashes from U+2012 through U+2015 in Japanese prose                                                                     |
| `4.3.1.丸かっこ（）`           | Report | Flags half-width parentheses in Japanese prose                                                                                |
| `4.3.2.大かっこ［］`           | Report | Flags half-width square brackets in Japanese prose                                                                            |
| `4.3.3.かぎかっこ「」`         | Report | Flags unmatched Japanese corner brackets within a paragraph                                                                   |
| `4.3.4.二重かぎかっこ『』`     | Report | Flags unmatched double corner brackets within a paragraph                                                                     |
| `4.3.5.二重引用符`             | Report | Flags an odd number of ASCII double quotation marks in a paragraph                                                            |
| `4.3.6.中かっこ{ }`            | Report | Flags unmatched braces within a paragraph                                                                                     |
| `4.3.7.山かっこ<>`             | Report | Flags unmatched angle brackets within a paragraph, excluding arrows and operators such as `<=`                                |
| `4.3.8.一重引用符`             | None   | Accepts the preset's configuration key and produces no findings                                                               |

## Hiragana, forms with さ, and synonyms {#japanese-wording}

Enable each of these seven rules by its ID.
The hiragana rules use part of speech and nearby words to check how a word is used.

| Rule ID                              | Action | What it does                                                                          |
| ------------------------------------ | ------ | ------------------------------------------------------------------------------------- |
| `ja-hiragana-hojodoushi`             | Report | Flags auxiliary verbs written in kanji, such as 致し in お願い致します                |
| `ja-hiragana-keishikimeishi`         | Report | Flags formal nouns written in kanji, such as 事 in 見た事がある                       |
| `ja-keishikimeishi`                  | Report | Flags formal nouns written in kanji, with options to exclude ほう and うえ            |
| `ja-hiragana-fukushi`                | Report | Flags kanji spellings of adverbs normally written in hiragana, such as 既に           |
| `ja-hiragana-daimeishi`              | Report | Flags kanji spellings of pronouns normally written in hiragana, such as 何処          |
| `@textlint-ja/no-insert-dropping-sa` | Report | Flags an extra さ, as in 辛さそう, or a missing さ, as in 良そう                      |
| `@textlint-ja/no-synonyms`           | Report | Flags inconsistent forms for the same term in a document, such as サーバ and サーバー |

## Language-independent checks {#general}

| Rule ID                 | Action | What it does                                                                                                          |
| ----------------------- | ------ | --------------------------------------------------------------------------------------------------------------------- |
| `no-todo`               | Report | Flags `TODO:` and unchecked task items                                                                                |
| `date-weekday-mismatch` | Report | Flags dates followed by a weekday in parentheses that falls on a different day of the week                            |
| `no-dead-link`          | Report | Flags missing local link targets, failed URL requests, and redirects. External URLs are checked only by `jafix check` |

## English spelling and style {#english}

Enable `common-misspellings` on its own to check for frequently misspelled words.
If you already use `spelling`, set it to `false` to limit the check to the list.
When using the dictionary-based `spelling` rule, use `skipPatterns` for names and technical terms used in your project.

| Rule ID               | Action         | What it does                                                                                                                                                                          |
| --------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `terminology`         | Fix and report | Checks the spelling and capitalization of technical terms, brands, and abbreviations. Applies replacements from your `terms` setting and reports matches from the built-in dictionary |
| `spelling`            | Report         | Flags words outside the US English Hunspell dictionary and provides suggestions when configured                                                                                       |
| `common-misspellings` | Report         | Shows listed misspellings and their correct forms                                                                                                                                     |
| `write-good`          | Report         | Flags patterns such as vague words, passive voice, and redundant phrases                                                                                                              |

## Custom replacements and filters {#custom-rules-and-filters}

Use dictionaries and patterns under `[builtin]` to specify project terminology or expressions to flag.

| Rule ID                  | Action         | What it does                                                                                       |
| ------------------------ | -------------- | -------------------------------------------------------------------------------------------------- |
| `prh`                    | Fix and report | Applies replacements from the prh dictionaries specified in `rulePaths`                            |
| `@textlint-rule/pattern` | Fix and report | Applies the `replace` value to a match in `patterns` when provided and reports the match otherwise |

Under `[filters]`, `comments` excludes ranges marked by HTML comments, and `allowlist` excludes ranges that match specified patterns. These exclusions apply to fixes and findings.
