# Windows 1-Click Installation Script for Acer Display Center & CLI (amctl)
$ErrorActionPreference = "Stop"
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "   🚀 Installing Acer Display Center & Monitor CLI Suite (amctl)" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

$InstallDir = Join-Path $env:LocalAppData "Programs\acer_monitor_cli"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExeTarget = Join-Path $InstallDir "amctl.exe"
$LegacyExeTarget = Join-Path $InstallDir "acer_monitor_cli.exe"
$GuiExeTarget = Join-Path $InstallDir "acer_display_center.exe"
$IcoTarget = Join-Path $InstallDir "app.ico"

# Stop any running instances before updating
Stop-Process -Name amctl,acer_monitor_cli,acer_display_center -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

# Find local build or download from GitHub release
$ScriptDir = $null
if ($MyInvocation.MyCommand -and $MyInvocation.MyCommand.Path) {
    $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if (-not $ScriptDir -and $PSScriptRoot) {
    $ScriptDir = $PSScriptRoot
}
if (-not $ScriptDir) {
    $ScriptDir = (Get-Location).Path
}

$LocalCandidates = @(
    (Join-Path $ScriptDir "target\release\acer_monitor_cli.exe"),
    (Join-Path $ScriptDir "target\release\amctl.exe"),
    "target\release\acer_monitor_cli.exe",
    "target\release\amctl.exe"
)
if ($env:CARGO_TARGET_DIR) {
    $LocalCandidates += (Join-Path $env:CARGO_TARGET_DIR "release\acer_monitor_cli.exe")
    $LocalCandidates += (Join-Path $env:CARGO_TARGET_DIR "release\amctl.exe")
}

$ExistingCandidates = @($LocalCandidates | Where-Object { Test-Path $_ } | Get-Item | Sort-Object LastWriteTime -Descending)

$Installed = $false
if ($ExistingCandidates.Count -gt 0) {
    $cand = $ExistingCandidates[0].FullName
    $candDir = Split-Path -Parent $cand
    $guiCand = Join-Path $candDir "acer_display_center.exe"

    Copy-Item -Path $cand -Destination $ExeTarget -Force
    Copy-Item -Path $cand -Destination $LegacyExeTarget -Force
    if (Test-Path $guiCand) {
        Copy-Item -Path $guiCand -Destination $GuiExeTarget -Force
    } else {
        Copy-Item -Path $cand -Destination $GuiExeTarget -Force
    }
    $Installed = $true
    Write-Host "[+] Installed newest local release binary from $cand." -ForegroundColor Green
}

if (-not $Installed) {
    Write-Host "[*] Downloading latest standalone release binaries from GitHub..." -ForegroundColor Yellow
    $BaseUrl = "https://github.com/sidhantjohnaind/acer-display-center/releases/latest/download"
    $RawUrl = "https://raw.githubusercontent.com/sidhantjohnaind/acer-display-center/main"
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri "$BaseUrl/amctl-x86_64-pc-windows-msvc.exe" -OutFile $ExeTarget -UseBasicParsing
        Copy-Item -Path $ExeTarget -Destination $LegacyExeTarget -Force
        
        try {
            Invoke-WebRequest -Uri "$BaseUrl/acer_display_center-x86_64-pc-windows-msvc.exe" -OutFile $GuiExeTarget -UseBasicParsing
        } catch {
            Copy-Item -Path $ExeTarget -Destination $GuiExeTarget -Force
        }

        Write-Host "[+] Downloaded binaries successfully." -ForegroundColor Green
    } catch {
        Write-Host "[!] Could not download pre-built binary. Building locally with Cargo..." -ForegroundColor Yellow
        cargo build --release
        Copy-Item -Path "target\release\acer_monitor_cli.exe" -Destination $ExeTarget -Force
        Copy-Item -Path "target\release\acer_monitor_cli.exe" -Destination $LegacyExeTarget -Force
        if (Test-Path "target\release\acer_display_center.exe") {
            Copy-Item -Path "target\release\acer_display_center.exe" -Destination $GuiExeTarget -Force
        } else {
            Copy-Item -Path $ExeTarget -Destination $GuiExeTarget -Force
        }
    }
}

# Install App Icon
if (Test-Path "app.ico") {
    Copy-Item -Path "app.ico" -Destination $IcoTarget -Force
} else {
    try {
        $RawUrl = "https://raw.githubusercontent.com/sidhantjohnaind/acer-display-center/main"
        Invoke-WebRequest -Uri "$RawUrl/app.ico" -OutFile $IcoTarget -UseBasicParsing
    } catch {}
}

# Add to User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*acer_monitor_cli*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "[+] Added $InstallDir to User PATH." -ForegroundColor Green
}

