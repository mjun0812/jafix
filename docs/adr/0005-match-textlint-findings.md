# ADR-0005: 移植したルールは、textlintと同じ箇所を指摘する

Status: superseded by ADR-0006

textlintの `preset-ja-technical-writing` から移植した組み込みのルールは、同じ入力に対して、textlintと同じ箇所を指摘する。見逃しや、許可リストが直後の文字まで許すような癖も写す。textlintから移っても指摘の数が変わらず、実物のtextlintの実行結果と比べて確かめられるからである。期待値は、textlint 15.8.0とpreset 12.0.2の実行結果から作り、`tests/fixtures/builtin/` に置く。

textlintと変えるのは、次の点だけである。

- textlintが誤った位置を出す3か所は、正しい位置に出す。`sentence-length` は文の先頭、`max-comma` はその文の最後のカンマ、`ja-no-mixed-period` は末尾の文字である
- 例外では止まらない。`arabic-kanji-numbers` は、textlintが例外を投げる漢数字の並びも指摘する
- 本文ではない箇所は見ない。裸のURL、省略形の参照リンクの文字列、画像には、本文の範囲ごとに当てるルールを当てない
- 列は文字で数える

## Considered Options

- 見逃しや許しすぎも直す。textlintより指摘の増える箇所が出て、実行結果と比べて確かめられる範囲が狭くなる
- 位置のずれも含めて同じにする。agentが誤った位置を見て、直す箇所を誤る

## Consequences

- textlintのpresetかルールのversionが上がったら、fixtureの期待値を作り直して比べる必要がある
- jafixとtextlintは、Markdownの解析器が違う。fixtureに無い入力では、nodeの分け方の違いで指摘の有無の変わることがある
