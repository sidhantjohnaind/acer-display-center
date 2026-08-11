
use std::collections::BTreeSet;

#[derive(Debug, Default, Clone)]
pub struct AcerFeatures {
    pub power: bool,
    pub brightness: bool,
    pub contrast: bool,

    pub refresh_num: bool,
    pub overdrive: bool,
    pub aim_point: bool,

    pub blue_light: bool,
    pub gamma: bool,
    pub color_temp: bool,

    pub display_mode: bool,
    pub color_space: bool,
    pub black_boost: bool,

    pub key_lock: bool,
    pub power_key: bool,
    pub power_indicator: bool,
}

#[derive(Debug, Default, Clone)]
pub struct MonitorCapabilities {
    pub vcp_codes: BTreeSet<u8>,
    pub acer: AcerFeatures,
    pub raw_capabilities: String,
}

impl MonitorCapabilities {
    pub fn report(&self, description: &str) -> String {
        let mut out = String::new();
        use std::fmt::Write as _;

        let _ = writeln!(out, "Monitor: {description}");
        let _ = writeln!(out, "Raw MCCS capabilities:");
        if self.raw_capabilities.is_empty() {
            let _ = writeln!(out, "  <empty>");
        } else {
            let _ = writeln!(out, "  {}", self.raw_capabilities);
        }

        let _ = writeln!(out, "\nParsed VCP codes:");
        if self.vcp_codes.is_empty() {
            let _ = writeln!(out, "  <none>");
        } else {
            for code in &self.vcp_codes {
                let _ = writeln!(out, "  0x{code:02X}");
            }
        }

        let _ = writeln!(out, "\nFeature flags:");
        let mut push_flag = |name: &str, value: bool| {
            let _ = writeln!(out, "  {name:<16} {value}");
        };

        push_flag("Power", self.acer.power);
        push_flag("Brightness", self.acer.brightness);
        push_flag("Contrast", self.acer.contrast);
        push_flag("KeyLock", self.acer.key_lock);
        push_flag("PowerKey", self.acer.power_key);
        push_flag("PowerIndicator", self.acer.power_indicator);
        push_flag("RefreshNum", self.acer.refresh_num);
        push_flag("OverDrive", self.acer.overdrive);
        push_flag("AimPoint", self.acer.aim_point);
        push_flag("BlueLight", self.acer.blue_light);
        push_flag("Gamma", self.acer.gamma);
        push_flag("ColorTemp", self.acer.color_temp);
        push_flag("DisplayMode", self.acer.display_mode);
        push_flag("ColorSpace", self.acer.color_space);
        push_flag("BlackBoost", self.acer.black_boost);

        out
    }
}

fn parse_vcp_codes(caps: &str) -> BTreeSet<u8> {
    let mut codes = BTreeSet::new();

    if let Some(vcp_start) = caps.find("vcp(") {
        let rest = &caps[vcp_start + 4..];
        if let Some(end) = rest.find(')') {
            let section = &rest[..end];
            for token in section.split_whitespace() {
                let token = token.trim_matches(|c: char| !c.is_ascii_hexdigit());
                if token.is_empty() {
                    continue;
                }
                if let Ok(code) = u8::from_str_radix(token, 16) {
                    codes.insert(code);
                }
            }
        }
    }

    codes
}

fn apply_feature_flags(
    cap: &mut MonitorCapabilities,
    has_d6: bool,
    has_e0_keylock: bool,
    has_e0_powerkey: bool,
    has_e0_indicator: bool,
    has_e0_refresh: bool,
    has_e0_od: bool,
    has_e0_aim: bool,
    has_e7_bluelight: bool,
    has_e7_gamma: bool,
    has_e7_colortemp: bool,
    has_e2: bool,
    has_e9: bool,
    has_e5: bool,
) {
    let vcp = &cap.vcp_codes;

    cap.acer.power = has_d6;
    cap.acer.brightness = vcp.contains(&0x10);
    cap.acer.contrast = vcp.contains(&0x12);

    cap.acer.key_lock = has_e0_keylock;
    cap.acer.power_key = has_e0_powerkey;
    cap.acer.power_indicator = has_e0_indicator;
    cap.acer.refresh_num = has_e0_refresh;
    cap.acer.overdrive = has_e0_od;
    cap.acer.aim_point = has_e0_aim;

    cap.acer.blue_light = has_e7_bluelight;
    cap.acer.gamma = has_e7_gamma;
    cap.acer.color_temp = has_e7_colortemp;

    cap.acer.display_mode = has_e2;
    cap.acer.black_boost = has_e5;
    cap.acer.color_space = has_e9;
}


