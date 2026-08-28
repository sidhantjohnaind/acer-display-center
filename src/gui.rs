use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use eframe::egui::{self, Color32, Margin, Pos2, Rect, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Tab {
    Display,
    Gaming,
    Color,
    Tools,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentTheme {
    CyberCyan,      // #38BDF8
    NitroCrimson,   // #F43F5E
    EmeraldMatrix,  // #10B981
    AmethystPurple, // #A855F7
    AmberSunset,    // #F59E0B
    StealthGray,    // #94A3B8
    SolarGold,      // #FACC15
    NeonPink,       // #EC4899
    MonochromeIce,  // #E2E8F0
}

fn default_gain() -> u32 { 50 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMonitorState {
    pub brightness: u32,
    pub contrast: u32,
    pub volume: u32,
    pub is_muted: bool,
    pub black_boost: u32,
    pub selected_preset: String,
    pub overdrive: u32,
    pub blue_light: u32,
    pub aimpoint: u32,
    pub hz_counter: bool,
    pub color_temp: String,
    pub gamma: String,
    pub color_space: String,
    pub selected_input: String,
    pub monitor_name: String,
    #[serde(default = "default_gain")]
    pub red_gain: u32,
    #[serde(default = "default_gain")]
    pub green_gain: u32,
    #[serde(default = "default_gain")]
    pub blue_gain: u32,
}

impl Default for CachedMonitorState {
    fn default() -> Self {
        Self {
            brightness: 80,
            contrast: 50,
            volume: 75,
            is_muted: false,
            black_boost: 5,
            selected_preset: "User".into(),
            overdrive: 1,
            blue_light: 0,
            aimpoint: 0,
            hz_counter: false,
            color_temp: "normal".into(),
            gamma: "22".into(),
            color_space: "sRGB".into(),
            selected_input: "DP".into(),
            monitor_name: "Acer Nitro VG271U".into(),
            red_gain: 50,
            green_gain: 50,
            blue_gain: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiSettings {
    pub theme: AccentTheme,
    pub unified_hdr_bridge: bool,
    #[serde(default)]
    pub last_state: CachedMonitorState,
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            theme: AccentTheme::CyberCyan,
            unified_hdr_bridge: true,
            last_state: CachedMonitorState::default(),
        }
    }
}

impl GuiSettings {
    pub fn config_path() -> PathBuf {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(local)
                .join("Programs")
                .join("acer_monitor_cli")
                .join("gui_settings.json");
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            p
        } else if let Ok(home) = std::env::var("HOME") {
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(home).join(".config"));
            let p = config_dir.join("acer_monitor_cli").join("gui_settings.json");
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            p
        } else {
            PathBuf::from("gui_settings.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<Self>(&data) {
                return cfg;
            }
        }
        let def = Self::default();
        let _ = def.save();
        def
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }
}

impl AccentTheme {
    pub const ALL: [AccentTheme; 9] = [
        AccentTheme::CyberCyan,
        AccentTheme::NitroCrimson,
        AccentTheme::EmeraldMatrix,
        AccentTheme::AmethystPurple,
        AccentTheme::AmberSunset,
        AccentTheme::StealthGray,
        AccentTheme::SolarGold,
        AccentTheme::NeonPink,
        AccentTheme::MonochromeIce,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::CyberCyan => "Cyber Cyan",
            Self::NitroCrimson => "Nitro Crimson",
            Self::EmeraldMatrix => "Emerald Green",
            Self::AmethystPurple => "Amethyst Violet",
            Self::AmberSunset => "Amber Sunset",
            Self::StealthGray => "Stealth Gray",
            Self::SolarGold => "Solar Gold",
            Self::NeonPink => "Neon Pink",
            Self::MonochromeIce => "Monochrome Ice",
        }
    }

    pub fn primary(&self) -> Color32 {
        match self {
            Self::CyberCyan => Color32::from_rgb(56, 189, 248),
            Self::NitroCrimson => Color32::from_rgb(244, 63, 94),
            Self::EmeraldMatrix => Color32::from_rgb(16, 185, 129),
            Self::AmethystPurple => Color32::from_rgb(168, 85, 247),
            Self::AmberSunset => Color32::from_rgb(245, 158, 11),
            Self::StealthGray => Color32::from_rgb(156, 163, 175),
            Self::SolarGold => Color32::from_rgb(250, 204, 21),
            Self::NeonPink => Color32::from_rgb(236, 72, 153),
            Self::MonochromeIce => Color32::from_rgb(226, 232, 240),
        }
    }

    pub fn badge_bg(&self) -> Color32 {
        match self {
            Self::CyberCyan => Color32::from_rgb(14, 40, 58),
            Self::NitroCrimson => Color32::from_rgb(56, 16, 26),
            Self::EmeraldMatrix => Color32::from_rgb(12, 46, 32),
            Self::AmethystPurple => Color32::from_rgb(44, 18, 62),
            Self::AmberSunset => Color32::from_rgb(54, 36, 12),
            Self::StealthGray => Color32::from_rgb(36, 42, 54),
            Self::SolarGold => Color32::from_rgb(52, 42, 12),
            Self::NeonPink => Color32::from_rgb(56, 16, 38),
            Self::MonochromeIce => Color32::from_rgb(34, 40, 52),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::CyberCyan => Self::NitroCrimson,
            Self::NitroCrimson => Self::EmeraldMatrix,
            Self::EmeraldMatrix => Self::AmethystPurple,
            Self::AmethystPurple => Self::AmberSunset,
            Self::AmberSunset => Self::StealthGray,
            Self::StealthGray => Self::SolarGold,
            Self::SolarGold => Self::NeonPink,
            Self::NeonPink => Self::MonochromeIce,
            Self::MonochromeIce => Self::CyberCyan,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MonitorStateUpdate {
    pub sync_id: Option<u64>,
    pub brightness: Option<u32>,
    pub contrast: Option<u32>,
    pub volume: Option<u32>,
    pub is_muted: Option<bool>,
    pub black_boost: Option<u32>,
    pub is_hdr_active: Option<bool>,
    pub selected_preset: Option<String>,
    pub overdrive: Option<u32>,
    pub blue_light: Option<u32>,
    pub aimpoint: Option<u32>,
    pub hz_counter: Option<bool>,
    pub color_temp: Option<String>,
    pub gamma: Option<String>,
    pub color_space: Option<String>,
    pub input: Option<String>,
    pub monitor_name: Option<String>,
    pub red_gain: Option<u32>,
    pub green_gain: Option<u32>,
    pub blue_gain: Option<u32>,
    pub status: Option<String>,
}

fn map_display_mode(dm: u32) -> &'static str {
    match dm {
        0 => "User",
        1 => "Standard",
        2 => "ECO Saver",
        3 => "Graphics",
        5 => "Action",
        6 => "Racing",
        7 => "Sports",
        11 => "HDR Game",
        _ => "Standard",
    }
}

fn map_input(input_val: u32) -> &'static str {
    match input_val {
        0x0F => "DP",
        0x11 => "HDMI 1",
        0x12 => "HDMI 2",
        _ => "AUTO",
    }
}

fn probe_hardware_state() -> MonitorStateUpdate {
    let mut update = MonitorStateUpdate::default();

    // 1. Windows OS Native HDR state
    let is_hdr = crate::hdr::get_os_hdr();
    update.is_hdr_active = Some(is_hdr);

    // 2. Direct DDC/CI Hardware registers
    if let Ok(mut set) = crate::monitor::MonitorSet::enumerate() {
        if let Some(mon) = set.monitors_mut().first_mut() {
            update.monitor_name = Some(mon.description.clone());

            let mut is_hardware_hdr = false;
            if let Ok((dm, _)) = mon.get_vcp(0xE2) {
                let preset_name = map_display_mode(dm);
                let mon_is_hdr = dm == 11 || preset_name.eq_ignore_ascii_case("HDR") || preset_name.eq_ignore_ascii_case("HDR Game");
                is_hardware_hdr = mon_is_hdr;
                let active_hdr = mon_is_hdr || is_hdr;
                update.selected_preset = Some(if active_hdr { "HDR Game".to_string() } else { preset_name.to_string() });
                update.is_hdr_active = Some(active_hdr);
            }

            let mut b_val: Option<u32> = None;
            for _ in 0..3 {
                if let Ok((b, _)) = mon.get_vcp(0x10) {
                    b_val = Some(b);
                    break;
                }
                std::thread::sleep(Duration::from_millis(80));
            }

            if is_hardware_hdr || is_hdr {
                if let Some(sdr_b) = crate::hdr::get_sdr_white_level() {
                    update.brightness = Some(sdr_b);
                }
            } else if let Some(b) = b_val {
                update.brightness = Some(b);
            }

            if let Ok((c, _)) = mon.get_vcp(0x12) {
                update.contrast = Some(c);
            }
            if let Ok((v, _)) = mon.get_vcp(0x62) {
                update.volume = Some(v);
            }
            if let Ok((m, _)) = mon.get_vcp(0x8D) {
                update.is_muted = Some(m == 1);
            }
            if let Ok((bb, _)) = mon.get_vcp(0xE5) {
                update.black_boost = Some(bb);
            }
            if let Ok((od, _)) = crate::acer::get_overdrive(mon) {
                update.overdrive = Some(od);
            }
            if let Ok((bl, _)) = crate::acer::get_blue_light(mon) {
                update.blue_light = Some(bl);
            }
            if let Ok((ct, _)) = crate::acer::get_color_temp(mon) {
                update.color_temp = Some(match ct {
                    0 => "warm",
                    1 => "normal",
                    2 => "cool",
                    3 => "bluelight",
                    4 => "user",
                    _ => "normal",
                }.to_string());
            }
            if let Ok((gm, _)) = crate::acer::get_gamma(mon) {
                update.gamma = Some(match gm {
                    0 => "18",
                    1 => "20",
                    2 => "22",
                    3 => "24",
                    4 => "26",
                    _ => "22",
                }.to_string());
            }
            if let Ok((inp, _)) = mon.get_vcp(0x60) {
                update.input = Some(map_input(inp).to_string());
            }
            if let Ok((r, _)) = mon.get_vcp(0x16) {
                update.red_gain = Some(r);
            }
            if let Ok((g, _)) = mon.get_vcp(0x18) {
                update.green_gain = Some(g);
            }
            if let Ok((b, _)) = mon.get_vcp(0x1A) {
                update.blue_gain = Some(b);
            }
            update.status = Some(format!("Synced with {}", mon.description));
        }
    }

    update
}

pub struct AcerQuickSettingsApp {
    pub selected_tab: Tab,
    pub theme: AccentTheme,
    pub brightness: u32,
    pub contrast: u32,
    pub volume: u32,
    pub is_muted: bool,
    pub black_boost: u32,
    pub selected_preset: String,
    pub is_hdr_active: bool,
    pub unified_hdr_bridge: bool,
    pub hotkeys_enabled: bool,
    pub hotkey_config: crate::hotkeys::HotkeyConfig,
    pub editing_hotkeys: bool,
    pub overdrive: u32,
    pub blue_light: u32,
    pub aimpoint: u32,
    pub hz_counter: bool,
    pub color_temp: String,
    pub gamma: String,
    pub color_space: String,
    pub selected_input: String,
    pub monitor_name: String,
    pub red_gain: u32,
    pub green_gain: u32,
    pub blue_gain: u32,
    pub status_text: String,

    // Async worker channels
    tx_cmd: Sender<(Vec<String>, Option<u64>)>,
    rx_status: Receiver<String>,
    rx_state: Receiver<MonitorStateUpdate>,

    // Debouncing
    last_b_change: Option<(Instant, u32)>,
    last_c_change: Option<(Instant, u32)>,
    last_v_change: Option<(Instant, u32)>,
    last_bb_change: Option<(Instant, u32)>,
    last_rg_change: Option<(Instant, u32)>,
    last_gg_change: Option<(Instant, u32)>,
    last_bg_change: Option<(Instant, u32)>,
    pub is_syncing: bool,
    sync_started_at: Option<Instant>,
    sync_counter: u64,
    pending_sync_id: Option<u64>,
    toast_message: Option<(String, Instant)>,

    // Oscillation Prevention Timestamps
    last_user_b_edit: Instant,
    last_user_c_edit: Instant,
    last_user_v_edit: Instant,
    last_user_bb_edit: Instant,
    last_user_preset_edit: Instant,
    last_user_rg_edit: Instant,
    last_user_gg_edit: Instant,
    last_user_bg_edit: Instant,
    created_at: Instant,
    has_been_focused: bool,
    pub is_pinned: bool,
    pub report_modal: Option<(String, String)>,
    rx_report: Receiver<(String, String)>,
}

impl AcerQuickSettingsApp {
    fn show_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), Instant::now()));
    }

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        #[cfg(windows)]
        {
            let win_dir = std::env::var("WINDIR")
                .or_else(|_| std::env::var("SystemRoot"))
                .unwrap_or_else(|_| "C:\\Windows".to_string());
            let fonts_dir = std::path::Path::new(&win_dir).join("Fonts");
            if let Ok(segui_bytes) = std::fs::read(fonts_dir.join("seguiemj.ttf")) {
                fonts.font_data.insert(
                    "seguiemj".to_owned(),
                    egui::FontData::from_owned(segui_bytes),
                );
                if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    vec.push("seguiemj".to_owned());
                }
            }
            if let Ok(sym_bytes) = std::fs::read(fonts_dir.join("seguisym.ttf")) {
                fonts.font_data.insert(
                    "seguisym".to_owned(),
                    egui::FontData::from_owned(sym_bytes),
                );
                if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    vec.push("seguisym".to_owned());
                }
            }
        }
        _cc.egui_ctx.set_fonts(fonts);

        #[cfg(target_os = "linux")]
        {
            // Activate window immediately and set UTILITY window type to prevent GNOME "is ready" notification
            std::thread::spawn(|| {
                for _ in 0..25 {
                    std::thread::sleep(Duration::from_millis(40));
                    if let Ok(output) = std::process::Command::new("xdotool")
                        .args(&["search", "--name", "Acer Display Center"])
                        .output()
                    {
                        if output.status.success() {
                            let win_ids = String::from_utf8_lossy(&output.stdout);
                            for id_str in win_ids.lines() {
                                let id = id_str.trim();
                                if !id.is_empty() {
                                    let _ = std::process::Command::new("xprop")
                                        .args(&["-id", id, "-f", "_NET_WM_WINDOW_TYPE", "4a", "-set", "_NET_WM_WINDOW_TYPE", "_NET_WM_WINDOW_TYPE_UTILITY"])
                                        .output();
                                    let _ = std::process::Command::new("xdotool")
                                        .args(&["windowactivate", "--sync", id])
                                        .status();
                                    let _ = std::process::Command::new("xdotool")
                                        .args(&["windowraise", id])
                                        .status();
                                    return;
                                }
                            }
                        }
                    }
                }
            });
        }

        let (tx_cmd, rx_cmd) = channel::<(Vec<String>, Option<u64>)>();
        let (tx_status, rx_status) = channel::<String>();
        let (tx_state, rx_state) = channel::<MonitorStateUpdate>();
        let (tx_report, rx_report) = channel::<(String, String)>();

        let is_hdr = crate::hdr::get_os_hdr();

        let tx_status_worker = tx_status.clone();
        let tx_state_worker = tx_state.clone();
        let tx_report_worker = tx_report.clone();
        std::thread::spawn(move || {
            while let Ok((args, maybe_id)) = rx_cmd.recv() {
                if args.first().map(|s| s.as_str()) == Some("refresh_state") {
                    let mut update = probe_hardware_state();
                    update.sync_id = maybe_id;
                    let _ = tx_state_worker.send(update);
                    let _ = tx_status_worker.send("Hardware state refreshed".into());
                    continue;
                }

                let first_arg = args.first().cloned().unwrap_or_default();
                let is_report_cmd = matches!(first_arg.as_str(), "diag" | "energy" | "edid" | "caps" | "info" | "scan");
                let is_mode_change_cmd = matches!(first_arg.as_str(), "preset" | "hdr" | "reset" | "od" | "bluelight" | "colortemp" | "gamma" | "colorspace" | "input" | "sdr" | "brightness" | "contrast" | "gain" | "rgb" | "red" | "green" | "blue");

                match crate::cli::dispatch_command(args) {
                    Ok(output) => {
                        if is_report_cmd && !output.is_empty() {
                            let title = match first_arg.as_str() {
                                "diag" => "Hardware Diagnostic Report",
                                "energy" => "Energy Consumption Calculation",
                                "edid" => "Connected Display EDID Dump",
                                "caps" => "Display Capabilities (VCP MCCS)",
                                "info" => "Monitor Hardware Info",
                                "scan" => "VCP Register Range Scan",
                                _ => "Output Report",
                            };
                            let _ = tx_report_worker.send((title.into(), output));
                        }
                        let _ = tx_status_worker.send("OK".into());

                        // Automatically re-probe and sync hardware state after mode changes!
                        if is_mode_change_cmd {
                            let delay_ms = if matches!(first_arg.as_str(), "preset" | "hdr" | "reset") { 1500 } else { 150 };
                            std::thread::sleep(Duration::from_millis(delay_ms));
                            let mut update = probe_hardware_state();
                            update.sync_id = maybe_id;
                            let _ = tx_state_worker.send(update);
                        } else if let Some(id) = maybe_id {
                            let mut update = MonitorStateUpdate::default();
                            update.sync_id = Some(id);
                            let _ = tx_state_worker.send(update);
                        }
                    }
                    Err(e) => {
                        let _ = tx_status_worker.send(format!("Error: {e}"));
                        if let Some(id) = maybe_id {
                            let mut update = MonitorStateUpdate::default();
                            update.sync_id = Some(id);
                            let _ = tx_state_worker.send(update);
                        }
                    }
                }
            }
        });

        // Fast background hardware probe (0ms UI startup lag!)
        let tx_state_periodic = tx_state.clone();
        std::thread::spawn(move || {
            let first_update = probe_hardware_state();
            let _ = tx_state_periodic.send(first_update);

            loop {
                std::thread::sleep(Duration::from_secs(10));
                let update = probe_hardware_state();
                if tx_state_periodic.send(update).is_err() {
                    break;
                }
            }
        });

        let past = Instant::now() - Duration::from_secs(10);
        let settings = GuiSettings::load();
        let initial_brightness = if is_hdr {
            crate::hdr::get_sdr_white_level().unwrap_or(settings.last_state.brightness)
        } else {
            settings.last_state.brightness
        };
        let initial_preset = if is_hdr {
            "✨ HDR Game".to_string()
        } else if settings.last_state.selected_preset.contains("HDR") {
            "⚡ Standard".to_string()
        } else {
            settings.last_state.selected_preset
        };
        Self {
            selected_tab: Tab::Display,
            theme: settings.theme,
            brightness: initial_brightness,
            contrast: settings.last_state.contrast,
            volume: settings.last_state.volume,
            is_muted: settings.last_state.is_muted,
            black_boost: settings.last_state.black_boost,
            selected_preset: initial_preset,
            is_hdr_active: is_hdr,
            unified_hdr_bridge: settings.unified_hdr_bridge,
            hotkeys_enabled: true,
            hotkey_config: crate::hotkeys::HotkeyConfig::load(),
            editing_hotkeys: false,
            overdrive: settings.last_state.overdrive,
            blue_light: settings.last_state.blue_light,
            aimpoint: settings.last_state.aimpoint,
            hz_counter: settings.last_state.hz_counter,
            color_temp: settings.last_state.color_temp,
            gamma: settings.last_state.gamma,
            color_space: settings.last_state.color_space,
            selected_input: settings.last_state.selected_input,
            monitor_name: settings.last_state.monitor_name,
            red_gain: settings.last_state.red_gain,
            green_gain: settings.last_state.green_gain,
            blue_gain: settings.last_state.blue_gain,
            status_text: "Connected via DDC/CI".into(),
            tx_cmd,
            rx_status,
            rx_state,
            last_b_change: None,
            last_c_change: None,
            last_v_change: None,
            last_bb_change: None,
            last_rg_change: None,
            last_gg_change: None,
            last_bg_change: None,
            is_syncing: false,
            sync_started_at: None,
            sync_counter: 0,
            pending_sync_id: None,
            toast_message: None,
            last_user_b_edit: past,
            last_user_c_edit: past,
            last_user_v_edit: past,
            last_user_bb_edit: past,
            last_user_preset_edit: past,
            last_user_rg_edit: past,
            last_user_gg_edit: past,
            last_user_bg_edit: past,
            created_at: Instant::now(),
            has_been_focused: false,
            is_pinned: false,
            report_modal: None,
            rx_report,
        }
    }

    fn save_settings(&self) {
        let settings = GuiSettings {
            theme: self.theme,
            unified_hdr_bridge: self.unified_hdr_bridge,
            last_state: CachedMonitorState {
                brightness: self.brightness,
                contrast: self.contrast,
                volume: self.volume,
                is_muted: self.is_muted,
                black_boost: self.black_boost,
                selected_preset: self.selected_preset.clone(),
                overdrive: self.overdrive,
                blue_light: self.blue_light,
                aimpoint: self.aimpoint,
                hz_counter: self.hz_counter,
                color_temp: self.color_temp.clone(),
                gamma: self.gamma.clone(),
                color_space: self.color_space.clone(),
                selected_input: self.selected_input.clone(),
                monitor_name: self.monitor_name.clone(),
                red_gain: self.red_gain,
                green_gain: self.green_gain,
                blue_gain: self.blue_gain,
            },
        };
        let _ = settings.save();
    }

    fn send_cmd(&mut self, args: &[&str]) {
        let first_arg = args.first().copied().unwrap_or_default();
        let is_sync_trigger = matches!(first_arg, "preset" | "hdr" | "reset" | "od" | "bluelight" | "colortemp" | "gamma" | "colorspace" | "input" | "sync" | "brightness" | "contrast" | "sdr" | "gain" | "rgb" | "red" | "green" | "blue");
        let id = if is_sync_trigger {
            self.sync_counter += 1;
            let cur_id = self.sync_counter;
            self.pending_sync_id = Some(cur_id);
            self.is_syncing = true;
            self.sync_started_at = Some(Instant::now());
            Some(cur_id)
        } else {
            None
        };
        let vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let _ = self.tx_cmd.send((vec, id));
    }

    fn refresh_hardware(&mut self) {
        self.sync_counter += 1;
        let id = self.sync_counter;
        self.pending_sync_id = Some(id);
        self.is_syncing = true;
        self.sync_started_at = Some(Instant::now());
        let past = Instant::now() - Duration::from_secs(10);
        self.last_user_b_edit = past;
        self.last_user_c_edit = past;
        self.last_user_v_edit = past;
        self.last_user_bb_edit = past;
        self.last_user_preset_edit = past;
        let _ = self.tx_cmd.send((vec!["refresh_state".into()], Some(id)));
    }
}

