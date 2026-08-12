# Acer Monitor Control CLI & Daemon Suite (`amctl`) 🖥️⚡

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-blue.svg)]()
[![Arch](https://img.shields.io/badge/Arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V%2064-purple.svg)]()

> A feature-rich, high-performance Rust CLI (`amctl` / `acer_monitor_cli`), IPC daemon, and system integration suite for controlling Acer (and generic VESA MCCS) monitors via DDC/CI on Linux (x86_64, ARM64, RISC-V 64) and Windows (x86_64, ARM64).

---

## 📋 Table of Contents

- [✨ Key Features](#-key-features)
- [📦 Installation](#-installation)
  - [Linux Quick Install](#linux-quick-install)
  - [Ubuntu / GNOME Top Bar Extension](#ubuntu--gnome-top-bar-extension)
  - [Windows Quick Install & Taskbar System Tray Widget](#windows-quick-install--taskbar-system-tray-widget)
  - [Building from Source & Architecture Targets](#building-from-source--architecture-targets)
- [🛠️ Usage & Command Reference](#️-usage--command-reference)
  - [Display & Audio Basics](#display--audio-basics)
  - [Hardware Presets & Acer Banked VCP Controls](#hardware-presets--acer-banked-vcp-controls)
  - [Smooth Fading & Transitions](#smooth-fading--transitions)
  - [Query, Information & Diagnostics](#query-information--diagnostics)
  - [Multi-Monitor Management](#multi-monitor-management)
  - [Automation & Background Daemons](#automation--background-daemons)
  - [Profile Management](#profile-management)
  - [Shell & Status Bar Integration](#shell--status-bar-integration)
- [⚡ Advanced: Raw Banked VCP Control](#-advanced-raw-banked-vcp-control)
- [📄 License](#-license)

---

## ✨ Key Features

* **🖥️ Full Display & Audio Control**: Adjust Brightness, Contrast, Volume, Mute, Power, and Input source (`DP`, `HDMI1`, `HDMI2`, `Auto`, `Next`).
* **🎮 Acer Hardware Banked VCPs**: Direct hardware access to Black Boost, OverDrive (`od`), AimPoint Crosshair (`aim`), Blue Light Filter (`bluelight`), Gamma, Color Temperature (`colortemp`), and Refresh Rate Counter (`refreshnum`).
* **🎛️ One-Touch Hardware Presets**: Apply native monitor OSD modes (`user`, `standard`, `eco`, `graphics`, `action`, `racing`, `sports`, `hdr`, `reading`, `movie`).
* **🌊 Smooth Parameter Fading**: Transition brightness, contrast, or volume smoothly over custom time durations (`fade`).
* **☀️ Solar Circadian Auto-Scheduler**: Automatically shifts display brightness and color temperature between day and night based on GPS coordinates.
* **🌙 Smart Inactivity Idle Dimmer**: Dims screen after inactivity, with **automatic inhibition during media playback** via MPRIS D-Bus checks.
* **⚡ Sub-Millisecond IPC Socket Server**: Background server daemon (`server`) listening on local IPC for instant hotkey handling (`send`).
* **🎮 Auto-Profile Switcher**: Automatically applies monitor profile JSONs when specified applications/games launch (`auto-profile`).
* **👀 Hotplug Monitor Watcher**: Monitors connected displays for live hotplug events (`watch-monitors`).
* **📊 Hardware EDID & VCP Inspector**: Decodes EDID (resolution, screen size, manufacture date, serial numbers) and probes active VCP feature codes with optional `--json` export.
* **💡 Real-Time Energy Calculator**: Calculates live wattage draw (~15.2W) and estimated annual electricity costs (~$6.68/year).
* **🎨 Diagnostic Test Patterns**: Renders full-screen RGB, alignment grids, and grayscale gradients for display testing.
* **⚙️ Multi-Monitor Support**: Target specific displays by index (`0`), model substring (`VG271U`), or all connected displays (`all`). Includes `sync` and brightness `balance`.
* **🧩 Desktop Integrations**:
  - **Linux GNOME Shell Extension**: Top bar Quick Settings widget (`acer-monitor@sidhant`) with sliders & submenus.
  - **Windows Taskbar System Tray Widget**: Native Windows Notification Area app (`acer-tray.ps1`).
  - **Shell & Status Bar**: Systemd User Service, Desktop Launcher, Waybar JSON config, and shell completion scripts (`bash`, `zsh`, `fish`).
* **🌍 Multi-Architecture Support**: Native cross-compilation binaries for **x86_64**, **ARM64 (aarch64)**, and **RISC-V 64 (riscv64gc)**.


---

## 📦 Installation

### Linux Quick Install

Make sure your user is in the `i2c` group and `i2c-dev` kernel module is enabled (`sudo modprobe i2c-dev`).

Run the automated installation script:
```bash
chmod +x install.sh
./install.sh
```

The script builds the release binary and installs `acer_monitor_cli` along with convenience symlinks `amctl` and `amc` into `/usr/local/bin/`. It also registers the desktop entry and systemd user service.

To enable the systemd background service:
```bash
systemctl --user enable --now acer-monitor
```

### Ubuntu / GNOME Top Bar Extension

To add an interactive Acer Monitor Control icon to your GNOME Shell top bar:
```bash
chmod +x install-gnome-extension.sh
./install-gnome-extension.sh
```

### Windows Quick Install & Taskbar System Tray Widget

Run PowerShell as User or Administrator, or launch `install.bat`:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
.\install.ps1
```

This performs the complete Windows setup:
1. Installs `acer_monitor_cli.exe` into `%LocalAppData%\Programs\acer_monitor_cli` and adds it to your user `PATH`.
2. Installs and auto-starts the **Windows System Tray Widget** (`acer-tray.ps1`) in the Windows Taskbar Notification Area (next to the clock).
3. Registers a Start Menu shortcut and Task Scheduler logon trigger for the Smart Idle Dimmer.

To launch or restart the Windows System Tray Widget manually at any time:
```powershell
powershell -WindowStyle Hidden -ExecutionPolicy Bypass -File "$env:LocalAppData\Programs\acer_monitor_cli\acer-tray.ps1"
```


### ⚙️ Building from Source

#### 1. Prerequisites & Toolchain Setup

First, install the Rust compiler and toolchain if you haven't already:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Linux Prerequisites**:
Ensure `build-essential`, `pkg-config`, and the `i2c-dev` kernel module are installed:
```bash
sudo apt update && sudo apt install -y build-essential pkg-config
sudo modprobe i2c-dev
```

#### 2. Native Compilation

Clone the repository and build the optimized release binary:
```bash
git clone https://github.com/sidhantjohnaind/acer_monitor_cli.git
cd acer_monitor_cli
cargo build --release
```

- **Linux Output**: `target/release/acer_monitor_cli`
- **Windows Output**: `target/release/acer_monitor_cli.exe`

#### 3. Cross-Compiling for Other Architectures

You can cross-compile `acer_monitor_cli` for Linux ARM64 (Raspberry Pi), Linux RISC-V 64, and Windows on ARM:

```bash
# 🐧 Linux ARM64 / Raspberry Pi (aarch64)
sudo apt install -y gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
cargo build --release --target aarch64-unknown-linux-gnu
# Binary: target/aarch64-unknown-linux-gnu/release/acer_monitor_cli

# 🧪 Linux RISC-V 64 (riscv64gc)
sudo apt install -y gcc-riscv64-linux-gnu
rustup target add riscv64gc-unknown-linux-gnu
CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=riscv64-linux-gnu-gcc \
cargo build --release --target riscv64gc-unknown-linux-gnu
# Binary: target/riscv64gc-unknown-linux-gnu/release/acer_monitor_cli

# 🪟 Windows x86_64 (from Linux)
sudo apt install -y mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# Binary: target/x86_64-pc-windows-gnu/release/acer_monitor_cli.exe

# 💻 Windows on ARM / Snapdragon X Elite (ARM64)
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc
# Binary: target/aarch64-pc-windows-msvc/release/acer_monitor_cli.exe
```

---

## 🧩 Desktop Widgets & Integrations

`amctl` includes native widgets and shortcuts tailored for every desktop environment:

1. **Linux GNOME Shell Quick Settings Extension** (`gnome-extension/`):
   - Adds sliders and popup submenus for Brightness, Contrast, Volume, Presets, Inputs, Black Boost, and Solar schedule to the top bar.
   - Install with `./install-gnome-extension.sh`.

2. **Windows Taskbar System Tray Widget** (`acer-tray.ps1`):
   - Adds a native notification area system tray icon next to the Windows clock with right-click popups.
   - Installed automatically via `.\install.ps1`.

3. **Rofi / Dmenu Quick Launcher Widget** (`rofi-acer.sh`):
   - Fast keyboard-driven menu for i3, Sway, Hyprland, XFCE, and Openbox window managers.
   - Launch anytime with `./rofi-acer.sh`.

4. **Waybar Status Bar Widget** (`waybar/`):
   - Custom widget configuration (`config.jsonc`) and glassmorphic CSS (`style.css`) for Hyprland/Sway bars.



---

## 🛠️ Usage & Command Reference

You can use `amctl`, `amc`, or `acer_monitor_cli` interchangeably.

### Display & Audio Basics

```bash
amctl brightness 80            # Set brightness to 80%
amctl brightness +10 --osd     # Increase brightness by 10% with visual OSD banner
amctl contrast 75              # Set contrast to 75%
amctl volume 50                # Set volume to 50%
amctl volume -5 --osd          # Decrease volume by 5% with OSD banner
amctl mute toggle --osd        # Toggle mute/unmute with OSD banner
amctl input next               # Cycle to the next input channel
amctl input dp                 # Switch directly to DisplayPort (dp, hdmi1, hdmi2, auto)
amctl power off                # Turn off monitor display power (on | off)
```

### Hardware Presets & Acer Banked VCP Controls

```bash
amctl preset hdr               # Apply hardware HDR mode (user, standard, eco, graphics, action, racing, sports, hdr, reading, movie)
amctl blackboost 5             # Adjust Black Boost level (0-10)
amctl bluelight 70             # Set Blue Light filter (0, 50, 60, 70, 80)
amctl colortemp warm           # Set color temperature (warm, normal, cool, bluelight, user)
amctl gamma 22                 # Set gamma level (18, 20, 22, 24, 26)
amctl od 2                     # Set OverDrive level (0=Off, 1=Normal, 2=Extreme)
amctl aim 1                    # Set AimPoint crosshair overlay type (0=Off, 1-3=Crosshair style)
amctl refreshnum on            # Enable hardware refresh rate counter OSD (on | off)
amctl indicator off            # Turn off front power LED indicator (on | off)
amctl keylock on               # Lock front panel OSD buttons (on | off)
amctl unlock                   # Emergency unlock OSD buttons and power button
amctl reset                    # Factory reset monitor settings
```

### Smooth Fading & Transitions

Fades brightness, contrast, or volume smoothly from a starting value to an ending value over a specified duration in milliseconds (default: 1000ms):

```bash
amctl fade brightness 10 90 2000    # Fade brightness from 10% to 90% over 2 seconds
amctl fade volume 80 10 1500        # Fade volume from 80% down to 10% over 1.5 seconds
```

### Query, Information & Diagnostics

```bash
amctl list                     # List connected DDC/CI monitors
amctl list --json              # Output monitor list in JSON format
amctl info                     # Print detailed capabilities and current VCP values
amctl info --json              # Output detailed monitor info as JSON
amctl caps                     # Display raw MCCS capability string report
amctl scan                     # Probe common VCP feature codes
amctl scan --json              # Output probe scan results as JSON
amctl edid                     # Display hardware EDID panel readout
amctl diag                     # Generate a full system diagnostic report
amctl energy                   # Estimate current wattage draw and annual energy cost
amctl test-pattern gradient    # Render diagnostic test pattern (red, green, blue, white, black, grid, gradient)
amctl get 0x10                 # Read raw VCP code (e.g. 0x10 = Brightness)
amctl get-bluelight            # Get current Blue Light level
amctl get-gamma                # Get current Gamma level
amctl get-colortemp            # Get current Color Temp
amctl get-od                   # Get current OverDrive level
amctl get-blackboost           # Get current Black Boost level
```

### Multi-Monitor Management

Pass a monitor specifier at the end of any command to target specific displays:
- **By Index**: `0`, `1`, etc.
- **By Model Substring**: `VG271U`, `Nitro`, etc.
- **All Monitors**: `all`

```bash
amctl brightness 100 0         # Set brightness of monitor 0 to 100%
amctl preset eco VG271U        # Apply ECO preset to monitor matching 'VG271U'
amctl brightness 50 all        # Set brightness to 50% across ALL monitors
amctl sync                     # Sync secondary monitors' brightness & contrast to master monitor
amctl balance --offset -15     # Balance secondary monitors with relative offset to master display
```

### Automation & Background Daemons

```bash
# Solar circadian schedule based on latitude/longitude coordinates
amctl solar-schedule --lat 28.61 --lon 77.20 --day-b 90 --night-b 15 --night-ct warm

# Smart Idle Dimmer (dims display after 5 mins idle, pauses dimming when watching video via MPRIS)
amctl idle-dimmer --idle-secs 300 --dim-to 10

# Auto-Profile Switcher (applies profile JSON when process is active)
amctl auto-profile --rule "hl2.exe:gaming.json"

# Hotplug Monitor Watcher (detects display connection changes)
amctl watch-monitors

# IPC Server Daemon (runs IPC socket server for sub-millisecond hotkey commands)
amctl server
amctl send brightness +10      # Send command to active server daemon
```

### Profile Management

Save monitor settings to JSON configuration files and restore them at any time:

```bash
amctl profile save workspace.json    # Save current monitor settings to JSON
amctl profile load workspace.json    # Restore settings from JSON file
```

### Shell & Status Bar Integration

```bash
# Output Waybar configuration block
amctl waybar-config

# Generate shell completion script
amctl completions bash > /etc/bash_completion.d/amctl
amctl completions zsh > ~/.zsh/completion/_amctl
amctl completions fish > ~/.config/fish/completions/amctl.fish
```

---

## ⚡ Advanced: Raw Banked VCP Control

Acer hardware utilizes custom banked register mapping over VCP codes `0xE0`, `0xE7`, and `0xE9`. Advanced users can directly read or write banked registers:

```bash
# Write selector register and set bank value
amctl rawbank e0 0x04 2        # Write OverDrive selector (0x04) = 2 (Extreme)

# Read banked register selector value
amctl getbank e7 0x00          # Read Blue Light selector (0x00)
```

---

## 📄 License

Licensed under the [MIT License](LICENSE).

