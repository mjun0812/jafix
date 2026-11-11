# textlintとの速度の比較

jafixとtextlintに同じルールを設定して、`check`・`fix`・hookの3つの場面で速さを比べた。jafixは、どの場面とファイルの大きさでも、textlintより23〜133倍速く終わり、メモリも3分の1以下で済んだ。計測は2026-11-11に `scripts/bench-vs-textlint.sh` で行った。

## 比べ方

両方の設定に、同じ働きの5つのルールを入れた。jafixの組み込みの修正ルール3つと、設定の `[[rules]]` に書いた指摘ルール2つである。textlintでは、同じ働きを次のルールで書いた。

| 働き                               | jafix                        | textlint                                                |
| ---------------------------------- | ---------------------------- | ------------------------------------------------------- |
| 全角括弧を半角にする               | `fullwidth-paren`            | `prh`                                                   |
| 日本語と英数字の間の空白を削除する | `space-between-ja-and-alnum` | `ja-space-between-half-and-full-width` (`space: never`) |
| 日本語と括弧の間の空白を削除する   | `space-between-ja-and-paren` | `@textlint-rule/pattern` の置き換え                     |
| 30語の単語リストを指摘する         | `[[rules]]` の `words`       | `@textlint-rule/pattern`                                |
| 漢字が6文字以上続く箇所を指摘する  | `[[rules]]` の `pattern`     | `@textlint-rule/pattern`                                |

場面は3つである。`check` はファイルを変えずに指摘を出し、`fix` はファイル全体を直して書き戻す。hookは、Claude CodeのEditで50行目から20行を書き換えた入力を `jafix hook claude` に渡す。textlintにはhookの入力を読む機能が無く、直す行も絞れない。そこでtextlint側では、ファイル全体に `textlint --fix` をかけてから `textlint` で指摘を出す2回の実行を、hookの1回分とした。

入力は4つのファイルである。見出し・段落・リスト・強調・コードブロックを含み、どのルールにも当たる約350バイトの塊を、10回・100回・1000回並べたものを3つ作った。残りの1つは、実際の文書としてのjafixの `README.md` である。hyperfineで3回の予行の後に20回(343KBのファイルだけ10回)計り、その平均を使った。

## 同じ仕事をしているか

ルールの働きごとに数えた件数は、ほぼ一致した。3KBのファイルでは、全角括弧と、括弧の間の空白と2つの指摘ルールの件数が一致した。差が出たのは日本語と英数字の間の空白だけで、jafixのほうが30件多い。

| 働き                                      | jafix | textlint |
| ----------------------------------------- | ----: | -------: |
| 全角括弧                                  |    20 |       20 |
| 日本語と英数字の間の空白                  |   120 |       90 |
| 日本語と括弧の間の空白と、2つの指摘ルール |    70 |       70 |

この差は、textlintの `ja-space-between-half-and-full-width` が見逃す空白による。長音符の後の `サーバー API` の空白と、漢字の後の `系 38` の空白を、textlintは削除しない。jafixのほうが多く仕事をしているので、この差は比べる上でjafixに不利に働く。

## 結果

計測の環境は、Apple M5 (macOS、arm64)である。jafix 0.1.0 (d25cafb)のrelease buildと、textlint v15.8.0 (Node.js v26.13.0)を、hyperfine 2.0.0で計った。時間は壁時計時間の平均、メモリは最大常駐メモリの平均である。倍率は、textlintの時間をjafixの時間で割ったものである。

| 場面  | ファイル               |          jafix |         textlint |  倍率 |
| ----- | ---------------------- | -------------: | ---------------: | ----: |
| check | blocks-10.md (3KB)     |    2.3ms / 6MB |  156.9ms / 123MB |  69倍 |
| check | README.md (13KB)       |    2.8ms / 6MB |  185.4ms / 127MB |  67倍 |
| check | blocks-100.md (34KB)   |     7ms / 12MB |  248.1ms / 189MB |  36倍 |
| check | blocks-1000.md (343KB) |  89.7ms / 73MB |   2024ms / 328MB |  23倍 |
| fix   | blocks-10.md (3KB)     |    2.4ms / 5MB |  161.4ms / 136MB |  68倍 |
| fix   | README.md (13KB)       |    2.9ms / 6MB |  201.4ms / 155MB |  70倍 |
| fix   | blocks-100.md (34KB)   |   6.5ms / 12MB |  313.8ms / 203MB |  48倍 |
| fix   | blocks-1000.md (343KB) |  73.1ms / 72MB | 2892.8ms / 425MB |  40倍 |
| hook  | blocks-10.md (3KB)     |    2.5ms / 6MB |  335.8ms / 136MB | 133倍 |
| hook  | README.md (13KB)       |      4ms / 6MB |  375.1ms / 153MB |  94倍 |
| hook  | blocks-100.md (34KB)   |   8.9ms / 13MB |  549.1ms / 203MB |  62倍 |
| hook  | blocks-1000.md (343KB) | 123.1ms / 89MB | 3436.5ms / 420MB |  28倍 |

## 読み取れること

小さいファイルほど、差が大きくなる。textlintは、1回の実行ごとにNode.jsの起動とルールの読み込みで約150msかかる。jafixは同じ部分が約2msで終わる。agentの1回の編集で書くような数KBから数十KBのファイルでは、この固定の時間が実行時間の大半を占める。

hookの場面で差が最も開くのは、textlintが直す処理と指摘する処理を別々に実行するからである。textlintは起動の固定の時間を2回払い、ファイル全体を2回解析する。jafixは1つのプロセスで直してから指摘を集め、直すのも今回変えた行だけである。

ファイルが大きくなると、倍率は縮む。本文を解析する時間が固定の時間より大きくなるためである。それでも343KBのファイルで、jafixは23〜40倍速い。ただしjafixは、変えた行が少なくてもファイル全体を解析するので、hookの時間はファイルの大きさに比例して伸びる。

メモリの差も大きい。jafixは数KBのファイルで6MB、343KBでも89MBに収まった。textlintは、どのファイルでも120MB以上を使った。

この計測は、予行の後の2回目以降の実行の時間である。textlintの初回の実行は、ディスクのキャッシュを使えないので、さらに遅くなる。

## 測り直す

`scripts/bench-vs-textlint.sh` を実行すると、同じ表を標準出力に出す。引数にディレクトリを渡すと、hyperfineの結果のJSONをそこへ残す。

```sh
scripts/bench-vs-textlint.sh /tmp/bench-results
```

textlintは、本体とルールを入れたprefix (`<prefix>/bin/textlint` と `<prefix>/node_modules`)を、環境変数 `TEXTLINT_PREFIX` で渡す。未設定なら `mise where npm:textlint` の場所を使う。prefixには、`textlint-rule-prh`、`textlint-rule-ja-space-between-half-and-full-width`、`@textlint-rule/textlint-rule-pattern` を入れておく。ほかに、`hyperfine` と `jq` が要る。
