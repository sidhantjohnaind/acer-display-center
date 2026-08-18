use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub name: String,
    pub description: String,
    pub mod_ctrl: bool,
    pub mod_alt: bool,
    pub mod_shift: bool,
    pub mod_win: bool,
    pub key: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortcutActionTemplate {
    pub label: &'static str,
    pub description: &'static str,
    pub command: &'static [&'static str],
    pub default_key: &'static str,
}

pub const ACTION_TEMPLATES: &[ShortcutActionTemplate] = &[
    ShortcutActionTemplate { label: "Open Display Center Flyout", description: "Opens the Acer Quick Access flyout GUI", command: &["gui"], default_key: "M" },
    ShortcutActionTemplate { label: "Toggle Unified HDR", description: "Coordinates Windows 11 HDR & Monitor Hardware HDR", command: &["hdr", "both", "toggle"], default_key: "H" },
    ShortcutActionTemplate { label: "Brightness +10%", description: "Increases screen brightness with on-screen OSD", command: &["brightness", "+10", "--osd"], default_key: "UP" },
    ShortcutActionTemplate { label: "Brightness -10%", description: "Decreases screen brightness with on-screen OSD", command: &["brightness", "-10", "--osd"], default_key: "DOWN" },
    ShortcutActionTemplate { label: "ECO Mode Preset", description: "Switches monitor to low-power ECO profile", command: &["preset", "eco"], default_key: "E" },
    ShortcutActionTemplate { label: "Action Gaming Preset", description: "Switches monitor to high-performance Action mode (100% Brightness)", command: &["preset", "action"], default_key: "A" },
    ShortcutActionTemplate { label: "Racing Mode Preset", description: "Switches monitor to ultra-low latency Racing mode", command: &["preset", "racing"], default_key: "R" },
    ShortcutActionTemplate { label: "Sports Mode Preset", description: "Switches monitor to Sports mode", command: &["preset", "sports"], default_key: "S" },
    ShortcutActionTemplate { label: "Standard Preset", description: "Switches monitor to Standard mode", command: &["preset", "standard"], default_key: "N" },
    ShortcutActionTemplate { label: "Movie Cinema Preset", description: "Switches monitor to Movie mode", command: &["preset", "movie"], default_key: "C" },
    ShortcutActionTemplate { label: "Graphics Preset", description: "Switches monitor to Graphics mode", command: &["preset", "graphics"], default_key: "G" },
    ShortcutActionTemplate { label: "User Custom Preset", description: "Switches monitor to User custom mode", command: &["preset", "user"], default_key: "U" },
    ShortcutActionTemplate { label: "Dim Screen (10%)", description: "Instantly dims screen to 10% brightness", command: &["brightness", "10", "--osd"], default_key: "D" },
    ShortcutActionTemplate { label: "Max Brightness (100%)", description: "Instantly sets screen to maximum 100% brightness", command: &["brightness", "100", "--osd"], default_key: "B" },
    ShortcutActionTemplate { label: "Volume +10%", description: "Increases monitor speaker volume", command: &["volume", "+10", "--osd"], default_key: "RIGHT" },
    ShortcutActionTemplate { label: "Volume -10%", description: "Decreases monitor speaker volume", command: &["volume", "-10", "--osd"], default_key: "LEFT" },
    ShortcutActionTemplate { label: "Toggle Audio Mute", description: "Mutes or unmutes monitor audio", command: &["mute", "toggle", "--osd"], default_key: "X" },
    ShortcutActionTemplate { label: "AimPoint Crosshair Toggle", description: "Toggles hardware AimPoint crosshair on/off", command: &["aim", "1"], default_key: "P" },
    ShortcutActionTemplate { label: "FPS / Hz HUD Counter Toggle", description: "Toggles on-screen refresh rate counter", command: &["refreshnum", "on"], default_key: "F" },
    ShortcutActionTemplate { label: "Switch Input to DisplayPort", description: "Switches video input to DP", command: &["input", "dp"], default_key: "1" },
    ShortcutActionTemplate { label: "Switch Input to HDMI 1", description: "Switches video input to HDMI 1", command: &["input", "hdmi1"], default_key: "2" },
    ShortcutActionTemplate { label: "Switch Input to HDMI 2", description: "Switches video input to HDMI 2", command: &["input", "hdmi2"], default_key: "3" },
    ShortcutActionTemplate { label: "Turn Display Off", description: "Sends DDC/CI power off command to monitor", command: &["power", "off"], default_key: "O" },
    ShortcutActionTemplate { label: "Lock Physical OSD Keys", description: "Locks physical monitor buttons to prevent tampering", command: &["keylock", "on"], default_key: "L" },
    ShortcutActionTemplate { label: "Unlock Physical OSD Keys", description: "Unlocks physical monitor buttons", command: &["unlock"], default_key: "K" },
    ShortcutActionTemplate { label: "Sync All Displays", description: "Synchronizes settings across multiple monitors", command: &["sync"], default_key: "Y" },
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub bindings: Vec<HotkeyBinding>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bindings: vec![
                HotkeyBinding {
                    name: "Open Flyout GUI".into(),
                    description: "Toggles the Acer Display Center Quick Access UI".into(),
                    mod_ctrl: true,
                    mod_alt: true,
                    mod_shift: false,
                    mod_win: false,
                    key: "M".into(),
                    command: vec!["gui".into()],
                },
                HotkeyBinding {
                    name: "Toggle Unified HDR".into(),
                    description: "Coordinates Windows 11 HDR & Monitor Hardware HDR".into(),
                    mod_ctrl: true,
                    mod_alt: true,
                    mod_shift: false,
                    mod_win: false,
                    key: "H".into(),
                    command: vec!["hdr".into(), "both".into(), "toggle".into()],
                },
                HotkeyBinding {
                    name: "Brightness +10%".into(),
                    description: "Increases screen brightness with on-screen OSD".into(),
                    mod_ctrl: true,
                    mod_alt: true,
                    mod_shift: false,
                    mod_win: false,
                    key: "UP".into(),
                    command: vec!["brightness".into(), "+10".into(), "--osd".into()],
                },
                HotkeyBinding {
                    name: "Brightness -10%".into(),
                    description: "Decreases screen brightness with on-screen OSD".into(),
                    mod_ctrl: true,
                    mod_alt: true,
                    mod_shift: false,
                    mod_win: false,
                    key: "DOWN".into(),
                    command: vec!["brightness".into(), "-10".into(), "--osd".into()],
                },
                HotkeyBinding {
                    name: "ECO Mode Preset".into(),
                    description: "Switches monitor to low-power ECO profile".into(),
                    mod_ctrl: true,
                    mod_alt: true,
                    mod_shift: false,
                    mod_win: false,
                    key: "E".into(),
                    command: vec!["preset".into(), "eco".into()],
                },
            ],
        }
    }
}

