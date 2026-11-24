#!/bin/sh
set -eu

# When piped to sh, parse the whole block before running any install steps.
{
    platform=$(uname -sm)
    case "$platform" in
        'Darwin arm64') target=aarch64-apple-darwin ;;
        'Linux x86_64') target=x86_64-unknown-linux-musl ;;
        'Linux aarch64') target=aarch64-unknown-linux-musl ;;
        *)
            printf 'jafix: unsupported platform: %s\n' "$platform" >&2
            exit 1
            ;;
    esac

    base_url=https://github.com/mjun0812/jafix/releases/latest/download
    archive=jafix-$target.tar.gz
    install_dir=${JAFIX_INSTALL_DIR:-"$HOME/.local/bin"}
    work=$(mktemp -d "${TMPDIR:-/tmp}/jafix-install.XXXXXXXX")
    trap 'rm -rf "$work"' 0

    curl -fsSL "$base_url/checksums.txt" -o "$work/checksums.txt"
    checksum=$(awk -v name="$archive" '$2 == name { print; exit }' "$work/checksums.txt")
    if [ -z "$checksum" ]; then
        printf 'jafix: checksum missing for %s\n' "$archive" >&2
        exit 1
    fi
    curl -fsSL "$base_url/$archive" -o "$work/$archive"
    (
        cd "$work"
        case "$platform" in
            Darwin*) printf '%s\n' "$checksum" | shasum -a 256 -c ;;
            Linux*) printf '%s\n' "$checksum" | sha256sum -c ;;
        esac
    )

    tar -xzf "$work/$archive" -C "$work" jafix
    mkdir -p "$install_dir"
    install -m 755 "$work/jafix" "$install_dir/jafix"
    printf 'Installed %s\n' "$install_dir/jafix"
    case ":$PATH:" in
        *":$install_dir:"*) ;;
        *) printf 'Add %s to PATH to run jafix.\n' "$install_dir" ;;
    esac
}
