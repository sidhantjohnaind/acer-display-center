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
$IcoTarget = Join-Path $InstallDir "app.ico"

# Stop any running instances before updating
Stop-Process -Name amctl,acer_monitor_cli -Force -ErrorAction SilentlyContinue

# Find local build or download from GitHub release
$LocalCandidates = @(
    "D:\temp\rust\target\release\acer_monitor_cli.exe",
    "target\release\acer_monitor_cli.exe",
    "target\x86_64-pc-windows-msvc\release\acer_monitor_cli.exe",
    "target\x86_64-pc-windows-gnu\release\acer_monitor_cli.exe"
)

$Installed = $false
foreach ($cand in $LocalCandidates) {
    if (Test-Path $cand) {
        Copy-Item -Path $cand -Destination $ExeTarget -Force
        Copy-Item -Path $cand -Destination $LegacyExeTarget -Force
        $Installed = $true
        Write-Host "[+] Installed local release binary." -ForegroundColor Green
        break
    }
}

if (-not $Installed) {
    Write-Host "[*] Downloading latest standalone release binary from GitHub..." -ForegroundColor Yellow
    $DownloadUrl = "https://github.com/sidhantjohnaind/acer-display-center/releases/latest/download/amctl-x86_64-pc-windows-msvc.exe"
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ExeTarget -UseBasicParsing
        Copy-Item -Path $ExeTarget -Destination $LegacyExeTarget -Force
        Write-Host "[+] Downloaded binary successfully." -ForegroundColor Green
    } catch {
        Write-Host "[!] Could not download pre-built binary. Building locally with Cargo..." -ForegroundColor Yellow
        cargo build --release
        Copy-Item -Path "target\release\acer_monitor_cli.exe" -Destination $ExeTarget -Force
        Copy-Item -Path "target\release\acer_monitor_cli.exe" -Destination $LegacyExeTarget -Force
    }
}

# Install App Icon
if (Test-Path "app.ico") {
    Copy-Item -Path "app.ico" -Destination $IcoTarget -Force
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
$StartShortcut = $WshShell.CreateShortcut((Join-Path $StartMenuDir "Acer Display Center.lnk"))
$StartShortcut.TargetPath = $ExeTarget
$StartShortcut.Arguments = "gui"
$StartShortcut.WorkingDirectory = $InstallDir
$StartShortcut.Description = "Acer Display Center - Monitor Quick Settings"
if (Test-Path $IcoTarget) { $StartShortcut.IconLocation = "$IcoTarget,0" }
$StartShortcut.Save()
Write-Host "[+] Created Start Menu Shortcut: Acer Display Center" -ForegroundColor Green

# Register Startup Daemon (amctl tray)
$StartupDir = Join-Path $env:AppData "Microsoft\Windows\Start Menu\Programs\Startup"
$StartupShortcut = $WshShell.CreateShortcut((Join-Path $StartupDir "Acer Display Center.lnk"))
$StartupShortcut.TargetPath = $ExeTarget
$StartupShortcut.Arguments = "tray"
$StartupShortcut.WorkingDirectory = $InstallDir
$StartupShortcut.Description = "Acer Display Center Background System Tray Daemon"
if (Test-Path $IcoTarget) { $StartupShortcut.IconLocation = "$IcoTarget,0" }
$StartupShortcut.Save()
Write-Host "[+] Registered System Tray background daemon at Windows Startup." -ForegroundColor Green

# Also configure Windows Run Registry Key for reliability
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "AcerDisplayCenter" -Value "`"$ExeTarget`" tray" -Force
Write-Host "[+] Registered Windows Run registry startup entry." -ForegroundColor Green

# Launch Tray Daemon Now
Start-Process -FilePath $ExeTarget -ArgumentList "tray" -WindowStyle Hidden
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

