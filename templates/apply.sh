#!/bin/bash
set -eu
SRC="{{source}}"
ENV_FILE="$HOME/.local/share/shell/env.sh"

mkdir -p "$(dirname "$ENV_FILE")"
if [ -f "$SRC/env.sh" ]; then
  cp -f "$SRC/env.sh" "$ENV_FILE"
fi

if [ -d "$SRC/home" ]; then
  cp -a "$SRC/home/." "$HOME/"
fi

if [ -n "{{term}}" ] && [ -f "$SRC/terminfo" ] && command -v tic >/dev/null 2>&1; then
  if ! infocmp "{{term}}" >/dev/null 2>&1; then
    tic -x "$SRC/terminfo" >/dev/null 2>&1 || true
  fi
fi

for rc in "$HOME/.bashrc" "$HOME/.profile" "$HOME/.zshrc"; do
  touch "$rc"
  if ! grep -q "shell-manager" "$rc" 2>/dev/null; then
    {
      echo ""
      echo "# >>> shell-manager >>>"
      echo "[ -f \"$ENV_FILE\" ] && . \"$ENV_FILE\""
      echo "[ -d \"\$HOME/Workspace\" ] && cd \"\$HOME/Workspace\""
      echo "# <<< shell-manager <<<"
    } >> "$rc"
  fi
done

if command -v zsh >/dev/null 2>&1 && command -v sudo >/dev/null 2>&1; then
  zsh_bin="$(command -v zsh)"
  current="$(getent passwd "$(id -un)" | cut -d: -f7 || true)"
  if [ "$current" != "$zsh_bin" ]; then
    if ! grep -qx "$zsh_bin" /etc/shells 2>/dev/null; then
      echo "$zsh_bin" | sudo -n tee -a /etc/shells >/dev/null 2>&1 || true
    fi
    sudo -n chsh -s "$zsh_bin" "$(id -un)" >/dev/null 2>&1 || true
  fi
fi

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"
cat > "$BIN_DIR/xdg-open" <<'SHIM'
#!/bin/sh
url="$1"
case "$url" in
  http://* | https://*) ;;
  *)
    echo "xdg-open: only http(s) URLs are supported" >&2
    exit 2
    ;;
esac
if command -v curl >/dev/null 2>&1 && \
  curl -fsS --max-time 5 -X POST --data-binary "$url" "http://127.0.0.1:{{browser_port}}/open" >/dev/null 2>&1; then
  exit 0
fi
printf 'shell: the host browser bridge is unavailable; open manually:\n  %s\n' "$url" >&2
exit 1
SHIM
chmod +x "$BIN_DIR/xdg-open"
ln -sf xdg-open "$BIN_DIR/x-www-browser"
ln -sf xdg-open "$BIN_DIR/www-browser"