impl eframe::App for AcerQuickSettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(16));

        // Track when window actually gains focus
        if ctx.input(|i| i.viewport().focused == Some(true)) {
            self.has_been_focused = true;
        }

        // 1. Dismiss on Escape Key
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // 2. Auto-dismiss on outside click / focus loss ONLY if not pinned, has been focused, and after 1500ms grace period
        if !self.is_pinned && self.has_been_focused && self.created_at.elapsed() > Duration::from_millis(1500) {
            let lost_focus = ctx.input(|i| i.viewport().focused == Some(false));
            if lost_focus {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }

            #[cfg(windows)]
            {
                use std::os::windows::ffi::OsStrExt;
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
                use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, GetCursorPos, GetWindowRect};
                use windows_sys::Win32::Foundation::{POINT, RECT};

                let l_down = (unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } as u16 & 0x8000) != 0;
                let r_down = (unsafe { GetAsyncKeyState(VK_RBUTTON as i32) } as u16 & 0x8000) != 0;

                if l_down || r_down {
                    unsafe {
                        let mut pt: POINT = std::mem::zeroed();
                        GetCursorPos(&mut pt);
                        let title: Vec<u16> = std::ffi::OsStr::new("Acer Display Center")
                            .encode_wide()
                            .chain(Some(0))
                            .collect();
                        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
                        if hwnd != 0 as _ {
                            let mut rect: RECT = std::mem::zeroed();
                            GetWindowRect(hwnd, &mut rect);
                            let inside_gui = pt.x >= rect.left && pt.x <= rect.right && pt.y >= rect.top && pt.y <= rect.bottom;
                            if !inside_gui {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                return;
                            }
                        }
                    }
                }
            }
        }

        let now = Instant::now();

        while let Ok(msg) = self.rx_status.try_recv() {
            self.status_text = msg;
        }

        while let Ok((title, body)) = self.rx_report.try_recv() {
            self.report_modal = Some((title, body));
        }

        // Safety timeout for sync spinner (max 6s)
        if let Some(t) = self.sync_started_at {
            if t.elapsed() > Duration::from_secs(6) {
                self.is_syncing = false;
                self.sync_started_at = None;
            }
        }

        let mut had_state_change = false;
        while let Ok(st) = self.rx_state.try_recv() {
            had_state_change = true;
            let is_sync_reply = st.sync_id.is_some();
            if let Some(b) = st.brightness {
                if (is_sync_reply || now.duration_since(self.last_user_b_edit) > Duration::from_millis(600)) && self.last_b_change.is_none() {
                    self.brightness = b;
                }
            }
            if let Some(c) = st.contrast {
                if (is_sync_reply || now.duration_since(self.last_user_c_edit) > Duration::from_millis(600)) && self.last_c_change.is_none() {
                    self.contrast = c;
                }
            }
            if let Some(v) = st.volume {
                if (is_sync_reply || now.duration_since(self.last_user_v_edit) > Duration::from_millis(600)) && self.last_v_change.is_none() {
                    self.volume = v;
                }
            }
            if let Some(m) = st.is_muted { self.is_muted = m; }
            if let Some(bb) = st.black_boost {
                if (is_sync_reply || now.duration_since(self.last_user_bb_edit) > Duration::from_millis(600)) && self.last_bb_change.is_none() {
                    self.black_boost = bb;
                }
            }
            if let Some(hdr) = st.is_hdr_active {
                if is_sync_reply || now.duration_since(self.last_user_preset_edit) > Duration::from_millis(600) {
                    self.is_hdr_active = hdr;
                }
            }
            if let Some(p) = st.selected_preset {
                if is_sync_reply || now.duration_since(self.last_user_preset_edit) > Duration::from_millis(600) {
                    self.selected_preset = p;
                }
            }
            if let Some(od) = st.overdrive { self.overdrive = od; }
            if let Some(bl) = st.blue_light { self.blue_light = bl; }
            if let Some(ct) = st.color_temp { self.color_temp = ct; }
            if let Some(gm) = st.gamma { self.gamma = gm; }
            if let Some(cs) = st.color_space { self.color_space = cs; }
            if let Some(inp) = st.input { self.selected_input = inp; }
            if let Some(name) = st.monitor_name { self.monitor_name = name; }
            if let Some(r) = st.red_gain {
                if (is_sync_reply || now.duration_since(self.last_user_rg_edit) > Duration::from_millis(600)) && self.last_rg_change.is_none() {
                    self.red_gain = r;
                }
            }
            if let Some(g) = st.green_gain {
                if (is_sync_reply || now.duration_since(self.last_user_gg_edit) > Duration::from_millis(600)) && self.last_gg_change.is_none() {
                    self.green_gain = g;
                }
            }
            if let Some(b) = st.blue_gain {
                if (is_sync_reply || now.duration_since(self.last_user_bg_edit) > Duration::from_millis(600)) && self.last_bg_change.is_none() {
                    self.blue_gain = b;
                }
            }
            if let Some(stat) = st.status { self.status_text = stat; }

            // Clear is_syncing only AFTER applying the state
            if let Some(id) = st.sync_id {
                if self.pending_sync_id == Some(id) || self.pending_sync_id.map(|p| id >= p).unwrap_or(false) {
                    self.is_syncing = false;
                    self.pending_sync_id = None;
                }
            }
        }

        if had_state_change {
            self.save_settings();
        }

        // Debounced Sliders
        if let Some((time, val)) = self.last_b_change {
            if now.duration_since(time) >= Duration::from_millis(80) {
                if self.is_hdr_active {
                    self.send_cmd(&["sdr", &val.to_string()]);
                } else {
                    self.send_cmd(&["brightness", &val.to_string()]);
                }
                self.last_b_change = None;
                self.save_settings();
            }
        }
        if let Some((time, val)) = self.last_c_change {
            if now.duration_since(time) >= Duration::from_millis(80) {
                self.send_cmd(&["contrast", &val.to_string()]);
                self.last_c_change = None;
                self.save_settings();
            }
        }
        if let Some((time, val)) = self.last_v_change {
            if now.duration_since(time) >= Duration::from_millis(80) {
                self.send_cmd(&["volume", &val.to_string()]);
                self.last_v_change = None;
                self.save_settings();
            }
        }
        if let Some((time, val)) = self.last_rg_change {
            if now.duration_since(time) >= Duration::from_millis(80) {
                self.send_cmd(&["gain", "red", &val.to_string()]);
                self.last_rg_change = None;
                self.save_settings();
            }
        }
        if let Some((time, val)) = self.last_gg_change {
            if now.duration_since(time) >= Duration::from_millis(80) {
                self.send_cmd(&["gain", "green", &val.to_string()]);
                self.last_gg_change = None;
                self.save_settings();
            }
        }
        if let Some((time, val)) = self.last_bg_change {
            if now.duration_since(time) >= Duration::from_millis(80) {
                self.send_cmd(&["gain", "blue", &val.to_string()]);
                self.last_bg_change = None;
                self.save_settings();
            }
        }
        if let Some((time, val)) = self.last_bb_change {
            if now.duration_since(time) >= Duration::from_millis(80) {
                self.send_cmd(&["bb", &val.to_string()]);
                self.last_bb_change = None;
                self.save_settings();
            }
        }

        let theme = self.theme;
        let accent = theme.primary();

        // Dark Studio Aesthetic Visuals
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(11, 13, 18);
        visuals.window_fill = Color32::from_rgb(11, 13, 18);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(18, 21, 28);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);
        visuals.widgets.hovered.rounding = Rounding::same(6.0);
        visuals.widgets.active.rounding = Rounding::same(6.0);
        visuals.selection.bg_fill = accent;
        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(11, 13, 18))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(32, 36, 48)))
                    .rounding(Rounding::same(10.0))
                    .inner_margin(Margin::symmetric(14.0, 12.0)),
            )
            .show(ctx, |ui| {
                // Header Bar with Drag handle, Sync, Theme Cycler, and Pin
                let header_rect = ui.horizontal(|ui| {
                    // Left: Logo Dot & Title
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 3.5, accent);
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new("ACER DISPLAY CENTER").strong().size(11.5).color(Color32::WHITE));
                    ui.label(egui::RichText::new("·").size(11.0).color(Color32::from_rgb(70, 75, 90)));
                    ui.label(egui::RichText::new("VG271U").size(10.5).color(Color32::from_rgb(148, 163, 184)));

                    // Right: Actions Cluster (Right-to-Left)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 1. Pin [ 📌 ] Button (Prevents auto-dismiss)
                        let (pin_rect, pin_resp) = ui.allocate_exact_size(Vec2::new(24.0, 22.0), egui::Sense::click());
                        let is_pin_hov = pin_resp.hovered();
                        let pin_bg = if self.is_pinned { theme.badge_bg() } else if is_pin_hov { Color32::from_rgb(34, 40, 56) } else { Color32::from_rgb(22, 25, 34) };
                        let pin_stroke = if self.is_pinned { Stroke::new(1.0, accent) } else { Stroke::new(1.0, Color32::from_rgb(38, 44, 58)) };
                        ui.painter().rect_filled(pin_rect, Rounding::same(4.0), pin_bg);
                        ui.painter().rect_stroke(pin_rect, Rounding::same(4.0), pin_stroke);
                        ui.painter().text(
                            pin_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "📌",
                            egui::FontId::proportional(10.5),
                            if self.is_pinned { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) },
                        );
                        if pin_resp.clicked() {
                            self.is_pinned = !self.is_pinned;
                            self.save_settings();
                            if self.is_pinned {
                                self.show_toast("Window Pinned (Stays Open)");
                            } else {
                                self.show_toast("Window Unpinned (Auto-Dismiss Enabled)");
                            }
                        }

                        // 2. Theme Button with Active Color Circle
                        let (th_rect, th_resp) = ui.allocate_exact_size(Vec2::new(60.0, 22.0), egui::Sense::click());
                        let is_th_hov = th_resp.hovered();
                        let th_bg = if is_th_hov { Color32::from_rgb(28, 34, 48) } else { Color32::from_rgb(22, 25, 34) };
                        ui.painter().rect_filled(th_rect, Rounding::same(4.0), th_bg);
                        ui.painter().rect_stroke(th_rect, Rounding::same(4.0), Stroke::new(1.0, Color32::from_rgb(38, 44, 58)));
                        ui.painter().circle_filled(Pos2::new(th_rect.left() + 9.0, th_rect.center().y), 3.0, accent);
                        ui.painter().text(
                            Pos2::new(th_rect.left() + 17.0, th_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Theme",
                            egui::FontId::proportional(10.0),
                            Color32::WHITE,
                        );
                        if th_resp.clicked() {
                            self.theme = self.theme.next();
                            self.save_settings();
                            self.show_toast(format!("Theme: {}", self.theme.name()));
                        }

                        // 3. Sync Button with Live Dynamic Spinner (Runs for >= 2.2s so user sees smooth feedback)
                        let min_sync_dur = Duration::from_millis(2200);
                        let is_animating_sync = self.is_syncing || self.sync_started_at.map(|t| t.elapsed() < min_sync_dur).unwrap_or(false);
                        if is_animating_sync {
                            ctx.request_repaint();
                        } else {
                            self.sync_started_at = None;
                        }

                        let (sync_rect, sync_resp) = ui.allocate_exact_size(
                            Vec2::new(if is_animating_sync { 62.0 } else { 48.0 }, 22.0),
                            egui::Sense::click(),
                        );
                        let is_sync_hov = sync_resp.hovered();
                        let sync_bg = if is_animating_sync {
                            Color32::from_rgba_unmultiplied(accent.r() / 6, accent.g() / 6, accent.b() / 6, 255)
                        } else if is_sync_hov {
                            Color32::from_rgb(22, 32, 48)
                        } else {
                            Color32::from_rgb(16, 24, 36)
                        };
                        let sync_stroke = if is_animating_sync {
                            Stroke::new(1.2, accent)
                        } else if is_sync_hov {
                            Stroke::new(1.0, accent)
                        } else {
                            Stroke::new(1.0, Color32::from_rgb(40, 52, 72))
                        };

                        ui.painter().rect_filled(sync_rect, Rounding::same(4.0), sync_bg);
                        ui.painter().rect_stroke(sync_rect, Rounding::same(4.0), sync_stroke);

                        if is_animating_sync {
                            let angle = (ui.input(|i| i.time) * 10.0) as f32;
                            let center = sync_rect.left_center() + Vec2::new(10.0, 0.0);
                            let r = 4.0;
                            ui.painter().circle_stroke(center, r, Stroke::new(1.2, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 100)));
                            let p1 = center + Vec2::new(angle.cos() * r, angle.sin() * r);
                            let p2 = center + Vec2::new((angle + 1.8).cos() * r, (angle + 1.8).sin() * r);
                            ui.painter().line_segment([p1, p2], Stroke::new(1.8, accent));
                            ui.painter().circle_filled(p1, 1.8, Color32::WHITE);

                            ui.painter().text(
                                Pos2::new(sync_rect.left() + 20.0, sync_rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                "Syncing",
                                egui::FontId::proportional(9.5),
                                accent,
                            );
                        } else {
                            ui.painter().text(
                                sync_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Sync",
                                egui::FontId::proportional(10.0),
                                if is_sync_hov { accent } else { Color32::from_rgb(200, 215, 235) },
                            );
                        }

                        if sync_resp.clicked() && !is_animating_sync {
                            self.refresh_hardware();
                            self.show_toast("Refreshing monitor state...");
                        }
                    });
                }).response.rect;

                // Drag window when dragging header bar
                if ui.rect_contains_pointer(header_rect) && ui.input(|i| i.pointer.primary_down()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.add_space(8.0);

                if let Some((title, report_body)) = self.report_modal.clone() {
                    // Inline Report View inside CentralPanel (Zero window overflow)
                    ui.horizontal(|ui| {
                        let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 3.0, accent);
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new(&title).strong().size(12.0).color(Color32::WHITE));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(
                                egui::Button::new(egui::RichText::new("✕ Back").size(10.5).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(34, 40, 56))
                                    .rounding(Rounding::same(4.0))
                                    .min_size(Vec2::new(54.0, 22.0)),
                            ).clicked() {
                                self.report_modal = None;
                            }

                            if ui.add(
                                egui::Button::new(egui::RichText::new("Copy").size(10.5).color(accent))
                                    .fill(Color32::from_rgb(18, 26, 38))
                                    .stroke(Stroke::new(1.0, accent))
                                    .rounding(Rounding::same(4.0))
                                    .min_size(Vec2::new(48.0, 22.0)),
                            ).clicked() {
                                ui.output_mut(|o| o.copied_text = report_body.clone());
                                self.show_toast("Report copied to clipboard");
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    egui::Frame::none()
                        .fill(Color32::from_rgb(14, 16, 22))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(28, 34, 46)))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(ui.available_height() - 40.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(&report_body)
                                            .monospace()
                                            .size(10.0)
                                            .color(Color32::from_rgb(215, 225, 240)),
                                    );
                                });
                        });
                } else if self.editing_hotkeys {
                    self.render_hotkey_editor(ui, accent);
                } else {
                    // Pixel-Perfect Segmented Navigation Bar
                    let tabs = [
                        (Tab::Display, "☀ Display"),
                        (Tab::Gaming, "🎯 Gaming"),
                        (Tab::Color, "🎨 Color"),
                        (Tab::Tools, "🛠 Tools"),
                    ];

                    let target_tab_idx = match self.selected_tab {
                        Tab::Display => 0.0,
                        Tab::Gaming => 1.0,
                        Tab::Color => 2.0,
                        Tab::Tools => 3.0,
                    };
                    let anim_tab_idx = ctx.animate_value_with_time(egui::Id::new("tab_anim_pos"), target_tab_idx, 0.16);

                    let nav_w = ui.available_width();
                    let nav_h = 30.0;
                    let (nav_rect, _response) = ui.allocate_exact_size(Vec2::new(nav_w, nav_h), egui::Sense::hover());

                    // Nav Track Background
                    ui.painter().rect_filled(nav_rect, Rounding::same(6.0), Color32::from_rgb(15, 18, 24));
                    ui.painter().rect_stroke(nav_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgb(28, 34, 46)));

                    let tab_w = (nav_rect.width() - 8.0) / 4.0;
                    let pill_x = nav_rect.left() + 4.0 + anim_tab_idx * tab_w;
                    let pill_rect = Rect::from_min_size(Pos2::new(pill_x, nav_rect.top() + 3.0), Vec2::new(tab_w, nav_h - 6.0));

                    // Glowing Active Tab Indicator Pill
                    ui.painter().rect_filled(pill_rect, Rounding::same(4.0), theme.badge_bg());
                    ui.painter().rect_stroke(pill_rect, Rounding::same(4.0), Stroke::new(1.0, accent));

                    for (idx, (tab, name)) in tabs.into_iter().enumerate() {
                        let btn_rect = Rect::from_min_size(
                            Pos2::new(nav_rect.left() + 4.0 + (idx as f32) * tab_w, nav_rect.top() + 3.0),
                            Vec2::new(tab_w, nav_h - 6.0),
                        );
                        let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
                        let is_sel = self.selected_tab == tab;
                        let is_hov = resp.hovered();
                        let text_color = if is_sel {
                            Color32::WHITE
                        } else if is_hov {
                            Color32::from_rgb(225, 235, 250)
                        } else {
                            Color32::from_rgb(148, 163, 184)
                        };

                        let clean_name = name.replace(['\u{FE0F}', '\u{FE0E}'], "");
                        ui.painter().text(
                            btn_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            clean_name,
                            egui::FontId::proportional(11.5),
                            text_color,
                        );

                        if resp.clicked() {
                            self.selected_tab = tab;
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Content View for Selected Tab
                    match self.selected_tab {
                        Tab::Display => self.render_display_tab(ui, accent),
                        Tab::Gaming => self.render_gaming_tab(ui, accent),
                        Tab::Color => self.render_color_tab(ui, accent),
                        Tab::Tools => self.render_tools_tab(ui, accent),
                    }
                }

                // Floating Action Toast Capsule
                if let Some((msg, created)) = &self.toast_message {
                    let elapsed = created.elapsed().as_secs_f32();
                    if elapsed < 2.2 {
                        let alpha = if elapsed < 0.2 {
                            elapsed / 0.2
                        } else if elapsed > 1.8 {
                            (2.2 - elapsed) / 0.4
                        } else {
                            1.0
                        };

                        let toast_h = 26.0;
                        let toast_w = (ui.available_width() - 40.0).max(180.0);
                        let toast_rect = Rect::from_center_size(
                            Pos2::new(ui.min_rect().center().x, ui.max_rect().bottom() - 34.0),
                            Vec2::new(toast_w, toast_h),
                        );

                        let bg_color = Color32::from_rgba_unmultiplied(16, 20, 30, (245.0 * alpha) as u8);
                        let border_color = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), (220.0 * alpha) as u8);
                        let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * alpha) as u8);

                        ui.painter().rect_filled(toast_rect, Rounding::same(13.0), bg_color);
                        ui.painter().rect_stroke(toast_rect, Rounding::same(13.0), Stroke::new(1.0, border_color));
                        ui.painter().text(
                            toast_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{msg}"),
                            egui::FontId::proportional(11.0),
                            text_color,
                        );
                    }
                }

                // Bottom Live Hardware Status Bar
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        let dot_color = if self.is_hdr_active { Color32::from_rgb(34, 197, 94) } else { accent };
                        let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 3.0, dot_color);

                        let hdr_txt = if self.is_hdr_active { "HDR ACTIVE" } else { "HDR OFF" };
                        ui.label(egui::RichText::new(hdr_txt).size(9.5).color(Color32::from_rgb(148, 163, 184)));
                        ui.label(egui::RichText::new("|").size(9.5).color(Color32::from_rgb(50, 55, 68)));
                        ui.label(egui::RichText::new(format!("Preset: {}", self.selected_preset)).strong().size(9.5).color(accent));
                        ui.label(egui::RichText::new("|").size(9.5).color(Color32::from_rgb(50, 55, 68)));
                        ui.label(egui::RichText::new(&self.status_text).size(9.0).color(Color32::from_rgb(120, 130, 150)));
                    });
                    ui.separator();
                });
            });
    }
}

