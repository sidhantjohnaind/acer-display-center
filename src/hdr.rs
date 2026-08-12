use std::process::Command;

/// Toggles OS-level HDR (Windows OS HDR or Linux Wayland HDR)
pub fn set_os_hdr(enable: bool) {
    #[cfg(windows)]
    {
        // Toggle Windows OS HDR via Registry & PowerShell
        let val = if enable { "1" } else { "0" };
        let script = format!(
            "$reg = 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\VideoSettings'; \
             if (-not (Test-Path $reg)) {{ New-Item -Path $reg -Force | Out-Null }}; \
             Set-ItemProperty -Path $reg -Name 'EnableHDR' -Value {val} -ErrorAction SilentlyContinue"
        );
        let _ = Command::new("powershell")
            .args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .output();
    }

    #[cfg(unix)]
    {
        // Toggle Linux Wayland / KDE / Hyprland HDR
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            let d = desktop.to_lowercase();
            if d.contains("hyprland") {
                let val = if enable { "1" } else { "0" };
                let _ = Command::new("hyprctl")
                    .args(&["keyword", "experimental:hdr", val])
                    .output();
            } else if d.contains("kde") {
                let status = if enable { "enable" } else { "disable" };
                let _ = Command::new("kscreen-doctor")
                    .arg(format!("output.1.hdr.{status}"))
                    .output();
            }
        }
    }
}
