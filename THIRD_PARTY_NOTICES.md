# Third-party notices

この文書は、jafixのbinaryに含まれる他者の著作物のうち、Rustのcrateのライセンスの一覧では示しきれないものの出典と条件を書いたものです。binaryに埋め込んだ辞書やデータ、textlintのルールから写した部分、crateの一覧に載らない著作権表示とソースの入手先を扱います。binaryにリンクされるすべてのRustのcrateの名前・version・ライセンスと、ライセンスの文面は、GitHub Releasesの配布物に入っている `CRATE_LICENSES.txt` にあります。`CRATE_LICENSES.txt` は、`scripts/package.sh` が梱包のときに、cargo-aboutで `Cargo.lock` と梱包するtargetの依存から作ります。

jafixのbinaryは、形態素解析のライブラリのlinderaと、linderaが埋め込むIPADICの辞書を含みます。linderaはMIT Licenseで、辞書の元のmecab-ipadicは「mecab-ipadic」の節の条件で公開されています。linderaのcrateのpackageはライセンスのファイルを含まないので、`CRATE_LICENSES.txt` にはMIT Licenseの標準の文面だけが載ります。linderaの著作権表示は、次の表に書いています。

| 名前         | version        | 著作権表示                                                                                      |
| ------------ | -------------- | ----------------------------------------------------------------------------------------------- |
| lindera      | 6.2.0          | Copyright (c) 2019 by the project authors.                                                      |
| mecab-ipadic | 2.7.0-20250920 | Copyright 2000, 2001, 2002, 2003 Nara Institute of Science and Technology. All Rights Reserved. |

jafixの組み込みのルールのうち、textlintと同じIDのものは、次のtextlintのルールとライブラリの挙動に合わせて書き直したものです。正規表現、語の表、辞書、判定の手順の一部を、これらのソースから写しています。どれもMIT Licenseで公開されています。

