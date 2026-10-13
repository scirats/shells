#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="shell"
INSTALL_DIR="$HOME/.local/bin"
DATA_DIR="$HOME/.local/share/shell"
LEGACY_DIRS=("$HOME/.config/shell" "$HOME/.local/state/shell")

usage() {
    cat <<EOF
Usage: $(basename "$0")

Removes $BIN_NAME completely: stops the shells and deletes the binary, its
templates and every data directory.

Options:
  -h, --help   Show this help
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

if command -v limactl >/dev/null 2>&1 && [ -d "$DATA_DIR/lima" ]; then
    while IFS= read -r instance; do
        [ -n "$instance" ] || continue
        LIMA_HOME="$DATA_DIR/lima" limactl stop -f "$instance" >/dev/null 2>&1 || true
    done < <(LIMA_HOME="$DATA_DIR/lima" limactl list -q 2>/dev/null || true)
fi

bin="$INSTALL_DIR/$BIN_NAME"

if [ -e "$bin" ]; then
    rm -f "$bin"
    echo "Removed $bin"
fi

if [ -d "$INSTALL_DIR/templates" ]; then
    rm -rf "$INSTALL_DIR/templates"
    echo "Removed $INSTALL_DIR/templates"
fi

rmdir "$INSTALL_DIR" 2>/dev/null && echo "Removed empty $INSTALL_DIR" || true

if [ -d "$DATA_DIR" ]; then
    rm -rf "$DATA_DIR"
    echo "Removed configuration $DATA_DIR"
fi

for dir in "${LEGACY_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        rm -rf "$dir"
        echo "Removed legacy $dir"
    fi
done

echo "$BIN_NAME uninstalled."