impl AcerQuickSettingsApp {
    fn render_card_button(
        ui: &mut egui::Ui,
        id: egui::Id,
        text: &str,
        is_selected: bool,
        size: Vec2,
        accent: Color32,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let is_hovered = response.hovered();

        let anim_val = ui.ctx().animate_value_with_time(id, if is_selected { 1.0 } else if is_hovered { 0.4 } else { 0.0 }, 0.12);

        let bg_color = if is_selected {
            Color32::from_rgba_unmultiplied(accent.r() / 6, accent.g() / 6, accent.b() / 6, 255)
        } else if is_hovered {
            Color32::from_rgb(24, 28, 38)
        } else {
            Color32::from_rgb(16, 19, 26)
        };

        let stroke_color = if is_selected {
            accent
        } else if is_hovered {
            Color32::from_rgb(60, 72, 95)
        } else {
            Color32::from_rgb(28, 34, 46)
        };

        let text_color = if is_selected {
            Color32::WHITE
        } else if is_hovered {
            Color32::from_rgb(225, 235, 250)
        } else {
            Color32::from_rgb(148, 163, 184)
        };

        let rounding = Rounding::same(5.0);
        let painter = ui.painter();
        painter.rect_filled(rect, rounding, bg_color);
        painter.rect_stroke(rect, rounding, Stroke::new(1.0 + anim_val * 0.5, stroke_color));

        let clean_text = text.replace(['\u{FE0F}', '\u{FE0E}'], "");
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            clean_text,
            egui::FontId::proportional(11.0),
            text_color,
        );