#[cfg(target_os = "linux")]
mod platform {
    use super::{apply_feature_flags, MonitorCapabilities};
    use ddc::Ddc;
    use std::{collections::BTreeSet, fs, path::Path};

    pub struct Monitor {
        pub description: String,
        pub capabilities: MonitorCapabilities,
        device_path: String,
    }

    pub struct MonitorSet {
        monitors: Vec<Monitor>,
    }

    fn connected_monitor_name() -> Option<String> {
        let entries = fs::read_dir("/sys/class/drm").ok()?;

        for entry in entries.flatten() {
            let path = entry.path();
            let status = path.join("status");
            let edid = path.join("edid");

            if !status.exists() || !edid.exists() {
                continue;
            }

            let status_text = fs::read_to_string(&status).ok()?;
            if status_text.trim() != "connected" {
                continue;
            }

            if let Some(name) = edid_display_name(&edid) {
                return Some(name);
            }
        }

        None
    }

    fn edid_display_name(edid_path: &Path) -> Option<String> {
        let data = fs::read(edid_path).ok()?;
        if data.len() < 128 {
            return None;
        }

        // Search for the monitor name descriptor (tag 0xFC).
        for block in data.chunks(128) {
            if block.len() < 128 {
                continue;
            }

            // Detailed descriptor blocks start at offset 54.
            for desc in block[54..126].chunks(18) {
                if desc.len() < 18 {
                    continue;
                }
                if desc[0] == 0x00 && desc[1] == 0x00 && desc[2] == 0x00 && desc[3] == 0xFC {
                    let text = String::from_utf8_lossy(&desc[5..18]);
                    let name = text.trim_matches(char::from(0)).trim().to_string();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }

        None
    }

    impl Monitor {
        fn new(device_path: String) -> Result<Option<Self>, String> {
            let mut handle = match ddc_i2c::from_i2c_device(device_path.as_str()) {
                Ok(h) => h,
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Permission denied") || err_str.contains("EACCES") {
                        return Err(err_str);
                    }
                    return Ok(None);
                }
            };

            let usable = handle.get_vcp_feature(0x10).is_ok()
                || handle.get_vcp_feature(0xDF).is_ok();

            if !usable {
                return Ok(None);
            }

            let monitor_name = connected_monitor_name().unwrap_or_else(|| device_path.clone());
            Ok(Some(Self {
                description: format!("{} ({})", monitor_name, device_path),
                capabilities: MonitorCapabilities::default(),
                device_path,
            }))
        }

        pub fn set_vcp(&mut self, code: u8, value: u32) -> Result<(), String> {
            let mut handle = ddc_i2c::from_i2c_device(self.device_path.as_str())
                .map_err(|e| e.to_string())?;
            let value: u16 = value
                .try_into()
                .map_err(|_| format!("Value {} exceeds u16 range", value))?;
            handle.set_vcp_feature(code, value).map_err(|e| e.to_string())
        }

        pub fn get_vcp(&mut self, code: u8) -> Result<(u32, u32), String> {
            let mut handle = ddc_i2c::from_i2c_device(self.device_path.as_str())
                .map_err(|e| e.to_string())?;
            let v = handle.get_vcp_feature(code).map_err(|e| e.to_string())?;
            Ok((u32::from(v.value()), u32::from(v.maximum())))
        }

        fn probe_e0(&mut self, selector: u32) -> bool {
            self.set_vcp(0xE0, selector).is_ok() && self.get_vcp(0xE1).is_ok()
        }

        fn probe_e7(&mut self, selector: u32) -> bool {
            self.set_vcp(0xE7, selector).is_ok() && self.get_vcp(0xE8).is_ok()
        }

        fn probe_e9(&mut self, selector: u32) -> bool {
            self.set_vcp(0xE9, selector).is_ok() && self.get_vcp(0xEA).is_ok()
        }

        pub fn update_capabilities(&mut self) -> Result<(), String> {
            let mut codes = BTreeSet::new();
            for code in [
                0x04u8, 0x06, 0x08, 0x0B, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A,
                0x60, 0x62, 0x8D, 0xD6, 0xE2, 0xE5, 0xE7, 0xE8, 0xE9, 0xEA,
            ] {
                if self.get_vcp(code).is_ok() {
                    codes.insert(code);
                }
            }

            self.capabilities.raw_capabilities = format!(
                "i2c-probed(vcp({}))",
                codes.iter().map(|c| format!("{c:02X}")).collect::<Vec<_>>().join(" ")
            );
            self.capabilities.vcp_codes = codes;

            let has_d6 = self.get_vcp(0xD6).is_ok();
            let has_e2 = self.get_vcp(0xE2).is_ok();
            let has_e5 = self.get_vcp(0xE5).is_ok();
            let has_e0_keylock = self.probe_e0(0x00);
            let has_e0_powerkey = self.probe_e0(0x01);
            let has_e0_indicator = self.probe_e0(0x02);
            let has_e0_refresh = self.probe_e0(0x05);
            let has_e0_od = self.probe_e0(0x04);
            let has_e0_aim = self.probe_e0(0x06);
            let has_e7_bluelight = self.probe_e7(0x00);
            let has_e7_gamma = self.probe_e7(0x01);
            let has_e7_colortemp = self.probe_e7(0x02);
            let has_e9_colorspace = self.probe_e9(0x00);

            apply_feature_flags(
                &mut self.capabilities,
                has_d6,
                has_e0_keylock,
                has_e0_powerkey,
                has_e0_indicator,
                has_e0_refresh,
                has_e0_od,
                has_e0_aim,
                has_e7_bluelight,
                has_e7_gamma,
                has_e7_colortemp,
                has_e2,
                has_e9_colorspace,
                has_e5,
            );

            Ok(())
        }
    }

