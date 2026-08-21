use crate::{
    acer, ddc::parse_u32, edid::EdidInfo, energy, idle, monitor::MonitorSet, osd::show_osd_banner,
    pattern, server, solar::SolarSchedule,
};

pub fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return launch_gui_with_tray();
    }

    let output = dispatch_command(args)?;
    if !output.is_empty() {
        println!("{output}");
    }
    Ok(())
}

fn launch_gui_with_tray() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        let exe = std::env::current_exe().unwrap_or_else(|_| {
            std::path::PathBuf::from("amctl.exe")
        });
        // Ensure tray daemon is running in background if not already active
        let _ = Command::new(&exe)
            .arg("tray")
            .creation_flags(0x08000000 /* CREATE_NO_WINDOW */)
            .spawn();
    }
    #[cfg(not(windows))]
    {
        std::thread::spawn(|| {
            let _ = crate::tray::run_tray();
        });
    }

    crate::gui::run_gui()
}

pub fn dispatch_command(mut args: Vec<String>) -> Result<String, String> {
    if args.is_empty() {
        return Ok(get_help_string());
    }

    let cmd = args.remove(0).to_ascii_lowercase();
    match cmd.as_str() {
        "help" | "-h" | "--help" => Ok(get_help_string()),

        "preset" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli preset <user|standard|eco|graphics|hdr|action|racing|sports|reading|movie|0-11> [specifier] [--unified]".to_string());
            }
            let preset_name = args[0].to_ascii_lowercase();
            let is_unified = args.iter().any(|a| a == "--unified" || a == "-u");
            let filtered: Vec<String> = args[1..].iter().filter(|a| *a != "--unified" && *a != "-u").cloned().collect();
            let spec = parse_optional_specifier(&filtered);

            let val: u32 = match preset_name.as_str() {
                "user" => 0,
                "standard" | "normal" => 1,
                "eco" => 2,
                "graphics" => 3,
                "action" | "gaming" => 5,
                "racing" => 6,
                "sports" => 7,
                "hdr" | "hdr400" => 11,
                "reading" | "text" => 2,
                "movie" | "cinema" => 3,
                other => parse_u32(other)?,
            };

            let is_hdr = val == 11 || preset_name == "hdr" || preset_name == "hdr400";

            // If switching to an SDR preset while Windows OS HDR is active, disable OS HDR first
            // so the monitor hardware scaler unlocks and actually switches modes.
            if !is_hdr && crate::hdr::get_os_hdr() {
                crate::hdr::set_os_hdr(false);
                std::thread::sleep(std::time::Duration::from_millis(200));
            } else if is_hdr && !crate::hdr::get_os_hdr() {
                crate::hdr::set_os_hdr(true);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            with_monitor(spec, |mon| {
                // Direct hardware Display Mode register (Acer 0xE2)
                let _ = mon.set_vcp(0xE2, val);
                Ok(())
            })?;

            Ok(format!("Applied hardware preset '{preset_name}' (VCP 0xE2 = {val})."))
        }

        "hotkeys" => {
            if args.is_empty() {
                let state = crate::tray::are_hotkeys_enabled();
                return Ok(format!("Global hotkeys are {}.", if state { "ENABLED" } else { "DISABLED" }));
            }
            let on = parse_on_off(&args[0])?;
            crate::tray::set_hotkeys_enabled(on);
            Ok(format!("Global hotkeys set to {}.", if on { "ENABLED" } else { "DISABLED" }))
        }

        "unlock" => {
            let spec = parse_optional_specifier(&args);
            with_monitor(spec, |mon| {
                acer::key_lock(mon, false)?;
                acer::power_key(mon, false)?;
                Ok(())
            })?;
            Ok("Unlocked OSD keys and power button.".to_string())
        }

        "balance" => {
            let offset = parse_flag_val(&args, "--offset").unwrap_or(-15.0) as i32;
            let filtered: Vec<String> = args.into_iter().filter(|a| !a.starts_with("--offset")).collect();
            let master_spec = parse_optional_specifier(&filtered);

            let mut set = MonitorSet::enumerate()?;
            if set.monitors_mut().len() < 2 {
                return Ok("Only one monitor detected; balance requires 2+ displays.".to_string());
            }

            let master_b = {
                let master_mon = set.pick_mut_by_specifier(master_spec)?;
                let (b, _) = master_mon.get_vcp(0x10)?;
                b
            };

            for (i, mon) in set.monitors_mut().iter_mut().enumerate() {
                if i > 0 {
                    let sec_b = ((master_b as i32) + offset).clamp(0, 100) as u32;
                    let _ = acer::brightness(mon, sec_b);
                }
            }
            Ok(format!("Balanced secondary displays with offset {offset} relative to master brightness ({master_b}%)."))
        }

        "diag" => {
            let spec = parse_optional_specifier(&args);
            let mut report = String::new();
            with_monitor(spec, |mon| {
                mon.update_capabilities()?;
                report.push_str(&format!("=== Diagnostic Report for '{}' ===\n\n", mon.description));
                if let Some(edid) = EdidInfo::inspect_connected() {
                    report.push_str(&edid.report());
                    report.push_str("\n");
                }
                report.push_str(&mon.capabilities.report(&mon.description));
                report.push_str("\n");
                report.push_str(&format_monitor_info(mon)?);
                Ok(())
            })?;
            Ok(report)
        }

        "idle-dimmer" => {
            let idle_secs = parse_flag_u64(&args, "--idle-secs").unwrap_or(300);
            let dim_to = parse_flag_u32(&args, "--dim-to").unwrap_or(10);
            let filtered: Vec<String> = args
                .into_iter()
                .filter(|a| !a.starts_with("--idle-secs") && !a.starts_with("--dim-to"))
                .collect();
            let spec = parse_optional_specifier(&filtered);
            idle::run_idle_dimmer(idle_secs, dim_to, spec)?;
            Ok("Idle dimmer stopped.".to_string())
        }

        "energy" => {
            let is_hdr_flag = args.iter().any(|a| a == "--hdr");
            let filtered: Vec<String> = args.iter().filter(|a| *a != "--hdr").cloned().collect();
            let spec = parse_optional_specifier(&filtered);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (b, _) = mon.get_vcp(0x10).unwrap_or((80, 100));
                let is_hdr = is_hdr_flag || mon.get_vcp(0xE2).map(|(v, _)| v == 11).unwrap_or(false);
                out = energy::report_energy(b, &mon.description, is_hdr);
                Ok(())
            })?;
            Ok(out)
        }


        "test-pattern" => {
            let name = args.first().map(|s| s.as_str()).unwrap_or("grid");
            Ok(pattern::render_pattern(name))
        }

        "report" => {
            let title = if !args.is_empty() { args.remove(0) } else { "Acer Monitor Report".into() };
            let content = args.join(" ");
            crate::gui::run_report_gui(title, content)?;
            Ok(String::new())
        }

        "gui" | "app" | "flyout" => {
            launch_gui_with_tray()?;
            Ok(String::new())
        }

        "tray" => {
            crate::tray::run_tray()?;
            Ok(String::new())
        }

        "watch-vcp" | "watch" => run_watch_vcp(args),

        "record" | "record-vcp" | "diff" | "diff-vcp" | "learn" => run_record_vcp(args),

        "watch-monitors" => {
            println!("Hotplug Monitor Watcher running... Press Ctrl+C to stop.");
            let mut last_count = 0;
            loop {
                if let Ok(mut set) = MonitorSet::enumerate() {
                    let current_count = set.monitors_mut().len();
                    if current_count != last_count {
                        println!("Monitor event detected! Current connected count: {current_count}");
                        last_count = current_count;
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        }

        "auto-profile" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli auto-profile --rule \"process_name:profile.json\"".to_string());
            }
            println!("Auto-Profile Switcher running... Press Ctrl+C to stop.");

            let mut rules = Vec::new();
            let mut i = 0;
            while i < args.len() {
                if let Some(rule) = args[i].strip_prefix("--rule=") {
                    rules.push(rule.to_string());
                } else if args[i] == "--rule" && i + 1 < args.len() {
                    rules.push(args[i + 1].clone());
                    i += 1;
                }
                i += 1;
            }

            if rules.is_empty() {
                return Err("No valid rules specified. Usage: acer_monitor_cli auto-profile --rule \"process_name:profile.json\"".to_string());
            }

            loop {
                for rule in &rules {
                    let parts: Vec<&str> = rule.split(':').collect();
                    if parts.len() == 2 {
                        let proc_name = parts[0];
                        let profile_path = parts[1];

                        #[cfg(unix)]
                        {
                            if let Ok(out) = std::process::Command::new("pgrep").arg(proc_name).output() {
                                if !out.stdout.is_empty() {
                                    let _ = with_monitor(None, |mon| profile_load(mon, profile_path));
                                }
                            }
                        }

                        #[cfg(windows)]
                        {
                            if let Ok(out) = std::process::Command::new("tasklist").args(&["/FI", &format!("IMAGENAME eq {proc_name}")]).output() {
                                let text = String::from_utf8_lossy(&out.stdout);
                                if text.contains(proc_name) {
                                    let _ = with_monitor(None, |mon| profile_load(mon, profile_path));
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }


        "hdr" => {
            let target_sub = args.first().map(|s| s.as_str()).unwrap_or("both");
            
            let (mode, action) = match target_sub {
                "os" => ("os", args.get(1).map(|s| s.as_str()).unwrap_or("toggle")),
                "monitor" | "display" | "hardware" => ("monitor", args.get(1).map(|s| s.as_str()).unwrap_or("toggle")),
                "both" => ("both", args.get(1).map(|s| s.as_str()).unwrap_or("toggle")),
                act => ("both", act),
            };

            if mode == "os" {
                let enable = match action {
                    "on" | "enable" | "1" => true,
                    "off" | "disable" | "0" => false,
                    _ => true,
                };
                crate::hdr::set_os_hdr(enable);
                let os_str = if enable { "ON" } else { "OFF" };
                return Ok(format!("OS-level HDR set to {os_str}."));
            }

            let filtered: Vec<String> = args.iter()
                .filter(|a| *a != "os" && *a != "monitor" && *a != "display" && *a != "hardware" && *a != "both" && *a != "on" && *a != "off" && *a != "enable" && *a != "disable" && *a != "toggle" && *a != "1" && *a != "0")
                .cloned()
                .collect();
            let spec = parse_optional_specifier(&filtered);
            let mut out = String::new();

            with_monitor(spec, |mon| {
                let enable = match action {
                    "on" | "enable" | "1" => true,
                    "off" | "disable" | "0" => false,
                    "toggle" => {
                        let is_hdr = mon.get_vcp(0xE2).map(|(v, _)| v == 11).unwrap_or(false);
                        !is_hdr
                    }
                    _ => true,
                };

                let mode_val = if enable { 11 } else { 1 };
                let mode_name = if enable { "HDR Game Mode" } else { "Standard Mode" };
                acer::display_mode(mon, mode_val)?;
                if enable {
                    let _ = acer::brightness(mon, 100);
                }
                if mode == "both" {
                    crate::hdr::set_os_hdr(enable);
                    let os_hdr_str = if enable { "ON" } else { "OFF" };
                    out = format!("Unified OS + Hardware HDR set to {os_hdr_str}! Display mode set to '{mode_name}' and OS HDR toggled {os_hdr_str}.");
                } else {
                    let hdr_str = if enable { "ON" } else { "OFF" };
                    out = format!("Hardware Display HDR set to {hdr_str} ('{mode_name}').");
                }
                Ok(())
            })?;
            Ok(out)
        }

        "sdr" | "sdr-brightness" | "sdr-white-level" => {
            let cur_opt = crate::hdr::get_sdr_white_level();
            if args.is_empty() {
                if let Some(cur) = cur_opt {
                    let nits = 80.0 + (cur as f32 / 100.0 * 400.0);
                    return Ok(format!("Windows SDR Content Brightness: {cur}% (~{nits:.0} nits)"));
                } else {
                    return Err("Failed to query Windows SDR Content Brightness (HDR may be disabled).".to_string());
                }
            }
            let val_str = &args[0];
            let val = if val_str.starts_with('+') || val_str.starts_with('-') {
                let cur = cur_opt.unwrap_or(50);
                parse_relative_or_abs(val_str, cur, 100)?
            } else {
                parse_u32(val_str)?
            };
            let show_osd = args.iter().any(|a| a == "--osd");
            crate::hdr::set_sdr_white_level(val)?;
            let clamped = val.min(100);
            let nits = 80.0 + (clamped as f32 / 100.0 * 400.0);
            if show_osd {
                show_osd_banner("SDR Brightness", clamped, 100);
            }
            Ok(format!("Windows SDR Content Brightness set to {clamped}% (~{nits:.0} nits)."))
        }

        "server" => {
            server::run_server()?;
            Ok("Server stopped.".to_string())
        }


        "send" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli send <command...>".to_string());
            }
            server::send_command(&args)
        }

        "install-service" => install_systemd_service(),

        "install-desktop" => install_desktop_entry(),

        "sync" => {
            let master_spec = parse_optional_specifier(&args);
            sync_monitors(master_spec)
        }

        "solar" | "solar-schedule" => {
            let lat = parse_flag_val(&args, "--lat").unwrap_or(28.61);
            let lon = parse_flag_val(&args, "--lon").unwrap_or(77.20);
            let day_b = parse_flag_u32(&args, "--day-b").unwrap_or(80);
            let night_b = parse_flag_u32(&args, "--night-b").unwrap_or(20);
            let day_bl = parse_flag_u32(&args, "--day-bl").unwrap_or(0);
            let night_bl = parse_flag_u32(&args, "--night-bl").unwrap_or(3);

            let day_ct = parse_flag_str(&args, "--day-ct").and_then(|s| parse_color_temp(&s).ok());
            let night_ct = parse_flag_str(&args, "--night-ct").and_then(|s| parse_color_temp(&s).ok());

            let schedule = SolarSchedule::new(lat, lon, day_b, night_b, day_bl, night_bl, day_ct, night_ct);
            let (target_b, target_bl, target_ct) = schedule.calculate_targets();

            let is_day = schedule.is_daytime();
            let mode_name = if is_day { "Day Mode" } else { "Night Mode" };

            let mut filtered = Vec::new();
            let mut skip_next = false;
            for arg in &args {
                if skip_next {
                    skip_next = false;
                    continue;
                }
                if arg == "--lat" || arg == "--lon" || arg == "--day-b" || arg == "--night-b" || arg == "--day-bl" || arg == "--night-bl" || arg == "--day-ct" || arg == "--night-ct" {
                    skip_next = true;
                    continue;
                }
                filtered.push(arg.clone());
            }

            let spec = parse_optional_specifier(&filtered);

            with_monitor(spec, |mon| {
                acer::brightness(mon, target_b)?;
                acer::blue_light(mon, target_bl)?;
                if let Some(ct) = target_ct {
                    acer::color_temp(mon, ct)?;
                }
                Ok(())
            })?;

            Ok(format!("Applied {mode_name} (Brightness: {target_b}, BlueLight level: {target_bl}) based on coordinates ({lat}, {lon})."))
        }

        "waybar-config" => Ok(get_waybar_config_string()),

        "completions" => {
            let shell = args.first().map(|s| s.as_str()).unwrap_or("bash");
            Ok(get_completions_string(shell))
        }

        "edid" => {
            if let Some(parsed) = EdidInfo::inspect_connected() {
                Ok(parsed.report())
            } else {
                Err("Could not read or parse connected monitor EDID.".to_string())
            }
        }

        "list" => {
            let mut set = MonitorSet::enumerate()?;
            if args.iter().any(|a| a == "--json" || a == "-j") {
                let items: Vec<String> = set
                    .monitors_mut()
                    .iter()
                    .enumerate()
                    .map(|(i, m)| {
                        let desc_escaped = m.description.replace('"', "\\\"");
                        format!("  {{\"index\": {i}, \"description\": \"{desc_escaped}\"}}")
                    })
                    .collect();
                Ok(format!("[\n{}\n]", items.join(",\n")))
            } else {
                let mut out = String::new();
                for (i, m) in set.monitors_mut().iter().enumerate() {
                    out.push_str(&format!("[{i}] {}\n", m.description));
                }
                Ok(out.trim_end().to_string())
            }
        }

        "caps" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                mon.update_capabilities()?;
                out.push_str(&mon.capabilities.report(&mon.description));
                Ok(())
            })?;
            Ok(out)
        }

        "get" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli get <vcp_code> [specifier]".to_string());
            }
            let code = parse_u32(&args[0])? as u8;
            let spec = parse_optional_specifier(&args[1..]);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = mon.get_vcp(code)?;
                out = format!("0x{code:02X}: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "set" => {
            if args.len() < 2 {
                return Err("Usage: acer_monitor_cli set <vcp_code> <value> [specifier]".to_string());
            }
            let code = parse_u32(&args[0])? as u8;
            let value = parse_u32(&args[1])?;
            let spec = parse_optional_specifier(&args[2..]);
            with_monitor(spec, |mon| mon.set_vcp(code, value))?;
            Ok("OK".to_string())
        }

        "power" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli power <on|off> [specifier]".to_string());
            }
            let on = parse_on_off(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::power_mode(mon, on))?;
            Ok("OK".to_string())
        }

        "reset" => {
            let spec = parse_optional_specifier(&args);
            with_monitor(spec, acer::factory_reset)?;
            Ok("Factory reset applied.".to_string())
        }

        "volume" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli volume <0-100 | +val | -val> [--osd] [specifier]".to_string());
            }
            let show_osd = args.iter().any(|a| a == "--osd");
            let filtered: Vec<String> = args.into_iter().filter(|a| a != "--osd").collect();
            let val_str = &filtered[0];
            let spec = parse_optional_specifier(&filtered[1..]);

            with_monitor(spec, |mon| {
                let new_val = if val_str.starts_with('+') || val_str.starts_with('-') {
                    let (cur, max) = mon.get_vcp(0x62)?;
                    parse_relative_or_abs(val_str, cur, max)?
                } else {
                    parse_u32(val_str)?
                };
                acer::volume(mon, new_val)?;
                if show_osd {
                    let (_, max) = mon.get_vcp(0x62).unwrap_or((new_val, 100));
                    show_osd_banner("Volume", new_val, max);
                }
                Ok(())
            })?;
            Ok("OK".to_string())
        }

        "mute" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli mute <on|off|toggle> [--osd] [specifier]".to_string());
            }
            let show_osd = args.iter().any(|a| a == "--osd");
            let filtered: Vec<String> = args.into_iter().filter(|a| a != "--osd").collect();
            let val_str = &filtered[0];
            let spec = parse_optional_specifier(&filtered[1..]);

            if val_str.eq_ignore_ascii_case("toggle") {
                with_monitor(spec, |mon| {
                    let (cur, _) = mon.get_vcp(0x8D)?;
                    let is_muted = cur == 1;
                    acer::mute(mon, !is_muted)?;
                    if show_osd {
                        show_osd_banner("Mute", if !is_muted { 100 } else { 0 }, 100);
                    }
                    Ok(())
                })?;
            } else {
                let on = parse_on_off(val_str)?;
                with_monitor(spec, |mon| {
                    acer::mute(mon, on)?;
                    if show_osd {
                        show_osd_banner("Mute", if on { 100 } else { 0 }, 100);
                    }
                    Ok(())
                })?;
            }
            Ok("OK".to_string())
        }

        "keylock" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli keylock <on|off> [specifier]".to_string());
            }
            let on = parse_on_off(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| {
                acer::key_lock(mon, on)?;
                Ok(())
            })?;
            Ok(if on { "OSD keys locked.".to_string() } else { "OSD keys unlocked.".to_string() })
        }

        "powerkey" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli powerkey <on|off> [specifier]".to_string());
            }
            let on = parse_on_off(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::power_key(mon, on))?;
            Ok("OK".to_string())
        }

        "indicator" | "led" | "power-led" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (val, _) = with_monitor(spec, |mon| acer::get_power_indicator(mon))?;
                return Ok(format!("Power LED Indicator is {}", if val == 1 { "ON" } else { "OFF" }));
            }
            let val_str = &args[0];
            let spec = parse_optional_specifier(&args[1..]);
            if val_str.eq_ignore_ascii_case("toggle") {
                with_monitor(spec, |mon| {
                    let (cur, _) = acer::get_power_indicator(mon).unwrap_or((1, 1));
                    acer::power_indicator(mon, cur != 1)
                })?;
            } else {
                let on = parse_on_off(val_str)?;
                with_monitor(spec, |mon| acer::power_indicator(mon, on))?;
            }
            Ok("OK".to_string())
        }

        "od" | "overdrive" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (val, _) = with_monitor(spec, |mon| acer::get_overdrive(mon))?;
                let name = match val { 0 => "Off", 1 => "Normal", 2 => "Extreme", _ => "Off" };
                return Ok(format!("OverDrive: {name} ({val})"));
            }
            let val_str = &args[0];
            let value = match val_str.to_ascii_lowercase().as_str() {
                "off" | "0" => 0,
                "normal" | "1" => 1,
                "extreme" | "2" => 2,
                other => parse_u32(other)?,
            };
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::overdrive(mon, value))?;
            Ok("OK".to_string())
        }

        "aim" | "aimpoint" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (val, _) = with_monitor(spec, |mon| acer::get_aim_type(mon))?;
                let name = match val { 0 => "Off", 1 => "Dot", 2 => "Cross", 3 => "Triangle", _ => "Unknown" };
                return Ok(format!("AimPoint: {name} ({val})"));
            }
            let val_str = &args[0];
            let spec = parse_optional_specifier(&args[1..]);
            if val_str.eq_ignore_ascii_case("next") || val_str.eq_ignore_ascii_case("cycle") {
                with_monitor(spec, |mon| {
                    let (cur, _) = acer::get_aim_type(mon).unwrap_or((0, 3));
                    let next = (cur + 1) % 4;
                    acer::aim_type(mon, next)
                })?;
            } else {
                let value = match val_str.to_ascii_lowercase().as_str() {
                    "off" | "0" => 0,
                    "dot" | "1" => 1,
                    "cross1" | "cross" | "2" => 2,
                    "cross2" | "triangle" | "3" => 3,
                    other => parse_u32(other)?,
                };
                with_monitor(spec, |mon| acer::aim_type(mon, value))?;
            }
            Ok("OK".to_string())
        }

        "refreshnum" | "fps" | "hz" | "refresh-rate" => {
            let spec = parse_optional_specifier(&args[1.min(args.len())..]);
            if args.is_empty() {
                let (cur, _) = with_monitor(spec, |mon| acer::get_refresh_rate_num(mon))?;
                return Ok(format!("Refresh Rate (FPS/Hz) HUD is {}", if cur == 1 { "ON" } else { "OFF" }));
            }
            let val_str = &args[0];
            if val_str.eq_ignore_ascii_case("toggle") {
                with_monitor(spec, |mon| {
                    let cur = acer::get_refresh_rate_num(mon).map(|(v, _)| v == 1).unwrap_or(false);
                    acer::refresh_rate_num(mon, !cur)
                })?;
            } else {
                let on = parse_on_off(val_str)?;
                with_monitor(spec, |mon| acer::refresh_rate_num(mon, on))?;
            }
            Ok("OK".to_string())
        }

        "bluelight" | "blue-light" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (val, _) = with_monitor(spec, |mon| acer::get_blue_light(mon))?;
                return Ok(format!("Blue Light Shield: Level {val}"));
            }
            let value = parse_bluelight(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::blue_light(mon, value))?;
            Ok("OK".to_string())
        }

        "gamma" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (val, _) = with_monitor(spec, |mon| acer::get_gamma(mon))?;
                let name = match val { 0 => "1.8", 1 => "2.2", 2 => "2.4", _ => "2.2" };
                return Ok(format!("Gamma: {name} ({val})"));
            }
            let value = parse_gamma(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::gamma(mon, value))?;
            Ok("OK".to_string())
        }

        "colortemp" | "color-temp" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (val, _) = with_monitor(spec, |mon| acer::get_color_temp(mon))?;
                let name = match val { 0 => "Warm", 1 => "Normal", 2 => "Cool", 3 => "BlueLight", 4 => "User", _ => "Normal" };
                return Ok(format!("Color Temperature: {name} ({val})"));
            }
            if crate::hdr::get_os_hdr() {
                crate::hdr::set_os_hdr(false);
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            let value = parse_color_temp(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::color_temp(mon, value))?;
            Ok("OK".to_string())
        }

        "displaymode" | "display-mode" | "mode" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli displaymode <value> [specifier]".to_string());
            }
            let value = parse_u32(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::display_mode(mon, value))?;
            Ok("OK".to_string())
        }

        "colorspace" | "color-space" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli colorspace <srgb|rec709|hdr|ebu|dcip3|smpte-c|general> OR <calib_idx> <space_idx>".to_string());
            }
            let (calib, space, spec_start) = match args[0].to_ascii_lowercase().as_str() {
                "srgb" => (0, 0, 1),
                "rec709" => (0, 1, 1),
                "hdr" => (0, 2, 1),
                "ebu" => (0, 3, 1),
                "dcip3" | "dci" => (0, 4, 1),
                "smpte-c" | "smpte" => (0, 5, 1),
                "general" => (0, 6, 1),
                _ => {
                    if args.len() < 2 {
                        return Err("Usage: acer_monitor_cli colorspace <srgb|rec709|hdr|ebu|dcip3|smpte-c> OR <calib_idx> <space_idx>".to_string());
                    }
                    let c = parse_u32(&args[0])?;
                    let s = parse_u32(&args[1])?;
                    (c, s, 2)
                }
            };
            let spec = parse_optional_specifier(&args[spec_start..]);
            with_monitor(spec, |mon| acer::color_space(mon, calib, space))?;
            Ok("OK".to_string())
        }

        "gain" | "rgb" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (r, g, b) = with_monitor(spec, |mon| acer::get_rgb_gain(mon))?;
                return Ok(format!("Hardware RGB Gain: Red={r}%, Green={g}%, Blue={b}%"));
            }

            let sub = args[0].to_ascii_lowercase();
            match sub.as_str() {
                "reset" => {
                    let spec = parse_optional_specifier(&args[1..]);
                    with_monitor(spec, |mon| {
                        acer::set_rgb_gain(mon, 50, 50, 50)
                    })?;
                    Ok("Hardware RGB Gain reset to default (50 / 50 / 50).".to_string())
                }
                "red" | "r" => {
                    if args.len() < 2 {
                        let spec = parse_optional_specifier(&args[1..]);
                        let (r, _) = with_monitor(spec, |mon| mon.get_vcp(0x16))?;
                        return Ok(format!("Red Gain: {r}%"));
                    }
                    let val_str = &args[1];
                    let spec = parse_optional_specifier(&args[2..]);
                    with_monitor(spec, |mon| {
                        let val = if val_str.starts_with('+') || val_str.starts_with('-') {
                            let (cur, max) = mon.get_vcp(0x16).unwrap_or((50, 100));
                            parse_relative_or_abs(val_str, cur, max)?
                        } else {
                            parse_u32(val_str)?
                        };
                        acer::red_gain(mon, val.clamp(0, 100))
                    })?;
                    Ok("OK".to_string())
                }
                "green" | "g" => {
                    if args.len() < 2 {
                        let spec = parse_optional_specifier(&args[1..]);
                        let (g, _) = with_monitor(spec, |mon| mon.get_vcp(0x18))?;
                        return Ok(format!("Green Gain: {g}%"));
                    }
                    let val_str = &args[1];
                    let spec = parse_optional_specifier(&args[2..]);
                    with_monitor(spec, |mon| {
                        let val = if val_str.starts_with('+') || val_str.starts_with('-') {
                            let (cur, max) = mon.get_vcp(0x18).unwrap_or((50, 100));
                            parse_relative_or_abs(val_str, cur, max)?
                        } else {
                            parse_u32(val_str)?
                        };
                        acer::green_gain(mon, val.clamp(0, 100))
                    })?;
                    Ok("OK".to_string())
                }
                "blue" | "b" => {
                    if args.len() < 2 {
                        let spec = parse_optional_specifier(&args[1..]);
                        let (b, _) = with_monitor(spec, |mon| mon.get_vcp(0x1A))?;
                        return Ok(format!("Blue Gain: {b}%"));
                    }
                    let val_str = &args[1];
                    let spec = parse_optional_specifier(&args[2..]);
                    with_monitor(spec, |mon| {
                        let val = if val_str.starts_with('+') || val_str.starts_with('-') {
                            let (cur, max) = mon.get_vcp(0x1A).unwrap_or((50, 100));
                            parse_relative_or_abs(val_str, cur, max)?
                        } else {
                            parse_u32(val_str)?
                        };
                        acer::blue_gain(mon, val.clamp(0, 100))
                    })?;
                    Ok("OK".to_string())
                }
                _ => {
                    if args.len() >= 3 {
                        let r = parse_u32(&args[0])?.clamp(0, 100);
                        let g = parse_u32(&args[1])?.clamp(0, 100);
                        let b = parse_u32(&args[2])?.clamp(0, 100);
                        let spec = parse_optional_specifier(&args[3..]);
                        with_monitor(spec, |mon| acer::set_rgb_gain(mon, r, g, b))?;
                        Ok(format!("Hardware RGB Gain set to Red={r}%, Green={g}%, Blue={b}%."))
                    } else {
                        Err("Usage: amctl gain <red|green|blue|reset> <value> OR amctl gain <r> <g> <b>".to_string())
                    }
                }
            }
        }

        "red" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (r, _) = with_monitor(spec, |mon| mon.get_vcp(0x16))?;
                return Ok(format!("Red Gain: {r}%"));
            }
            let val_str = &args[0];
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| {
                let val = if val_str.starts_with('+') || val_str.starts_with('-') {
                    let (cur, max) = mon.get_vcp(0x16).unwrap_or((50, 100));
                    parse_relative_or_abs(val_str, cur, max)?
                } else {
                    parse_u32(val_str)?
                };
                acer::red_gain(mon, val.clamp(0, 100))
            })?;
            Ok("OK".to_string())
        }

        "green" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (g, _) = with_monitor(spec, |mon| mon.get_vcp(0x18))?;
                return Ok(format!("Green Gain: {g}%"));
            }
            let val_str = &args[0];
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| {
                let val = if val_str.starts_with('+') || val_str.starts_with('-') {
                    let (cur, max) = mon.get_vcp(0x18).unwrap_or((50, 100));
                    parse_relative_or_abs(val_str, cur, max)?
                } else {
                    parse_u32(val_str)?
                };
                acer::green_gain(mon, val.clamp(0, 100))
            })?;
            Ok("OK".to_string())
        }

        "blue" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (b, _) = with_monitor(spec, |mon| mon.get_vcp(0x1A))?;
                return Ok(format!("Blue Gain: {b}%"));
            }
            let val_str = &args[0];
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| {
                let val = if val_str.starts_with('+') || val_str.starts_with('-') {
                    let (cur, max) = mon.get_vcp(0x1A).unwrap_or((50, 100));
                    parse_relative_or_abs(val_str, cur, max)?
                } else {
                    parse_u32(val_str)?
                };
                acer::blue_gain(mon, val.clamp(0, 100))
            })?;
            Ok("OK".to_string())
        }

        "bias" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (r, _) = with_monitor(spec, |mon| mon.get_vcp(0x6C))?;
                let (g, _) = with_monitor(spec, |mon| mon.get_vcp(0x6E))?;
                let (b, _) = with_monitor(spec, |mon| mon.get_vcp(0x70))?;
                return Ok(format!("Hardware RGB Bias: Red={r}%, Green={g}%, Blue={b}%"));
            }
            let sub = args[0].to_ascii_lowercase();
            match sub.as_str() {
                "red" | "r" => {
                    let val = parse_u32(&args[1])?.clamp(0, 100);
                    let spec = parse_optional_specifier(&args[2..]);
                    with_monitor(spec, |mon| acer::red_bias(mon, val))?;
                    Ok("OK".to_string())
                }
                "green" | "g" => {
                    let val = parse_u32(&args[1])?.clamp(0, 100);
                    let spec = parse_optional_specifier(&args[2..]);
                    with_monitor(spec, |mon| acer::green_bias(mon, val))?;
                    Ok("OK".to_string())
                }
                "blue" | "b" => {
                    let val = parse_u32(&args[1])?.clamp(0, 100);
                    let spec = parse_optional_specifier(&args[2..]);
                    with_monitor(spec, |mon| acer::blue_bias(mon, val))?;
                    Ok("OK".to_string())
                }
                _ => {
                    if args.len() >= 3 {
                        let r = parse_u32(&args[0])?.clamp(0, 100);
                        let g = parse_u32(&args[1])?.clamp(0, 100);
                        let b = parse_u32(&args[2])?.clamp(0, 100);
                        let spec = parse_optional_specifier(&args[3..]);
                        with_monitor(spec, |mon| {
                            acer::red_bias(mon, r)?;
                            acer::green_bias(mon, g)?;
                            acer::blue_bias(mon, b)?;
                            Ok(())
                        })?;
                        Ok(format!("Hardware RGB Bias set to Red={r}%, Green={g}%, Blue={b}%."))
                    } else {
                        Err("Usage: amctl bias <red|green|blue> <value> OR amctl bias <r> <g> <b>".to_string())
                    }
                }
            }
        }

        "blackboost" | "black-boost" | "bb" => {
            if args.is_empty() {
                let spec = parse_optional_specifier(&args);
                let (val, max) = with_monitor(spec, |mon| acer::get_black_boost(mon))?;
                return Ok(format!("Black Boost: {val}/{max}"));
            }
            let value = parse_u32(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::black_boost(mon, value))?;
            Ok("OK".to_string())
        }

        "raw" | "rawbank" => {
            if args.len() < 3 {
                return Err("Usage: acer_monitor_cli raw <bank_hex> <selector_hex> <value_hex> [specifier]\nExample: acer_monitor_cli raw e0 02 01".to_string());
            }
            let bank = u8::from_str_radix(args[0].trim_start_matches("0x").trim_start_matches("0X"), 16)
                .map_err(|e| format!("Invalid bank hex: {e}"))?;
            let selector = u32::from_str_radix(args[1].trim_start_matches("0x").trim_start_matches("0X"), 16)
                .map_err(|e| format!("Invalid selector hex: {e}"))?;
            let value = u32::from_str_radix(args[2].trim_start_matches("0x").trim_start_matches("0X"), 16)
                .map_err(|e| format!("Invalid value hex: {e}"))?;
            let spec = parse_optional_specifier(&args[3..]);
            with_monitor(spec, |mon| acer::raw_bank(mon, bank, selector, value))?;
            Ok("OK".to_string())
        }

        "brightness" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli brightness <0-100 | +val | -val> [--osd] [specifier]".to_string());
            }
            let show_osd = args.iter().any(|a| a == "--osd");
            let filtered: Vec<String> = args.into_iter().filter(|a| a != "--osd").collect();
            let val_str = &filtered[0];
            let spec = parse_optional_specifier(&filtered[1..]);

            with_monitor(spec, |mon| {
                let new_val = if val_str.starts_with('+') || val_str.starts_with('-') {
                    let (cur, max) = mon.get_vcp(0x10)?;
                    parse_relative_or_abs(val_str, cur, max)?
                } else {
                    parse_u32(val_str)?
                };
                acer::brightness(mon, new_val)?;
                if show_osd {
                    let (_, max) = mon.get_vcp(0x10).unwrap_or((new_val, 100));
                    show_osd_banner("Brightness", new_val, max);
                }
                Ok(())
            })?;
            Ok("OK".to_string())
        }

        "contrast" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli contrast <0-100 | +val | -val> [--osd] [specifier]".to_string());
            }
            let show_osd = args.iter().any(|a| a == "--osd");
            let filtered: Vec<String> = args.into_iter().filter(|a| a != "--osd").collect();
            let val_str = &filtered[0];
            let spec = parse_optional_specifier(&filtered[1..]);

            with_monitor(spec, |mon| {
                let new_val = if val_str.starts_with('+') || val_str.starts_with('-') {
                    let (cur, max) = mon.get_vcp(0x12)?;
                    parse_relative_or_abs(val_str, cur, max)?
                } else {
                    parse_u32(val_str)?
                };
                acer::contrast(mon, new_val)?;
                if show_osd {
                    let (_, max) = mon.get_vcp(0x12).unwrap_or((new_val, 100));
                    show_osd_banner("Contrast", new_val, max);
                }
                Ok(())
            })?;
            Ok("OK".to_string())
        }

        "fade" => {
            if args.len() < 3 {
                return Err("Usage: acer_monitor_cli fade <brightness|contrast|volume> <from> <to> [duration_ms] [specifier]".to_string());
            }
            let feature = args[0].to_ascii_lowercase();
            let code = match feature.as_str() {
                "brightness" => 0x10u8,
                "contrast" => 0x12u8,
                "volume" => 0x62u8,
                other => return Err(format!("Unsupported fade feature '{other}'. Use brightness, contrast, or volume.")),
            };

            let start_val = parse_u32(&args[1])?;
            let end_val = parse_u32(&args[2])?;
            let duration_ms = if args.len() >= 4 { parse_u32(&args[3])? as u64 } else { 1000u64 };
            let spec = parse_optional_specifier(&args[4..]);

            with_monitor(spec, |mon| acer::fade_vcp(mon, code, start_val, end_val, duration_ms))?;
            Ok("Fade transition complete.".to_string())
        }

        "input" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli input <auto|dp|hdmi1|hdmi2|next|raw_value> [specifier]".to_string());
            }
            let val_str = &args[0];
            let spec = parse_optional_specifier(&args[1..]);
            if val_str.eq_ignore_ascii_case("next") {
                with_monitor(spec, |mon| {
                    let (cur, _) = mon.get_vcp(0x60)?;
                    let next_input = match cur {
                        0x01 => 0x0F,
                        0x0F => 0x11,
                        0x11 => 0x12,
                        _ => 0x01,
                    };
                    acer::input(mon, next_input)
                })?;
            } else {
                let value = parse_input_source(val_str)?;
                with_monitor(spec, |mon| acer::input(mon, value))?;
            }
            Ok("OK".to_string())
        }

        "info" => {
            let is_json = args.iter().any(|a| a == "--json" || a == "-j");
            let filtered: Vec<String> = args.into_iter().filter(|a| a != "--json" && a != "-j").collect();
            let spec = parse_optional_specifier(&filtered);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                mon.update_capabilities()?;
                if is_json {
                    out = format_monitor_info_json(mon)?;
                } else {
                    out = format_monitor_info(mon)?;
                }
                Ok(())
            })?;
            Ok(out)
        }

        "scan" => {
            let is_json = args.iter().any(|a| a == "--json" || a == "-j");
            let filtered: Vec<String> = args.into_iter().filter(|a| a != "--json" && a != "-j").collect();
            let spec = parse_optional_specifier(&filtered);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                mon.update_capabilities()?;
                if is_json {
                    out = format_vcp_scan_json(mon)?;
                } else {
                    out = format_vcp_scan(mon)?;
                }
                Ok(())
            })?;
            Ok(out)
        }



        "getbank" => {
            if args.len() < 2 {
                return Err("Usage: acer_monitor_cli getbank <e0|e7|e9> <selector> [specifier]".to_string());
            }
            let bank = parse_bank(&args[0])?;
            let selector = parse_u32(&args[1])?;
            let spec = parse_optional_specifier(&args[2..]);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_raw_bank(mon, bank, selector)?;
                out = format!("Bank 0x{bank:02X} [selector={selector}]: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-bluelight" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_blue_light(mon)?;
                out = format!("BlueLight: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-gamma" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_gamma(mon)?;
                out = format!("Gamma: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-colortemp" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_color_temp(mon)?;
                out = format!("ColorTemp: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-od" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_overdrive(mon)?;
                out = format!("OverDrive: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-aim" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_aim_type(mon)?;
                out = format!("AimPoint: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-blackboost" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_black_boost(mon)?;
                out = format!("BlackBoost: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-keylock" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_key_lock(mon)?;
                out = format!("KeyLock: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "get-indicator" => {
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (cur, max) = acer::get_power_indicator(mon)?;
                out = format!("PowerIndicator: current={cur} max={max}");
                Ok(())
            })?;
            Ok(out)
        }

        "profile" => {
            if args.len() < 2 {
                return Err("Usage: acer_monitor_cli profile <save|load> <path.json> [specifier]".to_string());
            }
            let action = args[0].to_ascii_lowercase();
            let path = &args[1];
            let spec = parse_optional_specifier(&args[2..]);
            match action.as_str() {
                "save" => {
                    with_monitor(spec, |mon| profile_save(mon, path))?;
                    Ok(format!("Saved profile to '{path}'"))
                }
                "load" => {
                    with_monitor(spec, |mon| profile_load(mon, path))?;
                    Ok(format!("Loaded profile from '{path}'"))
                }
                _ => Err(format!("Unknown profile action '{action}'. Use 'save' or 'load'.")),
            }
        }


        _ => Err(format!("Unknown command: {cmd}\nUse 'acer_monitor_cli --help' for usage.")),
    }
}

fn install_systemd_service() -> Result<String, String> {
    let service_content = r#"[Unit]
Description=Acer Monitor DDC/CI Control Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/acer_monitor_cli server
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
"#;

    let home = std::env::var("HOME").map_err(|_| "Could not determine HOME directory.".to_string())?;
    let service_dir = std::path::Path::new(&home).join(".config/systemd/user");
    std::fs::create_dir_all(&service_dir).map_err(|e| format!("Failed to create service dir: {e}"))?;
    let service_path = service_dir.join("acer-monitor.service");
    std::fs::write(&service_path, service_content).map_err(|e| format!("Failed to write service file: {e}"))?;

    let _ = std::process::Command::new("systemctl")
        .args(&["--user", "daemon-reload"])
        .status();

    Ok(format!(
        "Systemd user service installed at '{}'.\nEnable and start with:\n  systemctl --user enable --now acer-monitor",
        service_path.display()
    ))
}

fn install_desktop_entry() -> Result<String, String> {
    #[cfg(unix)]
    {
        let desktop_content = r#"[Desktop Entry]
Name=Acer Monitor Control
Comment=Control Acer Monitor Brightness, Presets, and Solar Schedules
Exec=/usr/local/bin/acer_monitor_cli brightness 50 --osd
Icon=display
Terminal=false
Type=Application
Categories=Utility;Settings;HardwareSettings;
Actions=DayMode;NightMode;GamingPreset;EcoPreset;

[Desktop Action DayMode]
Name=Apply Day Mode
Exec=/usr/local/bin/acer_monitor_cli solar-schedule --lat 28.61 --lon 77.20 --day-b 90

[Desktop Action NightMode]
Name=Apply Night Mode
Exec=/usr/local/bin/acer_monitor_cli brightness 15 --osd

[Desktop Action GamingPreset]
Name=Gaming HDR Mode
Exec=/usr/local/bin/acer_monitor_cli preset hdr

[Desktop Action EcoPreset]
Name=ECO Power Saving Mode
Exec=/usr/local/bin/acer_monitor_cli preset eco
"#;

        let home = std::env::var("HOME").map_err(|_| "Could not determine HOME directory.".to_string())?;
        let app_dir = std::path::Path::new(&home).join(".local/share/applications");
        std::fs::create_dir_all(&app_dir).map_err(|e| format!("Failed to create applications dir: {e}"))?;
        let file_path = app_dir.join("acer-monitor-control.desktop");
        std::fs::write(&file_path, desktop_content).map_err(|e| format!("Failed to write desktop entry: {e}"))?;

        Ok(format!(
            "Installed Desktop Entry at '{}'.\n'Acer Monitor Control' is now available in your Application Launcher menu!",
            file_path.display()
        ))
    }

    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| "Could not determine APPDATA directory.".to_string())?;
        let start_menu = std::path::Path::new(&appdata).join("Microsoft\\Windows\\Start Menu\\Programs");
        std::fs::create_dir_all(&start_menu).map_err(|e| format!("Failed to create Start Menu dir: {e}"))?;
        let link_path = start_menu.join("Acer Monitor Control.bat");

        let bat_content = "@echo off\r\nacer_monitor_cli.exe brightness 50 --osd\r\n";
        std::fs::write(&link_path, bat_content).map_err(|e| format!("Failed to write Start Menu shortcut: {e}"))?;

        Ok(format!(
            "Installed Windows Start Menu shortcut at '{}'.\n'Acer Monitor Control' is now available in your Start Menu!",
            link_path.display()
        ))
    }
}

fn sync_monitors(master_spec: Option<&str>) -> Result<String, String> {
    let mut set = MonitorSet::enumerate()?;
    if set.monitors_mut().len() < 2 {
        return Ok("Only one monitor detected; no secondary displays to sync.".to_string());
    }

    let (master_brightness, master_contrast) = {
        let master_mon = set.pick_mut_by_specifier(master_spec)?;
        let (b, _) = master_mon.get_vcp(0x10)?;
        let (c, _) = master_mon.get_vcp(0x12)?;
        (b, c)
    };

    let mut synced_count = 0;
    for mon in set.monitors_mut() {
        let _ = acer::brightness(mon, master_brightness);
        let _ = acer::contrast(mon, master_contrast);
        synced_count += 1;
    }

    Ok(format!("Synchronized brightness ({master_brightness}) and contrast ({master_contrast}) across {synced_count} monitors."))
}

fn parse_flag_val(args: &[String], flag: &str) -> Option<f64> {
    let idx = args.iter().position(|a| a == flag)?;
    args.get(idx + 1).and_then(|v| v.parse::<f64>().ok())
}

fn parse_flag_u32(args: &[String], flag: &str) -> Option<u32> {
    let idx = args.iter().position(|a| a == flag)?;
    args.get(idx + 1).and_then(|v| v.parse::<u32>().ok())
}

fn parse_flag_u64(args: &[String], flag: &str) -> Option<u64> {
    let idx = args.iter().position(|a| a == flag)?;
    args.get(idx + 1).and_then(|v| v.parse::<u64>().ok())
}

fn parse_flag_str(args: &[String], flag: &str) -> Option<String> {
    let idx = args.iter().position(|a| a == flag)?;
    args.get(idx + 1).cloned()
}

fn with_monitor<T, F>(spec: Option<&str>, mut f: F) -> Result<T, String>
where
    F: FnMut(&mut crate::monitor::Monitor) -> Result<T, String>,
{
    let mut set = MonitorSet::enumerate()?;
    if let Some("all") = spec {
        let mut last = None;
        for mon in set.monitors_mut() {
            last = Some(f(mon)?);
        }
        last.ok_or_else(|| "No monitors found".to_string())
    } else {
        let mon = set.pick_mut_by_specifier(spec)?;
        f(mon)
    }
}

fn parse_optional_specifier(args: &[String]) -> Option<&str> {
    args.first().map(|s| s.as_str())
}

fn parse_relative_or_abs(s: &str, current: u32, max: u32) -> Result<u32, String> {
    if let Some(rest) = s.strip_prefix('+') {
        let delta: u32 = rest.parse().map_err(|e| format!("Invalid relative adjustment '{s}': {e}"))?;
        Ok((current + delta).min(max))
    } else if let Some(rest) = s.strip_prefix('-') {
        let delta: u32 = rest.parse().map_err(|e| format!("Invalid relative adjustment '{s}': {e}"))?;
        Ok(current.saturating_sub(delta))
    } else {
        parse_u32(s)
    }
}

fn parse_on_off(s: &str) -> Result<bool, String> {
    match s.to_ascii_lowercase().as_str() {
        "on" | "1" | "true" | "yes" => Ok(true),
        "off" | "0" | "false" | "no" => Ok(false),
        _ => Err(format!("Expected on/off, got '{s}'")),
    }
}

fn parse_bluelight(s: &str) -> Result<u32, String> {
    match s.to_ascii_lowercase().as_str() {
        "0" | "off" | "standard" => Ok(0),
        "1" | "50" | "50%" => Ok(1),
        "2" | "60" | "60%" => Ok(2),
        "3" | "70" | "70%" => Ok(3),
        "4" | "80" | "80%" => Ok(4),
        _ => Err("Use 0/off/standard, 50, 60, 70, or 80".into()),
    }
}

fn parse_gamma(s: &str) -> Result<u32, String> {
    match s.trim() {
        "1.8" | "18" => Ok(0),
        "2.0" | "20" => Ok(1),
        "2.2" | "22" | "default" | "standard" => Ok(2),
        "2.4" | "24" => Ok(3),
        "2.6" | "26" => Ok(4),
        _ => Err("Use 1.8 (18), 2.0 (20), 2.2 (22), 2.4 (24), or 2.6 (26)".into()),
    }
}

fn parse_color_temp(s: &str) -> Result<u32, String> {
    match s.to_ascii_lowercase().as_str() {
        "warm" => Ok(0),
        "normal" | "std" | "standard" => Ok(1),
        "cool" => Ok(2),
        "bluelight" | "blue-light" | "blue" => Ok(3),
        "user" | "custom" => Ok(4),
        other => parse_u32(other),
    }
}

fn parse_input_source(s: &str) -> Result<u32, String> {
    match s.to_ascii_lowercase().replace([' ', '_', '-'], "").as_str() {
        "auto" => Ok(0x01),
        "dp" | "displayport" => Ok(0x0F),
        "hdmi1" => Ok(0x11),
        "hdmi2" => Ok(0x12),
        other => parse_u32(other),
    }
}

fn input_source_name(v: u32) -> String {
    match v {
        0x01 => "Auto".to_string(),
        0x0F => "DisplayPort".to_string(),
        0x11 => "HDMI1".to_string(),
        0x12 => "HDMI2".to_string(),
        other => format!("0x{other:02X}"),
    }
}

fn format_monitor_info(mon: &mut crate::monitor::Monitor) -> Result<String, String> {
    let mut out = String::new();
    use std::fmt::Write as _;

    let caps = mon.capabilities.clone();
    let _ = writeln!(out, "Monitor: {}", mon.description);
    let _ = writeln!(out);
    let _ = writeln!(out, "Feature flags:");
    let _ = writeln!(out, "  Power          {}", caps.acer.power);
    let _ = writeln!(out, "  Brightness     {}", caps.acer.brightness);
    let _ = writeln!(out, "  Contrast       {}", caps.acer.contrast);
    let _ = writeln!(out, "  Volume         {}", true);
    let _ = writeln!(out, "  Mute           {}", true);
    let _ = writeln!(out, "  Input          {}", true);
    let _ = writeln!(out, "  RefreshNum     {}", caps.acer.refresh_num);
    let _ = writeln!(out, "  OverDrive      {}", caps.acer.overdrive);
    let _ = writeln!(out, "  AimPoint       {}", caps.acer.aim_point);
    let _ = writeln!(out, "  BlueLight      {}", caps.acer.blue_light);
    let _ = writeln!(out, "  Gamma          {}", caps.acer.gamma);
    let _ = writeln!(out, "  ColorTemp      {}", caps.acer.color_temp);
    let _ = writeln!(out, "  DisplayMode    {}", caps.acer.display_mode);
    let _ = writeln!(out, "  ColorSpace     {}", caps.acer.color_space);
    let _ = writeln!(out, "  BlackBoost     {}", caps.acer.black_boost);
    let _ = writeln!(out);
    let _ = writeln!(out, "Current values (best-effort):");

    for (name, code) in [
        ("Brightness", 0x10u8),
        ("Contrast", 0x12u8),
        ("Volume", 0x62u8),
        ("Mute", 0x8Du8),
        ("Power", 0xD6u8),
        ("DisplayMode", 0xE2u8),
        ("BlackBoost", 0xE5u8),
        ("ColorSpace", 0xEAu8),
    ] {
        match mon.get_vcp(code) {
            Ok((cur, max)) => { let _ = writeln!(out, "  {name:<12} current={cur} max={max}"); }
            Err(e) => { let _ = writeln!(out, "  {name:<12} <unsupported> ({e})"); }
        }
    }

    match mon.get_vcp(0x60) {
        Ok((cur, max)) => { let _ = writeln!(out, "  {:<12} current={} ({}) max={}", "Input", cur, input_source_name(cur), max); }
        Err(e) => { let _ = writeln!(out, "  {:<12} <unsupported> ({e})", "Input"); }
    }

    Ok(out)
}

fn format_monitor_info_json(mon: &mut crate::monitor::Monitor) -> Result<String, String> {
    let caps = mon.capabilities.clone();
    let desc_escaped = mon.description.replace('"', "\\\"");

    let mut out = String::new();
    use std::fmt::Write as _;

    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"description\": \"{desc_escaped}\",");
    let _ = writeln!(out, "  \"feature_flags\": {{");
    let _ = writeln!(out, "    \"power\": {},", caps.acer.power);
    let _ = writeln!(out, "    \"brightness\": {},", caps.acer.brightness);
    let _ = writeln!(out, "    \"contrast\": {},", caps.acer.contrast);
    let _ = writeln!(out, "    \"volume\": true,");
    let _ = writeln!(out, "    \"mute\": true,");
    let _ = writeln!(out, "    \"input\": true,");
    let _ = writeln!(out, "    \"refresh_num\": {},", caps.acer.refresh_num);
    let _ = writeln!(out, "    \"overdrive\": {},", caps.acer.overdrive);
    let _ = writeln!(out, "    \"aim_point\": {},", caps.acer.aim_point);
    let _ = writeln!(out, "    \"blue_light\": {},", caps.acer.blue_light);
    let _ = writeln!(out, "    \"gamma\": {},", caps.acer.gamma);
    let _ = writeln!(out, "    \"color_temp\": {},", caps.acer.color_temp);
    let _ = writeln!(out, "    \"display_mode\": {},", caps.acer.display_mode);
    let _ = writeln!(out, "    \"color_space\": {},", caps.acer.color_space);
    let _ = writeln!(out, "    \"black_boost\": {}", caps.acer.black_boost);
    let _ = writeln!(out, "  }},");
    let _ = writeln!(out, "  \"current_values\": {{");

    let mut first = true;
    for (name, code) in [
        ("brightness", 0x10u8),
        ("contrast", 0x12u8),
        ("volume", 0x62u8),
        ("mute", 0x8Du8),
        ("power", 0xD6u8),
        ("display_mode", 0xE2u8),
        ("black_boost", 0xE5u8),
        ("color_space", 0xEAu8),
    ] {
        if !first {
            let _ = writeln!(out, ",");
        }
        first = false;
        match mon.get_vcp(code) {
            Ok((cur, max)) => { let _ = write!(out, "    \"{name}\": {{\"current\": {cur}, \"max\": {max}}}"); }
            Err(_) => { let _ = write!(out, "    \"{name}\": null"); }
        }
    }

    match mon.get_vcp(0x60) {
        Ok((cur, max)) => {
            let name_str = input_source_name(cur);
            let _ = writeln!(out, ",");
            let _ = write!(out, "    \"input\": {{\"current\": {cur}, \"name\": \"{name_str}\", \"max\": {max}}}");
        }
        Err(_) => {
            let _ = writeln!(out, ",");
            let _ = write!(out, "    \"input\": null");
        }
    }
    let _ = writeln!(out, "\n  }}");
    let _ = writeln!(out, "}}");

    Ok(out)
}

fn format_vcp_scan(mon: &mut crate::monitor::Monitor) -> Result<String, String> {
    let mut out = String::new();
    use std::fmt::Write as _;

    let _ = writeln!(out, "Probe results:");
    for code in [
        0x04u8, 0x06, 0x08, 0x0B, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A,
        0x60, 0x62, 0x8D, 0xD6, 0xE2, 0xE5, 0xE7, 0xE8, 0xE9, 0xEA
    ] {
        match mon.get_vcp(code) {
            Ok((cur, max)) => { let _ = writeln!(out, "  0x{code:02X}: current={cur} max={max}"); }
            Err(_) => { let _ = writeln!(out, "  0x{code:02X}: <no read response>"); }
        }
    }
    Ok(out)
}

fn format_vcp_scan_json(mon: &mut crate::monitor::Monitor) -> Result<String, String> {
    let mut out = String::new();
    use std::fmt::Write as _;

    let desc_escaped = mon.description.replace('"', "\\\"");
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"description\": \"{desc_escaped}\",");
    let _ = writeln!(out, "  \"probe_results\": {{");

    let codes = [
        0x04u8, 0x06, 0x08, 0x0B, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A,
        0x60, 0x62, 0x8D, 0xD6, 0xE2, 0xE5, 0xE7, 0xE8, 0xE9, 0xEA
    ];

    for (i, &code) in codes.iter().enumerate() {
        let comma = if i + 1 < codes.len() { "," } else { "" };
        match mon.get_vcp(code) {
            Ok((cur, max)) => { let _ = writeln!(out, "    \"0x{code:02X}\": {{\"current\": {cur}, \"max\": {max}}}{comma}"); }
            Err(_) => { let _ = writeln!(out, "    \"0x{code:02X}\": null{comma}"); }
        }
    }

    let _ = writeln!(out, "  }}");
    let _ = writeln!(out, "}}");
    Ok(out)
}