        response
    }

    fn render_custom_slider(
        ui: &mut egui::Ui,
        val: &mut u32,
        min: u32,
        max: u32,
        accent: Color32,
    ) -> bool {
        let width = ui.available_width();
        let height = 18.0;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click_and_drag());
        let mut changed = false;

        let track_h = 5.0;
        let track_rect = Rect::from_center_size(rect.center(), Vec2::new(rect.width() - 12.0, track_h));

        if resp.clicked() || resp.dragged() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let t = ((pos.x - track_rect.left()) / track_rect.width()).clamp(0.0, 1.0);
                let new_val = (min as f32 + t * (max - min) as f32).round() as u32;
                if new_val != *val {
                    *val = new_val;
                    changed = true;
                }
            }
        }

        let anim_val = if resp.dragged() {
            *val as f32
        } else {
            ui.ctx().animate_value_with_time(resp.id.with("slider_anim"), *val as f32, 0.12)
        };
        let progress = ((anim_val - min as f32) / (max - min) as f32).clamp(0.0, 1.0);
        let knob_x = track_rect.left() + progress * track_rect.width();
        let knob_center = Pos2::new(knob_x, track_rect.center().y);

        let is_hovered = resp.hovered();
        let is_dragged = resp.dragged();
        let knob_radius = if is_dragged { 7.5 } else if is_hovered { 6.5 } else { 5.5 };

        let painter = ui.painter();

        // 1. Inactive Track Background
        painter.rect_filled(track_rect, Rounding::same(2.5), Color32::from_rgb(22, 26, 36));
        painter.rect_stroke(track_rect, Rounding::same(2.5), Stroke::new(1.0, Color32::from_rgb(36, 42, 58)));

        // 2. Active Filled Track
        if progress > 0.005 {
            let fill_rect = Rect::from_min_max(
                track_rect.min,
                Pos2::new(knob_x, track_rect.max.y),
            );
            painter.rect_filled(fill_rect, Rounding::same(2.5), accent);
        }

        // 3. Glowing Interactive Knob
        if is_hovered || is_dragged {
            painter.circle_filled(knob_center, knob_radius + 4.0, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 60));
        }
        painter.circle_filled(knob_center, knob_radius, Color32::WHITE);
        painter.circle_stroke(knob_center, knob_radius, Stroke::new(1.5, accent));

        changed
    }

    fn render_hdr_card(&mut self, ui: &mut egui::Ui, accent: Color32) {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 46.0), egui::Sense::click());

        let is_hovered = resp.hovered();
        let bg_color = if self.unified_hdr_bridge {
            Color32::from_rgba_unmultiplied(accent.r() / 7, accent.g() / 7, accent.b() / 7, 255)
        } else if is_hovered {
            Color32::from_rgb(22, 26, 34)
        } else {
            Color32::from_rgb(16, 19, 26)
        };

        let stroke_color = if self.unified_hdr_bridge {
            accent
        } else if is_hovered {
            Color32::from_rgb(52, 60, 80)
        } else {
            Color32::from_rgb(28, 34, 46)
        };

        let painter = ui.painter();
        painter.rect(rect, Rounding::same(6.0), bg_color, Stroke::new(1.0, stroke_color));

        // Title and description
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 15.0),
            egui::Align2::LEFT_CENTER,
            "✨ Unified HDR Bridge",
            egui::FontId::proportional(11.5),
            Color32::WHITE,
        );
        let subtext = if self.unified_hdr_bridge {
            "ON: Presets sync BOTH Windows 11 & Monitor HDR"
        } else {
            "OFF: Presets ONLY change Monitor Hardware"
        };
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 31.0),
            egui::Align2::LEFT_CENTER,
            subtext,
            egui::FontId::proportional(9.5),
            if self.unified_hdr_bridge { accent } else { Color32::from_rgb(148, 163, 184) },
        );

        // Status Pill Badge
        let (pill_bg, pill_stroke, status_text, status_color) = if self.unified_hdr_bridge {
            (
                self.theme.badge_bg(),
                Stroke::new(1.0, accent),
                "ON",
                Color32::WHITE,
            )
        } else {
            (
                Color32::from_rgb(24, 28, 36),
                Stroke::new(1.0, Color32::from_rgb(45, 52, 68)),
                "OFF",
                Color32::from_rgb(148, 163, 184),
            )
        };

        let pill_rect = Rect::from_center_size(
            Pos2::new(rect.right() - 34.0, rect.center().y),
            Vec2::new(48.0, 22.0),
        );
        painter.rect(pill_rect, Rounding::same(4.0), pill_bg, pill_stroke);
        painter.text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            status_text,
            egui::FontId::proportional(10.5),
            status_color,
        );

        if resp.clicked() {
            self.unified_hdr_bridge = !self.unified_hdr_bridge;
            self.save_settings();
            if self.unified_hdr_bridge {
                self.show_toast("Unified HDR Bridge ON (Syncs Windows + Monitor)");
            } else {
                self.show_toast("Unified HDR Bridge OFF (Monitor Only)");
            }
        }
    }

    fn render_hotkeys_card(&mut self, ui: &mut egui::Ui, accent: Color32) {
        let card_h = 46.0;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), card_h), egui::Sense::click());
        let is_hovered = resp.hovered();

        let bg_color = if self.hotkeys_enabled {
            self.theme.badge_bg()
        } else if is_hovered {
            Color32::from_rgb(22, 26, 34)
        } else {
            Color32::from_rgb(16, 19, 26)
        };

        let stroke_color = if self.hotkeys_enabled {
            accent
        } else if is_hovered {
            Color32::from_rgb(52, 60, 80)
        } else {
            Color32::from_rgb(28, 34, 46)
        };

        let painter = ui.painter();
        painter.rect(rect, Rounding::same(6.0), bg_color, Stroke::new(1.0, stroke_color));

        // Title and description
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 15.0),
            egui::Align2::LEFT_CENTER,
            "⌨  Global Hotkeys & Shortcuts",
            egui::FontId::proportional(11.5),
            Color32::WHITE,
        );
        let subtext = if self.hotkeys_enabled {
            "ACTIVE: System-wide shortcuts enabled"
        } else {
            "DISABLED: System-wide shortcuts paused"
        };
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 31.0),
            egui::Align2::LEFT_CENTER,
            subtext,
            egui::FontId::proportional(9.5),
            if self.hotkeys_enabled { accent } else { Color32::from_rgb(148, 163, 184) },
        );

        // Status Pill Badge
        let (pill_bg, pill_stroke, status_text, status_color) = if self.hotkeys_enabled {
            (
                self.theme.badge_bg(),
                Stroke::new(1.0, accent),
                "ON",
                Color32::WHITE,
            )
        } else {
            (
                Color32::from_rgb(24, 28, 36),
                Stroke::new(1.0, Color32::from_rgb(45, 52, 68)),
                "OFF",
                Color32::from_rgb(148, 163, 184),
            )
        };

        let pill_rect = Rect::from_center_size(
            Pos2::new(rect.right() - 34.0, rect.center().y),
            Vec2::new(48.0, 22.0),
        );
        painter.rect(pill_rect, Rounding::same(4.0), pill_bg, pill_stroke);
        painter.text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            status_text,
            egui::FontId::proportional(10.5),
            status_color,
        );

        if resp.clicked() {
            self.hotkeys_enabled = !self.hotkeys_enabled;
            self.send_cmd(&["hotkeys", if self.hotkeys_enabled { "on" } else { "off" }]);
            if self.hotkeys_enabled {
                self.show_toast("Global Hotkeys Enabled");
            } else {
                self.show_toast("Global Hotkeys Disabled");
            }
        }
    }

    fn render_slider_card(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        val: &mut u32,
        max: u32,
        unit: &str,
        quick_chips: &[u32],
        on_change: &mut Option<(Instant, u32)>,
        accent: Color32,
    ) -> bool {
        let mut user_interacted = false;

        egui::Frame::none()
            .fill(Color32::from_rgb(16, 19, 26))
            .stroke(Stroke::new(1.0, Color32::from_rgb(28, 34, 46)))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(12.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(title).strong().size(11.5).color(Color32::WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}{}", *val, unit))
                                .strong()
                                .size(11.5)
                                .color(accent),
                        );
                    });
                });

                ui.add_space(2.0);

                if Self::render_custom_slider(ui, val, 0, max, accent) {
                    user_interacted = true;
                    *on_change = Some((Instant::now(), *val));
                }

                if !quick_chips.is_empty() {
                    ui.add_space(3.0);
                    let chip_w = (ui.available_width() - (quick_chips.len() as f32 - 1.0) * 4.0) / (quick_chips.len() as f32);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        for &chip in quick_chips {
                            let is_sel = *val == chip;
                            let btn_id = egui::Id::new(format!("chip_{title}_{chip}"));
                            let chip_txt = format!("{chip}{unit}");

                            if Self::render_card_button(ui, btn_id, &chip_txt, is_sel, Vec2::new(chip_w, 20.0), accent).clicked() {
                                user_interacted = true;
                                *val = chip;
                                *on_change = Some((Instant::now(), chip));
                            }
                        }
                    });
                }
            });

        user_interacted
    }

    fn render_display_tab(&mut self, ui: &mut egui::Ui, accent: Color32) {
        let b_title = if self.is_hdr_active {
            "☀ SDR Brightness (HDR)"
        } else {
            "☀ Brightness"
        };
        let mut b_val = self.brightness;
        let mut b_change = self.last_b_change;
        if self.render_slider_card(ui, b_title, &mut b_val, 100, "%", &[0, 25, 50, 75, 100], &mut b_change, accent) {
            self.last_user_b_edit = Instant::now();
        }
        self.brightness = b_val;
        self.last_b_change = b_change;

        ui.add_space(4.0);

        let mut c_val = self.contrast;
        let mut c_change = self.last_c_change;
        if self.render_slider_card(ui, "🌓 Contrast", &mut c_val, 100, "%", &[40, 50, 60, 75], &mut c_change, accent) {
            self.last_user_c_edit = Instant::now();
        }
        self.contrast = c_val;
        self.last_c_change = c_change;

        ui.add_space(4.0);

        // Volume Card with Clean Mute Badge
        let mut v_val = self.volume;
        let mut v_change = self.last_v_change;
        let mut user_v_interacted = false;

        egui::Frame::none()
            .fill(Color32::from_rgb(16, 19, 26))
            .stroke(Stroke::new(1.0, Color32::from_rgb(28, 34, 46)))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(12.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔊 Volume").strong().size(11.5).color(Color32::WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mute_txt = if self.is_muted { "MUTED" } else { "MUTE" };
                        let mute_color = if self.is_muted { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(148, 163, 184) };
                        let mute_resp = ui.add(
                            egui::Button::new(egui::RichText::new(mute_txt).size(9.5).color(mute_color))
                                .fill(Color32::from_rgb(22, 26, 36))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 60)))
                                .rounding(Rounding::same(3.0))
                                .min_size(Vec2::new(42.0, 16.0)),
                        );
                        if mute_resp.clicked() {
                            self.is_muted = !self.is_muted;
                            let cmd_val = if self.is_muted { "1" } else { "0" };
                            self.show_toast(if self.is_muted { "Monitor Audio Muted" } else { "Monitor Audio Unmuted" });
                            self.send_cmd(&["mute", cmd_val]);
                        }

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!("{}%", v_val))
                                .strong()
                                .size(11.5)
                                .color(accent),
                        );
                    });
                });

                ui.add_space(2.0);

                if Self::render_custom_slider(ui, &mut v_val, 0, 100, accent) {
                    user_v_interacted = true;
                    v_change = Some((Instant::now(), v_val));
                }
            });

        if user_v_interacted {
            self.last_user_v_edit = Instant::now();
        }
        self.volume = v_val;
        self.last_v_change = v_change;

        ui.add_space(6.0);
        ui.label(egui::RichText::new("🎮 Display Presets").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        // 4x2 Preset Grid with Clean Typography (8 Exact Hardware Presets - Movie Removed)
        let presets = [
            ("⚔ Action", "action"),
            ("⚡ Standard", "standard"),
            ("✨ HDR Game", "hdr"),
            ("🌱 ECO", "eco"),
            ("🏁 Racing", "racing"),
            ("⚽ Sports", "sports"),
            ("🎨 Graphics", "graphics"),
            ("👤 User", "user"),
        ];

        let col_w = (ui.available_width() - 12.0) / 4.0;
        egui::Grid::new("presets_grid").num_columns(4).spacing([4.0, 4.0]).show(ui, |ui| {
            for (i, &(label, cmd)) in presets.iter().enumerate() {
                let clean_sel = self.selected_preset
                    .replace(['⚔', '⚡', '✨', '🌱', '🏁', '⚽', '🎨', '👤', ' '], "")
                    .to_ascii_lowercase();
                let clean_label = label
                    .replace(['⚔', '⚡', '✨', '🌱', '🏁', '⚽', '🎨', '👤', ' '], "")
                    .to_ascii_lowercase();
                let is_sel = self.selected_preset.eq_ignore_ascii_case(cmd)
                    || self.selected_preset.eq_ignore_ascii_case(label)
                    || clean_sel == cmd
                    || clean_sel == clean_label
                    || (clean_sel.contains("hdr") && cmd == "hdr")
                    || (clean_sel.contains("eco") && cmd == "eco")
                    || (clean_sel.contains("standard") && cmd == "standard")
                    || (self.is_hdr_active && cmd == "hdr");
                let btn_id = egui::Id::new(format!("preset_btn_{cmd}"));

                if Self::render_card_button(ui, btn_id, label, is_sel, Vec2::new(col_w, 26.0), accent).clicked() {
                    let is_target_hdr = cmd == "hdr" || label.eq_ignore_ascii_case("HDR Game");
                    self.selected_preset = label.into();
                    self.is_hdr_active = is_target_hdr;
                    self.last_user_preset_edit = Instant::now();
                    self.last_b_change = None;
                    self.last_c_change = None;
                    self.show_toast(format!("Applied Preset: {label}"));

                    if self.unified_hdr_bridge {
                        // Unified HDR Bridge is ON: Change BOTH Monitor Hardware & Windows OS HDR!
                        self.send_cmd(&["preset", cmd, "--unified"]);
                    } else {
                        // Unified HDR Bridge is OFF: ONLY change Monitor Hardware! Windows OS HDR untouched.
                        self.send_cmd(&["preset", cmd]);
                    }
                    self.save_settings();
                }
                if (i + 1) % 4 == 0 { ui.end_row(); }
            }
        });

        // Quick HDR Mode Card on Display Tab
        ui.add_space(5.0);
        let (hdr_rect, hdr_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), egui::Sense::click());
        let is_hdr_hov = hdr_resp.hovered();
        let (hdr_bg, hdr_stroke, hdr_title, hdr_sub, badge_txt, badge_bg, badge_col) = if self.is_hdr_active {
            (
                Color32::from_rgba_unmultiplied(accent.r() / 8, accent.g() / 8, accent.b() / 8, 255),
                Stroke::new(1.2, accent),
                "✨ High Dynamic Range (HDR)",
                "Windows 11 HDR & Monitor HDR are active",
                "HDR ON",
                self.theme.badge_bg(),
                Color32::WHITE,
            )
        } else {
            (
                if is_hdr_hov { Color32::from_rgb(22, 26, 36) } else { Color32::from_rgb(16, 19, 26) },
                Stroke::new(1.0, if is_hdr_hov { Color32::from_rgb(60, 72, 95) } else { Color32::from_rgb(28, 34, 46) }),
                "✨ High Dynamic Range (HDR)",
                "Standard Dynamic Range (SDR) active",
                "HDR OFF",
                Color32::from_rgb(24, 28, 36),
                Color32::from_rgb(148, 163, 184),
            )
        };

        let painter = ui.painter();
        painter.rect_filled(hdr_rect, Rounding::same(6.0), hdr_bg);
        painter.rect_stroke(hdr_rect, Rounding::same(6.0), hdr_stroke);
        painter.text(
            Pos2::new(hdr_rect.left() + 10.0, hdr_rect.top() + 11.0),
            egui::Align2::LEFT_CENTER,
            hdr_title,
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );
        painter.text(
            Pos2::new(hdr_rect.left() + 10.0, hdr_rect.top() + 23.0),
            egui::Align2::LEFT_CENTER,
            hdr_sub,
            egui::FontId::proportional(9.0),
            if self.is_hdr_active { accent } else { Color32::from_rgb(148, 163, 184) },
        );

        let badge_rect = Rect::from_center_size(
            Pos2::new(hdr_rect.right() - 36.0, hdr_rect.center().y),
            Vec2::new(54.0, 20.0),
        );
        painter.rect(badge_rect, Rounding::same(4.0), badge_bg, Stroke::new(1.0, if self.is_hdr_active { accent } else { Color32::from_rgb(45, 52, 68) }));
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge_txt,
            egui::FontId::proportional(10.0),
            badge_col,
        );

        if hdr_resp.clicked() {
            let next_hdr = !self.is_hdr_active;
            self.is_hdr_active = next_hdr;
            if next_hdr {
                self.selected_preset = "✨ HDR Game".into();
                self.brightness = 100;
                self.show_toast("Enabling HDR (Windows OS + Display)...");
                self.send_cmd(&["hdr", "both", "on"]);
            } else {
                self.selected_preset = "⚡ Standard".into();
                self.brightness = 80;
                self.show_toast("Disabling HDR (Standard SDR Mode)...");
                self.send_cmd(&["hdr", "both", "off"]);
                self.send_cmd(&["brightness", "80"]);
            }
            self.save_settings();
            self.refresh_hardware();
        }

        ui.add_space(6.0);
        ui.label(egui::RichText::new("🔌 Input Source").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        let inputs = [("🔌 DP", "dp"), ("📺 HDMI 1", "hdmi1"), ("📺 HDMI 2", "hdmi2"), ("🔄 AUTO", "auto"), ("⏭ NEXT", "next")];
        let inp_w = (ui.available_width() - 16.0) / 5.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for &(label, cmd) in &inputs {
                let is_sel = self.selected_input.eq_ignore_ascii_case(cmd)
                    || self.selected_input.eq_ignore_ascii_case(label);
                let btn_id = egui::Id::new(format!("inp_btn_{cmd}"));

                if Self::render_card_button(ui, btn_id, label, is_sel, Vec2::new(inp_w, 24.0), accent).clicked() {
                    self.selected_input = label.into();
                    self.show_toast(format!("Input switched to {label}"));
                    self.send_cmd(&["input", cmd]);
                }
            }
        });
    }

    fn render_gaming_tab(&mut self, ui: &mut egui::Ui, accent: Color32) {
        ui.label(egui::RichText::new("⚡  Response Time (OverDrive)").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        let ods = [("Off", 0, "off"), ("⚡ Normal", 1, "normal"), ("🔥 Extreme", 2, "extreme")];
        let od_w = (ui.available_width() - 8.0) / 3.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for &(name, val, cmd) in &ods {
                let is_sel = self.overdrive == val;
                let btn_id = egui::Id::new(format!("od_btn_{val}"));

                if Self::render_card_button(ui, btn_id, name, is_sel, Vec2::new(od_w, 26.0), accent).clicked() {
                    self.overdrive = val;
                    self.show_toast(format!("OverDrive: {name}"));
                    self.send_cmd(&["od", cmd]);
                }
            }
        });

        ui.add_space(5.0);

        let mut bb_val = self.black_boost;
        let mut bb_change = self.last_bb_change;
        if self.render_slider_card(ui, "🌑  Black Boost (Shadow Enhancer)", &mut bb_val, 10, "", &[0, 3, 5, 8, 10], &mut bb_change, accent) {
            self.last_user_bb_edit = Instant::now();
        }
        self.black_boost = bb_val;
        self.last_bb_change = bb_change;

        ui.add_space(5.0);
        ui.label(egui::RichText::new("🎯  AimPoint Hardware Crosshair").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        let aims = [("Off", 0), ("🔴 Dot", 1), ("✚ Cross", 2), ("▲ Triangle", 3)];
        let aim_w = (ui.available_width() - 12.0) / 4.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for &(name, val) in &aims {
                let is_sel = self.aimpoint == val;
                let btn_id = egui::Id::new(format!("aim_btn_{val}"));

                if Self::render_card_button(ui, btn_id, name, is_sel, Vec2::new(aim_w, 24.0), accent).clicked() {
                    self.aimpoint = val;
                    self.show_toast(format!("AimPoint: {name}"));
                    self.send_cmd(&["aim", &val.to_string()]);
                }
            }
        });

        ui.add_space(5.0);
        ui.label(egui::RichText::new("📊  Refresh Rate (Hz) OSD Counter").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        let hz_w = (ui.available_width() - 4.0) / 2.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let on_id = egui::Id::new("hz_on");
            let off_id = egui::Id::new("hz_off");

            if Self::render_card_button(ui, on_id, "Show FPS / Hz HUD", self.hz_counter, Vec2::new(hz_w, 26.0), accent).clicked() {
                self.hz_counter = true;
                self.show_toast("Refresh Rate HUD Enabled");
                self.send_cmd(&["refreshnum", "1"]);
            }
            if Self::render_card_button(ui, off_id, "Hide FPS HUD", !self.hz_counter, Vec2::new(hz_w, 26.0), accent).clicked() {
                self.hz_counter = false;
                self.show_toast("Refresh Rate HUD Disabled");
                self.send_cmd(&["refreshnum", "0"]);
            }
        });
    }

    fn render_color_tab(&mut self, ui: &mut egui::Ui, accent: Color32) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("🌡  Color Temperature").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(2.0);

                let temps = [
                    ("Warm (4000K)", "warm"),
                    ("Normal (6500K)", "normal"),
                    ("Cool (9300K)", "cool"),
                    ("BlueLight", "bluelight"),
                    ("User", "user"),
                ];
                let ct_w = (ui.available_width() - 16.0) / 5.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for &(name, cmd) in &temps {
                        let is_sel = self.color_temp.eq_ignore_ascii_case(cmd);
                        let short_name = name.split(' ').next().unwrap_or(name);
                        let btn_id = egui::Id::new(format!("ct_btn_{cmd}"));

                        if Self::render_card_button(ui, btn_id, short_name, is_sel, Vec2::new(ct_w, 24.0), accent).clicked() {
                            self.color_temp = cmd.into();
                            self.show_toast(format!("Color Temp set to {name}"));
                            self.send_cmd(&["colortemp", cmd]);
                        }
                    }
                });

                ui.add_space(5.0);
                ui.label(egui::RichText::new("🛡  Blue Light Eye Shield").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(2.0);

                let bls = [("Off", 0), ("50%", 1), ("60%", 2), ("70%", 3), ("80%", 4)];
                let bl_w = (ui.available_width() - 16.0) / 5.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for &(name, val) in &bls {
                        let is_sel = self.blue_light == val;
                        let btn_id = egui::Id::new(format!("bl_btn_{val}"));

                        if Self::render_card_button(ui, btn_id, name, is_sel, Vec2::new(bl_w, 24.0), accent).clicked() {
                            self.blue_light = val;
                            self.show_toast(format!("Blue Light: {name}"));
                            self.send_cmd(&["bluelight", &val.to_string()]);
                        }
                    }
                });

                ui.add_space(5.0);
                ui.label(egui::RichText::new("📐  Gamma Curve").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(2.0);

                let gammas = [("1.8", "18"), ("2.0", "20"), ("2.2 (Std)", "22"), ("2.4", "24"), ("2.6", "26")];
                let gm_w = (ui.available_width() - 16.0) / 5.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for &(name, cmd) in &gammas {
                        let is_sel = self.gamma == cmd;
                        let btn_id = egui::Id::new(format!("gm_btn_{cmd}"));

                        if Self::render_card_button(ui, btn_id, name, is_sel, Vec2::new(gm_w, 24.0), accent).clicked() {
                            self.gamma = cmd.into();
                            self.show_toast(format!("Gamma Curve set to {name}"));
                            self.send_cmd(&["gamma", cmd]);
                        }
                    }
                });

                ui.add_space(5.0);
                ui.label(egui::RichText::new("🎨  Color Space Profile").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(2.0);

                let spaces = [
                    ("sRGB", "0", "0"),
                    ("Rec.709", "0", "1"),
                    ("HDR", "0", "2"),
                    ("EBU", "0", "3"),
                    ("DCI-P3", "0", "4"),
                    ("General", "0", "6"),
                ];
                let sp_w = (ui.available_width() - 8.0) / 3.0;
                egui::Grid::new("color_space_grid").num_columns(3).spacing([4.0, 4.0]).show(ui, |ui| {
                    for (i, &(name, cal, sp)) in spaces.iter().enumerate() {
                        let is_sel = self.color_space.eq_ignore_ascii_case(name);
                        let btn_id = egui::Id::new(format!("cs_btn_{name}"));

                        if Self::render_card_button(ui, btn_id, name, is_sel, Vec2::new(sp_w, 25.0), accent).clicked() {
                            self.color_space = name.into();
                            self.show_toast(format!("Color Space: {name}"));
                            self.send_cmd(&["colorspace", cal, sp]);
                        }
                        if (i + 1) % 3 == 0 { ui.end_row(); }
                    }
                });

                ui.add_space(7.0);

                // Hardware RGB Gain / Balance Header & Reset Button (Positioned at bottom)
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎨  Hardware RGB Gain / Balance").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let reset_btn = ui.add(
                            egui::Button::new(egui::RichText::new("↺ Reset 50/50/50").size(9.5).color(Color32::WHITE))
                                .fill(Color32::from_rgb(28, 34, 48))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 56, 76)))
                                .rounding(Rounding::same(3.0))
                                .min_size(Vec2::new(75.0, 18.0)),
                        );
                        if reset_btn.clicked() {
                            self.red_gain = 50;
                            self.green_gain = 50;
                            self.blue_gain = 50;
                            self.last_rg_change = None;
                            self.last_gg_change = None;
                            self.last_bg_change = None;
                            self.show_toast("Reset RGB Gain to 50 / 50 / 50");
                            self.send_cmd(&["gain", "reset"]);
                            self.save_settings();
                        }
                    });
                });
                ui.add_space(3.0);

                // Red Gain Slider
                let mut rg_val = self.red_gain;
                let mut rg_change = self.last_rg_change;
                let red_accent = Color32::from_rgb(239, 68, 68);
                if self.render_slider_card(ui, "🔴 Red Gain (0x16)", &mut rg_val, 100, "%", &[25, 50, 75, 100], &mut rg_change, red_accent) {
                    self.last_user_rg_edit = Instant::now();
                }
                self.red_gain = rg_val;
                self.last_rg_change = rg_change;

                ui.add_space(3.0);

                // Green Gain Slider
                let mut gg_val = self.green_gain;
                let mut gg_change = self.last_gg_change;
                let green_accent = Color32::from_rgb(34, 197, 94);
                if self.render_slider_card(ui, "🟢 Green Gain (0x18)", &mut gg_val, 100, "%", &[25, 50, 75, 100], &mut gg_change, green_accent) {
                    self.last_user_gg_edit = Instant::now();
                }
                self.green_gain = gg_val;
                self.last_gg_change = gg_change;

                ui.add_space(3.0);

                // Blue Gain Slider
                let mut bg_val = self.blue_gain;
                let mut bg_change = self.last_bg_change;
                let blue_accent = Color32::from_rgb(59, 130, 246);
                if self.render_slider_card(ui, "🔵 Blue Gain (0x1A)", &mut bg_val, 100, "%", &[25, 50, 75, 100], &mut bg_change, blue_accent) {
                    self.last_user_bg_edit = Instant::now();
                }
                self.blue_gain = bg_val;
                self.last_bg_change = bg_change;

                ui.add_space(4.0);
            });
    }

    fn render_tools_tab(&mut self, ui: &mut egui::Ui, accent: Color32) {
        self.render_hdr_card(ui, accent);
        ui.add_space(4.0);
        self.render_hotkeys_card(ui, accent);
        ui.add_space(3.0);

        let cfg_resp = ui.add(
            egui::Button::new(egui::RichText::new("⚙ Customize Shortcuts & Keys").size(10.5).color(Color32::WHITE))
                .fill(Color32::from_rgb(22, 26, 36))
                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 60)))
                .min_size(Vec2::new(ui.available_width(), 26.0))
                .rounding(Rounding::same(4.0)),
        );
        if cfg_resp.clicked() {
            self.editing_hotkeys = true;
        }

        ui.add_space(6.0);

        ui.label(egui::RichText::new("🎨  Theme Accent Color").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        let theme_w = (ui.available_width() - 8.0) / 3.0;
        egui::Grid::new("theme_grid").num_columns(3).spacing([4.0, 4.0]).show(ui, |ui| {
            for (i, &t) in AccentTheme::ALL.iter().enumerate() {
                let is_sel = self.theme == t;
                let btn_id = egui::Id::new(format!("theme_pick_{}", t.name()));

                if Self::render_card_button(ui, btn_id, t.name(), is_sel, Vec2::new(theme_w, 25.0), t.primary()).clicked() {
                    self.theme = t;
                    self.save_settings();
                    self.show_toast(format!("Theme set to {}", t.name()));
                }
                if (i + 1) % 3 == 0 { ui.end_row(); }
            }
        });

        ui.add_space(6.0);
        ui.label(egui::RichText::new("⚡  Quick Maintenance").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        let half_w = (ui.available_width() - 4.0) / 2.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let unl_id = egui::Id::new("tool_unlock");
            if Self::render_card_button(ui, unl_id, "🔓 Unlock OSD", false, Vec2::new(half_w, 25.0), accent).clicked() {
                self.show_toast("OSD Keys Unlocked");
                self.send_cmd(&["unlock"]);
            }

            let lck_id = egui::Id::new("tool_lock");
            if Self::render_card_button(ui, lck_id, "🔒 Lock OSD", false, Vec2::new(half_w, 25.0), Color32::from_rgb(245, 158, 11)).clicked() {
                self.show_toast("OSD Keys Locked");
                self.send_cmd(&["keylock", "on"]);
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let sync_id = egui::Id::new("tool_sync");
            if Self::render_card_button(ui, sync_id, "🔄 Sync Displays", false, Vec2::new(half_w, 25.0), accent).clicked() {
                self.show_toast("All Displays Synchronized");
                self.send_cmd(&["sync"]);
            }

            let pwr_id = egui::Id::new("tool_pwr_off");
            if Self::render_card_button(ui, pwr_id, "🌙 Display Off", false, Vec2::new(half_w, 25.0), Color32::from_rgb(239, 68, 68)).clicked() {
                self.show_toast("Monitor Powering Off (DDC/CI)");
                self.send_cmd(&["power", "off"]);
            }
        });

        ui.add_space(6.0);
        ui.label(egui::RichText::new("📊  Diagnostics & Factory Control").strong().size(10.5).color(Color32::from_rgb(148, 163, 184)));
        ui.add_space(2.0);

        let tri_w = (ui.available_width() - 8.0) / 3.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let diag_id = egui::Id::new("tool_diag");
            if Self::render_card_button(ui, diag_id, "💻 Diag", false, Vec2::new(tri_w, 25.0), accent).clicked() {
                self.show_toast("Querying Diagnostics...");
                self.send_cmd(&["diag"]);
            }

            let nrg_id = egui::Id::new("tool_nrg");
            if Self::render_card_button(ui, nrg_id, "⚡ Energy", false, Vec2::new(tri_w, 25.0), accent).clicked() {
                self.show_toast("Calculating Energy...");
                self.send_cmd(&["energy"]);
            }

            let reset_id = egui::Id::new("tool_reset");
            if Self::render_card_button(ui, reset_id, "⚠ Reset", false, Vec2::new(tri_w, 25.0), Color32::from_rgb(239, 68, 68)).clicked() {
                self.show_toast("Monitor Reset to Defaults");
                self.send_cmd(&["reset"]);
            }
        });
    }

    fn render_hotkey_editor(&mut self, ui: &mut egui::Ui, accent: Color32) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("⚙ Hotkeys Customizer")
                    .strong()
                    .size(12.5)
                    .color(Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("✕ Back").size(10.0).color(Color32::WHITE))
                            .fill(Color32::from_rgb(38, 44, 60))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(58, 68, 92)))
                            .rounding(Rounding::same(4.0))
                            .min_size(Vec2::new(42.0, 22.0)),
                    )
                    .clicked()
                {
                    self.editing_hotkeys = false;
                }

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("💾 Save & Apply")
                                .size(10.0)
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(accent)
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(76.0, 22.0)),
                    )
                    .clicked()
                {
                    if let Err(e) = self.hotkey_config.save() {
                        self.show_toast(format!("Save error: {e}"));
                    } else {
                        self.show_toast("Shortcuts Saved & Applied!");
                        self.editing_hotkeys = false;
                    }
                }

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("↺ Defaults")
                                .size(10.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        )
                        .fill(Color32::from_rgb(22, 26, 36))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 60)))
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(56.0, 22.0)),
                    )
                    .clicked()
                {
                    self.hotkey_config = crate::hotkeys::HotkeyConfig::default();
                    let _ = self.hotkey_config.save();
                    self.show_toast("Restored Default Shortcuts");
                }
            });
        });

        ui.add_space(2.0);

        // Add Shortcut From Scratch Button Bar
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Configure shortcuts below or create new ones from scratch:")
                    .size(9.0)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let add_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("➕ Add New Shortcut")
                            .size(10.0)
                            .color(Color32::WHITE)
                            .strong(),
                    )
                    .fill(Color32::from_rgb(16, 185, 129)) // Emerald green
                    .rounding(Rounding::same(4.0))
                    .min_size(Vec2::new(115.0, 22.0)),
                );

                if add_btn.clicked() {
                    let new_idx = self.hotkey_config.bindings.len() + 1;
                    self.hotkey_config.bindings.push(crate::hotkeys::HotkeyBinding {
                        name: format!("Custom Action {new_idx}"),
                        description: "Custom monitor shortcut".into(),
                        mod_ctrl: true,
                        mod_alt: true,
                        mod_shift: false,
                        mod_win: false,
                        key: "F1".into(),
                        command: vec!["preset".into(), "action".into()],
                    });
                    self.show_toast("New shortcut added! Configure it below.");
                }
            });
        });

        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 10.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let keys_options = [
                    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
                    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                    "UP", "DOWN", "LEFT", "RIGHT", "SPACE",
                    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"
                ];

                let mut to_remove: Option<usize> = None;

                for (i, binding) in self.hotkey_config.bindings.iter_mut().enumerate() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(16, 19, 26))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 34, 46)))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::symmetric(10.0, 7.0))
                        .show(ui, |ui| {
                            // Row 1: Action Dropdown & Delete Button
                            ui.horizontal(|ui| {
                                // Find current matching template or custom
                                let cur_label = crate::hotkeys::ACTION_TEMPLATES
                                    .iter()
                                    .find(|t| t.command.iter().map(|s| s.to_string()).collect::<Vec<_>>() == binding.command)
                                    .map(|t| t.label)
                                    .unwrap_or("Custom CLI Command");

                                egui::ComboBox::from_id_source(format!("action_combo_{i}"))
                                    .selected_text(cur_label)
                                    .width(220.0)
                                    .show_ui(ui, |ui| {
                                        for t in crate::hotkeys::ACTION_TEMPLATES {
                                            let is_sel = t.command.iter().map(|s| s.to_string()).collect::<Vec<_>>() == binding.command;
                                            if ui.selectable_label(is_sel, t.label).clicked() {
                                                binding.name = t.label.into();
                                                binding.description = t.description.into();
                                                binding.command = t.command.iter().map(|s| s.to_string()).collect();
                                            }
                                        }
                                    });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Delete button
                                    let del_btn = ui.add(
                                        egui::Button::new(egui::RichText::new("🗑").size(10.0).color(Color32::from_rgb(239, 68, 68)))
                                            .fill(Color32::from_rgb(32, 20, 24))
                                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(70, 30, 36)))
                                            .rounding(Rounding::same(3.0))
                                            .min_size(Vec2::new(20.0, 18.0)),
                                    );
                                    if del_btn.clicked() {
                                        to_remove = Some(i);
                                    }

                                    ui.label(
                                        egui::RichText::new(binding.to_display_string())
                                            .strong()
                                            .size(10.5)
                                            .color(accent),
                                    );
                                });
                            });

                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(&binding.description)
                                    .size(8.5)
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;

                                let ctrl_resp = ui.selectable_label(binding.mod_ctrl, "Ctrl");
                                if ctrl_resp.clicked() { binding.mod_ctrl = !binding.mod_ctrl; }

                                let alt_resp = ui.selectable_label(binding.mod_alt, "Alt");
                                if alt_resp.clicked() { binding.mod_alt = !binding.mod_alt; }

                                let shift_resp = ui.selectable_label(binding.mod_shift, "Shift");
                                if shift_resp.clicked() { binding.mod_shift = !binding.mod_shift; }

                                let win_resp = ui.selectable_label(binding.mod_win, "Win");
                                if win_resp.clicked() { binding.mod_win = !binding.mod_win; }

                                ui.label(egui::RichText::new("+").size(10.0).color(Color32::from_rgb(148, 163, 184)));

                                egui::ComboBox::from_id_source(format!("combo_key_{i}"))
                                    .selected_text(&binding.key)
                                    .width(65.0)
                                    .show_ui(ui, |ui| {
                                        for &k in &keys_options {
                                            ui.selectable_value(&mut binding.key, k.to_string(), k);
                                        }
                                    });
                            });
                        });
                    ui.add_space(4.0);
                }

                if let Some(del_idx) = to_remove {
                    self.hotkey_config.bindings.remove(del_idx);
                    self.show_toast("Shortcut removed");
                }
            });
    }
}

