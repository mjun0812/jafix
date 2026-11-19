#!/usr/bin/env bash
# 指定したtargetでjafixをビルドし、バイナリとライセンスの文書をdistに梱包する
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "usage: $0 <target>" >&2
    exit 2
fi
target=$1

cd "$(dirname "$0")/.."
cargo build --release --locked --target "$target"
# binaryにリンクされるcrateのライセンスの一覧を、Cargo.lockとそのtargetの依存から作る。
# about.toml の accepted に無いライセンスのcrateがあると、ここで止まる。
# cargo-aboutは、targetを指定しても全platformのpackageのmetadataを読むので、buildが取得しないほかのOS向けのcrateも先に取得しておく。
# そのうえで --frozen を付け、crateのrepositoryからライセンスのファイルを取りに行かず、取得したcrateのソースだけを読ませる
cargo fetch --locked
licenses="target/$target/release/CRATE_LICENSES.txt"
about_log="target/$target/cargo-about.log"
rm -f "$licenses"
if ! cargo about generate --frozen --fail --target "$target" -o "$licenses.tmp" about.hbs 2>"$about_log"; then
    cat "$about_log" >&2
    exit 1
fi
cat "$about_log" >&2
# clarifyのchecksumが合わないとき、cargo-aboutはWARNを出すだけで、clarifyを使わずに一覧を作る
if grep -q WARN "$about_log"; then
    echo "error: cargo about generate がWARNを出しました。about.toml のclarifyのchecksumが、crateのファイルと合っているかを確かめてください" >&2
    exit 1
fi
mv "$licenses.tmp" "$licenses"
mkdir -p dist
# binaryにリンクされるcrateのライセンスの一覧と、IPADICの辞書などcrateの一覧に載らないものの著作権表示と条件を書いた文書を、一緒に配る。
notices=(LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md)
if [[ $target == *-windows-* ]]; then
    python -m zipfile -c "dist/jafix-$target.zip" "target/$target/release/jafix.exe" "$licenses" "${notices[@]}"
else
    # 2つ目の -C は1つ目からの相対になるので、絶対パスで書く
    tar -czf "dist/jafix-$target.tar.gz" -C "target/$target/release" jafix CRATE_LICENSES.txt \
        -C "$PWD" "${notices[@]}"
fi
