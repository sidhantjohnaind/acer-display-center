#!/usr/bin/env bash
set -e

echo "=== Acer Monitor CLI & Service Installer (Linux) ==="

# Build release binary if not present
if [ ! -f "target/release/acer_monitor_cli" ]; then
    echo "Building release binary..."
    cargo build --release
fi

# Determine target bin directory
TARGET_DIR="/usr/local/bin"
if [ -w "$TARGET_DIR" ]; then
    cp target/release/acer_monitor_cli "$TARGET_DIR/acer_monitor_cli"
    chmod +x "$TARGET_DIR/acer_monitor_cli"
    echo "Installed binary to $TARGET_DIR/acer_monitor_cli"
else
    echo "Requesting sudo privileges to install binary to $TARGET_DIR..."
    sudo cp target/release/acer_monitor_cli "$TARGET_DIR/acer_monitor_cli"
    sudo chmod +x "$TARGET_DIR/acer_monitor_cli"
    echo "Installed binary to $TARGET_DIR/acer_monitor_cli"
fi

# Ensure i2c-dev module is loaded
if ! lsmod | grep -q "i2c_dev"; then
    echo "Loading kernel module i2c-dev..."
    sudo modprobe i2c-dev || true
fi

# Install desktop entry and systemd user service
acer_monitor_cli install-desktop || true
acer_monitor_cli install-service || true

echo ""
echo "Installation complete!"
echo "Run 'acer_monitor_cli --help' to get started."