| package                                                       | version | 著作権表示                                   |
| ------------------------------------------------------------- | ------- | -------------------------------------------- |
| sentence-splitter                                             | 5.0.1   | Copyright (c) 2015 azu                       |
| textlint-rule-sentence-length                                 | 5.2.1   | Copyright (c) 2015 azu                       |
| textlint-rule-max-comma                                       | 4.0.0   | Copyright (c) 2016 azu                       |
| textlint-rule-max-kanji-continuous-len                        | 1.1.1   | Copyright (c) 2016 azu                       |
| textlint-rule-preset-jtf-style                                | 3.0.3   | Copyright (c) 2015 azu                       |
| analyze-desumasu-dearu                                        | 2.1.5   | Copyright (c) 2015 azu                       |
| moji                                                          | 0.5.1   | Copyright (c) 2015 niwaringo                 |
| textlint-rule-ja-no-mixed-period                              | 3.0.2   | Copyright (c) 2016 azu                       |
| check-ends-with-period                                        | 3.0.2   | Copyright (c) 2017 azu                       |
| textlint-rule-no-nfd                                          | 2.0.2   | Copyright (c) 2016 azu                       |
| @textlint-rule/textlint-rule-no-invalid-control-character     | 3.0.0   | Copyright (c) 2017 azu                       |
| textlint-rule-no-zero-width-spaces                            | 1.0.1   | Copyright (c) 2021 Tomoyuki Hata             |
| textlint-rule-no-exclamation-question-mark                    | 1.1.0   | Copyright (c) 2016 azu                       |
| textlint-rule-no-hankaku-kana                                 | 2.0.1   | Copyright (c) 2016 azu                       |
| textlint-rule-ja-unnatural-alphabet                           | 2.0.1   | Copyright (c) 2017 azu                       |
| @textlint-rule/textlint-rule-no-unmatched-pair                | 2.0.4   | Copyright (c) 2018 azu                       |
| textlint-rule-max-ten                                         | 5.0.0   | Copyright (c) 2015 azu                       |
| textlint-rule-no-doubled-joshi                                | 5.1.1   | Copyright (c) 2015 azu                       |
| textlint-rule-no-doubled-conjunctive-particle-ga              | 3.0.0   | Copyright (c) 2016 takahashim                |
| textlint-rule-no-doubled-conjunction                          | 3.0.1   | Copyright (c) 2016 takahashim                |
| textlint-rule-no-mix-dearu-desumasu                           | 6.0.4   | Copyright (c) 2015 azu                       |
| analyze-desumasu-dearu                                        | 5.0.2   | Copyright (c) 2015 azu                       |
| textlint-rule-no-double-negative-ja                           | 2.0.1   | Copyright (c) 2015 azu                       |
| textlint-rule-no-dropping-the-ra                              | 3.0.0   | Copyright (c) 2015 azu                       |
| textlint-rule-ja-no-weak-phrase                               | 2.0.0   | Copyright (c) 2016 azu                       |
| textlint-rule-ja-no-successive-word                           | 2.0.1   | Copyright (c) 2016 azu                       |
| textlint-rule-ja-no-abusage                                   | 3.0.0   | Copyright (c) 2016 azu                       |
| textlint-rule-ja-no-redundant-expression                      | 4.0.1   | Copyright (c) 2016 azu                       |
| morpheme-match                                                | 2.0.4   | Copyright (c) 2016 azu                       |
| @textlint-ja/textlint-rule-preset-ai-writing                  | 1.7.0   | Copyright (c) 2025 azu                       |
| textlint-rule-preset-ai-words-ja                              | 1.2.1   | Copyright (c) 2026 p1ass                     |
| textlint-rule-prh                                             | 6.1.1   | Copyright (c) 2015 azu                       |
| prh                                                           | 6.1.0   | Copyright (c) 2026 vvakame                   |
| @textlint-rule/textlint-rule-pattern                          | 2.0.2   | Copyright (c) 2022 azu                       |
| @textlint/regexp-string-matcher                               | 2.0.2   | Copyright (c) 2018 azu                       |
| textlint-rule-preset-ja-spacing                               | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-nakaguro-or-halfwidth-space-between-katakana | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-no-space-around-parentheses                  | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-no-space-around-slash                        | 3.0.4   | Copyright (c) azu                            |
| textlint-rule-ja-no-space-between-full-width                  | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-space-between-half-and-full-width            | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-space-after-exclamation                      | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-space-after-question                         | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-space-around-code                            | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-space-around-emphasis                        | 3.0.4   | Copyright (c) azu                            |
| textlint-rule-ja-space-around-link                            | 3.0.4   | Copyright (c) 2024 azu                       |
| textlint-rule-ja-space-around-strong                          | 3.0.4   | Copyright (c) azu                            |
| textlint-rule-no-mixed-zenkaku-and-hankaku-alphabet           | 1.0.1   | Copyright (c) 2017 azu                       |
| textlint-rule-no-kangxi-radicals                              | 0.2.2   | 著作権表示なし(作者は xl1)                   |
| textlint-rule-ja-hiragana-hojodoushi                          | 1.1.0   | 著作権表示なし(作者は Hiroshi Takase)        |
| textlint-rule-ja-hiragana-keishikimeishi                      | 1.1.0   | 著作権表示なし(作者は Hiroshi Takase)        |
| textlint-rule-ja-keishikimeishi                               | 1.0.5   | 著作権表示なし(作者は Yosuke Ota)            |
| textlint-rule-ja-hiragana-fukushi                             | 1.3.0   | 著作権表示なし(作者は Hiroshi Takase)        |
| textlint-rule-ja-hiragana-daimeishi                           | 1.0.0   | 著作権表示なし(作者は Hiroshi Takase)        |
| @textlint-ja/textlint-rule-no-insert-dropping-sa              | 2.0.1   | Copyright (c) 2017 azu                       |
| @textlint-ja/textlint-rule-no-synonyms                        | 1.3.0   | Copyright (c) 2019 azu                       |
| textlint-filter-rule-comments                                 | 1.3.0   | Copyright (c) 2016 azu                       |
| textlint-filter-rule-allowlist                                | 4.0.0   | Copyright (c) 2016 azu                       |
| textlint-rule-no-todo                                         | 2.0.1   | Copyright (c) 2015 azu                       |
| textlint-rule-date-weekday-mismatch                           | 1.1.1   | Copyright (c) 2016 azu                       |
| chrono-node                                                   | 1.4.9   | Copyright (c) 2014, Wanasit Tanakitrungruang |
| textlint-rule-no-dead-link                                    | 6.2.3   | 著作権表示なし(作者は nodaguti)              |
| textlint-rule-terminology                                     | 5.2.16  | Copyright 2019 Artem Sapegin, contributors   |
| textlint-rule-write-good                                      | 2.0.0   | 著作権表示なし(作者は nodaguti)              |
| write-good                                                    | 1.0.8   | Copyright (c) 2014-2019 Brian Ford           |
| weasel-words                                                  | 0.1.1   | 著作権表示なし(作者は Brian Ford)            |
| passive-voice                                                 | 0.1.0   | 著作権表示なし(作者は Brian Ford)            |
| adverb-where                                                  | 0.2.6   | Copyright (c) 2014 Matt Blair                |
| too-wordy                                                     | 0.3.6   | Copyright (c) 2014 Matt Blair                |
| no-cliches                                                    | 0.3.6   | Copyright (c) 2021 Matt Blair                |
| e-prime                                                       | 0.10.4  | Copyright (c) 2015 Paul J. Miller            |

