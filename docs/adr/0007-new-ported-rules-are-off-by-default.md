# ADR-0007: 追加で移植するルールは、設定で有効にしたものだけが動き、presetの名前でも有効にできる

`textlint-rule-preset-ja-technical-writing` の23ルールの後に移植するルールは、設定に書かなければ無効である。利用者は、設定の `[builtin]` に、ルールのIDか、presetの名前(`"@textlint-ja/preset-ai-writing" = true` など)を書いて有効にする。今の26ルールは、これまでどおり設定に書かなくても有効である。presetの名前の値は `true` か `false` だけで、同じpresetのルールのIDのキーも書いたときは、書いた順によらずルールのIDのキーを使う。

設定に書かなければ無効にするのは、追加で移植するルールに、同じ書き方を重ねて見るものが多いからである。たとえば日本語と英数字の間の空白は、jafixの修正ルール、`preset-jtf-style`、`preset-ja-spacing`、`preset-smarthr` の4つが見る。全部を最初から有効にすると、同じ箇所に複数の指摘が出る。無効にしておけば、設定を書かない利用者の指摘は今と変わらず、選んだpresetの指摘だけが増える。presetの名前で有効にできないと、37ルールを持つ `preset-jtf-style` を使うのに37行を書くことになる。`preset-ja-technical-writing = false` も書けるので、別のpresetだけを使う利用者は23ルールをまとめて止められる。

## Considered Options

- 元のpresetの既定値どおり有効にし、jafixの既存のルールと逆の書き方を求めるものだけを無効にする。重なる指摘は残る
- すべて最初から有効にし、要らないものを利用者が切る
- presetの名前の値に、textlintと同じくルールごとの設定の表を書けるようにする。ルールのIDは全体で1つなので、ルールのIDのキーで足りる

## Consequences

- 新しいルールを足すspecは、どのpresetの名前で有効になるかと、presetを有効にしたときの各ルールの有効・無効とオプションを決める
- 同じルールのIDを、別のpresetが別のオプションで束ねる場合(`preset-smarthr` の `sentence-length` など)は、presetの名前で有効にしたときのオプションの決め方を、そのspecで見直す
