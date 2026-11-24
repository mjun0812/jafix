# ADR-0002: Rustで実装し、形態素解析にlinderaを使う

jafixはRustで書き、形態素解析が要るルール(単語リスト、読点の数え方、助詞の重複)にはIPADICを埋め込んだlinderaを使う。textlintの単語リストはkuromoji(IPADIC)が返す動詞の原形と品詞で語を見分けている。linderaで同じ辞書を使うと、起動と1文の解析に合わせて約1.5msかかった。Goのkagomeは起動のたびに辞書を展開するので約272msかかり、hookとしてはtextlintの1回の実行(0.16〜0.33秒)と変わらなかった。

## Considered Options

- Rustで形態素解析を使わず、単語リストを表層の文字列で探す。動詞の活用と品詞の条件を再現できず、短い名詞が熟語の一部に当たるような誤検知が増える
- Goで形態素解析を使わない。kagomeが起動の目標に合わず、単語リストも表層の文字列で探すことになる

## Consequences

- 辞書を埋め込むので、binaryは約49MBになる。配布はGitHub Releasesのビルド済みbinaryを主にする
