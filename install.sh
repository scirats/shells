#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="shell"
INSTALL_DIR="$HOME/.local/bin"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<EOF
Usage: $(basename "$0")

Builds $BIN_NAME in release mode and installs the binary and its templates to
$INSTALL_DIR.
EOF
}

if [ $# -gt 0 ]; then
    case "$1" in
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found; install Rust from https://rustup.rs" >&2
    exit 1
fi

echo "Building $BIN_NAME (release)..."
cargo build --release --manifest-path "$REPO_DIR/Cargo.toml"

binary="$REPO_DIR/target/release/$BIN_NAME"
if [ ! -f "$binary" ]; then
    echo "error: expected binary not found at $binary" >&2
    exit 1
fi

mkdir -p "$INSTALL_DIR"
install -m 0755 "$binary" "$INSTALL_DIR/$BIN_NAME"
echo "Installed $INSTALL_DIR/$BIN_NAME"

templates_dir="$REPO_DIR/templates"
if [ -d "$templates_dir" ]; then
    rm -rf "$INSTALL_DIR/templates"
    cp -R "$templates_dir" "$INSTALL_DIR/templates"
    echo "Installed templates to $INSTALL_DIR/templates"
fi

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo "Note: $INSTALL_DIR is not in your PATH. Add it with:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        ;;
esac
