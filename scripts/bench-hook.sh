#!/usr/bin/env bash
# 既定で無効な組み込みのルールもすべて有効にし、`prh` に1,300ルールの辞書を、`@textlint-rule/pattern` に30語と
# 1つの正規表現の31要素を書いた設定で、約30KBのMarkdownへのEditの入力の `jafix hook claude` をhyperfineで計り、
# 平均が0.05秒未満ならexit 0、以上ならexit 1で終える
set -euo pipefail

threshold=0.05

cd "$(dirname "$0")/.."
cargo build --release --locked
jafix=$PWD/target/release/jafix

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/home" "$work/xdg/jafix"
export HOME=$work/home XDG_CONFIG_HOME=$work/xdg
# 辞書は、公開の辞書で最も大きい `WEB+DB_PRESS.yml` と同じ規模にする。文字列の `pattern` 1,200 (うち80に `specs`)、
# `expected` だけ50、後読みと先読みと `$1` を使う正規表現50である。組み込みの修正ルールと逆の置き換えは入れない
awk 'BEGIN {
    print "rules:"
    for (i = 1; i <= 1200; i++) {
        printf "  - expected: 用語%d\n    pattern: ようご%d\n", i, i
        if (i <= 80) printf "    specs:\n      - from: ようご%dを使う\n        to: 用語%dを使う\n", i, i
    }
    for (i = 1; i <= 50; i++) printf "  - expected: Term%d\n", i
    for (i = 1; i <= 50; i++) printf "  - expected: サーバー%d$1\n    pattern: \x27/(?<=の)サーバ%d(?!ー)(を|が)/\x27\n", i, i
}' >"$XDG_CONFIG_HOME/jafix/prh.yml"

words=(
    照合 正本 検査 経路 堅牢 包括的 網羅 活用 実現 最適化
    担保 示唆 留意 顕著 多岐 包含 円滑 推進 促進 寄与
    着実 迅速 柔軟 効率的 革新的 画期的 重要 不可欠 鍵となる 本質的
)
{
    cat <<'TOML'
[builtin]
"@textlint-ja/preset-ai-writing" = true
preset-ai-words-ja = true
no-short-topic-comma = true
preset-ja-spacing = true
ja-space-around-emphasis = true
ja-space-around-strong = true
no-mixed-zenkaku-and-hankaku-alphabet = true
no-kangxi-radicals = true
prh = { rulePaths = ["prh.yml"] }
TOML
    for word in "${words[@]}"; do
        printf '\n[[builtin."@textlint-rule/pattern".patterns]]\nmessage = "AIが多用する語"\npattern = "%s"\n' "$word"
    done
    printf '\n[[builtin."@textlint-rule/pattern".patterns]]\nmessage = "漢字が長く続く"\npattern = %s\n' "'/\\p{sc=Han}{6,}/'"
} >"$XDG_CONFIG_HOME/jafix/config.toml"

block='## 見出し Rust の使い方

日本語 Rust で （補足） です。サーバー API を使う場合は、設定 (config) を読む。

- リスト の 項目 A と B を 並べる
- **強調** の 中 の Rust 1.0 を 書く

```sh
cargo build --release
```
'
for _ in $(seq 115); do
    printf '%s\n' "$block"
done >"$work/original.md"
cp "$work/original.md" "$work/t.md"

# 500行目から20行を、Editで書き換えた行として `+` にする
start=500
patch_lines=$(sed -n "${start},$((start + 19))p" "$work/original.md" | jq -R '"+" + .' | jq -s -c .)
jq -n -c --arg path "$work/t.md" --argjson start "$start" --argjson lines "$patch_lines" '{
  hook_event_name: "PostToolUse",
  tool_name: "Edit",
  tool_input: {file_path: $path, old_string: "", new_string: "", replace_all: false},
  tool_response: {
    filePath: $path,
    originalFile: "",
    structuredPatch: [{oldStart: $start, oldLines: 0, newStart: $start, newLines: ($lines | length), lines: $lines}],
    userModified: false,
    replaceAll: false
  }
}' >"$work/input.json"

echo "Markdown: $(wc -c <"$work/original.md" | tr -d ' ') bytes" >&2
# 指摘があるとhookはexit 2で終えるので、exit codeの0以外を失敗とみなさない
hyperfine -N --runs 30 --warmup 3 --ignore-failure \
    --prepare "cp $work/original.md $work/t.md" \
    --input "$work/input.json" \
    --export-csv "$work/result.csv" \
    "$jafix hook claude"

# hyperfine 1.xは `mean`、2.xは `time_wall_clock_mean` の列に秒で書く
mean=$(awk -F, 'NR == 1 { for (i = 1; i <= NF; i++) if ($i == "mean" || $i == "time_wall_clock_mean") col = i } NR == 2 { print $col }' "$work/result.csv")
echo "mean: ${mean}s (threshold: ${threshold}s)"
awk -v mean="$mean" -v threshold="$threshold" 'BEGIN { exit !(mean < threshold) }'
