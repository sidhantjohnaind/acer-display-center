# Windows Installation Script for Acer Monitor CLI & Daemon
Write-Host "=== Acer Monitor CLI & Daemon Installer (Windows) ===" -ForegroundColor Cyan

# Locate executable
$ExeSource = ""
if (Test-Path "target\x86_64-pc-windows-gnu\release\acer_monitor_cli.exe") {
    $ExeSource = "target\x86_64-pc-windows-gnu\release\acer_monitor_cli.exe"
} elseif (Test-Path "target\release\acer_monitor_cli.exe") {
    $ExeSource = "target\release\acer_monitor_cli.exe"
} else {
    Write-Host "Building release binary for Windows..." -ForegroundColor Yellow
    cargo build --release --target x86_64-pc-windows-gnu
    $ExeSource = "target\x86_64-pc-windows-gnu\release\acer_monitor_cli.exe"
}

# Create installation folder in LocalAppData
$InstallDir = Join-Path $env:LocalAppData "Programs\acer_monitor_cli"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExeTarget = Join-Path $InstallDir "acer_monitor_cli.exe"
Copy-Item -Path $ExeSource -Destination $ExeTarget -Force
Write-Host "[+] Installed binary to: $ExeTarget" -ForegroundColor Green

# Copy Taskbar System Tray Application script
if (Test-Path "acer-tray.ps1") {
    $TrayTarget = Join-Path $InstallDir "acer-tray.ps1"
    Copy-Item -Path "acer-tray.ps1" -Destination $TrayTarget -Force
    Write-Host "[+] Installed System Tray Widget script to: $TrayTarget" -ForegroundColor Green
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
$ShortcutPath = Join-Path $StartMenuDir "Acer Monitor Control.bat"
$BatContent = "@echo off`r`n`"$ExeTarget`" brightness 50 --osd`r`n"
Set-Content -Path $ShortcutPath -Value $BatContent
Write-Host "[+] Created Start Menu Shortcut: $ShortcutPath" -ForegroundColor Green

# Register Windows Startup Shortcut for System Tray Widget
$StartupDir = Join-Path $env:AppData "Microsoft\Windows\Start Menu\Programs\Startup"
$TrayBatPath = Join-Path $StartupDir "Acer Monitor Tray.bat"
$TrayBatContent = "@echo off`r`nstart /b powershell.exe -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$InstallDir\acer-tray.ps1`"`r`n"
Set-Content -Path $TrayBatPath -Value $TrayBatContent
Write-Host "[+] Registered Windows System Tray Widget auto-start in Startup folder." -ForegroundColor Green

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
Write-Host "The Windows Taskbar System Tray Widget is ready and will start automatically on logon."
Write-Host "You can also launch it manually anytime with:"
Write-Host "  powershell -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$InstallDir\acer-tray.ps1`""

