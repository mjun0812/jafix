# ADR-0014: 組み込みのルールのオプションのパターンも、JavaScriptの正規表現で読む

組み込みのルールのオプションのパターンは、`prh`・`@textlint-rule/pattern` と同じく、JavaScriptの正規表現(`u` フラグ付き)の構文と意味で読む。対象は、`sentence-length` の `skipPatterns`、`no-exclamation-question-mark`・`ja-unnatural-alphabet`・`ja-no-successive-word` の `allow`、`ja-no-redundant-expression` の `dictOptions` の `allows`、AIらしい書き方の6ルールの `allows`、`ja-space-between-half-and-full-width` の `allows` である。`allowlist` のパターンも同じく読む。読むには `regress` crateを使う。`/source/flags` の形の見分け方とフラグは `@textlint-rule/pattern` と同じで、`y` は不正である。組み込みの既定のパターンも、同じ方言で読む。この決定は、ADR-0012のうち、組み込みのルールのオプションをRustの `regex` crateで読むとした部分を覆す。

ADR-0012では、利用者がtextlintで使っているパターンをそのまま書けることを重視した。この理由は組み込みのルールのオプションにも当てはまる。`.textlintrc` の `allow` や `skipPatterns` には、後読みや後方参照を使ったものがある。方言を1つにすれば、利用者は書く場所によって方言を書き分けずに済み、READMEも書き分けを説明せずに済む。`allowlist` を入れるときに、ADR-0012が残した問いをまとめて決めることになっていた。2026-11-16に利用者がこの案を選んだ。

後方互換は保たないので、今の利用者の設定のうち、次のものは意味が変わる。1つ目に、Rustの `regex` crateだけが読む書き方(`\p{Han}`・`(?i)`・`[[:alpha:]]`・`\z`)は読み込みの誤りになる。`u` 付きのJavaScriptが断る書き方(`\-` のような意味の無い退避と、閉じていない `{`)も同じである。2つ目に、`\d`・`\w`・`\b` はASCIIで判定するので、全角の数字や日本語の文字に当たらなくなる。3つ目に、`//` は、どこにでも当たる空の正規表現から文字列そのもの `//` に変わり、`/a/d` は文字列そのものから正規表現 `a` に変わる。今のfixture・READMEの設定の例・組み込みの既定のパターンには、2つの方言で意味の変わる書き方が無い。そのため期待値は変えない。

## Considered Options

- `allowlist` だけをJavaScriptで読み、組み込みのルールのオプションは今のRustの `regex` crateのまま読む。ADR-0012の分け方に沿うが、設定の中に2つの方言が残る
- `allowlist` も、組み込みのルールのオプションと同じRustの `regex` crateで読む。後読みを使った許可リストを写せない
- JavaScriptで読み、ルールごとにtextlintの元の版に合わせて `u` の有無を変える。textlintとの一致は上がるが、同じ書き方のパターンがルールによって違う意味になる

## Consequences

- `builtin::pattern` (Rustの `regex` crateで読む関数)を削り、組み込みのルールのオプションのパターンの型は `regress` の正規表現になる
- READMEの「パターン」の節は、JavaScriptの方言を説明する。「正規表現」の節は、利用者の置き換えと組み込みのルールのオプションを書き分けない
- 方言を変えた後も、hookの速度の基準(`scripts/bench-hook.sh` の平均が50ms未満)を満たすかを確かめる
