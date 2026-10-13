if [ -z "${SHELL_MANAGER_ENV_LOADED:-}" ]; then
  export SHELL_MANAGER_ENV_LOADED=1
  export SHELL_MANAGER_NAME="{{name}}"
  export PATH="$HOME/.local/share/mise/shims:$HOME/.local/bin:$PATH"
  export BROWSER="${BROWSER:-xdg-open}"
fi