impl HotkeyConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(local).join("Programs").join("acer_monitor_cli").join("hotkeys.json");
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            p
        } else {
            PathBuf::from("hotkeys.json")
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

    pub fn find_shortcut_for_command(&self, cmd: &[&str]) -> Option<String> {
        if !self.enabled {
            return None;
        }
        for binding in &self.bindings {
            let matches = binding.command.len() == cmd.len()
                && binding
                    .command
                    .iter()
                    .zip(cmd.iter())
                    .all(|(a, b)| a.eq_ignore_ascii_case(b));
            if matches {
                return Some(binding.to_display_string());
            }
        }
        None
    }

    pub fn menu_item(&self, text: &str, cmd: &[&str]) -> String {
        if let Some(sc) = self.find_shortcut_for_command(cmd) {
            format!("{text}\t{sc}")
        } else {
            text.to_string()
        }
    }
}

impl HotkeyBinding {
    pub fn to_display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.mod_ctrl { parts.push("Ctrl"); }
        if self.mod_alt { parts.push("Alt"); }
        if self.mod_shift { parts.push("Shift"); }
        if self.mod_win { parts.push("Win"); }
        parts.push(self.key.as_str());
        parts.join(" + ")
    }

    pub fn win32_modifiers(&self) -> u32 {
        let mut m = 0x4000; // MOD_NOREPEAT
        if self.mod_alt { m |= 0x0001; }
        if self.mod_ctrl { m |= 0x0002; }
        if self.mod_shift { m |= 0x0004; }
        if self.mod_win { m |= 0x0008; }
        m
    }

    pub fn win32_vk(&self) -> u32 {
        match self.key.to_ascii_uppercase().as_str() {
            "UP" => 0x26,
            "DOWN" => 0x28,
            "LEFT" => 0x25,
            "RIGHT" => 0x27,
            "SPACE" => 0x20,
            "F1" => 0x70,
            "F2" => 0x71,
            "F3" => 0x72,
            "F4" => 0x73,
            "F5" => 0x74,
            "F6" => 0x75,
            "F7" => 0x76,
            "F8" => 0x77,
            "F9" => 0x78,
            "F10" => 0x79,
            "F11" => 0x7A,
            "F12" => 0x7B,
            s if s.len() == 1 => s.chars().next().unwrap() as u32,
            _ => 'M' as u32,
        }
    }
}
