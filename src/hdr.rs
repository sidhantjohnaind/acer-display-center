#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(windows)]
pub fn set_os_hdr(enable: bool) {
    // Avoid redundant OS display pipeline tear-down & black screening if already in desired state
    if get_os_hdr() == enable {
        return;
    }

    use windows::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo,
        DisplayConfigSetDeviceInfo,
        GetDisplayConfigBufferSizes,
        DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_PATH_INFO,
        QDC_ONLY_ACTIVE_PATHS,
        DISPLAYCONFIG_MODE_INFO,
        QueryDisplayConfig,
    };

    const DISPLAYCONFIG_DEVICE_INFO_SET_HDR_STATE: i32 = 16;
    const DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO: i32 = 9;

    #[repr(C)]
    struct DisplayConfigSetHdrState {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        value: u32,
    }

    #[repr(C)]
    struct DisplayConfigGetAdvancedColorInfo {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        value: u32,
        color_encoding: u32,
        bits_per_channel: u32,
    }

    unsafe {
        let mut path_count: u32 = 0;
        let mut mode_count: u32 = 0;

        let result = GetDisplayConfigBufferSizes(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            &mut mode_count,
        );

        if result.0 != 0 {
            return;
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

        let result = QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        );

        if result.0 != 0 {
            return;
        }

        for path in &paths {
            let mut info = DisplayConfigGetAdvancedColorInfo {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE(DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO),
                    size: std::mem::size_of::<DisplayConfigGetAdvancedColorInfo>() as u32,
                    adapterId: path.targetInfo.adapterId,
                    id: path.targetInfo.id,
                },
                value: 0,
                color_encoding: 0,
                bits_per_channel: 0,
            };

            // Only attempt to set HDR on displays that support advanced color (info.value & 0x1)
            if DisplayConfigGetDeviceInfo(&mut info.header) == 0 {
                let advanced_color_supported = (info.value & 0x1) != 0;
                if advanced_color_supported {
                    let mut hdr = DisplayConfigSetHdrState {
                        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                            r#type: windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE(DISPLAYCONFIG_DEVICE_INFO_SET_HDR_STATE),
                            size: std::mem::size_of::<DisplayConfigSetHdrState>() as u32,
                            adapterId: path.targetInfo.adapterId,
                            id: path.targetInfo.id,
                        },
                        value: if enable { 1 } else { 0 },
                    };

                    let _ = DisplayConfigSetDeviceInfo(&mut hdr.header);
                }
            }
        }
    }
}

#[cfg(windows)]
pub fn get_os_hdr() -> bool {
    use windows::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo,
        GetDisplayConfigBufferSizes,
        DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_PATH_INFO,
        QDC_ONLY_ACTIVE_PATHS,
        DISPLAYCONFIG_MODE_INFO,
        QueryDisplayConfig,
    };

    const DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO: i32 = 9;

    #[repr(C)]
    struct DisplayConfigGetAdvancedColorInfo {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        value: u32,
        color_encoding: u32,
        bits_per_channel: u32,
    }

    unsafe {
        let mut path_count: u32 = 0;
        let mut mode_count: u32 = 0;
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count).0 != 0 {
            return false;
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

        if QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        ).0 != 0 {
            return false;
        }

        for path in &paths {
            let mut info = DisplayConfigGetAdvancedColorInfo {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE(DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO),
                    size: std::mem::size_of::<DisplayConfigGetAdvancedColorInfo>() as u32,
                    adapterId: path.targetInfo.adapterId,
                    id: path.targetInfo.id,
                },
                value: 0,
                color_encoding: 0,
                bits_per_channel: 0,
            };

            if DisplayConfigGetDeviceInfo(&mut info.header) == 0 {
                // Windows Display Config API Bitfield:
                // Bit 0 (0x1): advancedColorSupported
                // Bit 1 (0x2): advancedColorEnabled (true for both HDR and Windows 11 Auto Color Management / ACM)
                // Bit 2 (0x4): wideColorEnforced (true for ACM in SDR mode, false for native HDR)
                // Bit 3 (0x8): advancedColorForceDisabled
                let advanced_color_enabled = (info.value & 0x2) != 0;
                let wide_color_enforced = (info.value & 0x4) != 0;
                if advanced_color_enabled && !wide_color_enforced {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(windows)]
pub fn get_sdr_white_level() -> Option<u32> {
    if !get_os_hdr() {
        return None;
    }

    use windows::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo,
        GetDisplayConfigBufferSizes,
        DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_PATH_INFO,
        QDC_ONLY_ACTIVE_PATHS,
        DISPLAYCONFIG_MODE_INFO,
        QueryDisplayConfig,
    };

    const DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL: i32 = 11;

    #[repr(C)]
    struct DisplayConfigSdrWhiteLevel {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        sdr_white_level: u32,
    }

    unsafe {
        let mut path_count: u32 = 0;
        let mut mode_count: u32 = 0;
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count).0 != 0 {
            return None;
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

        if QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        ).0 != 0 {
            return None;
        }

        for path in &paths {
            let mut info = DisplayConfigSdrWhiteLevel {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE(DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL),
                    size: std::mem::size_of::<DisplayConfigSdrWhiteLevel>() as u32,
                    adapterId: path.targetInfo.adapterId,
                    id: path.targetInfo.id,
                },
                sdr_white_level: 0,
            };

            if DisplayConfigGetDeviceInfo(&mut info.header) == 0 {
                let raw = info.sdr_white_level;
                let pct = if raw >= 1000 {
                    ((raw - 1000) as f32 / (6000 - 1000) as f32 * 100.0).round() as u32
                } else {
                    0
                };
                return Some(pct.min(100));
            }
        }
    }
    None
}

