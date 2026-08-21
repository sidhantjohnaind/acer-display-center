# Acer Monitor CLI & Suite Clean Uninstaller for Windows PowerShell
Write-Host "=== Acer Monitor CLI Suite Uninstaller (Windows) ===" -ForegroundColor Cyan

# 1. Stop all running background processes and tray widgets
Write-Host "[*] Stopping running processes and background widgets..." -ForegroundColor Yellow
Stop-Process -Name "acer_monitor_cli" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "amctl" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "acer_display_center" -Force -ErrorAction SilentlyContinue

# Stop background PowerShell tray scripts if active
Get-Process powershell -ErrorAction SilentlyContinue | Where-Object {
    $_.MainWindowTitle -eq "" -and $_.Id -ne $PID
} | Stop-Process -Force -ErrorAction SilentlyContinue

# 2. Unregister Windows Task Scheduler jobs
Write-Host "[*] Removing Task Scheduler background tasks..." -ForegroundColor Yellow
try {
    Unregister-ScheduledTask -TaskName "AcerMonitorIdleDimmer" -Confirm:$false -ErrorAction SilentlyContinue | Out-Null
    Write-Host "[+] Removed Scheduled Task: AcerMonitorIdleDimmer" -ForegroundColor Green
} catch { }

# 3. Remove Startup auto-launch shortcut and Registry Run key
$StartupDir = [Environment]::GetFolderPath('Startup')
$StartupLnk = Join-Path $StartupDir "Acer Display Center.lnk"
$TrayBatPath = Join-Path $StartupDir "Acer Monitor Tray.bat"
if (Test-Path $StartupLnk) {
    Remove-Item -Path $StartupLnk -Force -ErrorAction SilentlyContinue
    Write-Host "[+] Removed Windows Startup shortcut: $StartupLnk" -ForegroundColor Green
}
if (Test-Path $TrayBatPath) {
    Remove-Item -Path $TrayBatPath -Force -ErrorAction SilentlyContinue
    Write-Host "[+] Removed Windows Startup item: $TrayBatPath" -ForegroundColor Green
}
try {
    Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "AcerDisplayCenter" -ErrorAction SilentlyContinue
    Write-Host "[+] Removed Windows Run registry entry." -ForegroundColor Green
} catch { }

# 4. Remove Start Menu and Desktop shortcuts
$StartMenuDir = Join-Path $env:AppData "Microsoft\Windows\Start Menu\Programs"
$DesktopDir = [Environment]::GetFolderPath("Desktop")

$Shortcuts = @(
    (Join-Path $StartMenuDir "Acer Display Center.lnk"),
    (Join-Path $StartMenuDir "Acer Display Center (Tray Only).lnk"),
    (Join-Path $StartMenuDir "Acer Monitor Control.bat"),
    (Join-Path $DesktopDir "Acer Display Center.lnk"),
    (Join-Path $DesktopDir "Acer Display Center (Tray Only).lnk")
)
foreach ($sc in $Shortcuts) {
    if (Test-Path $sc) {
        Remove-Item -Path $sc -Force -ErrorAction SilentlyContinue
        Write-Host "[+] Removed shortcut: $sc" -ForegroundColor Green
    }
}

# 5. Remove installed binaries and scripts
$InstallDirs = @(
    (Join-Path $env:LocalAppData "Programs\acer_monitor_cli"),
    (Join-Path $env:LocalAppData "Programs\AcerMonitorCLI")
)

foreach ($dir in $InstallDirs) {
    if (Test-Path $dir) {
        Remove-Item -Path $dir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "[+] Removed installation directory: $dir" -ForegroundColor Green
    }
}

# 6. Remove PATH entry from User Environment
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -match "acer_monitor_cli|AcerMonitorCLI") {
    $CleanPath = ($UserPath -split ';' | Where-Object { 
        $_ -and ($_ -notlike "*acer_monitor_cli*") -and ($_ -notlike "*AcerMonitorCLI*") 
    }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $CleanPath, [EnvironmentVariableTarget]::User)
    Write-Host "[+] Cleaned up User PATH environment variable." -ForegroundColor Green
}

Write-Host ""
Write-Host "=== Uninstallation Complete! ===" -ForegroundColor Green
Write-Host "Acer Monitor CLI, System Tray, Scheduled Tasks, and PATH entries have been completely removed."