`textlint-rule-no-kangxi-radicals` は、LICENSEのファイルを持たず、`package.json` に作者 `xl1` とMIT Licenseを書いています。`textlint-rule-ja-hiragana-hojodoushi`・`textlint-rule-ja-hiragana-keishikimeishi`・`textlint-rule-ja-hiragana-fukushi`・`textlint-rule-ja-hiragana-daimeishi` の4つも、LICENSEのファイルを持たず、`package.json` とREADMEに作者 `Hiroshi Takase` とMIT Licenseを書いています。`textlint-rule-ja-keishikimeishi` も同じく、作者 `Yosuke Ota` とMIT Licenseを書いています。5つのpackageの辞書(補助動詞・形式名詞・副詞・代名詞の語のトークンの並び)と、`@textlint-ja/textlint-rule-no-insert-dropping-sa` のさ入れ・さ抜きのパターンは、binaryに含まれます。

`chrono-node` は、`textlint-rule-date-weekday-mismatch` が日付を読むのに使うライブラリです。jafixは、`chrono-node` の日本語と英語の日付の書式の正規表現と、英語の月名と序数の語の表を写しています。`textlint-rule-no-dead-link` は、LICENSEのファイルを持たず、`package.json` に作者 `nodaguti` とMIT Licenseを、READMEにMIT Licenseを書いています。

`textlint-rule-terminology` の辞書 `terms.jsonc` は、binaryに含まれます。`write-good` から `e-prime` までの7つは、`textlint-rule-write-good` が使う `write-good` と、その6つの単語リストのpackageです。jafixは、`write-good` の正規表現と、6つのpackageの単語リストを写し、binaryに埋め込んでいます。`textlint-rule-write-good`・`weasel-words`・`passive-voice` の3つは、LICENSEのファイルを持ちません。`textlint-rule-write-good` は `package.json` とReadMe.mdに作者 `nodaguti` とMIT Licenseを書いています。`weasel-words` と `passive-voice` は、`package.json` に作者 `Brian Ford` とMIT Licenseを書いています。

## webpki-rootsの証明書の一覧

jafixのbinaryは、webpki-rootsの証明書の一覧を含みます。`no-dead-link` は、外部のURLへの接続でTLSの証明書を検証するのに、この一覧を使います。この一覧はCommunity Data License Agreement - Permissive - Version 2.0 (CDLA-Permissive-2.0)で公開されているデータで、改変せずに含みます。

| 名前         | version | ライセンス          |
| ------------ | ------- | ------------------- |
| webpki-roots | 1.0.12  | CDLA-Permissive-2.0 |

CDLA-Permissive-2.0は、データを共有するときに、その全文を一緒に示すことを条件にしています。webpki-rootsはcrateなので、webpki-roots 1.0.12の `LICENSE` の全文は、`CRATE_LICENSES.txt` に載っています。

## Sudachiの同義語辞書

jafixのbinaryは、`@textlint-ja/no-synonyms` が使う同義語辞書を含みます。この辞書は、sudachi-synonyms-dictionary 20.0.0の `sudachi-synonyms-dictionary.json` から作ったもので、そのJSONは、SudachiDictの同義語辞書 `synonyms.txt` から作られています。どちらもApache License 2.0で公開されています。Apache License 2.0の全文は、repositoryの [LICENSE-APACHE](LICENSE-APACHE) にあります。

| 名前                        | version | 著作権表示                                           |
| --------------------------- | ------- | ---------------------------------------------------- |
| sudachi-synonyms-dictionary | 20.0.0  | Copyright 2019 azu                                   |
| SudachiDict                 | -       | Copyright (c) 2017-2023 Works Applications Co., Ltd. |

