#!/usr/bin/env bash
set -e

echo "================================================================"
echo "   🚀 Installing Acer Display Center & Monitor CLI Suite (amctl)"
echo "================================================================"

# 1. Locate local release binary or download from GitHub release
BIN_PATH=""
if [ -f "target/release/acer_monitor_cli" ]; then
    BIN_PATH="target/release/acer_monitor_cli"
elif [ -f "target/x86_64-unknown-linux-gnu/release/acer_monitor_cli" ]; then
    BIN_PATH="target/x86_64-unknown-linux-gnu/release/acer_monitor_cli"
elif [ -f "target/aarch64-unknown-linux-gnu/release/acer_monitor_cli" ]; then
    BIN_PATH="target/aarch64-unknown-linux-gnu/release/acer_monitor_cli"
elif [ -f "target/riscv64gc-unknown-linux-gnu/release/acer_monitor_cli" ]; then
    BIN_PATH="target/riscv64gc-unknown-linux-gnu/release/acer_monitor_cli"
fi

if [ -z "$BIN_PATH" ]; then
    ARCH=$(uname -m)
    echo "[*] Downloading latest standalone binary for architecture ($ARCH)..."
    TMP_BIN="/tmp/amctl-linux"
    
    if [ "$ARCH" = "x86_64" ]; then
        DOWNLOAD_URL="https://github.com/sidhantjohnaind/acer-display-center/releases/latest/download/amctl-x86_64-unknown-linux-gnu"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        DOWNLOAD_URL="https://github.com/sidhantjohnaind/acer-display-center/releases/latest/download/amctl-aarch64-unknown-linux-gnu"
    elif [ "$ARCH" = "riscv64" ]; then
        DOWNLOAD_URL="https://github.com/sidhantjohnaind/acer-display-center/releases/latest/download/amctl-riscv64gc-unknown-linux-gnu"
    else
        DOWNLOAD_URL="https://github.com/sidhantjohnaind/acer-display-center/releases/latest/download/amctl-x86_64-unknown-linux-gnu"
    fi

    if curl -sSL --fail "$DOWNLOAD_URL" -o "$TMP_BIN" 2>/dev/null; then
        chmod +x "$TMP_BIN"
        BIN_PATH="$TMP_BIN"
        echo "[+] Downloaded binary successfully."
    else
        echo "[*] Compiling binary locally with Cargo..."
        cargo build --release
        BIN_PATH="target/release/acer_monitor_cli"
    fi
fi

# 2. Install binary to /usr/local/bin or ~/.local/bin
TARGET_DIR="/usr/local/bin"
if [ -w "$TARGET_DIR" ]; then
    cp "$BIN_PATH" "$TARGET_DIR/acer_monitor_cli"
    chmod +x "$TARGET_DIR/acer_monitor_cli"
    ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amctl"
    ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amc"
    echo "[+] Installed binary to $TARGET_DIR/amctl"
else
    if command -v sudo >/dev/null 2>&1; then
        sudo cp "$BIN_PATH" "$TARGET_DIR/acer_monitor_cli"
        sudo chmod +x "$TARGET_DIR/acer_monitor_cli"
        sudo ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amctl"
        sudo ln -sf "$TARGET_DIR/acer_monitor_cli" "$TARGET_DIR/amc"
        echo "[+] Installed binary to $TARGET_DIR/amctl"
    else
        USER_BIN="$HOME/.local/bin"
        mkdir -p "$USER_BIN"
        cp "$BIN_PATH" "$USER_BIN/acer_monitor_cli"
        chmod +x "$USER_BIN/acer_monitor_cli"
        ln -sf "$USER_BIN/acer_monitor_cli" "$USER_BIN/amctl"
        ln -sf "$USER_BIN/acer_monitor_cli" "$USER_BIN/amc"
        echo "[+] Installed binary to $USER_BIN/amctl"
    fi
fi

# 3. Ensure i2c-dev module is loaded and udev permission is configured
if command -v sudo >/dev/null 2>&1; then
    if ! lsmod | grep -q "i2c_dev"; then
        sudo modprobe i2c-dev || true
        echo "i2c-dev" | sudo tee -a /etc/modules-load.d/i2c-dev.conf >/dev/null 2>&1 || true
    fi
    # Add current user to i2c group if group exists
    sudo usermod -aG i2c "$USER" 2>/dev/null || true
fi

# 4. Configure Linux Systemd User Startup Service
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
mkdir -p "$SYSTEMD_USER_DIR"
cat << 'EOF' > "$SYSTEMD_USER_DIR/acer-display-center.service"
[Unit]
Description=Acer Display Center IPC & Background Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/amctl tray
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload || true
    systemctl --user enable --now acer-display-center.service || true
    echo "[+] Configured and started systemd user service (acer-display-center.service)"
fi

# 5. Configure XDG Autostart Desktop Entry (for non-systemd desktop sessions)
AUTOSTART_DIR="$HOME/.config/autostart"
mkdir -p "$AUTOSTART_DIR"
cat << 'EOF' > "$AUTOSTART_DIR/acer-display-center.desktop"
[Desktop Entry]
Type=Application
Name=Acer Display Center Tray Daemon
Comment=Acer Monitor Control Background Tray & Quick Settings
Exec=amctl tray
Icon=display
Terminal=false
Categories=Utility;Settings;
X-GNOME-Autostart-enabled=true
EOF
echo "[+] Created XDG autostart entry (~/.config/autostart/acer-display-center.desktop)"

# 6. Install Desktop App Launcher
APP_DIR="$HOME/.local/share/applications"
mkdir -p "$APP_DIR"
cat << 'EOF' > "$APP_DIR/acer-display-center.desktop"
[Desktop Entry]
Type=Application
Name=Acer Display Center
Comment=Acer Monitor Quick Settings & Hardware Control
Exec=amctl gui
Icon=display
Terminal=false
Categories=Utility;Settings;HardwareSettings;
EOF
echo "[+] Created Application Launcher in ~/.local/share/applications"

echo ""
echo "================================================================"
echo "   ✅ Linux Installation Successful!"
echo "   • Background IPC Daemon is active and starts on login"
echo "   • Run 'amctl --help' for CLI commands"
echo "   • Run 'amctl info' to inspect connected monitor hardware"
echo "================================================================"
