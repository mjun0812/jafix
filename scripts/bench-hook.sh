#!/usr/bin/env bash
# 30語の `words` と1つの `pattern` の指摘ルールを書いた設定で、約30KBのMarkdownへのEditの入力の `jafix hook claude` を
# hyperfineで計り、平均が0.05秒未満ならexit 0、以上ならexit 1で終える
set -euo pipefail

threshold=0.05

cd "$(dirname "$0")/.."
cargo build --release --locked
jafix=$PWD/target/release/jafix

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/home" "$work/xdg/jafix"
export HOME=$work/home XDG_CONFIG_HOME=$work/xdg
cat >"$XDG_CONFIG_HOME/jafix/config.toml" <<'TOML'
[[rules]]
id = "ai-words"
message = "AIが多用する語"
words = [
    "照合", "正本", "検査", "経路", "堅牢", "包括的", "網羅", "活用", "実現", "最適化",
    "担保", "示唆", "留意", "顕著", "多岐", "包含", "円滑", "推進", "促進", "寄与",
    "着実", "迅速", "柔軟", "効率的", "革新的", "画期的", "重要", "不可欠", "鍵となる", "本質的",
]

[[rules]]
id = "long-kanji"
message = "漢字が長く続く"
pattern = '\p{sc=Han}{6,}'
TOML

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