jafixはJSONから `@textlint-ja/no-synonyms` の判定に使う項目(同義語のグループ・語彙番号・英字の表記かどうか・見出し)だけを取り出しました。取り出した項目は、1行が1つのグループのTSVにして `src/rules/morph/no_synonyms/sudachi-synonyms.tsv` に置いています。ファイルの先頭の行にも、出典・version・ライセンスとこの変更を書いています。build時に、このTSVから見出しで引く表を作り、binaryに埋め込みます。

## JTF日本語標準スタイルガイドの辞書

jafixのbinaryは、textlint-rule-preset-jtf-style 3.0.3の辞書の4つのファイル `dict/2.1.5.yml`・`dict/2.1.6.yml`・`dict/2.2.1.yml`・`dict/2.2.3.yml` を、改変せずに含みます。packageのREADMEは、JTF日本語標準スタイルガイドに由来する部分を次のように表示しています。

```text
by Japan Translation Federation (CC BY-SA) www.jtf.jp
本著作物は「JTF日本語標準スタイルガイド2.0」(JTF, CC BY-SA)を改変して作成したものです。
```

4つの辞書は、このCC BY-SA(表示 - 継承)の条件で利用しています。CC BY-SAのversionは、packageのREADMEにもJTFのページにも書かれていません。条件は次の2つです。

- 表示: 原著作者がJapan Translation Federation(JTF)であることと、CC BY-SAで公開されていることを示す
- 継承: 辞書を改変して配布するときは、同じCC BY-SAの条件で公開する

packageのREADMEは、辞書以外のコードをMIT Licenseとしています。`2.1.2.漢字` が使う常用漢字の表は、packageの `src/2.1.2.js` から写したコードで、MIT Licenseの部分です。

## Wikipediaの誤綴りの一覧

jafixのbinaryは、`common-misspellings` が使う誤綴りの一覧(4,310件)を含みます。この一覧は、Wikipediaのページ「Wikipedia:Lists of common misspellings/For machines」のrevision 1199637275 (2024-01-27保存)から作りました。このrevisionのURLは `https://en.wikipedia.org/w/index.php?title=Wikipedia:Lists_of_common_misspellings/For_machines&oldid=1199637275` です。ページの文章は、CC BY-SA 3.0 (`https://creativecommons.org/licenses/by-sa/3.0/`)で公開されています。

jafixは、誤綴りを小文字にし、前後の空白を除き、1行1項目の「誤綴り<タブ>正しい語,正しい語…」の形にしました。変えた一覧は `src/rules/english/data/common-misspellings.txt` に置き、ファイルの先頭の行にも、出典・revisionのid・ライセンスとこの変更を書いています。一覧は、CC BY-SA 3.0(表示 - 継承)の条件で利用しています。条件は次の2つです。

- 表示: 出典のページと作者(ページの履歴にあるWikipediaの編集者)を示し、CC BY-SA 3.0で公開されていることと、変更したことを示す
- 継承: 一覧を改変して配布するときは、同じCC BY-SA 3.0の条件で公開する

## en_USのhunspellの辞書

jafixのbinaryは、`spelling` が使うen_USのhunspellの辞書を含みます。この辞書は、dictionary-en 4.0.0の `index.aff` と `index.dic` を、中身を変えずに `src/rules/english/data/en_US.aff` と `src/rules/english/data/en_US.dic` に写したものです。dictionary-enの `package.json` は、ライセンスを `(MIT AND BSD)` としています。辞書はSCOWLから作られ、SCOWLの条件と、affixのファイルの元になったGeoff KuenningのIspellのBSDの条件で公開されています。

| 名前          | version | 著作権表示                                                                 |
| ------------- | ------- | -------------------------------------------------------------------------- |
| dictionary-en | 4.0.0   | Copyright 2000-2018 by Kevin Atkinson、Copyright 1993, Geoff Kuenning ほか |

dictionary-en 4.0.0の `license` のファイルの全文です。

