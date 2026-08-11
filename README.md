# Acer Monitor Control CLI & Daemon Suite (`amctl`) 🖥️⚡

> A powerful, hyper-fast Rust CLI (`amctl`) and background daemon suite for controlling Acer (and generic VESA MCCS) monitors via DDC/CI on Linux and Windows.

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
* **🎛️ One-Touch Hardware Presets**: Switch native monitor OSD modes (`user`, `standard`, `eco`, `graphics`, `hdr`, `racing`, `sports`).
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
sudo cp target/release/acer_monitor_cli /usr/local/bin/acer_monitor_cli
sudo ln -sf /usr/local/bin/acer_monitor_cli /usr/local/bin/amctl
amctl install-desktop
amctl install-service
```

### Windows Quick Install
Run PowerShell as Administrator or User:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
.\install.ps1
```
This installs `amctl.exe` to `%AppData%\Local\Programs\acer_monitor_cli`, adds it to your `%PATH%`, and creates a Start Menu shortcut.

---

## 🛠️ Usage & Commands

### Basic Controls
```bash
amctl brightness 80            # Set brightness to 80%
amctl brightness +10 --osd     # Relative +10% with visual OSD banner
amctl volume -5 --osd          # Decrease volume with OSD banner
amctl mute toggle              # Toggle mute/unmute
amctl input next               # Cycle to next input channel
amctl input dp                 # Switch directly to DisplayPort
```

### Hardware Presets & Color Controls
```bash
amctl preset hdr               # Apply hardware HDR gaming mode
amctl preset eco               # Apply hardware ECO power saving mode
amctl blackboost 5             # Set Black Boost level (0-10)
amctl bluelight 3              # Set 70% Blue Light Filter
amctl colortemp warm           # Set warm color temperature
amctl od 2                     # Set Extreme OverDrive
```

### Hardware Info & Energy Calculations
```bash
amctl edid                     # Display hardware panel EDID readout
amctl energy                   # Show live wattage draw and yearly cost
amctl diag                     # Generate full system diagnostic report
amctl test-pattern gradient    # Render grayscale step gradient test
```

### Automation Daemons
```bash
# Solar circadian schedule
amctl solar-schedule --lat 28.61 --lon 77.20 --day-b 90 --night-b 15 --night-ct warm

# Smart Idle Dimmer (5 mins idle, dim to 10%, video playback inhibits dimming)
amctl idle-dimmer --idle-secs 300 --dim-to 10

# IPC Server Daemon (instant sub-millisecond hotkey IPC)
amctl server
amctl send brightness +10
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
