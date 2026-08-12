# Acer Monitor Control Windows Taskbar System Tray Application
# Runs natively in the Windows System Tray (Taskbar Notification Area next to clock)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# Determine binary location
$ExePath = "acer_monitor_cli.exe"
$LocalAppDataExe = Join-Path $env:LocalAppData "Programs\acer_monitor_cli\acer_monitor_cli.exe"

if (Test-Path $LocalAppDataExe) {
    $ExePath = $LocalAppDataExe
}

function Exec-Cli([string]$argsStr) {
    Start-Process -FilePath $ExePath -ArgumentList $argsStr -WindowStyle Hidden -ErrorAction SilentlyContinue
}

# Create Tray Notify Icon
$notifyIcon = New-Object System.Windows.Forms.NotifyIcon
$notifyIcon.Icon = [System.Drawing.SystemIcons]::Application
$notifyIcon.Text = "Acer Monitor Control (amctl)"
$notifyIcon.Visible = $true

# Build Context Menu
$contextMenu = New-Object System.Windows.Forms.ContextMenuStrip

# Header Title
$headerItem = New-Object System.Windows.Forms.ToolStripMenuItem("🖥️ Acer Monitor Control")
$headerItem.Enabled = $false
$contextMenu.Items.Add($headerItem) | Out-Null
$contextMenu.Items.Add("-") | Out-Null

# ☀️ Brightness Submenu
$brightnessMenu = New-Object System.Windows.Forms.ToolStripMenuItem("☀️ Brightness")
foreach ($b in @(100, 90, 80, 70, 60, 50, 40, 30, 20, 10)) {
    $val = $b
    $item = New-Object System.Windows.Forms.ToolStripMenuItem("$val%")
    $item.add_Click({ Exec-Cli "brightness $val --osd" })
    $brightnessMenu.DropDownItems.Add($item) | Out-Null
}
$contextMenu.Items.Add($brightnessMenu) | Out-Null

# 🔊 Volume Submenu
$volumeMenu = New-Object System.Windows.Forms.ToolStripMenuItem("🔊 Volume")
foreach ($v in @(100, 80, 60, 40, 20, 0)) {
    $val = $v
    $item = New-Object System.Windows.Forms.ToolStripMenuItem("$val%")
    $item.add_Click({ Exec-Cli "volume $val --osd" })
    $volumeMenu.DropDownItems.Add($item) | Out-Null
}
$muteItem = New-Object System.Windows.Forms.ToolStripMenuItem("🔇 Toggle Mute")
$muteItem.add_Click({ Exec-Cli "mute toggle --osd" })
$volumeMenu.DropDownItems.Add("-") | Out-Null
$volumeMenu.DropDownItems.Add($muteItem) | Out-Null
$contextMenu.Items.Add($volumeMenu) | Out-Null

# 🎛️ Display Presets Submenu
$presetsMenu = New-Object System.Windows.Forms.ToolStripMenuItem("🎛️ Display Presets")
$presetList = @(
    @{ Name = "Standard Mode"; Cmd = "preset standard" },
    @{ Name = "ECO Power Saver"; Cmd = "preset eco" },
    @{ Name = "HDR Game Mode"; Cmd = "preset hdr" },
    @{ Name = "Action Gaming"; Cmd = "preset action" },
    @{ Name = "Racing Mode"; Cmd = "preset racing" },
    @{ Name = "Sports Mode"; Cmd = "preset sports" },
    @{ Name = "Graphics Mode"; Cmd = "preset graphics" },
    @{ Name = "Reading / Text"; Cmd = "preset reading" },
    @{ Name = "Movie / Cinema"; Cmd = "preset movie" },
    @{ Name = "User Mode"; Cmd = "preset user" }
)
foreach ($p in $presetList) {
    $cmd = $p.Cmd
    $item = New-Object System.Windows.Forms.ToolStripMenuItem($p.Name)
    $item.add_Click({ Exec-Cli $cmd })
    $presetsMenu.DropDownItems.Add($item) | Out-Null
}
$contextMenu.Items.Add($presetsMenu) | Out-Null

# 🔌 Input Source Submenu
$inputMenu = New-Object System.Windows.Forms.ToolStripMenuItem("🔌 Input Source")
$inputList = @(
    @{ Name = "DisplayPort"; Cmd = "input dp" },
    @{ Name = "HDMI 1"; Cmd = "input hdmi1" },
    @{ Name = "HDMI 2"; Cmd = "input hdmi2" },
    @{ Name = "Auto Switch"; Cmd = "input auto" }
)
foreach ($inp in $inputList) {
    $cmd = $inp.Cmd
    $item = New-Object System.Windows.Forms.ToolStripMenuItem($inp.Name)
    $item.add_Click({ Exec-Cli $cmd })
    $inputMenu.DropDownItems.Add($item) | Out-Null
}
$contextMenu.Items.Add($inputMenu) | Out-Null

