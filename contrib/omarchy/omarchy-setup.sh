#!/usr/bin/env bash
# First-party Omarchy setup script for PomoGo.
set -euo pipefail

echo "==> Setting up PomoGo for Omarchy Linux..."

# 1. Install Quickshell Bar Widget
PLUGIN_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/omarchy/plugins/pomogo"
mkdir -p "$PLUGIN_DIR"
cp "$(dirname "$0")/quickshell-plugin/PomogoBar.qml" "$PLUGIN_DIR/"
cp "$(dirname "$0")/quickshell-plugin/plugin.json" "$PLUGIN_DIR/"
echo "✔ Quickshell plugin installed at: $PLUGIN_DIR"

# 2. Install Desktop Launcher
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$APP_DIR"
cp "$(dirname "$0")/../pomogo.desktop" "$APP_DIR/"
echo "✔ Desktop file installed at: $APP_DIR/pomogo.desktop"

# 3. Add Hyprland window rule for floating Pomogo scratchpad/modal if needed
HYPR_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/hypr/hyprland.conf"
if [ -f "$HYPR_CONFIG" ]; then
    if ! grep -q "pomogo" "$HYPR_CONFIG"; then
        echo "windowrulev2 = float, class:(pomogo)" >> "$HYPR_CONFIG"
        echo "✔ Added Hyprland floating window rule to $HYPR_CONFIG"
    fi
fi

echo "==> PomoGo Omarchy installation complete! Launch with 'pomogo'."

