---
layout: home
hero:
  name: jafix
  text: 日本語の文章を校正するCLI
  tagline: 括弧や空白の表記を修正し、機械的に直せない箇所を指摘します。
  actions:
    - theme: brand
      text: クイックスタート
      link: /quick-start
    - theme: alt
      text: ルール一覧
      link: /rules
features:
  - title: 修正と指摘
    details: fixは全角括弧や余分な空白を書き換えます。checkは語の重複や括弧の閉じ忘れも報告します。
    link: /quick-start
    linkText: 試す
  - title: 設定なしで使える
    details: 初期状態で10ルールが有効です。使うルールはTOMLで選べます。
    link: /configuration
    linkText: 設定する
  - title: agentのhook
    details: Claude CodeとCodexが編集したMarkdownの行を修正し、その行の指摘をagentに返します。
    link: /hooks
    linkText: 登録する
  - title: 高速
    details: 単一のバイナリで動きます。同じルールを設定したtextlintより14〜142倍速く終わります。
    link: /benchmark-vs-textlint
    linkText: 計測結果
---
