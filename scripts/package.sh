#!/usr/bin/env bash
# 指定したtargetでjafixをビルドし、dist/jafix-<target>.tar.gz に jafix だけを入れて梱包する
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <target>" >&2
  exit 2
fi
target=$1

cd "$(dirname "$0")/.."
cargo build --release --locked --target "$target"
mkdir -p dist
# -C で移動してから入れ、tarの中にディレクトリを含めない
tar -czf "dist/jafix-$target.tar.gz" -C "target/$target/release" jafix
