# Third-party notices

jafixのbinaryは、形態素解析のライブラリのlinderaと、linderaが埋め込むIPADICの辞書を含みます。linderaはMIT Licenseで、辞書の元のmecab-ipadicは「mecab-ipadic」の節の条件で公開されています。

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

`textlint-rule-no-kangxi-radicals` は、LICENSEのファイルを持たず、`package.json` に作者 `xl1` とMIT Licenseを書いています。`textlint-rule-ja-hiragana-hojodoushi`・`textlint-rule-ja-hiragana-keishikimeishi`・`textlint-rule-ja-hiragana-fukushi`・`textlint-rule-ja-hiragana-daimeishi` の4つも、LICENSEのファイルを持たず、`package.json` とREADMEに作者 `Hiroshi Takase` とMIT Licenseを書いています。`textlint-rule-ja-keishikimeishi` も同じく、作者 `Yosuke Ota` とMIT Licenseを書いています。5つのpackageの辞書(補助動詞・形式名詞・副詞・代名詞の語のトークンの並び)と、`@textlint-ja/textlint-rule-no-insert-dropping-sa` のさ入れ・さ抜きのパターンは、binaryに含まれます。

`chrono-node` は、`textlint-rule-date-weekday-mismatch` が日付を読むのに使うライブラリです。jafixは、`chrono-node` の日本語と英語の日付の書式の正規表現と、英語の月名と序数の語の表を写しています。`textlint-rule-no-dead-link` は、LICENSEのファイルを持たず、`package.json` に作者 `nodaguti` とMIT Licenseを、READMEにMIT Licenseを書いています。

## webpki-rootsの証明書の一覧

jafixのbinaryは、webpki-rootsの証明書の一覧を含みます。`no-dead-link` は、外部のURLへの接続でTLSの証明書を検証するのに、この一覧を使います。この一覧はCommunity Data License Agreement - Permissive - Version 2.0 (CDLA-Permissive-2.0)で公開されているデータで、改変せずに含みます。

| 名前         | version | ライセンス          |
| ------------ | ------- | ------------------- |
| webpki-roots | 1.0.12  | CDLA-Permissive-2.0 |

webpki-roots 1.0.12の `LICENSE` の全文です。

```text
# Community Data License Agreement - Permissive - Version 2.0

This is the Community Data License Agreement - Permissive, Version
2.0 (the "agreement"). Data Provider(s) and Data Recipient(s) agree
as follows:

## 1. Provision of the Data

1.1. A Data Recipient may use, modify, and share the Data made
available by Data Provider(s) under this agreement if that Data
Recipient follows the terms of this agreement.

1.2. This agreement does not impose any restriction on a Data
Recipient's use, modification, or sharing of any portions of the
Data that are in the public domain or that may be used, modified,
or shared under any other legal exception or limitation.

## 2. Conditions for Sharing Data

2.1. A Data Recipient may share Data, with or without modifications, so
long as the Data Recipient makes available the text of this agreement
with the shared Data.

## 3. No Restrictions on Results

3.1. This agreement does not impose any restriction or obligations
with respect to the use, modification, or sharing of Results.

## 4. No Warranty; Limitation of Liability

4.1. All Data Recipients receive the Data subject to the following
terms:

THE DATA IS PROVIDED ON AN "AS IS" BASIS, WITHOUT REPRESENTATIONS,
WARRANTIES OR CONDITIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED
INCLUDING, WITHOUT LIMITATION, ANY WARRANTIES OR CONDITIONS OF TITLE,
NON-INFRINGEMENT, MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.

NO DATA PROVIDER SHALL HAVE ANY LIABILITY FOR ANY DIRECT, INDIRECT,
INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING
WITHOUT LIMITATION LOST PROFITS), HOWEVER CAUSED AND ON ANY THEORY OF
LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE DATA OR RESULTS,
EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.

## 5. Definitions

5.1. "Data" means the material received by a Data Recipient under
this agreement.

5.2. "Data Provider" means any person who is the source of Data
provided under this agreement and in reliance on a Data Recipient's
agreement to its terms.

5.3. "Data Recipient" means any person who receives Data directly
or indirectly from a Data Provider and agrees to the terms of this
agreement.

5.4. "Results" means any outcome obtained by computational analysis
of Data, including for example machine learning models and models'
insights.
```

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
