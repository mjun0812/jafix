#!/usr/bin/env bash
# 約30KBのMarkdownへのEditの入力で `jafix hook claude` をhyperfineで計り、平均が0.05秒未満ならexit 0、以上ならexit 1で終える
set -euo pipefail

threshold=0.05

cd "$(dirname "$0")/.."
cargo build --release --locked
jafix=$PWD/target/release/jafix

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/home" "$work/xdg"
export HOME=$work/home XDG_CONFIG_HOME=$work/xdg

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
hyperfine -N --runs 30 --warmup 3 \
    --prepare "cp $work/original.md $work/t.md" \
    --input "$work/input.json" \
    --export-csv "$work/result.csv" \
    "$jafix hook claude"

# hyperfine 1.xは `mean`、2.xは `time_wall_clock_mean` の列に秒で書く
mean=$(awk -F, 'NR == 1 { for (i = 1; i <= NF; i++) if ($i == "mean" || $i == "time_wall_clock_mean") col = i } NR == 2 { print $col }' "$work/result.csv")
echo "mean: ${mean}s (threshold: ${threshold}s)"
awk -v mean="$mean" -v threshold="$threshold" 'BEGIN { exit !(mean < threshold) }'
