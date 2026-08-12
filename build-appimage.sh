#!/usr/bin/env bash
# Script to build standalone Linux .AppImage package for Acer Monitor CLI (amctl)
set -e

echo "📦 Building release binary..."
cargo build --release

APPDIR="AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/icons/hicolor/128x128/apps"

echo "📂 Structuring AppDir..."
cp target/release/acer_monitor_cli "$APPDIR/usr/bin/amctl"

# Create Desktop Entry
cat <<EOF > "$APPDIR/amctl.desktop"
[Desktop Entry]
Name=Acer Monitor CLI
Comment=Command Line & IPC Suite for Acer Monitors
Exec=amctl
Icon=amctl
Terminal=true
Type=Application
Categories=Settings;System;
EOF

# Create AppRun entrypoint script
cat <<'EOF' > "$APPDIR/AppRun"
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/amctl" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# Create a default display icon
curl -sL https://raw.githubusercontent.com/google/material-design-icons/master/png/hardware/desktop_windows/materialicons/48dp/2x/baseline_desktop_windows_black_48dp.png -o "$APPDIR/amctl.png" || touch "$APPDIR/amctl.png"

echo "⚡ Generating amctl-x86_64.AppImage..."
ARCH=x86_64 /tmp/appimagetool-bin/AppRun "$APPDIR" "amctl-x86_64.AppImage"

echo "✅ AppImage created successfully: amctl-x86_64.AppImage"
