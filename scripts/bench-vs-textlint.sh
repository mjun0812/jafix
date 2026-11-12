#!/usr/bin/env bash
# jafixとtextlintを、同じ機能 (3つの修正ルールと2つの指摘ルール。jafixの移植した12ルールは無効にする) の設定にして、check・fix・hookの3つの場面で
# hyperfineで比べ、結果をMarkdownの表で標準出力に出す。
#
# textlintは、本体と次のルールを入れたprefix (`<prefix>/bin/textlint` と `<prefix>/node_modules`) を
# 環境変数 TEXTLINT_PREFIX で渡す。未設定なら `mise where npm:textlint` を使う。
#   textlint-rule-prh, textlint-rule-ja-space-between-half-and-full-width, @textlint-rule/textlint-rule-pattern
# 引数にディレクトリを渡すと、hyperfineの結果のJSONをそこへ残す。
set -euo pipefail

cd "$(dirname "$0")/.."
cargo build --release --locked
jafix=$PWD/target/release/jafix

prefix=${TEXTLINT_PREFIX:-$(mise where npm:textlint)}
textlint=$prefix/bin/textlint
for package in textlint-rule-prh textlint-rule-ja-space-between-half-and-full-width @textlint-rule/textlint-rule-pattern; do
    if [[ ! -d $prefix/node_modules/$package ]]; then
        echo "$package が $prefix/node_modules に無い" >&2
        exit 2
    fi
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
results=${1:-$work/results}
mkdir -p "$results" "$work/home" "$work/xdg/jafix" "$work/files" "$work/run"
export HOME=$work/home XDG_CONFIG_HOME=$work/xdg

# 指摘ルールは scripts/bench-hook.sh と同じ30語と1つの正規表現にする
words=(
    照合 正本 検査 経路 堅牢 包括的 網羅 活用 実現 最適化
    担保 示唆 留意 顕著 多岐 包含 円滑 推進 促進 寄与
    着実 迅速 柔軟 効率的 革新的 画期的 重要 不可欠 鍵となる 本質的
)
{
    # 比べるのは同じ機能の5つのルールだけなので、移植した12ルールは無効にする
    cat <<'TOML'
[builtin]
sentence-length = false
max-comma = false
max-kanji-continuous-len = false
arabic-kanji-numbers = false
ja-no-mixed-period = false
no-nfd = false
no-invalid-control-character = false
no-zero-width-spaces = false
no-exclamation-question-mark = false
no-hankaku-kana = false
ja-unnatural-alphabet = false
no-unmatched-pair = false

TOML
    echo '[[rules]]'
    echo 'id = "ai-words"'
    echo 'message = "AIが多用する語"'
    printf 'words = [%s]\n' "$(printf '"%s", ' "${words[@]}" | sed 's/, $//')"
    echo
    echo '[[rules]]'
    echo 'id = "long-kanji"'
    echo 'message = "漢字が長く続く"'
    echo "pattern = '\\p{sc=Han}{6,}'"
} >"$XDG_CONFIG_HOME/jafix/config.toml"

