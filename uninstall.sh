#!/usr/bin/env bash
# Acer Monitor CLI & Suite Clean Uninstaller (Linux/macOS)
set -e

echo "🗑️ Uninstalling Acer Monitor CLI (amctl) and background components..."

# 1. Stop and disable background systemd service if running
if command -v systemctl &> /dev/null; then
    echo "  • Stopping systemd background daemon..."
    systemctl --user stop acer-monitor-server.service 2>/dev/null || true
    systemctl --user disable acer-monitor-server.service 2>/dev/null || true
    rm -f "$HOME/.config/systemd/user/acer-monitor-server.service"
    systemctl --user daemon-reload 2>/dev/null || true
fi

# 2. Remove binaries and symlinks
echo "  • Removing executable binaries..."
sudo rm -f /usr/local/bin/amctl /usr/local/bin/acer_monitor_cli 2>/dev/null || true
rm -f "$HOME/.local/bin/amctl" "$HOME/.local/bin/acer_monitor_cli" 2>/dev/null || true

# 3. Remove Desktop Launcher & Rofi helper
echo "  • Removing desktop launchers & shell integration..."
rm -f "$HOME/.local/share/applications/acer-monitor-control.desktop"
rm -f "$HOME/.local/bin/rofi-acer.sh"

# 4. Remove GNOME Shell Extension
echo "  • Removing GNOME Shell Extension..."
rm -rf "$HOME/.local/share/gnome-shell/extensions/acer-monitor-control@sidhant.ai" 2>/dev/null || true

# 5. Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi

echo "✅ Acer Monitor CLI Suite has been completely uninstalled from your system!"