#[cfg(windows)]
fn get_tray_popup_pos(width: f32, height: f32) -> Option<Pos2> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, GetDesktopWindow,
    };
    use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC, GetDeviceCaps, LOGPIXELSX};

    unsafe {
        let mut work_area: RECT = std::mem::zeroed();
        let success = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut work_area as *mut _ as *mut _,
            0,
        );

        if success != 0 {
            let desktop = GetDesktopWindow();
            let hdc = GetDC(desktop);
            let dpi_x = if hdc != 0 as _ {
                let d = GetDeviceCaps(hdc, LOGPIXELSX as i32);
                ReleaseDC(desktop, hdc);
                d as f32
            } else {
                96.0
            };
            let scale = (dpi_x / 96.0).max(1.0);

            // Convert physical work area to egui logical points (flush with bottom-right)
            let logical_right = (work_area.right as f32) / scale;
            let logical_bottom = (work_area.bottom as f32) / scale;

            let x = (logical_right - width).max(0.0);
            let y = (logical_bottom - height).max(0.0);
            Some(Pos2::new(x, y))
        } else {
            None
        }
    }
}

#[cfg(target_os = "linux")]
fn get_tray_popup_pos(width: f32, _height: f32) -> Option<Pos2> {
    // 1. Try querying _NET_WORKAREA via xprop (accurately detects top panel height and dock)
    if let Ok(output) = std::process::Command::new("xprop")
        .args(&["-root", "_NET_WORKAREA"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(vals_str) = s.split('=').nth(1) {
                let nums: Vec<f32> = vals_str
                    .split(',')
                    .filter_map(|p| p.trim().parse::<f32>().ok())
                    .collect();
                if nums.len() >= 4 {
                    let work_x = nums[0];
                    let work_y = nums[1];
                    let work_w = nums[2];
                    let x = (work_x + work_w - width - 12.0).max(0.0);
                    let y = (work_y + 6.0).max(0.0);
                    return Some(Pos2::new(x, y));
                }
            }
        }
    }

    // 2. Try querying screen size via xrandr
    if let Ok(output) = std::process::Command::new("xrandr")
        .arg("--current")
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines() {
                if line.contains("Screen 0:") && line.contains("current") {
                    let parts: Vec<&str> = line.split("current").collect();
                    if let Some(res) = parts.get(1) {
                        let dims: Vec<f32> = res
                            .split(',')
                            .next()
                            .unwrap_or("")
                            .split('x')
                            .filter_map(|d| d.trim().parse::<f32>().ok())
                            .collect();
                        if dims.len() >= 2 {
                            let screen_w = dims[0];
                            let x = (screen_w - width - 16.0).max(0.0);
                            let y = 38.0; // below top bar
                            return Some(Pos2::new(x, y));
                        }
                    }
                }
            }
        }
    }

    // Default top-right fallback
    Some(Pos2::new(1920.0 - width - 16.0, 38.0))
}