# textlintでは、全角括弧をprh、日本語と英数字の間の空白をja-space-between-half-and-full-width、
# 日本語と括弧の間の空白と2つの指摘ルールを@textlint-rule/patternで書く
printf 'version: 1\nrules:\n  - expected: (\n    pattern: （\n  - expected: )\n    pattern: ）\n' >"$work/prh.yml"
japanese='\\p{sc=Han}\\p{sc=Hiragana}\\p{sc=Katakana}ー'
fix_patterns="
        { \"message\": \"日本語と括弧の間の空白を削除する\", \"pattern\": \"/([$japanese])[ 　]+([(（])/u\", \"replace\": \"\$1\$2\" },
        { \"message\": \"日本語と括弧の間の空白を削除する\", \"pattern\": \"/([)）])[ 　]+([$japanese])/u\", \"replace\": \"\$1\$2\" }"
alternation=$(
    IFS='|'
    echo "${words[*]}"
)
report_patterns="$fix_patterns,
        { \"message\": \"AIが多用する語\", \"pattern\": \"/$alternation/u\" },
        { \"message\": \"漢字が長く続く\", \"pattern\": \"/\\\\p{sc=Han}{6,}/u\" }"
for kind in fix report; do
    patterns=$fix_patterns
    [[ $kind == report ]] && patterns=$report_patterns
    cat >"$work/textlint-$kind.json" <<JSON
{
  "rules": {
    "prh": { "rulePaths": ["$work/prh.yml"] },
    "ja-space-between-half-and-full-width": { "space": "never" },
    "@textlint-rule/pattern": { "patterns": [$patterns
    ] }
  }
}
JSON
done
tl=("$textlint" --rules-base-directory "$prefix/node_modules")

# textlintはhookの入力を読めず、対象の行も絞れないので、直してから指摘を出す処理をファイル全体にかける
cat >"$work/textlint-hook.sh" <<SH
#!/bin/sh
"$textlint" --rules-base-directory "$prefix/node_modules" --config "$work/textlint-fix.json" --fix "\$1" >/dev/null
"$textlint" --rules-base-directory "$prefix/node_modules" --config "$work/textlint-report.json" "\$1" >/dev/null
exit 0
SH
chmod +x "$work/textlint-hook.sh"

block='## 見出し Rust の使い方

日本語 Rust で （補足） です。サーバー API を使う場合は、設定 (config) を読む。

- リスト の 項目 A と B を 並べる
- **強調** の 中 の Rust 1.0 を 書く
- 結果を照合して正本と比べ、自然言語処理技術の要点を示す。

```sh
cargo build --release
```
'
for n in 10 100 1000; do
    for _ in $(seq "$n"); do
        printf '%s\n' "$block"
    done >"$work/files/blocks-$n.md"
done
cp README.md "$work/files/README.md"
files=(blocks-10 README blocks-100 blocks-1000)

echo "## 環境"
echo
echo "- $("$jafix" --version) ($(git rev-parse --short HEAD))、textlint $("$textlint" --version) (Node.js $(node --version))、$(hyperfine --version)"
echo "- $(uname -sm)$(sysctl -n machdep.cpu.brand_string 2>/dev/null | sed 's/^/、/')"
echo

# 同じ仕事をしているかを、最も小さいファイルでの機能ごとの件数で確かめる。
# textlintの@textlint-rule/patternは、日本語と括弧の間の空白と2つの指摘ルールをまとめて受け持つ
jafix_out=$("$jafix" check "$work/files/blocks-10.md" || true)
textlint_out=$("${tl[@]}" --config "$work/textlint-report.json" -f unix "$work/files/blocks-10.md" || true)
echo "## 機能ごとの件数 (blocks-10.md)"
echo
echo "| 機能 | jafix | textlint |"
echo "| --- | --: | --: |"
echo "| 全角括弧 | $(grep -c '\[fullwidth-paren\]$' <<<"$jafix_out" || true) | $(grep -c '/prh\]$' <<<"$textlint_out" || true) |"
echo "| 日本語と英数字の間の空白 | $(grep -c '\[space-between-ja-and-alnum\]$' <<<"$jafix_out" || true) | $(grep -c '/ja-space-between-half-and-full-width\]$' <<<"$textlint_out" || true) |"
echo "| 日本語と括弧の間の空白と、2つの指摘ルール | $(grep -c -E '\[(space-between-ja-and-paren|ai-words|long-kanji)\]$' <<<"$jafix_out" || true) | $(grep -c '/@textlint-rule/pattern\]$' <<<"$textlint_out" || true) |"
echo

for file in "${files[@]}"; do
    src=$work/files/$file.md
    runs=20
    [[ $file == blocks-1000 ]] && runs=10
    hyperfine -N --ignore-failure --warmup 3 --runs "$runs" --export-json "$results/check-$file.json" \
        -n jafix "$jafix check $src" \
        -n textlint "${tl[*]} --config $work/textlint-report.json $src" >/dev/null
    hyperfine -N --ignore-failure --warmup 3 --runs "$runs" --export-json "$results/fix-$file.json" \
        --prepare "cp $src $work/run/jafix.md" -n jafix "$jafix fix $work/run/jafix.md" \
        --prepare "cp $src $work/run/textlint.md" -n textlint "${tl[*]} --config $work/textlint-fix.json --fix $work/run/textlint.md" >/dev/null
    # 50行目から20行を、Editで書き換えた行として `+` にする
    lines=$(sed -n '50,69p' "$src" | jq -R '"+" + .' | jq -s -c .)
    jq -n -c --arg path "$work/run/hook.md" --argjson lines "$lines" '{
        hook_event_name: "PostToolUse", tool_name: "Edit", tool_input: {file_path: $path},
        tool_response: {filePath: $path, originalFile: "",
            structuredPatch: [{oldStart: 50, oldLines: 0, newStart: 50, newLines: ($lines | length), lines: $lines}]}
    }' >"$work/run/hook-$file.json"
    hyperfine -N --ignore-failure --warmup 3 --runs "$runs" --export-json "$results/hook-$file.json" \
        --input "$work/run/hook-$file.json" \
        --prepare "cp $src $work/run/hook.md" -n jafix "$jafix hook claude" \
        --prepare "cp $src $work/run/hook-textlint.md" -n textlint "$work/textlint-hook.sh $work/run/hook-textlint.md" >/dev/null
done

echo "## 結果"
echo
echo "時間は壁時計時間の平均、メモリは最大常駐メモリの平均。倍率はtextlintの時間をjafixの時間で割ったもの。"
echo
echo "| 場面 | ファイル | jafix | textlint | 倍率 |"
echo "| --- | --- | --: | --: | --: |"
for scene in check fix hook; do
    for file in "${files[@]}"; do
        size=$(($(wc -c <"$work/files/$file.md") / 1024))
        # hyperfine 2.xは `summary` に、1.xは結果の直下に平均を書く。メモリは2.xだけが書く
        jq -r --arg scene "$scene" --arg file "$file.md (${size}KB)" '
            def mean: .summary.time_wall_clock.mean // .mean;
            def cell: "\(mean * 10000 | round / 10)ms"
                + (if .summary.memory_peak_resident then " / \(.summary.memory_peak_resident.mean / 1048576 | round)MB" else "" end);
            .results as [$jafix, $textlint]
            | "| \($scene) | \($file) | \($jafix | cell) | \($textlint | cell) | \(($textlint | mean) / ($jafix | mean) | round)倍 |"
        ' "$results/$scene-$file.json"
    done
done
