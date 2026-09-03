# Windows 1-Click Installation Script for Acer Display Center & CLI (amctl)
$ErrorActionPreference = "Stop"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "   🚀 Installing Acer Display Center & Monitor CLI Suite (amctl)" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# 1. Resolve Directories Robustly
$UserHome = [Environment]::GetFolderPath('UserProfile')
if (-not $UserHome) { $UserHome = $env:USERPROFILE }
if (-not $UserHome) { $UserHome = "C:\Users\$env:USERNAME" }

$LocalAppData = [Environment]::GetFolderPath('LocalApplicationData')
if (-not $LocalAppData -or -not (Test-Path $LocalAppData)) {
    $LocalAppData = "$UserHome\AppData\Local"
}

$AppData = [Environment]::GetFolderPath('ApplicationData')
if (-not $AppData -or -not (Test-Path $AppData)) {
    $AppData = "$UserHome\AppData\Roaming"
}

$DesktopDir = [Environment]::GetFolderPath('Desktop')
if (-not $DesktopDir -or -not (Test-Path $DesktopDir)) {
    $DesktopDir = "$UserHome\Desktop"
}

$InstallDir = "$LocalAppData\Programs\acer_monitor_cli"
if (-not (Test-Path $InstallDir)) {
    [System.IO.Directory]::CreateDirectory($InstallDir) | Out-Null
}

$ExeTarget = "$InstallDir\amctl.exe"
$LegacyExeTarget = "$InstallDir\acer_monitor_cli.exe"
$GuiExeTarget = "$InstallDir\acer_display_center.exe"
$IcoTarget = "$InstallDir\app.ico"

# 2. Stop running processes
Stop-Process -Name amctl,acer_monitor_cli,acer_display_center -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

# 3. Check for local build or download pre-compiled release from GitHub
$Installed = $false

