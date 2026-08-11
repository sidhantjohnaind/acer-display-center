#!/usr/bin/env bash
set -e

echo "=== Installing Ubuntu GNOME Top Bar Extension ==="

EXT_UUID="acer-monitor@sidhant"
EXT_DIR="$HOME/.local/share/gnome-shell/extensions/$EXT_UUID"

mkdir -p "$EXT_DIR"
cp gnome-extension/metadata.json "$EXT_DIR/"
cp gnome-extension/extension.js "$EXT_DIR/"

echo "Installed extension to $EXT_DIR"

if command -v gnome-extensions >/dev/null 2>&1; then
    gnome-extensions enable "$EXT_UUID" || true
    echo "Extension enabled! If icon does not appear immediately, press Alt+F2, type 'r', and press Enter to reload GNOME Shell."
else
    echo "Extension installed. Enable using GNOME Extensions app or reload GNOME Shell."
fi
