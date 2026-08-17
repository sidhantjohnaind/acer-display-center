#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(windows)]
pub fn set_os_hdr(enable: bool) {
    use windows::Win32::Devices::Display::{
        DisplayConfigSetDeviceInfo,
        GetDisplayConfigBufferSizes,
        DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_PATH_INFO,
        QDC_ONLY_ACTIVE_PATHS,
        DISPLAYCONFIG_MODE_INFO,
        QueryDisplayConfig,
    };

    const DISPLAYCONFIG_DEVICE_INFO_SET_HDR_STATE: i32 = 16;
    const DISPLAYCONFIG_MODE_INFO_TYPE_TARGET: i32 = 2;

    #[repr(C)]
    struct DisplayConfigSetHdrState {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        value: u32,
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

        for target in modes.iter().filter(|m| m.infoType.0 == DISPLAYCONFIG_MODE_INFO_TYPE_TARGET) {
            let mut hdr = DisplayConfigSetHdrState {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE(DISPLAYCONFIG_DEVICE_INFO_SET_HDR_STATE),
                    size: std::mem::size_of::<DisplayConfigSetHdrState>() as u32,
                    adapterId: target.adapterId,
                    id: target.id,
                },
                value: if enable { 1 } else { 0 },
            };

            let _ = DisplayConfigSetDeviceInfo(&mut hdr.header);
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
                if (info.value & 0x2) != 0 {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(not(windows))]
pub fn get_os_hdr() -> bool {
    false
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
            }
        }
    }
}
