# jafix

jafixは、日本語のMarkdownを校正するCLIです。
全角括弧や空白などの表記をそろえ、機械的に直せない箇所は指摘します。
設定ファイルで、使うルールを選べます。

- `jafix check FILE.md` で指摘を確認する
- `jafix fix FILE.md` で修正を書き込む
- [textlint由来のルール一覧](./textlint-rules.md)を見る
- [textlintとの速度の比較](./benchmark-vs-textlint.md)を読む
