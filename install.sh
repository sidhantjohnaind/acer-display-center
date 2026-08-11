#!/usr/bin/env bash
set -e

echo "=== Acer Monitor Control (amctl) Installer (Linux) ==="

# Build release binary if not present
if [ ! -f "target/release/acer_monitor_cli" ]; then
    echo "Building release binary..."
    cargo build --release
fi

TARGET_DIR="/usr/local/bin"
if [ -w "$TARGET_DIR" ]; then
    cp target/release/acer_monitor_cli "$TARGET_DIR/acer_monitor_cli"
    chmod +x "$TARGET_DIR/acer_monitor_cli"
    ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amctl"
    ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amc"
    echo "Installed binary and 'amctl' symlink to $TARGET_DIR/"
else
    echo "Requesting sudo privileges to install amctl..."
    sudo cp target/release/acer_monitor_cli "$TARGET_DIR/acer_monitor_cli"
    sudo chmod +x "$TARGET_DIR/acer_monitor_cli"
    sudo ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amctl"
    sudo ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amc"
    echo "Installed binary and 'amctl' symlink to $TARGET_DIR/"
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
echo "Run 'amctl --help' to get started."
