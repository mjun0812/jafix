# ADR-0001: textlintのルールを読み込まず、規則を自前で実装する

AI agentが書いた日本語の文章をglobalなhookで直すため、jafixはtextlintのルール(npm package)を動かさず、必要な規則を自分の中に書き直す。textlintは作業中のrepoから本体・ルール・設定を探す作りで、globalなhookとして1つのbinaryで動かすことにも、agentが変えた行だけを直すことにも向かなかった。

## Considered Options

- textlintのルールを、埋め込みのJS実行環境(gojaやQuickJS)で動かす。textlint本体、Markdownの解析器、kuromoji.jsはNode.jsで動く前提である。kuromojiの辞書の展開が重く、Node.jsより遅くなるおそれが大きかった
- Valeで置き換える。単語リストと正規表現のルールは書けるが、ファイルを書き換えないので変えた行だけの修正が別に要り、英語の単語の区切りが前提である
- RedPen(Java製で起動が重い)、LanguageTool(日本語にほぼ対応していない)、suiko(診断が中心でファイルを書き換えない)、jafmt(日本語と英数字の間に空白を入れる方針で、逆である)