fn profile_save(mon: &mut crate::monitor::Monitor, path: &str) -> Result<(), String> {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!("  \"description\": \"{}\",\n", mon.description.replace('"', "\\\"")));

    let mut fields = Vec::new();
    if let Ok((v, _)) = mon.get_vcp(0x10) { fields.push(format!("  \"brightness\": {v}")); }
    if let Ok((v, _)) = mon.get_vcp(0x12) { fields.push(format!("  \"contrast\": {v}")); }
    if let Ok((v, _)) = mon.get_vcp(0x62) { fields.push(format!("  \"volume\": {v}")); }
    if let Ok((v, _)) = mon.get_vcp(0x8D) { fields.push(format!("  \"mute\": {}", if v == 1 { "true" } else { "false" })); }
    if let Ok((v, _)) = mon.get_vcp(0xE5) { fields.push(format!("  \"black_boost\": {v}")); }
    if let Ok((v, _)) = mon.get_vcp(0xE2) { fields.push(format!("  \"display_mode\": {v}")); }
    if let Ok((v, _)) = acer::get_blue_light(mon) { fields.push(format!("  \"blue_light\": {v}")); }
    if let Ok((v, _)) = acer::get_gamma(mon) { fields.push(format!("  \"gamma\": {v}")); }
    if let Ok((v, _)) = acer::get_color_temp(mon) { fields.push(format!("  \"color_temp\": {v}")); }
    if let Ok((v, _)) = acer::get_overdrive(mon) { fields.push(format!("  \"overdrive\": {v}")); }

    json.push_str(&fields.join(",\n"));
    json.push_str("\n}\n");

    std::fs::write(path, json).map_err(|e| format!("Failed to write profile to '{path}': {e}"))?;
    Ok(())
}

