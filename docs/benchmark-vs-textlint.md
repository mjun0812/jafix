# textlintとの速度の比較 {#benchmark}

同じ働きのルールで計測すると、textlintにかかった時間はjafixの5〜49倍でした。jafixの最大常駐メモリはtextlintの半分以下でした。
2026-11-25に `scripts/bench-vs-textlint.sh` を使い、3KBから343KBまでの4ファイルで `check`、`fix`、hookを比べました。

## 比べ方 {#method}

jafixとtextlintで、同じ働きの5ルールを有効にしました。

| 働き                               | jafix                        | textlint                                                |
| ---------------------------------- | ---------------------------- | ------------------------------------------------------- |
| 全角括弧を半角にする               | `fullwidth-paren`            | `prh`                                                   |
| 日本語と英数字の間の空白を削除する | `space-between-ja-and-alnum` | `ja-space-between-half-and-full-width` (`space: never`) |
| 日本語と括弧の間の空白を削除する   | `space-between-ja-and-paren` | `@textlint-rule/pattern` の置き換え                     |
| リストにある30語を指摘する         | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |
| 漢字が6文字以上続く箇所を指摘する  | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |

各場面で実行したコマンドを次の表に示します。

| 場面  | jafix               | textlint                           |
| ----- | ------------------- | ---------------------------------- |
| check | `jafix check`       | `textlint`                         |
| fix   | `jafix fix`         | `textlint --fix`                   |
| hook  | `jafix hook claude` | `textlint --fix` の後に `textlint` |

jafixのhookには、Claude CodeのEditで50行目から20行を書き換えた場合の入力を作りました。
textlintでは同じファイル全体に `--fix` と指摘を出すコマンドを順に実行し、2回の合計時間をhookの1回分としました。

入力は次の4つのファイルです。

| ファイル         | 大きさ | 内容                            |
| ---------------- | ------ | ------------------------------- |
| `blocks-10.md`   | 3KB    | 約350バイトの塊を10回並べたもの |
| `README.md`      | 5KB    | jafixのREADME                   |
| `blocks-100.md`  | 34KB   | 同じ塊を100回並べたもの         |
| `blocks-1000.md` | 343KB  | 同じ塊を1000回並べたもの        |

繰り返し使った約350バイトの断片には、見出し、段落、リスト、強調、コードブロックがあります。5つのルールすべてが当たる内容です。
hyperfineで3回予備実行し、その後の20回の平均を求めました。343KBのファイルでは予備実行後の10回を使いました。

## 件数の一致 {#work-performed}

3KBのファイルで件数に差が出たのは、日本語と英数字の間の空白です。

| 働き                                      | jafix | textlint |
| ----------------------------------------- | ----: | -------: |
| 全角括弧                                  |    20 |       20 |
| 日本語と英数字の間の空白                  |   120 |       90 |
| 日本語と括弧の間の空白と、2つの指摘ルール |    70 |       70 |

textlintの `ja-space-between-half-and-full-width` は、長音符の後(`サーバー API`)と漢字の後(`系 38`)の空白を削除しません。
この空白も処理するため、jafixでは修正箇所が30件多くなります。

## 結果 {#results}

| 項目      | 値                             |
| --------- | ------------------------------ |
| マシン    | Apple M5 (macOS、arm64)        |
| jafix     | 0.0.3 (608e127)のrelease build |
| textlint  | v16.1.2 (Node.js v26.13.1)     |
| hyperfine | 2.0.0                          |

各行には実行時間と最大常駐メモリの平均を示します。
倍率はtextlintの時間をjafixの時間で割った値です。

| 場面  | ファイル               |          jafix |         textlint | 倍率 |
| ----- | ---------------------- | -------------: | ---------------: | ---: |
| check | blocks-10.md (3KB)     |    3.8ms / 8MB |    97.2ms / 91MB | 26倍 |
| check | README.md (5KB)        |    3.5ms / 8MB |    78.6ms / 83MB | 22倍 |
| check | blocks-100.md (34KB)   |   9.9ms / 15MB |  119.7ms / 126MB | 12倍 |
| check | blocks-1000.md (343KB) |  98.8ms / 86MB | 1107.9ms / 179MB | 11倍 |
| fix   | blocks-10.md (3KB)     |    3.8ms / 8MB |    98.7ms / 92MB | 26倍 |
| fix   | README.md (5KB)        |    3.3ms / 8MB |    79.7ms / 86MB | 24倍 |
| fix   | blocks-100.md (34KB)   |   8.7ms / 16MB |  124.8ms / 120MB | 14倍 |
| fix   | blocks-1000.md (343KB) | 126.5ms / 93MB |  576.6ms / 223MB |  5倍 |
| hook  | blocks-10.md (3KB)     |    3.8ms / 8MB |   182.1ms / 92MB | 49倍 |
| hook  | README.md (5KB)        |    3.4ms / 8MB |     163ms / 86MB | 48倍 |
| hook  | blocks-100.md (34KB)   |  10.2ms / 16MB |    225ms / 120MB | 22倍 |
| hook  | blocks-1000.md (343KB) | 135.3ms / 93MB |  742.7ms / 223MB |  5倍 |

## 倍率の傾向 {#interpretation}

倍率が最も大きかったのは3KBのファイルです。
このファイルの `check` はtextlintで約97ms、jafixで約4msでした。
数KBから数十KBのファイルでは、Node.jsの起動やルールの読み込みにかかる時間の影響が大きいと考えられます。

3KBのファイルでは、hookで最も差が開きました。
この比較ではtextlintを修正と指摘で2回起動し、そのたびにファイル全体を解析します。
jafixは1つのプロセスで、編集した行を修正してから指摘を集めます。

hookの倍率は3KBで49倍、343KBで5倍でした。
jafixのhookもファイル全体を解析するため、ファイルが大きいほど解析時間の影響が増すと考えられます。
`check` の倍率は34KBと343KBで11〜12倍でした。`fix` の倍率は34KBで14倍、343KBで5倍でした。

jafixの最大常駐メモリは3KBのファイルで8MB、343KBのファイルで最大93MBでした。
textlintはどのファイルでも80MB以上でした。

表の値は予備実行後の計測結果です。
ディスクキャッシュの状態によっては、初回の実行にさらに時間がかかります。

## 測り直す {#reproduce}

同じ条件で測るには `scripts/bench-vs-textlint.sh` を実行します。結果の表は標準出力に表示されます。
ディレクトリを引数に渡すと、hyperfineの結果をその場所にJSON形式で保存します。

```sh
scripts/bench-vs-textlint.sh /tmp/bench-results
```

| 必要なもの | 渡し方                                                                                                                              |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| textlint   | 環境変数 `TEXTLINT_PREFIX` にprefixを指定する。未設定なら `mise where npm:textlint` の場所を使う                                    |
| ルール     | prefixに `textlint-rule-prh`、`textlint-rule-ja-space-between-half-and-full-width`、`@textlint-rule/textlint-rule-pattern` を入れる |
| コマンド   | `hyperfine` と `jq`                                                                                                                 |

prefixは `<prefix>/bin/textlint` と `<prefix>/node_modules` を持つディレクトリです。
