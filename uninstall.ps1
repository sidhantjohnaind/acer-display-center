# Acer Monitor CLI Clean Uninstaller for Windows PowerShell
Write-Host "🗑️ Uninstalling Acer Monitor CLI (amctl)..." -ForegroundColor Cyan

# 1. Stop running processes
Stop-Process -Name "acer_monitor_cli" -ErrorAction SilentlyContinue

# 2. Remove binary files
$targetDir = "$env:LOCALAPPDATA\Programs\AcerMonitorCLI"
if (Test-Path $targetDir) {
    Remove-Item -Path $targetDir -Recurse -Force
    Write-Host "  • Removed $targetDir" -ForegroundColor Yellow
}

# 3. Remove PATH entry from environment
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -like "*AcerMonitorCLI*") {
    $newPath = ($userPath -split ';' | Where-Object { $_ -notlike "*AcerMonitorCLI*" }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "  • Removed from User PATH" -ForegroundColor Yellow
}

Write-Host "✅ Acer Monitor CLI has been completely uninstalled!" -ForegroundColor Green