fn profile_load(mon: &mut crate::monitor::Monitor, path: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile from '{path}': {e}"))?;

    let parse_field = |key: &str| -> Option<u32> {
        let pattern = format!("\"{key}\":");
        let idx = content.find(&pattern)?;
        let rest = content[idx + pattern.len()..].trim();
        let val_str: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
        val_str.parse().ok()
    };

    let parse_bool_field = |key: &str| -> Option<bool> {
        let pattern = format!("\"{key}\":");
        let idx = content.find(&pattern)?;
        let rest = content[idx + pattern.len()..].trim();
        if rest.starts_with("true") { Some(true) }
        else if rest.starts_with("false") { Some(false) }
        else { None }
    };

    if let Some(v) = parse_field("brightness") { acer::brightness(mon, v)?; }
    if let Some(v) = parse_field("contrast") { acer::contrast(mon, v)?; }
    if let Some(v) = parse_field("volume") { acer::volume(mon, v)?; }
    if let Some(b) = parse_bool_field("mute") { acer::mute(mon, b)?; }
    if let Some(v) = parse_field("black_boost") { acer::black_boost(mon, v)?; }
    if let Some(v) = parse_field("display_mode") { acer::display_mode(mon, v)?; }
    if let Some(v) = parse_field("blue_light") { acer::blue_light(mon, v)?; }
    if let Some(v) = parse_field("gamma") { acer::gamma(mon, v)?; }
    if let Some(v) = parse_field("color_temp") { acer::color_temp(mon, v)?; }
    if let Some(v) = parse_field("overdrive") { acer::overdrive(mon, v)?; }

    Ok(())
}