#[cfg(windows)]
pub fn set_sdr_white_level(percent: u32) -> Result<(), String> {
    if !get_os_hdr() {
        return Err("Windows 10/11 HDR is not active.".into());
    }

    use windows::Win32::Devices::Display::{
        DisplayConfigSetDeviceInfo,
        GetDisplayConfigBufferSizes,
        DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_PATH_INFO,
        QDC_ONLY_ACTIVE_PATHS,
        DISPLAYCONFIG_MODE_INFO,
        QueryDisplayConfig,
    };

    const DISPLAYCONFIG_DEVICE_INFO_SET_SDR_WHITE_LEVEL: i32 = 0xFFFFFFEE_u32 as i32; // -18

    #[repr(C)]
    struct DisplayConfigSetSdrWhiteLevel {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        sdr_white_level: u32,
        final_value: u8,
    }

    let pct = percent.min(100);
    // 0% = 1000 (80 nits), 100% = 6000 (480 nits)
    let raw = 1000 + (pct as f32 / 100.0 * 5000.0).round() as u32;

    unsafe {
        let mut path_count: u32 = 0;
        let mut mode_count: u32 = 0;
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count).0 != 0 {
            return Err("Failed to get display config buffer sizes".into());
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

        if QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        ).0 != 0 {
            return Err("Failed to query display config".into());
        }

        for path in &paths {
            let mut req = DisplayConfigSetSdrWhiteLevel {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE(DISPLAYCONFIG_DEVICE_INFO_SET_SDR_WHITE_LEVEL),
                    size: std::mem::size_of::<DisplayConfigSetSdrWhiteLevel>() as u32,
                    adapterId: path.targetInfo.adapterId,
                    id: path.targetInfo.id,
                },
                sdr_white_level: raw,
                final_value: 1,
            };
            let res = DisplayConfigSetDeviceInfo(&mut req.header);
            if res != 0 {
                return Err(format!("DisplayConfigSetDeviceInfo error code: {}", res));
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn get_os_hdr() -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            let d = desktop.to_lowercase();
            if d.contains("kde") {
                if let Ok(out) = Command::new("kscreen-doctor").arg("-j").output() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    if text.contains("\"hdr\":true") || text.contains("\"hdr\": true") {
                        return true;
                    }
                }
            } else if d.contains("hyprland") {
                if let Ok(out) = Command::new("hyprctl").args(&["getoption", "experimental:hdr"]).output() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    if text.contains("int: 1") {
                        return true;
                    }
                }
                if let Ok(out) = Command::new("hyprctl").args(&["monitors", "-j"]).output() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    if text.contains("\"hdr\":true") || text.contains("\"hdr\": true") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(not(windows))]
pub fn get_sdr_white_level() -> Option<u32> {
    None
}

#[cfg(not(windows))]
pub fn set_sdr_white_level(_percent: u32) -> Result<(), String> {
    Err("SDR White Level slider is a Windows 10/11 CCD feature.".into())
}

#[cfg(not(windows))]
pub fn set_os_hdr(_enable: bool) {
    #[cfg(unix)]
    {
        use std::process::Command;
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            let d = desktop.to_lowercase();
            if d.contains("hyprland") {
                let val = if _enable { "1" } else { "0" };
                let _ = Command::new("hyprctl")
                    .args(&["keyword", "experimental:hdr", val])
                    .output();
            } else if d.contains("kde") {
                let status = if _enable { "enable" } else { "disable" };
                let _ = Command::new("kscreen-doctor")
                    .arg(format!("output.1.hdr.{status}"))
                    .output();
                let _ = Command::new("kscreen-doctor")
                    .arg(format!("output.DP-1.hdr.{status}"))
                    .output();
                let _ = Command::new("kscreen-doctor")
                    .arg(format!("output.HDMI-A-1.hdr.{status}"))
                    .output();
            }
        }
    }
}