    impl MonitorSet {
        pub fn enumerate() -> Result<Self, String> {
            let mut monitors = Vec::new();
            let mut perm_error = false;

            let entries = fs::read_dir("/dev").map_err(|e| e.to_string())?;
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !name.starts_with("i2c-") {
                    continue;
                }

                let device_path = path.to_string_lossy().to_string();
                match Monitor::new(device_path) {
                    Ok(Some(mon)) => monitors.push(mon),
                    Ok(None) => {}
                    Err(e) => {
                        if e.contains("Permission denied") || e.contains("EACCES") {
                            perm_error = true;
                        }
                    }
                }
            }

            if monitors.is_empty() {
                if perm_error {
                    Err("Permission denied accessing /dev/i2c-* devices.\nTo fix this, add your user to the i2c group:\n  sudo usermod -aG i2c $USER\nthen log out and log back in, or run with sudo.".to_string())
                } else {
                    Err("No DDC/CI monitors found under /dev/i2c-*".to_string())
                }
            } else {
                Ok(Self { monitors })
            }
        }

        pub fn print_list(&self) {
            if self.monitors.is_empty() {
                println!("No DDC/CI monitors found.");
                return;
            }

            for (i, m) in self.monitors.iter().enumerate() {
                println!("[{i}] {}", m.description);
            }
        }