```text
en_US Hunspell Dictionary
Version 2020.12.07
Mon Dec 7 20:14:35 2020 -0500 [5ef55f9]
http://wordlist.sourceforge.net

README file for English Hunspell dictionaries derived from SCOWL.

These dictionaries are created using the speller/make-hunspell-dict
script in SCOWL.

The following dictionaries are available:

  en_US (American)
  en_CA (Canadian)
  en_GB-ise (British with "ise" spelling)
  en_GB-ize (British with "ize" spelling)
  en_AU (Australian)

  en_US-large
  en_CA-large
  en_GB-large (with both "ise" and "ize" spelling)
  en_AU-large

The normal (non-large) dictionaries correspond to SCOWL size 60 and,
to encourage consistent spelling, generally only include one spelling
variant for a word.  The large dictionaries correspond to SCOWL size
70 and may include multiple spelling for a word when both variants are
considered almost equal.  The larger dictionaries however (1) have not
been as carefully checked for errors as the normal dictionaries and
thus may contain misspelled or invalid words; and (2) contain
uncommon, yet valid, words that might cause problems as they are
likely to be misspellings of more common words (for example, "ort" and
"calender").

To get an idea of the difference in size, here are 25 random words
only found in the large dictionary for American English:

  Bermejo Freyr's Guenevere Hatshepsut Nottinghamshire arrestment
  crassitudes crural dogwatches errorless fetial flaxseeds godroon
  incretion jalapeño's kelpie kishkes neuroglias pietisms pullulation
  stemwinder stenoses syce thalassic zees

The en_US, en_CA and en_AU are the official dictionaries for Hunspell.
The en_GB and large dictionaries are made available on an experimental
basis.  If you find them useful please send me a quick email at
kevina@gnu.org.

If none of these dictionaries suite you (for example, maybe you want
the normal dictionary that also includes common variants) additional
dictionaries can be generated at http://app.aspell.net/create or by
modifying speller/make-hunspell-dict in SCOWL.  Please do let me know
if you end up publishing a customized dictionary.

If a word is not found in the dictionary or a word is there you think
shouldn't be, you can lookup the word up at http://app.aspell.net/lookup
to help determine why that is.

General comments on these list can be sent directly to me at
kevina@gnu.org or to the wordlist-devel mailing lists
(https://lists.sourceforge.net/lists/listinfo/wordlist-devel).  If you
have specific issues with any of these dictionaries please file a bug
report at https://github.com/kevina/wordlist/issues.

IMPORTANT CHANGES INTRODUCED In 2016.11.20:

New Australian dictionaries thanks to the work of Benjamin Titze
(btitze@protonmail.ch).

IMPORTANT CHANGES INTRODUCED IN 2016.04.24:

The dictionaries are now in UTF-8 format instead of ISO-8859-1.  This
was required to handle smart quotes correctly.

IMPORTANT CHANGES INTRODUCED IN 2016.01.19:

"SET UTF8" was changes to "SET UTF-8" in the affix file as some
versions of Hunspell do not recognize "UTF8".

ADDITIONAL NOTES:

The NOSUGGEST flag was added to certain taboo words.  While I made an
honest attempt to flag the strongest taboo words with the NOSUGGEST
flag, I MAKE NO GUARANTEE THAT I FLAGGED EVERY POSSIBLE TABOO WORD.
The list was originally derived from Németh László, however I removed
some words which, while being considered taboo by some dictionaries,
are not really considered swear words in today's society.

COPYRIGHT, SOURCES, and CREDITS:

The English dictionaries come directly from SCOWL
and is thus under the same copyright of SCOWL.  The affix file is
a heavily modified version of the original english.aff file which was
released as part of Geoff Kuenning's Ispell and as such is covered by
his BSD license.  Part of SCOWL is also based on Ispell thus the
Ispell copyright is included with the SCOWL copyright.

The collective work is Copyright 2000-2018 by Kevin Atkinson as well
as any of the copyrights mentioned below:

  Copyright 2000-2018 by Kevin Atkinson

  Permission to use, copy, modify, distribute and sell these word
  lists, the associated scripts, the output created from the scripts,
  and its documentation for any purpose is hereby granted without fee,
  provided that the above copyright notice appears in all copies and
  that both that copyright notice and this permission notice appear in
  supporting documentation. Kevin Atkinson makes no representations
  about the suitability of this array for any purpose. It is provided
  "as is" without express or implied warranty.

Alan Beale <biljir@pobox.com> also deserves special credit as he has,
in addition to providing the 12Dicts package and being a major
contributor to the ENABLE word list, given me an incredible amount of
feedback and created a number of special lists (those found in the
Supplement) in order to help improve the overall quality of SCOWL.

The 10 level includes the 1000 most common English words (according to
the Moby (TM) Words II [MWords] package), a subset of the 1000 most
common words on the Internet (again, according to Moby Words II), and
frequently class 16 from Brian Kelk's "UK English Wordlist
with Frequency Classification".

The MWords package was explicitly placed in the public domain:

    The Moby lexicon project is complete and has
    been place into the public domain. Use, sell,
    rework, excerpt and use in any way on any platform.

    Placing this material on internal or public servers is
    also encouraged. The compiler is not aware of any
    export restrictions so freely distribute world-wide.

    You can verify the public domain status by contacting

    Grady Ward
    3449 Martha Ct.
    Arcata, CA  95521-4884

    grady@netcom.com
    grady@northcoast.com

The "UK English Wordlist With Frequency Classification" is also in the
Public Domain:

  Date: Sat, 08 Jul 2000 20:27:21 +0100
  From: Brian Kelk <Brian.Kelk@cl.cam.ac.uk>

  > I was wondering what the copyright status of your "UK English
  > Wordlist With Frequency Classification" word list as it seems to
  > be lacking any copyright notice.

  There were many many sources in total, but any text marked
  "copyright" was avoided. Locally-written documentation was one
  source. An earlier version of the list resided in a filespace called
  PUBLIC on the University mainframe, because it was considered public
  domain.

  Date: Tue, 11 Jul 2000 19:31:34 +0100

  > So are you saying your word list is also in the public domain?

  That is the intention.

The 20 level includes frequency classes 7-15 from Brian's word list.

The 35 level includes frequency classes 2-6 and words appearing in at
least 11 of 12 dictionaries as indicated in the 12Dicts package.  All
words from the 12Dicts package have had likely inflections added via
my inflection database.

The 12Dicts package and Supplement is in the Public Domain.

The WordNet database, which was used in the creation of the
Inflections database, is under the following copyright:

  This software and database is being provided to you, the LICENSEE,
  by Princeton University under the following license.  By obtaining,
  using and/or copying this software and database, you agree that you
  have read, understood, and will comply with these terms and
  conditions.:

  Permission to use, copy, modify and distribute this software and
  database and its documentation for any purpose and without fee or
  royalty is hereby granted, provided that you agree to comply with
  the following copyright notice and statements, including the
  disclaimer, and that the same appear on ALL copies of the software,
  database and documentation, including modifications that you make
  for internal use or for distribution.

  WordNet 1.6 Copyright 1997 by Princeton University.  All rights
  reserved.

  THIS SOFTWARE AND DATABASE IS PROVIDED "AS IS" AND PRINCETON
  UNIVERSITY MAKES NO REPRESENTATIONS OR WARRANTIES, EXPRESS OR
  IMPLIED.  BY WAY OF EXAMPLE, BUT NOT LIMITATION, PRINCETON
  UNIVERSITY MAKES NO REPRESENTATIONS OR WARRANTIES OF MERCHANT-
  ABILITY OR FITNESS FOR ANY PARTICULAR PURPOSE OR THAT THE USE OF THE
  LICENSED SOFTWARE, DATABASE OR DOCUMENTATION WILL NOT INFRINGE ANY
  THIRD PARTY PATENTS, COPYRIGHTS, TRADEMARKS OR OTHER RIGHTS.

  The name of Princeton University or Princeton may not be used in
  advertising or publicity pertaining to distribution of the software
  and/or database.  Title to copyright in this software, database and
  any associated documentation shall at all times remain with
  Princeton University and LICENSEE agrees to preserve same.

The 40 level includes words from Alan's 3esl list found in version 4.0
of his 12dicts package.  Like his other stuff the 3esl list is also in the
public domain.

The 50 level includes Brian's frequency class 1, words appearing
in at least 5 of 12 of the dictionaries as indicated in the 12Dicts
package, and uppercase words in at least 4 of the previous 12
dictionaries.  A decent number of proper names is also included: The
top 1000 male, female, and Last names from the 1990 Census report; a
list of names sent to me by Alan Beale; and a few names that I added
myself.  Finally a small list of abbreviations not commonly found in
other word lists is included.

The name files form the Census report is a government document which I
don't think can be copyrighted.

The file special-jargon.50 uses common.lst and word.lst from the
"Unofficial Jargon File Word Lists" which is derived from "The Jargon
File".  All of which is in the Public Domain.  This file also contain
a few extra UNIX terms which are found in the file "unix-terms" in the
special/ directory.

The 55 level includes words from Alan's 2of4brif list found in version
4.0 of his 12dicts package.  Like his other stuff the 2of4brif is also
in the public domain.

The 60 level includes all words appearing in at least 2 of the 12
dictionaries as indicated by the 12Dicts package.

The 70 level includes Brian's frequency class 0 and the 74,550 common
dictionary words from the MWords package.  The common dictionary words,
like those from the 12Dicts package, have had all likely inflections
added.  The 70 level also included the 5desk list from version 4.0 of
the 12Dics package which is in the public domain.

The 80 level includes the ENABLE word list, all the lists in the
ENABLE supplement package (except for ABLE), the "UK Advanced Cryptics
Dictionary" (UKACD), the list of signature words from the YAWL package,
and the 10,196 places list from the MWords package.

The ENABLE package, mainted by M\Cooper <thegrendel@theriver.com>,
is in the Public Domain:

  The ENABLE master word list, WORD.LST, is herewith formally released
  into the Public Domain. Anyone is free to use it or distribute it in
  any manner they see fit. No fee or registration is required for its
  use nor are "contributions" solicited (if you feel you absolutely
  must contribute something for your own peace of mind, the authors of
  the ENABLE list ask that you make a donation on their behalf to your
  favorite charity). This word list is our gift to the Scrabble
  community, as an alternate to "official" word lists. Game designers
  may feel free to incorporate the WORD.LST into their games. Please
  mention the source and credit us as originators of the list. Note
  that if you, as a game designer, use the WORD.LST in your product,
  you may still copyright and protect your product, but you may *not*
  legally copyright or in any way restrict redistribution of the
  WORD.LST portion of your product. This *may* under law restrict your
  rights to restrict your users' rights, but that is only fair.

UKACD, by J Ross Beresford <ross@bryson.demon.co.uk>, is under the
following copyright:

  Copyright (c) J Ross Beresford 1993-1999. All Rights Reserved.

  The following restriction is placed on the use of this publication:
  if The UK Advanced Cryptics Dictionary is used in a software package
  or redistributed in any form, the copyright notice must be
  prominently displayed and the text of this document must be included
  verbatim.

  There are no other restrictions: I would like to see the list
  distributed as widely as possible.

The 95 level includes the 354,984 single words, 256,772 compound
words, 4,946 female names and the 3,897 male names, and 21,986 names
from the MWords package, ABLE.LST from the ENABLE Supplement, and some
additional words found in my part-of-speech database that were not
found anywhere else.

Accent information was taken from UKACD.

The VarCon package was used to create the American, British, Canadian,
and Australian word list.  It is under the following copyright:

  Copyright 2000-2016 by Kevin Atkinson

  Permission to use, copy, modify, distribute and sell this array, the
  associated software, and its documentation for any purpose is hereby
  granted without fee, provided that the above copyright notice appears
  in all copies and that both that copyright notice and this permission
  notice appear in supporting documentation. Kevin Atkinson makes no
  representations about the suitability of this array for any
  purpose. It is provided "as is" without express or implied warranty.

  Copyright 2016 by Benjamin Titze

  Permission to use, copy, modify, distribute and sell this array, the
  associated software, and its documentation for any purpose is hereby
  granted without fee, provided that the above copyright notice appears
  in all copies and that both that copyright notice and this permission
  notice appear in supporting documentation. Benjamin Titze makes no
  representations about the suitability of this array for any
  purpose. It is provided "as is" without express or implied warranty.

  Since the original words lists come from the Ispell distribution:

  Copyright 1993, Geoff Kuenning, Granada Hills, CA
  All rights reserved.

  Redistribution and use in source and binary forms, with or without
  modification, are permitted provided that the following conditions
  are met:

  1. Redistributions of source code must retain the above copyright
     notice, this list of conditions and the following disclaimer.
  2. Redistributions in binary form must reproduce the above copyright
     notice, this list of conditions and the following disclaimer in the
     documentation and/or other materials provided with the distribution.
  3. All modifications to the source code must be clearly marked as
     such.  Binary redistributions based on modified source code
     must be clearly marked as modified versions in the documentation
     and/or other materials provided with the distribution.
  (clause 4 removed with permission from Geoff Kuenning)
  5. The name of Geoff Kuenning may not be used to endorse or promote
     products derived from this software without specific prior
     written permission.

  THIS SOFTWARE IS PROVIDED BY GEOFF KUENNING AND CONTRIBUTORS ``AS IS'' AND
  ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
  IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
  ARE DISCLAIMED.  IN NO EVENT SHALL GEOFF KUENNING OR CONTRIBUTORS BE LIABLE
  FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
  DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
  OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
  HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
  LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
  OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
  SUCH DAMAGE.

Build Date: Mon Dec  7 20:19:27 EST 2020
Wordlist Command: mk-list --accents=strip en_US 60
```

