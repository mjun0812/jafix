# ADR-0013: 組み込みの修正ルールの箇所が重なれば、ルールの一覧で前にあるルールの箇所を1件だけ残す

組み込みの修正ルールの箇所どうしが重なるときは、1件だけを残して直し、`jafix check` もその1件だけを出す。先に始まる箇所を残し、同じ位置で始まるなら長い箇所を残す。範囲が同じなら、ルールの一覧で前にあるルールの箇所を残す。一覧は、jafix独自の3つ、`preset-ja-technical-writing` の23ルール、AIらしい書き方の7ルール、利用者の置き換えの2ルール、`preset-ja-spacing` の11ルールと `no-mixed-zenkaku-and-hankaku-alphabet`・`no-kangxi-radicals` の順である。各ルールの判定は、それぞれのまま変えない。

重なりは、`preset-ja-spacing` の修正ルールが、jafix独自の修正ルールと同じ空白を同じ向きに直すことから生じる。`日本語 Rust` の空白には、`space-between-ja-and-alnum` と `ja-space-between-half-and-full-width` の両方が当たり、`語 （補足）` の空白には、`space-between-ja-and-paren` と `ja-no-space-around-parentheses` の両方が当たる。どちらのIDで直しても文書の結果は同じなので、問題は `jafix check` とhookの出力に、どちらのIDの行を出すかである。

jafix独自のルールのIDを残すと、3つの条件を同時に満たせる。1つ目に、設定を書かない利用者の出力が変わらない。2つ目に、`preset-ja-spacing = true` を足しても、同じ空白の行が増えない。3つ目に、移したルールの判定をtextlintから写すというADR-0010を守れる。代わりに、`ja-space-between-half-and-full-width` のIDは、jafix独自のルールが当たらない空白(`punctuation` を真にした `。 Rust` など)だけに出る。textlintと同じ狭い判定で直したい利用者は、`space-between-ja-and-alnum = false` を書く。2026-11-15に、利用者がこの案を選んだ。

## Considered Options

- 移したルールのIDを残す。presetを有効にした利用者の出力がtextlintのIDに揃うが、移したルールが当たらない空白(`ー` `々` の隣、2文字以上の空白、表のセルや太字の中)にだけ `space-between-ja-and-alnum` の行が残り、1つの文書の中で、同じ種類の空白が2つのIDで出る
- 同じ向きの2つを同時に有効にした設定を不正にする。`preset-ja-spacing = true` だけでは読み込めず、`space-between-ja-and-alnum = false` と `space-between-ja-and-paren = false` も要る。ADR-0008が断るのは、逆向きの組だけである
- `ja-space-between-half-and-full-width` の空白を消す側を、jafix独自の判定にする。2つは同じ箇所に当たるが、textlintの判定を写すADR-0010から外れる
- `jafix check` に両方の行を出し、直すのは1回にする。同じ箇所に2件の指摘が出ることは、ADR-0007が新しいルールを設定に書くまで無効にした理由そのものである

## Consequences

- 範囲が同じ箇所では、ルール一覧の順に従って残す箇所を決める。修正ルールを足すspecでは、一覧に追加する位置が、重なる箇所に出すIDを決める
- 利用者の置き換えの固定の部分を組み込みの修正ルールと比べる判定は、重なりを解く前の箇所を使い、当たったルールをすべて理由に挙げる。重なりを解いた後の箇所を使うと、理由に挙げたルールを無効にしても、残ったもう片方のルールで読み込みがまた失敗するためである
- `preset-jtf-style` の空白と括弧のルールを入れるspecで、同じ空白に当たるルールが増える。そのときに、残すルールの順を見直す