        pub fn print_list_json(&self) {
            let items: Vec<String> = self.monitors
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let desc_escaped = m.description.replace('"', "\\\"");
                    format!("  {{\"index\": {i}, \"description\": \"{desc_escaped}\"}}")
                })
                .collect();
            println!("[\n{}\n]", items.join(",\n"));
        }

        pub fn pick_mut(&mut self, idx: Option<usize>) -> Result<&mut Monitor, String> {
            let idx = idx.unwrap_or(0);
            let len = self.monitors.len();
            self.monitors
                .get_mut(idx)
                .ok_or_else(|| format!("Monitor index {idx} out of range (0..{})", len.saturating_sub(1)))
        }

        pub fn monitors_mut(&mut self) -> &mut [Monitor] {
            &mut self.monitors
        }

        pub fn pick_mut_by_specifier(&mut self, spec: Option<&str>) -> Result<&mut Monitor, String> {
            let Some(spec) = spec else {
                return self.pick_mut(Some(0));
            };

            if let Ok(idx) = spec.parse::<usize>() {
                return self.pick_mut(Some(idx));
            }

            let spec_lower = spec.to_lowercase();
            let matches: Vec<usize> = self.monitors
                .iter()
                .enumerate()
                .filter(|(_, m)| m.description.to_lowercase().contains(&spec_lower))
                .map(|(i, _)| i)
                .collect();

            if matches.is_empty() {
                Err(format!("No monitor description matching '{spec}'. Use 'acer_monitor_cli list' to see available monitors."))
            } else {
                Ok(&mut self.monitors[matches[0]])
            }
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::{apply_feature_flags, parse_vcp_codes, MonitorCapabilities};
    use crate::ddc::*;
    use std::ptr;

    fn last_error() -> String {
        unsafe { format!("Win32 error {}", GetLastError()) }
    }

    fn wide_to_string(buf: &[u16]) -> String {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }

    unsafe extern "system" fn enum_proc(hmonitor: HMONITOR, _hdc: HDC, _rect: *mut RECT, lparam: LPARAM) -> BOOL {
        let vec = &mut *(lparam as *mut Vec<HMONITOR>);
        vec.push(hmonitor);
        1
    }

    pub struct Monitor {
        pub description: String,
        pub capabilities: MonitorCapabilities,
        hphysical: HANDLE,
    }

    pub struct MonitorSet {
        monitors: Vec<Monitor>,
        raw_handles: Vec<PHYSICAL_MONITOR>,
    }

    impl Monitor {
        pub fn set_vcp(&mut self, code: u8, value: u32) -> Result<(), String> {
            unsafe {
                if SetVCPFeature(self.hphysical, code, value) == 0 {
                    Err(format!("SetVCPFeature(0x{code:02X}, {value}) failed: {}", last_error()))
                } else {
                    Ok(())
                }
            }
        }

        pub fn get_vcp(&mut self, code: u8) -> Result<(u32, u32), String> {
            unsafe {
                let mut current = 0u32;
                let mut max = 0u32;
                if GetVCPFeatureAndVCPFeatureReply(self.hphysical, code, &mut current, &mut max) == 0 {
                    Err(format!("GetVCPFeatureAndVCPFeatureReply(0x{code:02X}) failed: {}", last_error()))
                } else {
                    Ok((current, max))
                }
            }
        }
        pub fn probe_e0(&mut self, selector: u32) -> bool {
            self.set_vcp(0xE0, selector).is_ok() &&
            self.get_vcp(0xE1).is_ok()
        }

    
        pub fn probe_e7(&mut self, selector: u32) -> bool {
            self.set_vcp(0xE7, selector).is_ok() &&
            self.get_vcp(0xE8).is_ok()
        }

    
        pub fn probe_e9(&mut self, selector: u32) -> bool {
            self.set_vcp(0xE9, selector).is_ok() &&
            self.get_vcp(0xEA).is_ok()
        }
        pub fn capabilities_string(&mut self) -> Result<String, String> {
            unsafe {
                let mut len = 0u32;
                if GetCapabilitiesStringLength(self.hphysical, &mut len) == 0 {
                    return Err(format!("GetCapabilitiesStringLength failed: {}", last_error()));
                }
                if len == 0 {
                    return Ok(String::new());
                }
                let mut buf = vec![0i8; len as usize + 1];
                if CapabilitiesRequestAndCapabilitiesReply(self.hphysical, buf.as_mut_ptr(), len + 1) == 0 {
                    return Err(format!("CapabilitiesRequestAndCapabilitiesReply failed: {}", last_error()));
                }
                let bytes: Vec<u8> = buf.into_iter().take_while(|&c| c != 0).map(|c| c as u8).collect();
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            }
        }

        pub fn update_capabilities(&mut self) -> Result<(), String> {
            let caps = self.capabilities_string()?;
            self.capabilities.raw_capabilities = caps.clone();
            self.capabilities.vcp_codes = parse_vcp_codes(&caps);

            let has_d6 = self.get_vcp(0xD6).is_ok();

            let has_e0_keylock = self.probe_e0(0x00);
            let has_e0_powerkey = self.probe_e0(0x01);
            let has_e0_indicator = self.probe_e0(0x02);
            let has_e0_refresh = self.probe_e0(0x05);
            let has_e0_od = self.probe_e0(0x04);
            let has_e0_aim = self.probe_e0(0x06);

            let has_e7_bluelight = self.probe_e7(0x00);
            let has_e7_gamma = self.probe_e7(0x01);
            let has_e7_colortemp = self.probe_e7(0x02);

            let has_e9_colorspace = self.probe_e9(0x00);
            let has_e2 = self.get_vcp(0xE2).is_ok();
            let has_e5 = self.get_vcp(0xE5).is_ok();

            apply_feature_flags(
                &mut self.capabilities,
                has_d6,
                has_e0_keylock,
                has_e0_powerkey,
                has_e0_indicator,
                has_e0_refresh,
                has_e0_od,
                has_e0_aim,
                has_e7_bluelight,
                has_e7_gamma,
                has_e7_colortemp,
                has_e2,
                has_e9_colorspace,
                has_e5,
            );

            Ok(())
        }
    }

    impl MonitorSet {
        pub fn enumerate() -> Result<Self, String> {
            let mut hmonitors: Vec<HMONITOR> = Vec::new();
            unsafe {
                if EnumDisplayMonitors(ptr::null_mut(), ptr::null(), Some(enum_proc), &mut hmonitors as *mut _ as isize) == 0 {
                    return Err(format!("EnumDisplayMonitors failed: {}", last_error()));
                }
            }

            let mut monitors = Vec::new();
            let mut raw_handles = Vec::new();

            for hmonitor in hmonitors {
                unsafe {
                    let mut count = 0u32;
                    if GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut count) == 0 {
                        return Err(format!("GetNumberOfPhysicalMonitorsFromHMONITOR failed: {}", last_error()));
                    }
                    if count == 0 {
                        continue;
                    }
                    let mut phys = vec![PHYSICAL_MONITOR::default(); count as usize];
                    if GetPhysicalMonitorsFromHMONITOR(hmonitor, count, phys.as_mut_ptr()) == 0 {
                        return Err(format!("GetPhysicalMonitorsFromHMONITOR failed: {}", last_error()));
                    }
                    for p in &phys {
                        monitors.push(Monitor {
                            hphysical: p.hPhysicalMonitor,
                            description: wide_to_string(&p.szPhysicalMonitorDescription),
                            capabilities: MonitorCapabilities::default(),
                        });
                    }
                    raw_handles.extend_from_slice(&phys);
                }
            }

            if monitors.is_empty() {
                Err("No DDC/CI monitors found".to_string())
            } else {
                Ok(Self { monitors, raw_handles })
            }
        }

        pub fn print_list(&self) {
            if self.monitors.is_empty() {
                println!("No physical monitors found.");
                return;
            }
            for (i, m) in self.monitors.iter().enumerate() {
                println!("[{i}] {}", m.description);
            }
        }

        pub fn print_list_json(&self) {
            let items: Vec<String> = self.monitors
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let desc_escaped = m.description.replace('"', "\\\"");
                    format!("  {{\"index\": {i}, \"description\": \"{desc_escaped}\"}}")
                })
                .collect();
            println!("[\n{}\n]", items.join(",\n"));
        }

        pub fn pick_mut(&mut self, idx: Option<usize>) -> Result<&mut Monitor, String> {
            let idx = idx.unwrap_or(0);
            let len = self.monitors.len();
            self.monitors
                .get_mut(idx)
                .ok_or_else(|| format!("Monitor index {idx} out of range (0..{})", len.saturating_sub(1)))
        }

        pub fn monitors_mut(&mut self) -> &mut [Monitor] {
            &mut self.monitors
        }

        pub fn pick_mut_by_specifier(&mut self, spec: Option<&str>) -> Result<&mut Monitor, String> {
            let Some(spec) = spec else {
                return self.pick_mut(Some(0));
            };

            if let Ok(idx) = spec.parse::<usize>() {
                return self.pick_mut(Some(idx));
            }

            let spec_lower = spec.to_lowercase();
            let matches: Vec<usize> = self.monitors
                .iter()
                .enumerate()
                .filter(|(_, m)| m.description.to_lowercase().contains(&spec_lower))
                .map(|(i, _)| i)
                .collect();

            if matches.is_empty() {
                Err(format!("No monitor description matching '{spec}'. Use 'acer_monitor_cli list' to see available monitors."))
            } else {
                Ok(&mut self.monitors[matches[0]])
            }
        }
    }

    impl Drop for MonitorSet {
        fn drop(&mut self) {
            unsafe {
                if !self.raw_handles.is_empty() {
                    let _ = DestroyPhysicalMonitors(self.raw_handles.len() as u32, self.raw_handles.as_mut_ptr());
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub use platform::{Monitor, MonitorSet};
#[cfg(windows)]
pub use platform::{Monitor, MonitorSet};

#[cfg(not(any(target_os = "linux", windows)))]
compile_error!("This project currently supports only Linux and Windows");
