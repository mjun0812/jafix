# ADR-0003: hookで直す行を、hookの入力にある今回の編集から決める

jafixのhookは、今回の編集でagentが追加・変更した行だけを直す。対象行はhookの入力から決める。Claude Codeでは `tool_response.structuredPatch` の追加行を使う。Codexでは編集後のファイルでapply_patchのpatchのchunkを探し、位置が1つに決まるときだけ追加行を使う。対象行が決まらなければファイルを変えず、理由を標準エラーに書いてexit 1で終える。exit 1はどちらのagentの作業も止めない。

## Considered Options

- gitのHEADとの差分を使う。agentに依らず単純だが、既存の未commitの変更や、追跡外のファイルの既存の行まで直してしまう
- PreToolUseで編集前のファイルを退避し、PostToolUseで差分を取る。文書化された項目だけで両agentを同じ仕組みで扱えるが、hookの登録が倍になり、ツールが失敗したときの退避ファイルの片付けが要る

## Consequences

- Claude Codeの `structuredPatch` は文書化されていない。無くなったときはファイルを変えずexit 1で終わるので、仕様の変更に気づける
- Codexは元のファイルで位置を決めるが、jafixには編集後のファイルしか無い。元からある同じ並びの行とCodexが書いた行を見分けられない入力では、そのファイルを直さない