fn parse_bank(s: &str) -> Result<u8, String> {
    match s.to_ascii_lowercase().as_str() {
        "e0" => Ok(0xE0),
        "e7" => Ok(0xE7),
        "e9" => Ok(0xE9),
        _ => Err("Use e0, e7, or e9".into()),
    }
}

pub fn vcp_name(code: u8) -> &'static str {
    match code {
        0x04 => "Restore Factory Defaults",
        0x05 => "Restore Factory Brightness/Contrast",
        0x06 => "Restore Factory Geometry",
        0x08 => "Restore Factory Color Defaults",
        0x0A => "Restore Factory TV Defaults",
        0x0B => "Color Temperature Increment",
        0x0C => "Color Temperature Request",
        0x0E => "Clock",
        0x10 => "Brightness (Luminance)",
        0x12 => "Contrast",
        0x14 => "Select Color Preset",
        0x16 => "Red Video Gain",
        0x18 => "Green Video Gain",
        0x1A => "Blue Video Gain",
        0x1C => "Focus",
        0x1E => "Auto Setup",
        0x20 => "Horizontal Position",
        0x22 => "Horizontal Size",
        0x24 => "Horizontal Pincushion",
        0x26 => "Horizontal Pincushion Balance",
        0x28 => "Horizontal Convergence R/B",
        0x2A => "Horizontal Convergence M/G",
        0x2C => "Horizontal Linearity",
        0x2E => "Horizontal Linearity Balance",
        0x30 => "Vertical Position",
        0x32 => "Vertical Size",
        0x34 => "Vertical Pincushion",
        0x36 => "Vertical Pincushion Balance",
        0x38 => "Vertical Convergence R/B",
        0x3A => "Vertical Convergence M/G",
        0x3C => "Vertical Linearity",
        0x3E => "Vertical Linearity Balance",
        0x40 => "Parallelogram Distortion",
        0x42 => "Trapezoid Distortion",
        0x44 => "Tilt (Rotation)",
        0x46 => "Top Corner Flare",
        0x48 => "Top Corner Hook",
        0x4A => "Bottom Corner Flare",
        0x4C => "Bottom Corner Hook",
        0x52 => "Active Control",
        0x54 => "Ambient Light Sensor",
        0x56 => "Horizontal Moire",
        0x58 => "Vertical Moire",
        0x59 => "6-Axis Saturation (Red)",
        0x5A => "6-Axis Saturation (Yellow)",
        0x5B => "6-Axis Saturation (Green)",
        0x5C => "6-Axis Saturation (Cyan)",
        0x5D => "6-Axis Saturation (Blue)",
        0x5E => "6-Axis Saturation (Magenta)",
        0x5F => "6-Axis Hue Control",
        0x60 => "Input Source / Select",
        0x62 => "Audio Speaker Volume",
        0x63 => "Speaker Select",
        0x64 => "Audio Microphone Volume",
        0x66 => "Ambient Light Sensor",
        0x6C => "Black Level (Red)",
        0x6E => "Black Level (Green)",
        0x70 => "Black Level (Blue)",
        0x72 => "Gamma",
        0x7C => "Adjust Focal Plane",
        0x86 => "Display Scaling / Aspect Ratio",
        0x87 => "Sharpness",
        0x8A => "Velocity Scan Modulation",
        0x8D => "Audio Mute",
        0x8E => "Window Swap",
        0x8F => "Window Selection",
        0x90 => "Window Control",
        0x92 => "Window Background",
        0x9B => "6-Axis Hue (Red)",
        0x9C => "6-Axis Hue (Yellow)",
        0x9D => "6-Axis Hue (Green)",
        0x9E => "6-Axis Hue (Cyan)",
        0x9F => "6-Axis Hue (Blue)",
        0xA0 => "6-Axis Hue (Magenta)",
        0xAA => "Screen Orientation",
        0xAC => "Horizontal Frequency",
        0xAE => "Vertical Frequency",
        0xB0 => "Settings / Setup",
        0xB2 => "Flat Panel Sub-Pixel Layout",
        0xB6 => "Display Technology / HDR",
        0xC0 => "Display Usage Time",
        0xC6 => "Application Enable Key",
        0xC8 => "Display Controller ID",
        0xC9 => "Display Firmware Level",
        0xCA => "OSD Language",
        0xCC => "OSD / Power Indicator / User Controls",
        0xD4 => "Stereo Video Mode",
        0xD6 => "Power Mode / DPMS",
        0xDC => "Display Mode / Preset",
        0xDF => "VCP Version",
        0xE0 => "Manufacturer / Bank Selector",
        0xE1 => "Manufacturer / Bank Value",
        0xE2 => "Manufacturer / Display Mode",
        0xE5 => "Manufacturer / Black Boost",
        0xE7 => "Manufacturer / Color Bank Selector",
        0xE8 => "Manufacturer / Color Bank Value",
        0xE9 => "Manufacturer / Calibration Bank Selector",
        0xEA => "Manufacturer / Calibration Bank Value",
        0xEB..=0xFF => "Manufacturer Specific",
        _ => "Unknown VCP Feature",
    }
}

