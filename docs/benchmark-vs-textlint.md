# textlintとの速度の比較 {#benchmark}

同じルールを設定したtextlintと比べて、jafixは14〜142倍速く終わり、メモリは3分の1以下でした。
`check`、`fix`、hookの3つの場面を、3KBから343KBの4つのファイルで計りました。
計測日は2026-11-15で、`scripts/bench-vs-textlint.sh` を使いました。

## 比べ方 {#method}

両方に、同じ働きの5つのルールを設定しました。

| 働き                               | jafix                        | textlint                                                |
| ---------------------------------- | ---------------------------- | ------------------------------------------------------- |
| 全角括弧を半角にする               | `fullwidth-paren`            | `prh`                                                   |
| 日本語と英数字の間の空白を削除する | `space-between-ja-and-alnum` | `ja-space-between-half-and-full-width` (`space: never`) |
| 日本語と括弧の間の空白を削除する   | `space-between-ja-and-paren` | `@textlint-rule/pattern` の置き換え                     |
| 30語の単語リストを指摘する         | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |
| 漢字が6文字以上続く箇所を指摘する  | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |

場面ごとに次のコマンドを計りました。

| 場面  | jafix               | textlint                           |
| ----- | ------------------- | ---------------------------------- |
| check | `jafix check`       | `textlint`                         |
| fix   | `jafix fix`         | `textlint --fix`                   |
| hook  | `jafix hook claude` | `textlint --fix` の後に `textlint` |

hookの入力は、Claude CodeのEditで50行目から20行を書き換えたものです。
textlintはhookの入力を読めず、直す行も絞れません。
そのため、ファイル全体への2回の実行をhookの1回分としました。

入力は次の4つのファイルです。

| ファイル         | 大きさ | 内容                            |
| ---------------- | ------ | ------------------------------- |
| `blocks-10.md`   | 3KB    | 約350バイトの塊を10回並べたもの |
| `blocks-100.md`  | 34KB   | 同じ塊を100回並べたもの         |
| `blocks-1000.md` | 343KB  | 同じ塊を1000回並べたもの        |
| `README.md`      | 69KB   | jafixのREADME                   |

塊は見出し、段落、リスト、強調、コードブロックを含み、どのルールにも当たります。
hyperfineで3回の予行の後に20回(343KBのファイルは10回)計り、平均を使いました。

## 件数の一致 {#work-performed}

ルールが当たった件数はほぼ一致しました。
3KBのファイルで差が出たのは日本語と英数字の間の空白だけで、jafixのほうが30件多く当たります。

| 働き                                      | jafix | textlint |
| ----------------------------------------- | ----: | -------: |
| 全角括弧                                  |    20 |       20 |
| 日本語と英数字の間の空白                  |   120 |       90 |
| 日本語と括弧の間の空白と、2つの指摘ルール |    70 |       70 |

textlintの `ja-space-between-half-and-full-width` は、長音符の後(`サーバー API`)と漢字の後(`系 38`)の空白を削除しません。
jafixは処理する箇所が多いので、この差は比較でjafixに不利に働きます。

## 結果 {#results}

| 項目      | 値                             |
| --------- | ------------------------------ |
| マシン    | Apple M5 (macOS、arm64)        |
| jafix     | 0.1.0 (00b728a)のrelease build |
| textlint  | v15.8.0 (Node.js v26.13.0)     |
| hyperfine | 2.0.0                          |

時間は壁時計時間の平均、メモリは最大常駐メモリの平均です。
倍率は、textlintの時間をjafixの時間で割った値です。

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

## 倍率の傾向 {#interpretation}

差が最も大きいのは3KBのファイルです。
textlintは1回の実行ごとに、Node.jsの起動とルールの読み込みで約150msかかります。
jafixは同じ部分が約2msで終わります。
agentが1回の編集で書く数KBから数十KBのファイルでは、この固定の時間が実行時間の大半を占めます。

数KBのファイルでは、hookの差が最も開きます。
textlintは修正と指摘を別々に実行するので、起動の時間を2回払い、ファイル全体を2回解析します。
jafixは1つのプロセスで修正してから指摘を集め、修正するのも今回変えた行だけです。

ファイルが大きくなるとhookの倍率は縮み、3KBの142倍が343KBでは33倍になります。
本文を解析する時間が固定の時間より大きくなるためです。
jafixは変えた行が少なくてもファイル全体を解析するので、hookの時間はファイルの大きさに比例して伸びます。
checkとfixの倍率は34KBから343KBの間で14〜54倍で、大きさに従った変化は見えません。

メモリは、jafixが数KBのファイルで6〜7MB、343KBでも92MBに収まりました。
textlintはどのファイルでも120MB以上を使いました。

どの値も予行の後の実行を計ったものです。
textlintの初回の実行はディスクのキャッシュを使えないので、さらに遅くなります。

## 測り直す {#reproduce}

`scripts/bench-vs-textlint.sh` は同じ表を標準出力に出します。
引数にディレクトリを渡すと、hyperfineの結果のJSONをそこに残します。

```sh
scripts/bench-vs-textlint.sh /tmp/bench-results
```

| 必要なもの | 渡し方                                                                                                                              |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| textlint   | 環境変数 `TEXTLINT_PREFIX` にprefixを指定する。未設定なら `mise where npm:textlint` の場所を使う                                    |
| ルール     | prefixに `textlint-rule-prh`、`textlint-rule-ja-space-between-half-and-full-width`、`@textlint-rule/textlint-rule-pattern` を入れる |
| コマンド   | `hyperfine` と `jq`                                                                                                                 |

prefixは `<prefix>/bin/textlint` と `<prefix>/node_modules` を持つディレクトリです。
