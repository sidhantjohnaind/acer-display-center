use crate::{
    acer, ddc::parse_u32, edid::EdidInfo, energy, idle, monitor::MonitorSet, osd::show_osd_banner,
    pattern, server, solar::SolarSchedule,
};

pub fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("{}", get_help_string());
        return Ok(());
    }

    let output = dispatch_command(args)?;
    if !output.is_empty() {
        println!("{output}");
    }
    Ok(())
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
                return Err("Usage: acer_monitor_cli preset <user|standard|eco|graphics|hdr|action|racing|sports|reading|movie|0-7> [specifier]".to_string());
            }
            let preset_name = args[0].to_ascii_lowercase();
            let spec = parse_optional_specifier(&args[1..]);

            with_monitor(spec, |mon| {
                match preset_name.as_str() {
                    "user" => acer::display_mode(mon, 0),
                    "standard" | "normal" => acer::display_mode(mon, 1),
                    "eco" => acer::display_mode(mon, 2),
                    "graphics" => acer::display_mode(mon, 3),
                    "hdr" => acer::raw_bank(mon, 0xE7, 8, 1),
                    "action" | "gaming" => acer::display_mode(mon, 5),
                    "racing" => acer::display_mode(mon, 6),
                    "sports" => acer::display_mode(mon, 7),
                    "reading" | "text" => mon.set_vcp(0xDC, 0x02),
                    "movie" | "cinema" => mon.set_vcp(0xDC, 0x03),
                    other => {
                        let val = parse_u32(other)?;
                        acer::display_mode(mon, val)
                    }
                }
            })?;
            Ok(format!("Applied hardware preset '{preset_name}'."))
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
            let spec = parse_optional_specifier(&args);
            let mut out = String::new();
            with_monitor(spec, |mon| {
                let (b, _) = mon.get_vcp(0x10).unwrap_or((80, 100));
                out = energy::report_energy(b, &mon.description);
                Ok(())
            })?;
            Ok(out)
        }

        "test-pattern" => {
            let name = args.first().map(|s| s.as_str()).unwrap_or("grid");
            Ok(pattern::render_pattern(name))
        }

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
            loop {
                for arg in &args {
                    if let Some(rule) = arg.strip_prefix("--rule=") {
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
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
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

        "solar-schedule" => {
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
            with_monitor(spec, |mon| acer::key_lock(mon, on))?;
            Ok("OK".to_string())
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

        "indicator" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli indicator <on|off> [specifier]".to_string());
            }
            let on = parse_on_off(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::power_indicator(mon, on))?;
            Ok("OK".to_string())
        }

        "od" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli od <value> [specifier]".to_string());
            }
            let value = parse_u32(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::overdrive(mon, value))?;
            Ok("OK".to_string())
        }

        "aim" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli aim <value> [specifier]".to_string());
            }
            let value = parse_u32(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::aim_type(mon, value))?;
            Ok("OK".to_string())
        }

        "refreshnum" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli refreshnum <on|off> [specifier]".to_string());
            }
            let on = parse_on_off(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::refresh_rate_num(mon, on))?;
            Ok("OK".to_string())
        }

        "bluelight" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli bluelight <0|50|60|70|80> [specifier]".to_string());
            }
            let value = parse_bluelight(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::blue_light(mon, value))?;
            Ok("OK".to_string())
        }

        "gamma" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli gamma <18|20|22|24|26> [specifier]".to_string());
            }
            let value = parse_gamma(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::gamma(mon, value))?;
            Ok("OK".to_string())
        }

        "colortemp" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli colortemp <warm|normal|cool|bluelight|user> [specifier]".to_string());
            }
            let value = parse_color_temp(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::color_temp(mon, value))?;
            Ok("OK".to_string())
        }

        "displaymode" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli displaymode <value> [specifier]".to_string());
            }
            let value = parse_u32(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::display_mode(mon, value))?;
            Ok("OK".to_string())
        }

        "colorspace" => {
            if args.len() < 2 {
                return Err("Usage: acer_monitor_cli colorspace <calibration_index> <space_index> [specifier]".to_string());
            }
            let calib = parse_u32(&args[0])?;
            let space = parse_u32(&args[1])?;
            let spec = parse_optional_specifier(&args[2..]);
            with_monitor(spec, |mon| acer::color_space(mon, calib, space))?;
            Ok("OK".to_string())
        }

        "blackboost" => {
            if args.is_empty() {
                return Err("Usage: acer_monitor_cli blackboost <0-10> [specifier]".to_string());
            }
            let value = parse_u32(&args[0])?;
            let spec = parse_optional_specifier(&args[1..]);
            with_monitor(spec, |mon| acer::black_boost(mon, value))?;
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

        "rawbank" => {
            if args.len() < 3 {
                return Err("Usage: acer_monitor_cli rawbank <e0|e7|e9> <selector> <value> [specifier]".to_string());
            }
            let bank = parse_bank(&args[0])?;
            let selector = parse_u32(&args[1])?;
            let value = parse_u32(&args[2])?;
            let spec = parse_optional_specifier(&args[3..]);
            with_monitor(spec, |mon| acer::raw_bank(mon, bank, selector, value))?;
            Ok("OK".to_string())
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

fn with_monitor<T, F>(spec: Option<&str>, mut f: F) -> Result<(), String>
where
    F: FnMut(&mut crate::monitor::Monitor) -> Result<T, String>,
{
    let mut set = MonitorSet::enumerate()?;
    if let Some("all") = spec {
        for mon in set.monitors_mut() {
            f(mon)?;
        }
        Ok(())
    } else {
        let mon = set.pick_mut_by_specifier(spec)?;
        f(mon)?;
        Ok(())
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
        "50" => Ok(1),
        "60" => Ok(2),
        "70" => Ok(3),
        "80" => Ok(4),
        _ => Err("Use 0/off/standard, 50, 60, 70, or 80".into()),
    }
}

fn parse_gamma(s: &str) -> Result<u32, String> {
    match s {
        "18" => Ok(0),
        "20" => Ok(1),
        "22" => Ok(2),
        "24" => Ok(3),
        "26" => Ok(4),
        _ => Err("Use 18, 20, 22, 24, or 26".into()),
    }
}

fn parse_color_temp(s: &str) -> Result<u32, String> {
    match s.to_ascii_lowercase().as_str() {
        "warm" => Ok(0xFFFF),
        "normal" => Ok(0),
        "cool" => Ok(1),
        "bluelight" => Ok(2),
        "user" => Ok(3),
        _ => Err("Use warm, normal, cool, bluelight, or user".into()),
    }
}

fn parse_input_source(s: &str) -> Result<u32, String> {
    match s.to_ascii_lowercase().as_str() {
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

        "fish" => r#"complete -c acer_monitor_cli -f -n "__fish_use_subcommand" -a "list caps get set brightness contrast volume fade preset unlock balance diag idle-dimmer auto-profile energy watch-monitors test-pattern blackboost mute input info scan profile solar-schedule install-service sync server send edid waybar-config completions"#.to_string(),

        _ => r#"# bash completion for acer_monitor_cli
_acer_monitor_cli_completions() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local cmds="list caps get set brightness contrast volume fade preset unlock balance diag idle-dimmer auto-profile energy watch-monitors test-pattern blackboost mute input info scan profile solar-schedule install-service sync server send edid waybar-config completions power reset keylock powerkey indicator od aim refreshnum bluelight gamma colortemp displaymode colorspace rawbank getbank get-bluelight get-gamma get-colortemp get-od get-aim get-blackboost get-keylock get-indicator"
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