fn current_time_str() -> String {
    if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let total_secs = duration.as_secs();
        let secs = total_secs % 60;
        let mins = (total_secs / 60) % 60;
        let hours = (total_secs / 3600) % 24;
        format!("{hours:02}:{mins:02}:{secs:02}")
    } else {
        "00:00:00".to_string()
    }
}

fn run_watch_vcp(args: Vec<String>) -> Result<String, String> {
    let mut is_all = false;
    let mut is_json = false;
    let mut poll_interval_ms: u64 = 500;
    let mut explicit_codes = Vec::new();
    let mut specifier: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--all" || arg == "-a" {
            is_all = true;
        } else if arg == "--json" || arg == "-j" {
            is_json = true;
        } else if let Some(val) = arg.strip_prefix("--interval=") {
            poll_interval_ms = val.parse().map_err(|e| format!("Invalid interval '{val}': {e}"))?;
        } else if (arg == "--interval" || arg == "-i") && i + 1 < args.len() {
            poll_interval_ms = args[i + 1].parse().map_err(|e| format!("Invalid interval '{}': {e}", args[i + 1]))?;
            i += 1;
        } else if let Some(val) = arg.strip_prefix("--monitor=") {
            specifier = Some(val.to_string());
        } else if (arg == "--monitor" || arg == "-m") && i + 1 < args.len() {
            specifier = Some(args[i + 1].clone());
            i += 1;
        } else if let Ok(code) = parse_u32(arg) {
            if code <= 0xFF {
                explicit_codes.push(code as u8);
            } else {
                return Err(format!("VCP code 0x{code:X} exceeds 0xFF"));
            }
        } else {
            if specifier.is_none() {
                specifier = Some(arg.clone());
            }
        }
        i += 1;
    }

    let mut set = MonitorSet::enumerate()?;
    let mon = set.pick_mut_by_specifier(specifier.as_deref())?;

    let candidate_codes: Vec<u8> = if !explicit_codes.is_empty() {
        explicit_codes
    } else if is_all {
        (0x00u8..=0xFFu8).collect()
    } else {
        let _ = mon.update_capabilities();
        let mut codes_set = mon.capabilities.vcp_codes.clone();
        for code in [
            0x04u8, 0x06, 0x08, 0x0B, 0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1E,
            0x20, 0x30, 0x52, 0x54, 0x60, 0x62, 0x6C, 0x6E, 0x70, 0x72, 0x86, 0x87, 0x8D,
            0x90, 0xAA, 0xAC, 0xAE, 0xB0, 0xB6, 0xC6, 0xC8, 0xC9, 0xCA, 0xCC, 0xD4, 0xD6,
            0xDC, 0xDF, 0xE0, 0xE1, 0xE2, 0xE5, 0xE7, 0xE8, 0xE9, 0xEA,
        ] {
            codes_set.insert(code);
        }
        codes_set.into_iter().collect()
    };

    let mut active_codes: std::collections::BTreeMap<u8, (u32, u32)> = std::collections::BTreeMap::new();
    for &code in &candidate_codes {
        if let Ok((cur, max)) = mon.get_vcp(code) {
            active_codes.insert(code, (cur, max));
        }
    }

    if active_codes.is_empty() {
        return Err(format!(
            "No readable VCP codes found on '{}'. Ensure DDC/CI is enabled in the monitor's OSD menu.",
            mon.description
        ));
    }

    if is_json {
        let items: Vec<String> = active_codes
            .iter()
            .map(|(&c, &(cur, max))| {
                format!(
                    "{{\"code\":\"0x{c:02X}\",\"name\":\"{}\",\"current\":{cur},\"max\":{max}}}",
                    vcp_name(c)
                )
            })
            .collect();
        println!(
            "{{\"event\":\"init\",\"monitor\":\"{}\",\"interval_ms\":{poll_interval_ms},\"active_codes\":[{}]}}",
            mon.description.replace('"', "\\\""),
            items.join(",")
        );
    } else {
        println!("========================================================================");
        println!("[amctl] Real-Time VCP Watcher - Monitoring '{}'", mon.description);
        println!("========================================================================");
        println!(
            "Active readable VCP codes ({} detected, polling every {}ms):",
            active_codes.len(),
            poll_interval_ms
        );
        for (&code, &(cur, max)) in &active_codes {
            let name = vcp_name(code);
            println!("  0x{code:02X} ({name:<34}) current: {cur:<4} max: {max}");
        }
        println!("------------------------------------------------------------------------");
        println!("Press buttons on your monitor / change OSD settings to detect changes.");
        println!("Press Ctrl+C to stop.");
        println!("------------------------------------------------------------------------");
    }

    loop {
        std::thread::sleep(std::time::Duration::from_millis(poll_interval_ms));

        let current_keys: Vec<u8> = active_codes.keys().copied().collect();
        for code in current_keys {
            let old_entry = active_codes.get(&code).copied();
            if let Some((old_cur, old_max)) = old_entry {
                if let Ok((new_cur, new_max)) = mon.get_vcp(code) {
                    if new_cur != old_cur || new_max != old_max {
                        let name = vcp_name(code);
                        let timestamp = current_time_str();
                        if is_json {
                            println!(
                                "{{\"event\":\"change\",\"timestamp\":\"{timestamp}\",\"code\":\"0x{code:02X}\",\"name\":\"{name}\",\"old\":{old_cur},\"new\":{new_cur},\"max\":{new_max}}}"
                            );
                        } else {
                            println!(
                                "[{timestamp}] VCP CHANGE: 0x{code:02X} ({name}) changed: {old_cur} -> {new_cur} (max: {new_max})"
                            );
                        }
                        active_codes.insert(code, (new_cur, new_max));
                    }
                }
            }
        }
    }
}

