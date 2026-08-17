# Windows Installation Script for Acer Monitor CLI & Daemon
Write-Host "=== Acer Monitor CLI & Daemon Suite Installer (Windows) ===" -ForegroundColor Cyan

# Locate executable
$ExeSource = ""
$Candidates = @(
    "D:\temp\rust\target\release\acer_monitor_cli.exe",
    "target\release\acer_monitor_cli.exe",
    "target\x86_64-pc-windows-msvc\release\acer_monitor_cli.exe",
    "target\x86_64-pc-windows-gnu\release\acer_monitor_cli.exe"
)

foreach ($c in $Candidates) {
    if (Test-Path $c) {
        $ExeSource = $c
        break
    }
}

if (-not $ExeSource) {
    Write-Host "Building release binary for Windows..." -ForegroundColor Yellow
    cargo build --release
    foreach ($c in $Candidates) {
        if (Test-Path $c) {
            $ExeSource = $c
            break
        }
    }
}

# Create installation folder in LocalAppData
$InstallDir = Join-Path $env:LocalAppData "Programs\acer_monitor_cli"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExeTarget = Join-Path $InstallDir "acer_monitor_cli.exe"
Copy-Item -Path $ExeSource -Destination $ExeTarget -Force
Write-Host "[+] Installed binary to: $ExeTarget" -ForegroundColor Green

# Create amctl alias binary
$AmctlTarget = Join-Path $InstallDir "amctl.exe"
Copy-Item -Path $ExeSource -Destination $AmctlTarget -Force
Write-Host "[+] Created 'amctl' command shortcut at: $AmctlTarget" -ForegroundColor Green

# Copy Taskbar System Tray & Windows 11 Quick Settings Flyout
if (Test-Path "flyout.ps1") {
    $FlyoutTarget = Join-Path $InstallDir "flyout.ps1"
    Copy-Item -Path "flyout.ps1" -Destination $FlyoutTarget -Force
    Write-Host "[+] Installed Windows 11 Quick Settings Flyout to: $FlyoutTarget" -ForegroundColor Green
}
if (Test-Path "amctl-flyout.bat") {
    $FlyoutBatTarget = Join-Path $InstallDir "amctl-flyout.bat"
    Copy-Item -Path "amctl-flyout.bat" -Destination $FlyoutBatTarget -Force
    Write-Host "[+] Installed 'amctl-flyout' CLI launcher to: $FlyoutBatTarget" -ForegroundColor Green
}
if (Test-Path "acer-tray.ps1") {
    $TrayTarget = Join-Path $InstallDir "acer-tray.ps1"
    Copy-Item -Path "acer-tray.ps1" -Destination $TrayTarget -Force
}
if (Test-Path "amctl-tray.bat") {
    $TrayBatTarget = Join-Path $InstallDir "amctl-tray.bat"
    Copy-Item -Path "amctl-tray.bat" -Destination $TrayBatTarget -Force
}

# Add to User PATH if not already added
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*acer_monitor_cli*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    Write-Host "[+] Added $InstallDir to User PATH." -ForegroundColor Green
}

# Create Start Menu Shortcut
$StartMenuDir = Join-Path $env:AppData "Microsoft\Windows\Start Menu\Programs"
$ShortcutPath = Join-Path $StartMenuDir "Acer Monitor Quick Settings.bat"
$BatContent = "@echo off`r`nstart powershell.exe -Sta -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$InstallDir\flyout.ps1`"`r`n"
Set-Content -Path $ShortcutPath -Value $BatContent
Write-Host "[+] Created Start Menu Shortcut: $ShortcutPath" -ForegroundColor Green

# Register Windows Startup Shortcut for System Tray & Quick Settings Flyout
$StartupDir = Join-Path $env:AppData "Microsoft\Windows\Start Menu\Programs\Startup"
$TrayBatPath = Join-Path $StartupDir "Acer Monitor Tray.bat"
$TrayBatContent = "@echo off`r`nstart powershell.exe -Sta -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$InstallDir\flyout.ps1`"`r`n"
Set-Content -Path $TrayBatPath -Value $TrayBatContent
Write-Host "[+] Registered Windows 11 Quick Settings Monitor auto-start in Startup folder." -ForegroundColor Green

# Register Windows Task Scheduler task for Smart Idle Dimmer
$TaskName = "AcerMonitorIdleDimmer"
$Action = New-ScheduledTaskAction -Execute $ExeTarget -Argument "idle-dimmer --idle-secs 300 --dim-to 10"
$Trigger = New-ScheduledTaskTrigger -AtLogOn
try {
    Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -User $env:USERNAME -ErrorAction SilentlyContinue | Out-Null
    Write-Host "[+] Registered Windows Task Scheduler job '$TaskName' (runs at Logon)." -ForegroundColor Green
} catch {
    Write-Host "[!] Could not register Task Scheduler job automatically (Run as Admin if desired)." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Installation Complete! ===" -ForegroundColor Cyan
Write-Host "Both 'acer_monitor_cli' and 'amctl' commands are now installed and ready in your PATH."
Write-Host "The Native Rust System Tray widget is configured to start automatically on Windows logon."
Write-Host "Launch it immediately with:"
Write-Host "  amctl tray"

