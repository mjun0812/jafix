#!/usr/bin/env bash
# 指定したtargetでjafixをビルドし、dist/jafix-<target>.tar.gz に jafix とライセンスの文書を入れて梱包する
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <target>" >&2
  exit 2
fi
target=$1

cd "$(dirname "$0")/.."
cargo build --release --locked --target "$target"
mkdir -p dist
# binaryにIPADICの辞書が入るので、その著作権表示と条件を書いた文書も一緒に配る。
# -C で移動してから入れ、tarの中にディレクトリを含めない。2つ目の -C は1つ目からの相対になるので、絶対パスで書く
tar -czf "dist/jafix-$target.tar.gz" -C "target/$target/release" jafix \
  -C "$PWD" LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md
