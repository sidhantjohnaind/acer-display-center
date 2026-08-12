#!/usr/bin/env bash
# Rofi / Dmenu Quick Control Menu for Acer Monitor (amctl)
set -e

AMCTL="amctl"
if ! command -v amctl &> /dev/null; then
    AMCTL="/usr/local/bin/amctl"
fi

MENU_CMD=""
if command -v rofi &> /dev/null; then
    MENU_CMD="rofi -dmenu -i -p 🖥️ Acer Monitor"
elif command -v dmenu &> /dev/null; then
    MENU_CMD="dmenu -i -p Acer Monitor:"
else
    echo "Error: rofi or dmenu is required to run this menu."
    exit 1
fi

CHOICE=$(cat <<EOF | eval "$MENU_CMD"
☀️ Brightness: 100%
☀️ Brightness: 80%
☀️ Brightness: 50%
🌙 Brightness: 20% (Night)
🔊 Volume: 100%
🔊 Volume: 50%
🔇 Toggle Mute
🎛️ Preset: Standard
🎛️ Preset: ECO Power Saver
🎛️ Preset: HDR Game Mode
🎛️ Preset: Action Gaming
🎛️ Preset: Racing
🎛️ Preset: Sports
🎛️ Preset: Reading / Text
🎛️ Preset: Movie / Cinema
🔌 Input: DisplayPort
🔌 Input: HDMI 1
🔌 Input: HDMI 2
🔌 Input: Auto-Switch
🎮 Black Boost: Level 5
🎮 Black Boost: Level 10
👁️ Blue Light: Level 3 (70%)
👁️ Blue Light: Off (0%)
⚡ OverDrive: Extreme
☀️ Apply Day Mode (90%)
🌙 Apply Night Mode (20% + Warm)
🔓 Emergency Unlock OSD
EOF
)

case "$CHOICE" in
    "☀️ Brightness: 100%") $AMCTL brightness 100 --osd ;;
    "☀️ Brightness: 80%") $AMCTL brightness 80 --osd ;;
    "☀️ Brightness: 50%") $AMCTL brightness 50 --osd ;;
    "🌙 Brightness: 20% (Night)") $AMCTL brightness 20 --osd ;;
    "🔊 Volume: 100%") $AMCTL volume 100 --osd ;;
    "🔊 Volume: 50%") $AMCTL volume 50 --osd ;;
    "🔇 Toggle Mute") $AMCTL mute toggle --osd ;;
    "🎛️ Preset: Standard") $AMCTL preset standard ;;
    "🎛️ Preset: ECO Power Saver") $AMCTL preset eco ;;
    "🎛️ Preset: HDR Game Mode") $AMCTL preset hdr ;;
    "🎛️ Preset: Action Gaming") $AMCTL preset action ;;
    "🎛️ Preset: Racing") $AMCTL preset racing ;;
    "🎛️ Preset: Sports") $AMCTL preset sports ;;
    "🎛️ Preset: Reading / Text") $AMCTL preset reading ;;
    "🎛️ Preset: Movie / Cinema") $AMCTL preset movie ;;
    "🔌 Input: DisplayPort") $AMCTL input dp ;;
    "🔌 Input: HDMI 1") $AMCTL input hdmi1 ;;
    "🔌 Input: HDMI 2") $AMCTL input hdmi2 ;;
    "🔌 Input: Auto-Switch") $AMCTL input auto ;;
    "🎮 Black Boost: Level 5") $AMCTL blackboost 5 ;;
    "🎮 Black Boost: Level 10") $AMCTL blackboost 10 ;;
    "👁️ Blue Light: Level 3 (70%)") $AMCTL bluelight 70 ;;
    "👁️ Blue Light: Off (0%)") $AMCTL bluelight 0 ;;
    "⚡ OverDrive: Extreme") $AMCTL od 2 ;;
    "☀️ Apply Day Mode (90%)") $AMCTL brightness 90 --osd ;;
    "🌙 Apply Night Mode (20% + Warm)") $AMCTL brightness 20 --osd && $AMCTL colortemp warm ;;
    "🔓 Emergency Unlock OSD") $AMCTL unlock ;;
esac
