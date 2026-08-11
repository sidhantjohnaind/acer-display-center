# Acer Monitor CLI & Service Suite 🖥️⚡

> A powerful, hyper-fast Rust CLI and background daemon suite for controlling Acer (and generic VESA MCCS) monitors via DDC/CI on Linux and Windows.

---

## 🚀 Key Features

* **🖥️ Complete Display Controls**: Adjust Brightness, Contrast, Volume, Mute, Power, and Input sources (`DisplayPort`, `HDMI1`, `HDMI2`, `Auto`, `next`).
* **🎮 Acer Hardware Banked VCPs**: Control Black Boost, OverDrive (`od`), AimPoint Crosshairs (`aim`), Blue Light Filter (`bluelight`), Gamma, Color Temperature (`colortemp`), and Refresh Rate Counter (`refreshnum`).
* **☀️ Solar Circadian Auto-Scheduler**: Automatically shifts brightness and color temperature at sunset/sunrise based on GPS coordinates.
* **🌙 Smart Inactivity Idle Dimmer**: Dims screen after user inactivity, but **automatically pauses dimming when watching YouTube or movies** (via MPRIS D-Bus checks).
* **⚡ Sub-Millisecond IPC Socket Daemon**: Runs background daemon (`server`) listening on `/tmp/acer_monitor.sock` for instant hotkey execution.
* **📊 Hardware EDID Inspector**: Decodes 10-bit color, 1440p native timings, 27.2" screen size, manufacture year, and serial numbers.
* **💡 Real-Time Energy Calculator**: Calculates live wattage draw ($15.2\text{W}$) and estimated yearly electricity cost ($\$6.68/\text{year}$).
* **🎨 Diagnostic Test Patterns**: Full-screen RGB, alignment grid, and 0-100% grayscale gradient patterns.
* **🎛️ One-Touch Hardware Presets**: Switch native monitor OSD modes (`standard`, `eco`, `graphics`, `hdr`, `racing`, `sports`).
* **⚙️ Multi-Monitor Support**: Bulk target `all`, model name substring matching (`VG271U`), sync (`sync`), and relative brightness balance (`balance`).

---

## 📦 Installation

### Linux Quick Install
Run the automated installation script:
```bash
chmod +x install.sh
./install.sh
```
Or install manually:
```bash
cargo build --release
sudo cp target/release/acer_monitor_cli /usr/local/bin/
acer_monitor_cli install-desktop
acer_monitor_cli install-service
```

### Windows Quick Install
Run PowerShell as Administrator or User:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
.\install.ps1
```
This installs `acer_monitor_cli.exe` to `%AppData%\Local\Programs\acer_monitor_cli`, adds it to your `%PATH%`, and creates a Start Menu shortcut.

---

## 🛠️ Usage & Commands

### Basic Controls
```bash
acer_monitor_cli brightness 80            # Set brightness to 80%
acer_monitor_cli brightness +10 --osd     # Relative +10% with visual OSD banner
acer_monitor_cli volume -5 --osd          # Decrease volume with OSD banner
acer_monitor_cli mute toggle              # Toggle mute/unmute
acer_monitor_cli input next               # Cycle to next input channel
acer_monitor_cli input dp                 # Switch directly to DisplayPort
```

### Hardware Presets & Color Controls
```bash
acer_monitor_cli preset hdr               # Apply hardware HDR gaming mode
acer_monitor_cli preset eco               # Apply hardware ECO power saving mode
acer_monitor_cli blackboost 5             # Set Black Boost level (0-10)
acer_monitor_cli bluelight 3              # Set 70% Blue Light Filter
acer_monitor_cli colortemp warm           # Set warm color temperature
acer_monitor_cli od 2                     # Set Extreme OverDrive
```

### Hardware Info & Energy Calculations
```bash
acer_monitor_cli edid                     # Display hardware panel EDID readout
acer_monitor_cli energy                   # Show live wattage draw and yearly cost
acer_monitor_cli diag                     # Generate full system diagnostic report
acer_monitor_cli test-pattern gradient    # Render grayscale step gradient test
```

### Automation Daemons
```bash
# Solar circadian schedule
acer_monitor_cli solar-schedule --lat 28.61 --lon 77.20 --day-b 90 --night-b 15 --night-ct warm

# Smart Idle Dimmer (5 mins idle, dim to 10%, video playback inhibits dimming)
acer_monitor_cli idle-dimmer --idle-secs 300 --dim-to 10

# IPC Server Daemon (instant sub-millisecond hotkey IPC)
acer_monitor_cli server
acer_monitor_cli send brightness +10
```

---

## ⚙️ Building from Source

```bash
# Build Linux release binary
cargo build --release

# Cross-compile Windows binary on Linux
cargo build --release --target x86_64-pc-windows-gnu
```

---

## 📄 License
Licensed under MIT.
