# ADR-0018: プロジェクト設定で共通設定を上書きする

設定は組み込みの既定値、共通設定、プロジェクト設定の順に適用する。
共通設定のパスは `$XDG_CONFIG_HOME/jafix/config.toml` で、`XDG_CONFIG_HOME` が未設定なら `~/.config/jafix/config.toml` を使う。
対象ファイルの実体から親ディレクトリをたどり、最初の `.jafix.toml` を使う。
`.git` があるディレクトリまでを探すため、入れ子のリポジトリは外側の設定を継承しない。
CLIとhookは同じ探索順を使い、一度の実行で複数のプロジェクトを扱える。

`[builtin]` と `[filters]` は項目ごとに上書きし、同じ項目のオプションは表ごと置き換える。
各設定ファイルではpresetを先に適用し、次にルールIDの指定を適用する。
`exclude` と `report-exclude` は一覧ごと上書きする。
相対パスは指定を書いた設定ファイルを基準にする。
ルールどうしの競合はすべての設定を適用した後に判定する。

初期状態で使うのは、表記を整え、書き間違いを指摘する[10ルール](../../REFERENCE.md#組み込みのルール)とする。
文の長さや文体などは、利用者が書くものに合わせて選ぶ。
`no-kangxi-radicals` と `date-weekday-mismatch` は、[ADR-0007](0007-new-ported-rules-are-off-by-default.md) の例外とする。
