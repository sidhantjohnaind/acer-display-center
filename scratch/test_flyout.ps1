try {
    $script = "$env:LocalAppData\Programs\acer_monitor_cli\flyout.ps1"
    & $script
} catch {
    Write-Host "ERROR: $($_.Exception.ToString())" -ForegroundColor Red
}
