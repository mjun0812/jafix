# jafix

jafixはtextlintの日本語向けルールをRustで実装した高速なLinterです。
バイナリ1つで文章をチェックし、直せる箇所は自動で修正できます。

## 導入

macOS (Apple Silicon)とLinux (x86_64、ARM64)では、`curl` でインストールできます。

```sh
curl -fsSL https://github.com/mjun0812/jafix/releases/latest/download/install.sh | sh
```

[GitHub Releases](https://github.com/mjun0812/jafix/releases)から環境に合うバイナリをダウンロードします。展開した `jafix` または `jafix.exe` を `PATH` の通ったディレクトリに置きます。

[mise](https://mise.jdx.dev/)でも配布バイナリをインストールできます。

```sh
mise use -g github:mjun0812/jafix
```

配布ファイルの選び方とソースからのビルド手順は[導入の詳細](REFERENCE.md#導入)にあります。

## 使い方

チェックするテキストファイルを指定します。

```sh
jafix check example.txt
jafix fix example.txt
```

`check` はファイルを書き換えず、ルールに当たった箇所を表示します。`fix` は修正ルールを適用してファイルに保存します。
どちらもMarkdownの構文に従って本文を判定し、コードブロックなどを対象から外します。

入力はUTF-8とShift_JIS (CP932を含む)に対応します。文字コードはファイルごとに自動で判定します。[文字コードを指定する方法](REFERENCE.md#使い方)
Gitで追加・変更した行だけを校正するには `--diff` を指定します。ステージ済みの変更を確認するには `check --staged` を使います。

パイプで渡したテキストを校正するには、ファイルの代わりに `-` を指定します。`check` は指摘のパスを `<stdin>` として表示し、`fix` はファイルを書き換えずに修正後の本文を標準出力に書きます。[詳細](REFERENCE.md#標準入力)

```sh
echo "本文（補足）です。" | jafix check -
echo "本文（補足）です。" | jafix fix -
```

この例は、パイプへ文字をUTF-8で渡すシェルを前提にしています。[Windows PowerShell 5.1での渡し方](REFERENCE.md#標準入力)

組み込みのルールが今の設定で有効かどうかは、`jafix rules` で確かめられます。

```sh
jafix rules
```

1行に1ルールを、ルールID、`有効` か `無効`、属するpresetの名前(どのpresetにも属さなければ `-`)のタブ区切りで出力します。ファイルを渡すと、そのファイルに使うプロジェクト設定で答えます。`exclude` と `report-exclude` によるファイルの除外は反映しません。

各コマンドの実行例と終了コードは `jafix <コマンド> --help` で表示できます。agentやスクリプトから使う場合は、`jafix --usage` でコマンド体系を、`jafix check --schema` でJSON出力の形をJSONで取得できます。[詳細](REFERENCE.md#ルールの一覧)

## 設定ファイル

共通設定は `~/.config/jafix/config.toml` に書きます。
環境変数 `XDG_CONFIG_HOME` を設定している場合の共通設定は `$XDG_CONFIG_HOME/jafix/config.toml` です。
プロジェクト設定は `.jafix.toml` に書きます。
同じルールを両方で指定した場合は、対象ファイル(標準入力ではカレントディレクトリ)に最も近い `.jafix.toml` の設定が優先されます。

使うルールは `[builtin]` で選べます。
ルールのIDに `true` を指定すると有効になり、`false` を指定すると無効になります。
文の長さの上限などはオプションで指定できます。

```toml
[builtin]
sentence-length = { max = 120 }
no-unmatched-pair = false
common-misspellings = true
```

設定ファイルがなくても10ルールが有効です。
[初期設定のルール一覧](REFERENCE.md#組み込みのルール)

## hookへの登録

Claude CodeやCodexのPostToolUse hookに登録すると、jafixは `.md` ファイルの編集後に動きます。
編集で追加・変更した行を自動修正し、その行に残った指摘をagentに返します。

| 対象        | 設定ファイル    | matcher       | コマンド            |
| ----------- | --------------- | ------------- | ------------------- |
| Claude Code | `settings.json` | `Edit\|Write` | `jafix hook claude` |
| Codex       | `hooks.json`    | `apply_patch` | `jafix hook codex`  |

[登録用のJSONとhookの挙動](REFERENCE.md#hookへの登録)

## MCP

`jafix mcp` を登録すると、agentが下書きの文章を直接チェックし、自動修正した本文を受け取れます。[MCPの登録手順](docs/quick-start.md#下書きをmcpで校正する-mcp)

## Agent Skill

Agent Skillを導入すると、Claude CodeやCodexが日本語の文書を編集した後に、今回追加・変更した行と新規ファイルをjafixで校正します。
hookを登録していない環境でも、agentが自動修正と手修正を使い分け、指摘が無くなるまで確かめます。

```sh
npx skills add mjun0812/jafix
```

手で導入する場合は、[`skills/jafix`](skills/jafix)ディレクトリを次の場所にコピーします。

| 対象        | 置き場所                 |
| ----------- | ------------------------ |
| Claude Code | `~/.claude/skills/jafix` |
| Codex       | `~/.agents/skills/jafix` |

skillは `jafix` コマンドを使うので、先にjafixを導入します。

## ドキュメントサイト

- [ドキュメントサイト](https://mjunya.com/jafix/) ([English](https://mjunya.com/jafix/en/))
- [ルール一覧](docs/rules.md) ([English](docs/en/rules.md))
- [設定と動作のリファレンス](REFERENCE.md)
- [textlintとの速度の比較](docs/benchmark-vs-textlint.md)
- [ドキュメントサイトの開発と公開](REFERENCE.md#ドキュメントサイト)
- [リリースの手順](REFERENCE.md#リリース)

## ライセンス

[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のデュアルライセンスです。
依存ライブラリや辞書の出典とライセンスは [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にあります。