$CliCand = @(
    "$env:CARGO_TARGET_DIR\release\amctl.exe",
    "$env:CARGO_TARGET_DIR\release\acer_monitor_cli.exe",
    "C:\rust-target\release\amctl.exe",
    "C:\rust-target\release\acer_monitor_cli.exe",
    "$PSScriptRoot\target\release\amctl.exe",
    "$PSScriptRoot\target\release\acer_monitor_cli.exe",
    "target\release\amctl.exe",
    "target\release\acer_monitor_cli.exe",
    "$PSScriptRoot\dist\amctl.exe",
    "$PSScriptRoot\dist\acer_monitor_cli.exe",
    "dist\amctl.exe",
    "dist\acer_monitor_cli.exe"
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

$GuiCand = @(
    "$env:CARGO_TARGET_DIR\release\acer_display_center.exe",
    "C:\rust-target\release\acer_display_center.exe",
    "$PSScriptRoot\target\release\acer_display_center.exe",
    "target\release\acer_display_center.exe",
    "$PSScriptRoot\dist\acer_display_center.exe",
    "dist\acer_display_center.exe"
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

if ($CliCand) {
    try {
        Copy-Item -Path $CliCand -Destination $ExeTarget -Force
        Copy-Item -Path $CliCand -Destination $LegacyExeTarget -Force
        if ($GuiCand) {
            Copy-Item -Path $GuiCand -Destination $GuiExeTarget -Force
        } else {
            Copy-Item -Path $CliCand -Destination $GuiExeTarget -Force
        }
        $Installed = $true
        Write-Host "[+] Installed newest local release binary ($CliCand)." -ForegroundColor Green
    } catch {}
}

if (-not $Installed) {
    Write-Host "[*] Downloading latest standalone release binaries from GitHub..." -ForegroundColor Yellow
    $BaseUrl = "https://github.com/sidhantjohnaind/acer-display-center/releases/latest/download"
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
        $Installed = $true
    } catch {
        Write-Host "[!] Could not download pre-built binary. Building locally with Cargo..." -ForegroundColor Yellow
        cargo build --release
        $builtCli = @("target\release\amctl.exe", "target\release\acer_monitor_cli.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
        $builtGui = "target\release\acer_display_center.exe"
        if ($builtCli) {
            Copy-Item -Path $builtCli -Destination $ExeTarget -Force
            Copy-Item -Path $builtCli -Destination $LegacyExeTarget -Force
            if (Test-Path $builtGui) {
                Copy-Item -Path $builtGui -Destination $GuiExeTarget -Force
            } else {
                Copy-Item -Path $builtCli -Destination $GuiExeTarget -Force
            }
            $Installed = $true
        }
    }
}

# 4. Install App Icon
if (Test-Path "app.ico") {
    Copy-Item -Path "app.ico" -Destination $IcoTarget -Force
} else {
    try {
        $RawUrl = "https://raw.githubusercontent.com/sidhantjohnaind/acer-display-center/main/app.ico"
        Invoke-WebRequest -Uri $RawUrl -OutFile $IcoTarget -UseBasicParsing
    } catch {}
}

# 5. Add to User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*acer_monitor_cli*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "[+] Added $InstallDir to User PATH." -ForegroundColor Green
}

# 6. Create Start Menu & Desktop Shortcuts
try {
    $WshShell = New-Object -ComObject WScript.Shell
    $StartMenuDir = "$AppData\Microsoft\Windows\Start Menu\Programs"
    if (Test-Path $StartMenuDir) {
        $GuiShortcut = $WshShell.CreateShortcut("$StartMenuDir\Acer Display Center.lnk")
        $GuiShortcut.TargetPath = $GuiExeTarget
        $GuiShortcut.Arguments = ""
        $GuiShortcut.WorkingDirectory = $InstallDir
        $GuiShortcut.Description = "Acer Display Center - Monitor Quick Settings & System Tray"
        if (Test-Path $IcoTarget) { $GuiShortcut.IconLocation = "$IcoTarget,0" }
        $GuiShortcut.Save()

        $TrayShortcut = $WshShell.CreateShortcut("$StartMenuDir\Acer Display Center (Tray Only).lnk")
        $TrayShortcut.TargetPath = $GuiExeTarget
        $TrayShortcut.Arguments = "tray"
        $TrayShortcut.WorkingDirectory = $InstallDir
        $TrayShortcut.Description = "Acer Display Center - System Tray Daemon"
        if (Test-Path $IcoTarget) { $TrayShortcut.IconLocation = "$IcoTarget,0" }
        $TrayShortcut.Save()
    }

    if (Test-Path $DesktopDir) {
        $GuiDesktopShortcut = $WshShell.CreateShortcut("$DesktopDir\Acer Display Center.lnk")
        $GuiDesktopShortcut.TargetPath = $GuiExeTarget
        $GuiDesktopShortcut.Arguments = ""
        $GuiDesktopShortcut.WorkingDirectory = $InstallDir
        $GuiDesktopShortcut.Description = "Acer Display Center - Monitor Quick Settings & System Tray"
        if (Test-Path $IcoTarget) { $GuiDesktopShortcut.IconLocation = "$IcoTarget,0" }
        $GuiDesktopShortcut.Save()

        $TrayDesktopShortcut = $WshShell.CreateShortcut("$DesktopDir\Acer Display Center (Tray Only).lnk")
        $TrayDesktopShortcut.TargetPath = $GuiExeTarget
        $TrayDesktopShortcut.Arguments = "tray"
        $TrayDesktopShortcut.WorkingDirectory = $InstallDir
        $TrayDesktopShortcut.Description = "Acer Display Center - System Tray Daemon"
        if (Test-Path $IcoTarget) { $TrayDesktopShortcut.IconLocation = "$IcoTarget,0" }
        $TrayDesktopShortcut.Save()
    }
    Write-Host "[+] Created Shortcuts: Acer Display Center (Start Menu & Desktop)" -ForegroundColor Green
} catch {}

# 7. Configure Windows Run Registry Key for System Tray Daemon
try {
    Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "AcerDisplayCenter" -Value "`"$GuiExeTarget`" tray" -Force
    Write-Host "[+] Configured System Tray daemon to start on Windows logon." -ForegroundColor Green
} catch {}

# 8. Clean up legacy Startup folder items
$StartupDir = "$AppData\Microsoft\Windows\Start Menu\Programs\Startup"
if (Test-Path $StartupDir) {
    Remove-Item -Path "$StartupDir\Acer*.lnk" -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "$StartupDir\Acer*.bat" -Force -ErrorAction SilentlyContinue
}

# 9. Launch Tray Daemon Now
try {
    Start-Process -FilePath $GuiExeTarget -ArgumentList "tray" -WindowStyle Hidden
    Write-Host "[+] Started Acer Display Center System Tray Daemon!" -ForegroundColor Green
} catch {}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "   ✅ Installation Successful!                                  " -ForegroundColor Green
Write-Host "   • Press [Ctrl + Alt + M] to open Acer Display Center Flyout   " -ForegroundColor White
Write-Host "   • Right-click the system tray icon for quick monitor controls " -ForegroundColor White
Write-Host "   • Run 'amctl --help' in terminal for full CLI commands        " -ForegroundColor White
Write-Host "================================================================" -ForegroundColor Cyan
