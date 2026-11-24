# ADR-0004: hookは指摘をexit 2と標準エラーでagentへ返す

jafixのhookは今回変えた行に指摘ルールの指摘があれば、標準エラーに1件1行で書いてexit 2で終える。Claude CodeとCodexはどちらもPostToolUseがexit 2で終わったときの標準エラーをagentへ渡すので、同じ書式を使える。同じ実行で対象行を決められなかった理由があれば、それも標準エラーに書く。ADR-0003のexit 1は指摘が無いときに限る。

## Considered Options

- exit 0で標準出力にJSONの `hookSpecificOutput.additionalContext` を出す。元のtoolの出力は残るが、Claude CodeとCodexでJSONの形を分ける必要がある。Codexは未知のフィールドがあるとJSON全体を無効にする
- 対象行を決められなかった理由があれば、exit 1を優先する。exit 1の標準エラーは利用者にしか見えないので、同じ実行の指摘がagentへ届かない

## Consequences

- Codexでは、元のtoolの出力(`Success. Updated the following files:` など)が標準エラーの文面に置き換わる。そのため最初の行で、編集は適用済みであることと、直すか誤検知なら直さずに進めてよいことを伝える
- PostToolUseは編集が終わってから呼ばれるので、exit 2でもtoolの実行は取り消されない。ADR-0003のexit 1と同じく、agentの作業を止めない
