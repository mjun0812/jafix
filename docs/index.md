---
layout: home
hero:
  name: jafix
  text: 日本語に特化した高速なtextlint
  tagline: Rust製のバイナリ1つで、文章のチェックと自動修正ができます。
  actions:
    - theme: brand
      text: クイックスタート
      link: /quick-start
    - theme: alt
      text: ルール一覧
      link: /rules
benchmark:
  title: textlintとのベンチマーク
  sizeLabel: サイズ
  ratioLabel: 倍率
  ratioUnit: 倍
  link: /benchmark-vs-textlint
  linkText: 詳細
features:
  - title: チェックと自動修正
    details: checkはファイルを書き換えずに指摘を表示します。fixは修正できる箇所を直して保存します。
    link: /quick-start
    linkText: 試す
  - title: 設定なしで使える
    details: 設定ファイルがなくても10ルールが有効です。使うルールはTOMLで選べます。
    link: /configuration
    linkText: 設定する
  - title: agentのhook
    details: Claude CodeやCodexの編集後に、追加または変更した行を自動修正し、残った指摘を返します。
    link: /hooks
    linkText: 登録する
---
