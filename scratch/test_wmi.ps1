try {
    $levels = [byte[]](0..100)
    $instanceName = "DISPLAY\ACR0BDD\5&216e7962&0&UID37121_0"
    $wmiClass = [wmiclass]"root\wmi:WmiMonitorBrightness"
    $newInstance = $wmiClass.CreateInstance()
    $newInstance["Active"] = $true
    $newInstance["InstanceName"] = $instanceName
    $newInstance["CurrentBrightness"] = [byte]80
    $newInstance["Levels"] = [uint32]101
    $newInstance["Level"] = $levels
    $newInstance.Put()
    Write-Output "SUCCESS: Put instance into root\wmi:WmiMonitorBrightness"
} catch {
    Write-Output "ERROR: $($_.Exception.Message)"
}