fn run_record_vcp(args: Vec<String>) -> Result<String, String> {
    use std::io::{self, BufRead, Write};
    let spec = parse_optional_specifier(&args);
    let mut set = MonitorSet::enumerate()?;
    let mon = set.pick_mut_by_specifier(spec)?;

    println!("========================================================================");
    println!("[amctl] Hardware VCP Register Change Recorder");
    println!("Target Monitor: '{}'", mon.description);
    println!("========================================================================");

    let mut baseline: std::collections::BTreeMap<u8, (u32, u32)> = std::collections::BTreeMap::new();
    println!("[1/2] Scanning all readable VCP codes (0x00 - 0xFF) for baseline snapshot...");
    for code in 0x00u8..=0xFFu8 {
        if let Ok((cur, max)) = mon.get_vcp(code) {
            baseline.insert(code, (cur, max));
        }
    }
    println!("[+] Recorded baseline snapshot with {} active VCP registers.", baseline.len());

    loop {
        println!("------------------------------------------------------------------------");
        println!(">>> STEP 1: Change the target setting on your monitor OSD now");
        println!("           (e.g., adjust Red/Green/Blue Gain, Color Temp, or Preset).");
        println!(">>> STEP 2: Press [ENTER] in this terminal to detect what changed.");
        print!("\nPress [ENTER] to compare (or type 'q' and Enter to exit): ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let stdin = io::stdin();
        let _ = stdin.lock().read_line(&mut input);

        if input.trim().eq_ignore_ascii_case("q") {
            println!("Exited VCP recorder.");
            break;
        }

        println!("\n[2/2] Scanning monitor registers & comparing with baseline...");
        let mut changes = Vec::new();

        for code in 0x00u8..=0xFFu8 {
            let old = baseline.get(&code).copied();
            let new = mon.get_vcp(code).ok();

            match (old, new) {
                (Some((old_c, old_m)), Some((new_c, new_m))) => {
                    if old_c != new_c || old_m != new_m {
                        changes.push((code, Some(old_c), new_c, new_m));
                        baseline.insert(code, (new_c, new_m));
                    }
                }
                (None, Some((new_c, new_m))) => {
                    changes.push((code, None, new_c, new_m));
                    baseline.insert(code, (new_c, new_m));
                }
                _ => {}
            }
        }

        if changes.is_empty() {
            println!("------------------------------------------------------------------------");
            println!("[!] No register changes detected since baseline.");
            println!("Tips:");
            println!("  1. Some monitors only commit DDC/CI changes when you close the OSD menu.");
            println!("  2. Close the OSD on your monitor and press [ENTER] again to re-check.");
        } else {
            println!("========================================================================");
            println!("🎯 DETECTED HARDWARE VCP CHANGES ({} register(s)):", changes.len());
            println!("========================================================================");
            for (code, old_opt, new_c, new_m) in changes {
                let name = vcp_name(code);
                if let Some(old_c) = old_opt {
                    println!("  • Register 0x{code:02X} ({name:<30}): {old_c} -> {new_c} (max: {new_m})");
                } else {
                    println!("  • Register 0x{code:02X} ({name:<30}): [New] {new_c} (max: {new_m})");
                }
            }
            println!("========================================================================");
        }
    }

    Ok("VCP Recording complete.".to_string())
}

fn get_waybar_config_string() -> String {
    r#"{
  "custom/acer_monitor": {
    "format": "󰍹 {percentage}%",
    "exec": "acer_monitor_cli get 0x10 | awk '{print $2}' | cut -d= -f2",
    "interval": 5,
    "on-scroll-up": "acer_monitor_cli brightness +5 --osd",
    "on-scroll-down": "acer_monitor_cli brightness -5 --osd",
    "on-click": "acer_monitor_cli mute toggle --osd",
    "tooltip": false
  }
}"#.to_string()
}