# Create Start Menu & Desktop Shortcuts with Shell.Application COM
$WshShell = New-Object -ComObject WScript.Shell

$StartMenuDir = Join-Path $env:AppData "Microsoft\Windows\Start Menu\Programs"
$DesktopDir = [Environment]::GetFolderPath("Desktop")

# 1. Main GUI Shortcut (Runs Quick Settings GUI silently with NO console window)
$GuiShortcut = $WshShell.CreateShortcut((Join-Path $StartMenuDir "Acer Display Center.lnk"))
$GuiShortcut.TargetPath = $GuiExeTarget
$GuiShortcut.Arguments = ""
$GuiShortcut.WorkingDirectory = $InstallDir
$GuiShortcut.Description = "Acer Display Center - Monitor Quick Settings & System Tray"
if (Test-Path $IcoTarget) { $GuiShortcut.IconLocation = "$IcoTarget,0" }
$GuiShortcut.Save()

$GuiDesktopShortcut = $WshShell.CreateShortcut((Join-Path $DesktopDir "Acer Display Center.lnk"))
$GuiDesktopShortcut.TargetPath = $GuiExeTarget
$GuiDesktopShortcut.Arguments = ""
$GuiDesktopShortcut.WorkingDirectory = $InstallDir
$GuiDesktopShortcut.Description = "Acer Display Center - Monitor Quick Settings & System Tray"
if (Test-Path $IcoTarget) { $GuiDesktopShortcut.IconLocation = "$IcoTarget,0" }
$GuiDesktopShortcut.Save()
Write-Host "[+] Created 100% Silent GUI Shortcut: Acer Display Center (Start Menu & Desktop)" -ForegroundColor Green

# 2. Tray Daemon Shortcut (Background System Tray Daemon silently with NO console window)
$TrayShortcut = $WshShell.CreateShortcut((Join-Path $StartMenuDir "Acer Display Center (Tray Only).lnk"))
$TrayShortcut.TargetPath = $GuiExeTarget
$TrayShortcut.Arguments = "tray"
$TrayShortcut.WorkingDirectory = $InstallDir
$TrayShortcut.Description = "Acer Display Center - System Tray Daemon"
if (Test-Path $IcoTarget) { $TrayShortcut.IconLocation = "$IcoTarget,0" }
$TrayShortcut.Save()

$TrayDesktopShortcut = $WshShell.CreateShortcut((Join-Path $DesktopDir "Acer Display Center (Tray Only).lnk"))
$TrayDesktopShortcut.TargetPath = $GuiExeTarget
$TrayDesktopShortcut.Arguments = "tray"
$TrayDesktopShortcut.WorkingDirectory = $InstallDir
$TrayDesktopShortcut.Description = "Acer Display Center - System Tray Daemon"
if (Test-Path $IcoTarget) { $TrayDesktopShortcut.IconLocation = "$IcoTarget,0" }
$TrayDesktopShortcut.Save()
Write-Host "[+] Created 100% Silent Tray Shortcut: Acer Display Center (Tray Only) (Start Menu & Desktop)" -ForegroundColor Green

# Clean up any legacy Startup folder shortcuts to ensure ONLY the tray runs on boot
$StartupDir = Join-Path $env:AppData "Microsoft\Windows\Start Menu\Programs\Startup"
Remove-Item -Path (Join-Path $StartupDir "Acer*.lnk"), (Join-Path $StartupDir "Acer*.bat") -Force -ErrorAction SilentlyContinue

# Configure Windows Run Registry Key: ONLY the background tray runs on boot / startup (Zero console)
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "AcerDisplayCenter" -Value "`"$GuiExeTarget`" tray" -Force
Write-Host "[+] Configured ONLY the System Tray daemon to run on boot / startup (HKCU Run)." -ForegroundColor Green

# Launch Tray Daemon Now (Detached background process)
Start-Process -FilePath $GuiExeTarget -ArgumentList "tray" -WindowStyle Hidden
Write-Host "[+] Started Acer Display Center System Tray Daemon!" -ForegroundColor Green

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "   ✅ Installation Successful!                                  " -ForegroundColor Green
Write-Host "   • Press [Ctrl + Alt + M] to open Acer Display Center Flyout   " -ForegroundColor White
Write-Host "   • Right-click the system tray icon for quick monitor controls " -ForegroundColor White
Write-Host "   • Run 'amctl --help' in terminal for full CLI commands        " -ForegroundColor White
Write-Host "================================================================" -ForegroundColor Cyan

Write-Host "Both 'acer_monitor_cli' and 'amctl' commands are now installed and ready in your PATH."
Write-Host "The Native Rust System Tray widget is configured to start automatically on Windows logon."
Write-Host "Launch it immediately with:"
Write-Host "  amctl tray"