# 🎮 Gaming & Enhancements Submenu
$gamingMenu = New-Object System.Windows.Forms.ToolStripMenuItem("🎮 Gaming & Vision")

# Black Boost
$bbTitle = New-Object System.Windows.Forms.ToolStripMenuItem("Black Boost Level")
$bbTitle.Enabled = $false
$gamingMenu.DropDownItems.Add($bbTitle) | Out-Null
foreach ($bb in @(0, 2, 5, 8, 10)) {
    $lvl = $bb
    $item = New-Object System.Windows.Forms.ToolStripMenuItem("Boost $lvl")
    $item.add_Click({ Exec-Cli "blackboost $lvl" })
    $gamingMenu.DropDownItems.Add($item) | Out-Null
}

# Blue Light
$blTitle = New-Object System.Windows.Forms.ToolStripMenuItem("Blue Light Filter")
$blTitle.Enabled = $false
$gamingMenu.DropDownItems.Add("-") | Out-Null
$gamingMenu.DropDownItems.Add($blTitle) | Out-Null
$blList = @(
    @{ Name = "Off (0%)"; Cmd = "bluelight 0" },
    @{ Name = "Level 1 (50%)"; Cmd = "bluelight 50" },
    @{ Name = "Level 2 (60%)"; Cmd = "bluelight 60" },
    @{ Name = "Level 3 (70%)"; Cmd = "bluelight 70" },
    @{ Name = "Level 4 (80%)"; Cmd = "bluelight 80" }
)
foreach ($bl in $blList) {
    $cmd = $bl.Cmd
    $item = New-Object System.Windows.Forms.ToolStripMenuItem($bl.Name)
    $item.add_Click({ Exec-Cli $cmd })
    $gamingMenu.DropDownItems.Add($item) | Out-Null
}

# OverDrive
$odTitle = New-Object System.Windows.Forms.ToolStripMenuItem("Hardware OverDrive")
$odTitle.Enabled = $false
$gamingMenu.DropDownItems.Add("-") | Out-Null
$gamingMenu.DropDownItems.Add($odTitle) | Out-Null
$odNormal = New-Object System.Windows.Forms.ToolStripMenuItem("OverDrive: Normal (1)")
$odNormal.add_Click({ Exec-Cli "od 1" })
$gamingMenu.DropDownItems.Add($odNormal) | Out-Null

$odExtreme = New-Object System.Windows.Forms.ToolStripMenuItem("OverDrive: Extreme (2)")
$odExtreme.add_Click({ Exec-Cli "od 2" })
$gamingMenu.DropDownItems.Add($odExtreme) | Out-Null

# AimPoint
$aimItem = New-Object System.Windows.Forms.ToolStripMenuItem("AimPoint Crosshair Overlay")
$aimItem.add_Click({ Exec-Cli "aim 1" })
$gamingMenu.DropDownItems.Add($aimItem) | Out-Null

$contextMenu.Items.Add($gamingMenu) | Out-Null

# ☀️ Day / Night Quick Actions
$dayItem = New-Object System.Windows.Forms.ToolStripMenuItem("☀️ Apply Day Mode (Brightness 90%)")
$dayItem.add_Click({ Exec-Cli "brightness 90 --osd" })
$contextMenu.Items.Add($dayItem) | Out-Null

$nightItem = New-Object System.Windows.Forms.ToolStripMenuItem("🌙 Apply Night Mode (20% + Warm)")
$nightItem.add_Click({
    Exec-Cli "brightness 20 --osd"
    Exec-Cli "colortemp warm"
})
$contextMenu.Items.Add($nightItem) | Out-Null

# 🔓 Unlock OSD Keys
$unlockItem = New-Object System.Windows.Forms.ToolStripMenuItem("🔓 Emergency Unlock OSD Keys")
$unlockItem.add_Click({ Exec-Cli "unlock" })
$contextMenu.Items.Add($unlockItem) | Out-Null

# Exit
$contextMenu.Items.Add("-") | Out-Null
$exitItem = New-Object System.Windows.Forms.ToolStripMenuItem("Exit Tray App")
$exitItem.add_Click({
    $notifyIcon.Visible = $false
    $notifyIcon.Dispose()
    [System.Windows.Forms.Application]::Exit()
})
$contextMenu.Items.Add($exitItem) | Out-Null

$notifyIcon.ContextMenuStrip = $contextMenu

# Keep PowerShell running application loop
[System.Windows.Forms.Application]::Run()
