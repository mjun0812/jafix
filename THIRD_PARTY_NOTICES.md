# Third-party notices

jafixのbinaryは、形態素解析のライブラリのlinderaと、linderaが埋め込むIPADICの辞書を含みます。linderaはMIT Licenseで、辞書の元のmecab-ipadicは「mecab-ipadic」の節の条件で公開されています。

| 名前         | version        | 著作権表示                                                                                      |
| ------------ | -------------- | ----------------------------------------------------------------------------------------------- |
| lindera      | 6.2.0          | Copyright (c) 2019 by the project authors.                                                      |
| mecab-ipadic | 2.7.0-20250920 | Copyright 2000, 2001, 2002, 2003 Nara Institute of Science and Technology. All Rights Reserved. |

jafixの組み込みのルールのうち、textlintと同じIDのものは、次のtextlintのルールとライブラリの挙動に合わせて書き直したものです。正規表現、語の表、辞書、判定の手順の一部を、これらのソースから写しています。どれもMIT Licenseで公開されています。

| package                                                       | version | 著作権表示                       |
| ------------------------------------------------------------- | ------- | -------------------------------- |
| sentence-splitter                                             | 5.0.1   | Copyright (c) 2015 azu           |
| textlint-rule-sentence-length                                 | 5.2.1   | Copyright (c) 2015 azu           |
| textlint-rule-max-comma                                       | 4.0.0   | Copyright (c) 2016 azu           |
| textlint-rule-max-kanji-continuous-len                        | 1.1.1   | Copyright (c) 2016 azu           |
| textlint-rule-preset-jtf-style                                | 3.0.3   | Copyright (c) 2015 azu           |
| analyze-desumasu-dearu                                        | 2.1.5   | Copyright (c) 2015 azu           |
| moji                                                          | 0.5.1   | Copyright (c) 2015 niwaringo     |
| textlint-rule-ja-no-mixed-period                              | 3.0.2   | Copyright (c) 2016 azu           |
| check-ends-with-period                                        | 3.0.2   | Copyright (c) 2017 azu           |
| textlint-rule-no-nfd                                          | 2.0.2   | Copyright (c) 2016 azu           |
| @textlint-rule/textlint-rule-no-invalid-control-character     | 3.0.0   | Copyright (c) 2017 azu           |
| textlint-rule-no-zero-width-spaces                            | 1.0.1   | Copyright (c) 2021 Tomoyuki Hata |
| textlint-rule-no-exclamation-question-mark                    | 1.1.0   | Copyright (c) 2016 azu           |
| textlint-rule-no-hankaku-kana                                 | 2.0.1   | Copyright (c) 2016 azu           |
| textlint-rule-ja-unnatural-alphabet                           | 2.0.1   | Copyright (c) 2017 azu           |
| @textlint-rule/textlint-rule-no-unmatched-pair                | 2.0.4   | Copyright (c) 2018 azu           |
| textlint-rule-max-ten                                         | 5.0.0   | Copyright (c) 2015 azu           |
| textlint-rule-no-doubled-joshi                                | 5.1.1   | Copyright (c) 2015 azu           |
| textlint-rule-no-doubled-conjunctive-particle-ga              | 3.0.0   | Copyright (c) 2016 takahashim    |
| textlint-rule-no-doubled-conjunction                          | 3.0.1   | Copyright (c) 2016 takahashim    |
| textlint-rule-no-mix-dearu-desumasu                           | 6.0.4   | Copyright (c) 2015 azu           |
| analyze-desumasu-dearu                                        | 5.0.2   | Copyright (c) 2015 azu           |
| textlint-rule-no-double-negative-ja                           | 2.0.1   | Copyright (c) 2015 azu           |
| textlint-rule-no-dropping-the-ra                              | 3.0.0   | Copyright (c) 2015 azu           |
| textlint-rule-ja-no-weak-phrase                               | 2.0.0   | Copyright (c) 2016 azu           |
| textlint-rule-ja-no-successive-word                           | 2.0.1   | Copyright (c) 2016 azu           |
| textlint-rule-ja-no-abusage                                   | 3.0.0   | Copyright (c) 2016 azu           |
| textlint-rule-ja-no-redundant-expression                      | 4.0.1   | Copyright (c) 2016 azu           |
| morpheme-match                                                | 2.0.4   | Copyright (c) 2016 azu           |
| @textlint-ja/textlint-rule-preset-ai-writing                  | 1.7.0   | Copyright (c) 2025 azu           |
| textlint-rule-preset-ai-words-ja                              | 1.2.1   | Copyright (c) 2026 p1ass         |
| textlint-rule-prh                                             | 6.1.1   | Copyright (c) 2015 azu           |
| prh                                                           | 6.1.0   | Copyright (c) 2026 vvakame       |
| @textlint-rule/textlint-rule-pattern                          | 2.0.2   | Copyright (c) 2022 azu           |
| @textlint/regexp-string-matcher                               | 2.0.2   | Copyright (c) 2018 azu           |
| textlint-rule-preset-ja-spacing                               | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-nakaguro-or-halfwidth-space-between-katakana | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-no-space-around-parentheses                  | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-no-space-around-slash                        | 3.0.4   | Copyright (c) azu                |
| textlint-rule-ja-no-space-between-full-width                  | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-space-between-half-and-full-width            | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-space-after-exclamation                      | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-space-after-question                         | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-space-around-code                            | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-space-around-emphasis                        | 3.0.4   | Copyright (c) azu                |
| textlint-rule-ja-space-around-link                            | 3.0.4   | Copyright (c) 2024 azu           |
| textlint-rule-ja-space-around-strong                          | 3.0.4   | Copyright (c) azu                |
| textlint-rule-no-mixed-zenkaku-and-hankaku-alphabet           | 1.0.1   | Copyright (c) 2017 azu           |
| textlint-rule-no-kangxi-radicals                              | 0.2.2   | 著作権表示なし(作者は xl1)       |

`textlint-rule-no-kangxi-radicals` は、LICENSEのファイルを持たず、`package.json` に作者 `xl1` とMIT Licenseを書いています。

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