## spellbook

jafixのbinaryは、`spelling` が辞書を引くのに使うRustのcrateのspellbookを含みます。spellbookは、Mozilla Public License 2.0 (MPL-2.0)で公開されています。MPL-2.0は、実行形式で配るときに、そのソースの入手先を示すことを条件にしています。この条件は `CRATE_LICENSES.txt` の一覧と文面だけでは満たせないので、入手先をここに書きます。jafixは、spellbookのソースを変えずに使っています。

| 名前      | version | ライセンス | ソースの入手先                            |
| --------- | ------- | ---------- | ----------------------------------------- |
| spellbook | 0.4.2   | MPL-2.0    | https://github.com/helix-editor/spellbook |

spellbook 0.4.2のソースは、https://crates.io/crates/spellbook/0.4.2 からも入手できます。MPL-2.0の全文は、`CRATE_LICENSES.txt` と https://mozilla.org/MPL/2.0/ にあります。

## MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## mecab-ipadic

mecab-ipadic-2.7.0-20250920の `COPYING` の全文です。

Copyright 2000, 2001, 2002, 2003 Nara Institute of Science and Technology. All Rights Reserved.

Use, reproduction, and distribution of this software is permitted. Any copy of this software, whether in its original form or modified, must include both the above copyright notice and the following paragraphs.

