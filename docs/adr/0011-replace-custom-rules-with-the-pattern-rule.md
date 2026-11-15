# ADR-0011: 設定の `[[rules]]` を削り、利用者のパターンは `@textlint-rule/pattern` に一本化する

設定ファイルの `[[rules]]` (表ごとに `id`・`message`・`pattern` か `words` を書き、指摘だけを足す仕組み)を削る。利用者のパターンは、textlintと同じIDとオプションを持つ組み込みのルール `@textlint-rule/pattern` の `patterns` に書く。後方互換は保たない。`[[rules]]` を書いた設定は、知らないキーとして不正になる。

`@textlint-rule/pattern` は、`[[rules]]` の働き(パターンか文字列で、利用者の文面の指摘を出す)をすべて持ち、置き換え(`replace`)、`allows`、`allowNodeTypes` を足したものである。一本化すれば、同じ働きの仕組みを2つ持たずに済む。利用者の正規表現の方言も、prhと同じJavaScriptの1つになる(ADR-0012)。textlintとの速度の比較でも、`[[rules]]` の働きをtextlintの側では `@textlint-rule/pattern` で書いている。

失うものは2つある。1つ目は表ごとの `id` で、指摘の行は `[@textlint-rule/pattern]` になる。どのパターンかは、要素の `message` で見分ける。2つ目は、`words` の「同じ位置で重なる語は長い語の1件だけを指摘する」扱いである。要素ごとに当たるので、重なる語は両方とも指摘になる。

## Considered Options

- 両方を残す。移行は要らないが、同じ働きの仕組みと正規表現の方言が2つずつになり、READMEで両方を説明し続ける
- `@textlint-rule/pattern` を入れず、`[[rules]]` に置き換えと許可を足す。表ごとの `id` は残るが、textlintのIDとオプションの名前で設定を書ける、という約束から外れる
- 両方を残し、`[[rules]]` の方言だけをJavaScriptにする

## Consequences

- `[[rules]]` を使っていた設定は、`[builtin."@textlint-rule/pattern"]` の `patterns` へ書き直す必要がある。文字列の一覧(`words`)は、文字列そのもののパターンの要素で書く
- 指摘の行から、利用者の付けた `id` が消える
