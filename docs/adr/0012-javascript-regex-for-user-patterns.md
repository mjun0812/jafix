# ADR-0012: 利用者が書く正規表現はJavaScriptの構文と意味で読み、組み込みのルールのオプションは今のまま読む

Status: superseded by ADR-0014 (組み込みのルールのオプションのパターンをRustの `regex` crateで読む部分だけ)

prhの辞書(`pattern`・`patterns`)と `@textlint-rule/pattern` のパターン(`pattern`・`allows`)は、`regress` crateを使ってJavaScriptの正規表現(`u` フラグ付き)の構文と意味で読む。組み込みのルールのオプションのパターン(`allow`・`skipPatterns` など)は、従来どおりRustの `regex` crateで読む。

利用者が書く辞書とパターンは、textlintで使っているものをそのまま書けることが大事である。公開のprhの辞書は、後読み・先読み・後方参照を多く使う。`regex` crateはこれらを読めない。`fancy-regex` は読めるが、`\b`・`\d`・`\w` をUnicodeの意味で読むので、`日本語Rust` の `R` の前が単語の境界にならず、prhの `wordBoundary` の結果が変わる。`regress` はECMAScriptの構文と意味を目標にしたcrateである。試作では、後読み、後方参照、`\p{Script=Han}`、ASCIIの `\b` と `\d` がJavaScriptと同じに働き、公開の3つの辞書の `specs` 335件がすべて通った。

組み込みのルールのオプションまで揃えるかは、まだ決めない。揃えると、約30ルールのオプションの読み込みとテストが変わり、判定の前提(`\d`・`\b` の意味)も変わる。同じ問いを持つ `textlint-filter-rule-allowlist` を入れるときに、まとめて決める。それまでは、設定の中に2つの方言がある。

## Considered Options

- 組み込みのルールのオプションも、今JavaScriptの方言に揃える
- 利用者のパターンも `regex` crateで読み、後読みと後方参照を断る。公開の辞書の多くが読めなくなる
- `fancy-regex` を使う。正規表現の意味がJavaScriptと違い、辞書の結果が変わる

## Consequences

- READMEは、利用者の置き換えの正規表現と、組み込みのルールのオプションのパターンとで、方言が違うことを書き分ける
- `textlint-filter-rule-allowlist` を入れるspecで、方言をどちらに揃えるかを決め直す