pub fn run_gui() -> Result<(), String> {
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::System::Console::FreeConsole();
    }

    let width = 420.0;
    let height = 690.0;

    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([width, height])
        .with_min_inner_size([380.0, 620.0])
        .with_title("Acer Display Center")
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top()
        .with_active(true);

    #[cfg(any(windows, target_os = "linux"))]
    if let Some(pos) = get_tray_popup_pos(width, height) {
        builder = builder.with_position(pos);
    }

    let native_options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    eframe::run_native(
        "Acer Display Center",
        native_options,
        Box::new(|cc| Ok(Box::new(AcerQuickSettingsApp::new(cc)))),
    )
    .map_err(|e| format!("GUI launch failed: {e}"))
}

pub struct AcerReportApp {
    pub title: String,
    pub content: String,
    pub copied: bool,
    pub copied_at: Option<Instant>,
}

impl AcerReportApp {
    pub fn new(title: String, content: String) -> Self {
        Self {
            title,
            content,
            copied: false,
            copied_at: None,
        }
    }
}

impl eframe::App for AcerReportApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let accent = Color32::from_rgb(6, 182, 212); // Cyan accent

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(11, 13, 18);
        visuals.window_fill = Color32::from_rgb(11, 13, 18);
        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(11, 13, 18))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 36, 48)))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::symmetric(14.0, 12.0)),
            )
            .show(ctx, |ui| {
                // Top Header Bar
                ui.horizontal(|ui| {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.0, accent);
                    ui.add_space(2.0);

                    ui.label(
                        egui::RichText::new(&self.title)
                            .strong()
                            .size(12.5)
                            .color(Color32::WHITE),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("✕ Close").size(10.5).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(34, 40, 56))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 56, 76)))
                                    .rounding(Rounding::same(4.0))
                                    .min_size(Vec2::new(55.0, 24.0)),
                            )
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        let copy_label = if self.copied { "✔ Copied!" } else { "📋 Copy" };
                        let copy_color = if self.copied { Color32::from_rgb(16, 185, 129) } else { Color32::WHITE };
                        let copy_btn = ui.add(
                            egui::Button::new(egui::RichText::new(copy_label).size(10.5).color(copy_color).strong())
                                .fill(if self.copied { Color32::from_rgb(20, 48, 38) } else { accent })
                                .rounding(Rounding::same(4.0))
                                .min_size(Vec2::new(75.0, 24.0)),
                        );

                        if copy_btn.clicked() {
                            ui.output_mut(|o| o.copied_text = self.content.clone());
                            self.copied = true;
                            self.copied_at = Some(Instant::now());
                        }
                    });
                });

                if let Some(t) = self.copied_at {
                    if t.elapsed() > Duration::from_millis(2000) {
                        self.copied = false;
                        self.copied_at = None;
                    }
                }

                ui.add_space(8.0);

                // Code / Content Scrollable Area
                egui::Frame::none()
                    .fill(Color32::from_rgb(16, 19, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 34, 46)))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::both()
                            .max_height(ui.available_height() - 4.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&self.content)
                                        .monospace()
                                        .size(11.5)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                );
                            });
                    });
            });
    }
}

