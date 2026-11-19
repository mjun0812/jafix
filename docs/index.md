---
layout: home
hero:
  name: jafix
  text: 日本語の文章を校正するCLI
  tagline: UTF-8とShift_JISのテキストファイルを校正できます。
  actions:
    - theme: brand
      text: クイックスタート
      link: /quick-start
    - theme: alt
      text: ルール一覧
      link: /rules
features:
  - title: 修正と指摘
    details: fixは全角括弧や余分な空白を直します。checkは修正できる箇所に加え、語の重複や括弧の閉じ忘れも報告します。
    link: /quick-start
    linkText: 試す
  - title: 設定なしで使える
    details: 設定ファイルがなくても、既定の10ルールで校正できます。使うルールはTOMLで選べます。
    link: /configuration
    linkText: 設定する
  - title: agentのhook
    details: Claude CodeとCodexがMarkdownに追加または変更した行を校正し、指摘をagentに返します。
    link: /hooks
    linkText: 登録する
  - title: textlintとの速度比較
    details: 同じ働きのルールで計測すると、textlintにかかった時間はjafixの14〜142倍でした。
    link: /benchmark-vs-textlint
    linkText: 計測結果
---