fn get_completions_string(shell: &str) -> String {
    match shell.to_ascii_lowercase().as_str() {
        "zsh" => r#"#compdef acer_monitor_cli
_acer_monitor_cli() {
  local -a commands
  commands=(
    'list:List connected DDC/CI monitors'
    'caps:Query MCCS display capabilities'
    'get:Get raw VCP code'
    'set:Set raw VCP code'
    'brightness:Adjust display brightness'
    'contrast:Adjust contrast'
    'volume:Adjust audio volume'
    'fade:Smoothly transition brightness/contrast'
    'preset:Apply one-touch mode preset (gaming, reading, movie, eco)'
    'unlock:Unlock front panel OSD menu buttons'
    'balance:Balance secondary monitors with brightness offset'
    'diag:Output complete diagnostic report'
    'idle-dimmer:Smart Inactivity Idle Dimmer with Video Inhibit'
    'auto-profile:Auto-switch profiles based on running app/game'
    'energy:Estimate power consumption and electricity cost'
    'tray:Run native System Tray notification area application'
    'watch-vcp:Monitor real-time VCP code value changes'
    'watch-monitors:Monitor display hotplug events'
    'test-pattern:Display diagnostic test patterns'
    'blackboost:Adjust Black Boost'
    'mute:Mute/Unmute/Toggle audio'
    'input:Select input source (DP, HDMI1, HDMI2, Next)'
    'info:Show detailed monitor info'
    'scan:Probe VCP feature codes'
    'profile:Save or load monitor profile JSON'
    'solar-schedule:Apply day/night solar schedule'
    'install-service:Install systemd user service'
    'sync:Synchronize secondary monitors'
    'server:Run IPC socket daemon'
    'send:Send command to running daemon'
    'edid:Show EDID hardware info'
    'waybar-config:Output Waybar status bar JSON block'
    'completions:Output shell completion script'
  )
  _describe 'command' commands
}
_acer_monitor_cli "$@""#.to_string(),

        "fish" => r#"complete -c acer_monitor_cli -f -n "__fish_use_subcommand" -a "list caps get set brightness contrast volume fade preset unlock balance diag idle-dimmer auto-profile energy tray watch-vcp watch-monitors test-pattern blackboost mute input info scan profile solar-schedule install-service sync server send edid waybar-config completions"#.to_string(),

        _ => r#"# bash completion for acer_monitor_cli
_acer_monitor_cli_completions() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local cmds="list caps get set brightness contrast volume fade preset unlock balance diag idle-dimmer auto-profile energy tray watch-vcp watch-monitors test-pattern blackboost mute input info scan profile solar-schedule install-service sync server send edid waybar-config completions power reset keylock powerkey indicator od aim refreshnum bluelight gamma colortemp displaymode colorspace rawbank getbank get-bluelight get-gamma get-colortemp get-od get-aim get-blackboost get-keylock get-indicator"
  COMPREPLY=( $(compgen -W "${cmds}" -- ${cur}) )
}
complete -F _acer_monitor_cli_completions acer_monitor_cli"#.to_string(),
    }
}

