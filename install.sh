#!/usr/bin/env bash
# Build and install PomoGo for the current user.
#
#   ./install.sh              build, install to ~/.local/bin, set up desktop
#   ./install.sh --uninstall  remove everything this script installed
#
# Works on any Linux with a Rust toolchain. On Omarchy it also installs and
# enables the omarchy-shell bar widget. Set PREFIX to install elsewhere
# (the binary goes to $PREFIX/bin).
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
BASH_COMP_DIR="$DATA_HOME/bash-completion/completions"
ZSH_COMP_DIR="$DATA_HOME/zsh/site-functions"
FISH_COMP_DIR="$CONFIG_HOME/fish/completions"
APP_FILE="$DATA_HOME/applications/pomogo.desktop"

cd "$(dirname "$0")"

say() { printf '==> %s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

if [[ ${1:-} == --uninstall ]]; then
  if [[ -x $BIN_DIR/pomogo ]] && have omarchy; then
    "$BIN_DIR/pomogo" omarchy uninstall || true
  fi
  rm -f "$BIN_DIR/pomogo" "$APP_FILE" \
    "$BASH_COMP_DIR/pomogo" "$ZSH_COMP_DIR/_pomogo" "$FISH_COMP_DIR/pomogo.fish"
  say "Removed PomoGo (your sessions in $DATA_HOME/pomogo are kept)"
  exit 0
fi

if ! have cargo; then
  echo "cargo not found. Install Rust first:" >&2
  echo "  Arch/Omarchy: sudo pacman -S --needed rust base-devel" >&2
  echo "  Fedora:       sudo dnf install cargo gcc" >&2
  echo "  Other:        https://rustup.rs" >&2
  exit 1
fi

say "Building release binary"
cargo build --release --locked

say "Installing to $BIN_DIR"
install -Dm755 target/release/pomogo "$BIN_DIR/pomogo"
POMOGO="$BIN_DIR/pomogo"

say "Installing shell completions"
mkdir -p "$BASH_COMP_DIR" "$ZSH_COMP_DIR" "$FISH_COMP_DIR"
"$POMOGO" completion bash >"$BASH_COMP_DIR/pomogo"
"$POMOGO" completion zsh >"$ZSH_COMP_DIR/_pomogo"
"$POMOGO" completion fish >"$FISH_COMP_DIR/pomogo.fish"

say "Installing desktop entry"
install -Dm644 contrib/pomogo.desktop "$APP_FILE"
if [[ $BIN_DIR != "$HOME/.local/bin" && $BIN_DIR != /usr/bin && $BIN_DIR != /usr/local/bin ]]; then
  sed -i "s|^Exec=pomogo|Exec=$POMOGO|" "$APP_FILE"
fi

if have omarchy; then
  say "Omarchy detected: installing the bar widget"
  "$POMOGO" omarchy install
fi

# Warn when another pomogo (e.g. the Go release from the AUR) wins on PATH.
resolved="$(command -v pomogo || true)"
if [[ -n $resolved && $resolved != "$POMOGO" ]]; then
  echo
  echo "Note: \`pomogo\` on your PATH resolves to $resolved, not $POMOGO."
  echo "      Remove the other copy or put $BIN_DIR earlier in PATH."
elif [[ -z $resolved ]]; then
  echo
  echo "Note: $BIN_DIR is not on your PATH yet."
fi

echo
say "Done. Run \`pomogo\` to start, \`pomogo doctor\` to check your setup."