Nara Institute of Science and Technology (NAIST), the copyright holders, disclaims all warranties with regard to this software, including all implied warranties of merchantability and fitness, in no event shall NAIST be liable for any special, indirect or consequential damages or any damages whatsoever resulting from loss of use, data or profits, whether in an action of contract, negligence or other tortuous action, arising out of or in connection with the use or performance of this software.

A large portion of the dictionary entries originate from ICOT Free Software. The following conditions for ICOT Free Software applies to the current dictionary as well.

Each User may also freely distribute the Program, whether in its original form or modified, to any third party or parties, PROVIDED that the provisions of Section 3 ("NO WARRANTY") will ALWAYS appear on, or be attached to, the Program, which is distributed substantially in the same form as set out herein and that such intended distribution, if actually made, will neither violate or otherwise contravene any of the laws and regulations of the countries having jurisdiction over the User or the intended distribution itself.

NO WARRANTY

The program was produced on an experimental basis in the course of the research and development conducted during the project and is provided to users as so produced on an experimental basis. Accordingly, the program is provided without any warranty whatsoever, whether express, implied, statutory or otherwise. The term "warranty" used herein includes, but is not limited to, any warranty of the quality, performance, merchantability and fitness for a particular purpose of the program and the nonexistence of any infringement or violation of any right of any third party.

Each user of the program will agree and understand, and be deemed to have agreed and understood, that there is no warranty whatsoever for the program and, accordingly, the entire risk arising from or otherwise connected with the program is assumed by the user.

Therefore, neither ICOT, the copyright holder, or any other organization that participated in or was otherwise related to the development of the program and their respective officials, directors, officers and other employees shall be held liable for any and all damages, including, without limitation, general, special, incidental and consequential damages, arising out of or otherwise in connection with the use or inability to use the program or any product, material or result produced or otherwise obtained by using the program, regardless of whether they have been advised of, or otherwise had knowledge of, the possibility of such damages at any time during the project or thereafter. Each user will be deemed to have agreed to the foregoing by his or her commencement of use of the program. The term "use" as used herein includes, but is not limited to, the use, modification, copying and distribution of the program and the production of secondary products from the program.

In the case where the program, whether in its original form or modified, was distributed or delivered to or received by a user from any person, organization or entity other than ICOT, unless it makes or grants independently of ICOT any specific warranty to the user in writing, such person, organization or entity, will also be exempted from and not be held liable to the user for any such damages as noted above as far as the program is concerned.
