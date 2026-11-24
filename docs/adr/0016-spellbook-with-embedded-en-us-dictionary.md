# ADR-0016: `spelling` は、埋め込んだen_USの辞書を `spellbook` で引く

`spelling` は、英単語の綴りを、`dictionary-en` の4.0.0のen_USのhunspellの辞書(`index.aff`・`index.dic`)で判定する。辞書のファイルは中身を変えずにbinaryへ埋め込み、語の判定と候補には `spellbook` 0.4.2 (MPL-2.0)の `check` と `suggest` を使う。`language` オプションは `"en"` だけを受け、他の値は設定の不正にする。利用者が辞書のファイルを指定するオプションは作らない。

辞書を引くcrateを `spellbook` にしたのは、Rustだけで書かれ、`check` と `suggest` の両方を持つcrateが他に無いためである。jafixは1つのbinaryで配り、C++のbuildを持たない。指摘ルールにした `spelling` では、候補を文面に出してagentに示せることが役に立つ(ADR-0009)。`spellbook` のREADMEは自身を「alpha」とするが、試作では、既定の語の取り出し方で取り出せる語の判定がnspellと同じだった。辞書の読み込みは約7ms、30語の `check` は約0.14msだった。2026-11-16に、利用者がこの案を選んだ。

辞書を埋め込み、`language` を `"en"` に限ったのは、textlintの既定と同じ結果になる最小の形だからである。textlintでは、利用者が `dictionary-<language>` のpackageを入れて `language` で選ぶ。jafixにはnpmのpackageを探す仕組みが無い。既定の `"en"` の辞書を持てば、設定を書くだけで動く。他の言語とen_GBを使えないことは利用者から見える制約なので、2026-11-16に、利用者がこの案を選んだ。

## Considered Options

- `zspell` 0.5.5 (Apache-2.0)。`check` は安定しているが、`suggest` が未完成で、候補を文面に出せない。辞書を読むときに接辞を展開するので、メモリを多く使う
- `hunspell-rs` の0.4.0と `hunspell-sys` の0.3.1 (C++のhunspellのbinding)。判定はhunspellそのものだが、C++のlibraryのbuildかlinkが要り、配布する各targetのbuildが重くなる
- nspell (MIT)の判定と候補の手順を、Rustに書き直す。nspellと同じ結果になるが、affixの規則の解釈と候補の作り方を自前で持ち続けることになる
- 辞書を埋め込まず、利用者が辞書のファイルを置く。binaryは増えないが、設定を書くだけでは動かない
- 埋め込んだ辞書に加えて、jafix独自のオプションで利用者のhunspellの辞書を読む。他の言語を使えるが、textlintに無いオプションを足す

## Consequences

- 語が正しいかの判定と候補は、nspellではなく `spellbook` に従う。ハイフンで分けた語と末尾の `.` を含む語の判定と、候補の並びと数は、textlintと違う。READMEの「textlintとの違い」に書く
- `spellbook` は、既定の `Dictionary::new` がプロセスごとに種の変わるhasherで語の表を作るので、候補の並びが実行ごとに変わる。jafixは、`foldhash` 0.2.0 (Zlib)の固定の種の `FixedState` を `Dictionary::new_with_hasher` に渡す。`spellbook` の既定の機能を切り、`Dictionary::new` を使えなくする。種を変えると候補の並びが変わりうるので、変えたらfixtureとREADMEの例を見直す
- binaryは辞書(554,848バイト)と `spellbook` の分だけ大きくなる。jafixが辞書を読むのは `spelling` が有効で判定する語があるときの1回だけである
- `THIRD_PARTY_NOTICES.md` に、`dictionary-en` の `license` のファイルの文面(SCOWLとIspellの条件)と、`spellbook` のMPL-2.0とソースの入手先を書く。MPL-2.0はファイル単位の継承なので、jafixのコードのライセンスは変わらない
- `dictionary-en` か `spellbook` のversionを上げるときは、判定と候補が変わりうるので、fixtureを確かめ直す