fn get_help_string() -> String {
    r#"Acer Monitor CLI Suite

Usage:
  acer_monitor_cli list [--json]
  acer_monitor_cli caps [specifier]
  acer_monitor_cli get <vcp_code> [specifier]
  acer_monitor_cli set <vcp_code> <value> [specifier]
  acer_monitor_cli brightness <0-100 | +val | -val> [--osd] [specifier]
  acer_monitor_cli contrast <0-100 | +val | -val> [--osd] [specifier]
  acer_monitor_cli volume <0-100 | +val | -val> [--osd] [specifier]
  acer_monitor_cli preset <gaming|reading|movie|eco> [specifier]
  acer_monitor_cli unlock [specifier]
  acer_monitor_cli balance [--offset <val>] [master_specifier]
  acer_monitor_cli diag [specifier]
  acer_monitor_cli fade <brightness|contrast|volume> <from> <to> [duration_ms] [specifier]
  acer_monitor_cli idle-dimmer [--idle-secs 300] [--dim-to 10] [specifier]
  acer_monitor_cli auto-profile --rule "proc:profile.json"
  acer_monitor_cli energy [specifier]
  acer_monitor_cli tray
  acer_monitor_cli watch-vcp [--all] [--interval <ms>] [--json] [specifier]
  acer_monitor_cli watch-vcp [vcp_code1 vcp_code2 ...] [--interval <ms>] [specifier]
  acer_monitor_cli watch-monitors
  acer_monitor_cli test-pattern <red|green|blue|white|black|grid|gradient>
  acer_monitor_cli blackboost <0-10> [specifier]
  acer_monitor_cli mute <on|off|toggle> [--osd] [specifier]
  acer_monitor_cli input <auto|dp|hdmi1|hdmi2|next|raw_value> [specifier]
  acer_monitor_cli info [--json] [specifier]
  acer_monitor_cli scan [--json] [specifier]
  acer_monitor_cli edid [specifier]
  acer_monitor_cli profile <save|load> <path.json> [specifier]
  acer_monitor_cli solar-schedule --lat <lat> --lon <lon> [specifier]
  acer_monitor_cli install-service
  acer_monitor_cli sync [master_specifier]
  acer_monitor_cli server
  acer_monitor_cli send <command...>
  acer_monitor_cli waybar-config
  acer_monitor_cli completions <bash|zsh|fish>
  acer_monitor_cli reset [specifier]
  acer_monitor_cli keylock <on|off> [specifier]
  acer_monitor_cli powerkey <on|off> [specifier]
  acer_monitor_cli indicator <on|off> [specifier]
  acer_monitor_cli od <value> [specifier]
  acer_monitor_cli aim <value> [specifier]
  acer_monitor_cli refreshnum <on|off> [specifier]
  acer_monitor_cli bluelight <0|50|60|70|80> [specifier]
  acer_monitor_cli gamma <18|20|22|24|26> [specifier]
  acer_monitor_cli colortemp <warm|normal|cool|bluelight|user> [specifier]
  acer_monitor_cli displaymode <value> [specifier]
  acer_monitor_cli colorspace <calibration_index> <space_index> [specifier]
  acer_monitor_cli rawbank <e0|e7|e9> <selector> <value> [specifier]
  acer_monitor_cli getbank <e0|e7|e9> <selector> [specifier]
  acer_monitor_cli get-bluelight [specifier]
  acer_monitor_cli get-gamma [specifier]
  acer_monitor_cli get-colortemp [specifier]
  acer_monitor_cli get-od [specifier]
  acer_monitor_cli get-aim [specifier]
  acer_monitor_cli get-blackboost [specifier]
  acer_monitor_cli get-keylock [specifier]
  acer_monitor_cli get-indicator [specifier]
  acer_monitor_cli power off [specifier]

Note:
  specifier can be an index (0), a model name substring (VG271U), or 'all' to target all monitors.
"#.to_string()
}
