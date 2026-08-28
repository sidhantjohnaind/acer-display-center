#[cfg(unix)]
use std::process::Command;
use std::{
    thread::sleep,
    time::Duration,
};

use crate::acer;

pub fn is_video_playing() -> bool {
    #[cfg(unix)]
    {
        // Query MPRIS media players via qdbus / dbus-send / pgrep
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg("dbus-send --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep -i mpris")
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if !text.is_empty() {
                // Check if any player returns Playing status
                for line in text.lines() {
                    let dest = line.trim().trim_matches('"');
                    if dest.starts_with("org.mpris.MediaPlayer2") {
                        if let Ok(status_out) = Command::new("dbus-send")
                            .arg("--print-reply")
                            .arg(format!("--dest={dest}"))
                            .arg("/org/mpris/MediaPlayer2")
                            .arg("org.freedesktop.DBus.Properties.Get")
                            .arg("string:org.mpris.MediaPlayer2.Player")
                            .arg("string:PlaybackStatus")
                            .output()
                        {
                            let status_text = String::from_utf8_lossy(&status_out.stdout);
                            if status_text.contains("Playing") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn get_idle_time_ms() -> u64 {
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("xprintidle").output() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(ms) = text.parse::<u64>() {
                return ms;
            }
        }
    }
    0
}

pub fn run_idle_dimmer(
    idle_secs: u64,
    dim_to: u32,
    spec: Option<&str>,
) -> Result<(), String> {
    let target_idle_ms = idle_secs * 1000;
    println!("Starting Smart Idle Dimmer Daemon (Target Idle: {idle_secs}s, Dim to: {dim_to}%)...");
    println!("Video playback inhibit is active (watching videos will pause dimming).");
    println!("Press Ctrl+C to stop daemon.");

    let mut set = crate::monitor::MonitorSet::enumerate()?;
    let mon = set.pick_mut_by_specifier(spec)?;

    let mut active_brightness = match mon.get_vcp(0x10) {
        Ok((cur, _)) => cur,
        Err(_) => 80,
    };
    let mut is_dimmed = false;

    loop {
        let idle_ms = get_idle_time_ms();
        let video_playing = is_video_playing();

        if idle_ms >= target_idle_ms && !video_playing && !is_dimmed {
            if let Ok((cur, _)) = mon.get_vcp(0x10) {
                if cur > dim_to {
                    active_brightness = cur;
                }
            }
            println!("User idle for {idle_secs}s and no video playing. Dimming display to {dim_to}%...");
            let _ = acer::fade_vcp(mon, 0x10, active_brightness, dim_to, 1000);
            is_dimmed = true;
        } else if (idle_ms < target_idle_ms || video_playing) && is_dimmed {
            let reason = if video_playing { "Video playback detected" } else { "User activity detected" };
            println!("{reason}! Restoring brightness to {active_brightness}%...");
            let _ = acer::fade_vcp(mon, 0x10, dim_to, active_brightness, 500);
            is_dimmed = false;
        }

        sleep(Duration::from_secs(2));
    }
}