#[cfg(windows)]
fn get_center_popup_pos(width: f32, height: f32) -> Option<Pos2> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, GetDesktopWindow,
    };
    use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC, GetDeviceCaps, LOGPIXELSX};

    unsafe {
        let mut work_area: RECT = std::mem::zeroed();
        let success = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut work_area as *mut _ as *mut _,
            0,
        );

        if success != 0 {
            let desktop = GetDesktopWindow();
            let hdc = GetDC(desktop);
            let dpi_x = if hdc != 0 as _ {
                let d = GetDeviceCaps(hdc, LOGPIXELSX as i32);
                ReleaseDC(desktop, hdc);
                d as f32
            } else {
                96.0
            };
            let scale = (dpi_x / 96.0).max(1.0);

            let logical_width = ((work_area.right - work_area.left) as f32) / scale;
            let logical_height = ((work_area.bottom - work_area.top) as f32) / scale;

            let x = (logical_width - width) / 2.0;
            let y = (logical_height - height) / 2.0;
            Some(Pos2::new(x.max(0.0), y.max(0.0)))
        } else {
            None
        }
    }
}

pub fn run_report_gui(title: String, content: String) -> Result<(), String> {
    let width = 580.0;
    let height = 480.0;

    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([width, height])
        .with_min_inner_size([400.0, 300.0])
        .with_title(&title)
        .with_decorations(true)
        .with_resizable(true)
        .with_always_on_top()
        .with_active(true);

    #[cfg(windows)]
    if let Some(pos) = get_center_popup_pos(width, height) {
        builder = builder.with_position(pos);
    }

    let native_options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    let title_clone = title.clone();
    eframe::run_native(
        &title_clone,
        native_options,
        Box::new(move |_cc| Ok(Box::new(AcerReportApp::new(title, content)))),
    )
    .map_err(|e| format!("Report GUI launch failed: {e}"))
}
