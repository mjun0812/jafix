# textlintとの速度の比較

jafixとtextlintに同じルールを設定して、`check`・`fix`・hookの3つの場面で速さを比べた。jafixは、どの場面とファイルの大きさでも、textlintより14〜142倍速く終わり、メモリも3分の1以下で済んだ。計測は2026-11-15に `scripts/bench-vs-textlint.sh` で行った。

## 比べ方

両方の設定に、同じ働きの5つの規則を入れた。jafixは、組み込みの修正ルール3つと、`@textlint-rule/pattern` の2つの要素である。textlintでは、同じ働きを次のルールで書いた。

| 働き                               | jafix                        | textlint                                                |
| ---------------------------------- | ---------------------------- | ------------------------------------------------------- |
| 全角括弧を半角にする               | `fullwidth-paren`            | `prh`                                                   |
| 日本語と英数字の間の空白を削除する | `space-between-ja-and-alnum` | `ja-space-between-half-and-full-width` (`space: never`) |
| 日本語と括弧の間の空白を削除する   | `space-between-ja-and-paren` | `@textlint-rule/pattern` の置き換え                     |
| 30語の単語リストを指摘する         | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |
| 漢字が6文字以上続く箇所を指摘する  | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |

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

計測の環境は、Apple M5 (macOS、arm64)である。jafix 0.1.0 (00b728a)のrelease buildと、textlint v15.8.0 (Node.js v26.13.0)を、hyperfine 2.0.0で計った。時間は壁時計時間の平均、メモリは最大常駐メモリの平均である。倍率は、textlintの時間をjafixの時間で割ったものである。

| 場面  | ファイル               |          jafix |         textlint |  倍率 |
| ----- | ---------------------- | -------------: | ---------------: | ----: |
| check | blocks-10.md (3KB)     |    2.6ms / 7MB |  151.7ms / 122MB |  59倍 |
| check | blocks-100.md (34KB)   |   9.2ms / 14MB |  249.8ms / 189MB |  27倍 |
| check | README.md (69KB)       |  18.7ms / 12MB |  267.8ms / 178MB |  14倍 |
| check | blocks-1000.md (343KB) | 120.1ms / 79MB |   2755ms / 328MB |  23倍 |
| fix   | blocks-10.md (3KB)     |    2.3ms / 6MB |  167.8ms / 136MB |  73倍 |
| fix   | blocks-100.md (34KB)   |   6.1ms / 13MB |  330.6ms / 203MB |  54倍 |
| fix   | README.md (69KB)       |   5.7ms / 12MB |  262.5ms / 185MB |  46倍 |
| fix   | blocks-1000.md (343KB) |  89.5ms / 78MB | 4464.4ms / 406MB |  50倍 |
| hook  | blocks-10.md (3KB)     |    2.9ms / 7MB |  411.1ms / 135MB | 142倍 |
| hook  | blocks-100.md (34KB)   |  11.2ms / 14MB |   1031ms / 202MB |  92倍 |
| hook  | README.md (69KB)       |  10.5ms / 12MB |  490.7ms / 188MB |  47倍 |
| hook  | blocks-1000.md (343KB) | 192.6ms / 92MB | 6319.1ms / 408MB |  33倍 |

## 読み取れること

差が最も大きいのは、3KBの小さいファイルである。textlintは、1回の実行ごとにNode.jsの起動とルールの読み込みで約150msかかる。jafixは同じ部分が約2msで終わる。agentの1回の編集で書くような数KBから数十KBのファイルでは、この固定の時間が実行時間の大半を占める。

数KBのファイルでhookの場面の差が最も開くのは、textlintが直す処理と指摘する処理を別々に実行するからである。textlintは起動の固定の時間を2回払い、ファイル全体を2回解析する。jafixは1つのプロセスで直してから指摘を集め、直すのも今回変えた行だけである。

ファイルが大きくなると、hookの倍率は縮み、3KBの142倍が343KBの33倍になる。本文を解析する時間が固定の時間より大きくなるためである。checkとfixの倍率は、34KBから343KBの間では14〜54倍の範囲で、大きさに従った変化は見えない。343KBのファイルでも、jafixは23〜50倍速い。ただしjafixは、変えた行が少なくてもファイル全体を解析するので、hookの時間はファイルの大きさに比例して伸びる。

メモリの差も大きい。jafixは数KBのファイルで7MB、343KBでも92MBに収まった。textlintは、どのファイルでも120MB以上を使った。

この計測は、予行の後の2回目以降の実行の時間である。textlintの初回の実行は、ディスクのキャッシュを使えないので、さらに遅くなる。

## 測り直す

`scripts/bench-vs-textlint.sh` を実行すると、同じ表を標準出力に出す。引数にディレクトリを渡すと、hyperfineの結果のJSONをそこへ残す。

```sh
scripts/bench-vs-textlint.sh /tmp/bench-results
```

textlintは、本体とルールを入れたprefix (`<prefix>/bin/textlint` と `<prefix>/node_modules`)を、環境変数 `TEXTLINT_PREFIX` で渡す。未設定なら `mise where npm:textlint` の場所を使う。prefixには、`textlint-rule-prh`、`textlint-rule-ja-space-between-half-and-full-width`、`@textlint-rule/textlint-rule-pattern` を入れておく。ほかに、`hyperfine` と `jq` が要る。
