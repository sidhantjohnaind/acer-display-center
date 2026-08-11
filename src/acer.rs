use crate::monitor::Monitor;

pub fn power_mode(mon: &mut Monitor, on: bool) -> Result<(), String> {
    mon.set_vcp(0xD6, if on { 1 } else { 5 })
}

pub fn factory_reset(mon: &mut Monitor) -> Result<(), String> {
    mon.set_vcp(0x04, 1)
}

pub fn key_lock(mon: &mut Monitor, on: bool) -> Result<(), String> {
    mon.set_vcp(0xE0, 0x00)?;
    mon.set_vcp(0xE1, if on { 1 } else { 0 })
}

pub fn power_key(mon: &mut Monitor, on: bool) -> Result<(), String> {
    mon.set_vcp(0xE0, 0x01)?;
    mon.set_vcp(0xE1, if on { 1 } else { 0 })
}

pub fn power_indicator(mon: &mut Monitor, on: bool) -> Result<(), String> {
    mon.set_vcp(0xE0, 0x02)?;
    mon.set_vcp(0xE1, if on { 1 } else { 0 })
}

pub fn overdrive(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0xE0, 0x04)?;
    mon.set_vcp(0xE1, value)
}

pub fn aim_type(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0xE0, 0x06)?;
    mon.set_vcp(0xE1, value)
}

pub fn refresh_rate_num(mon: &mut Monitor, on: bool) -> Result<(), String> {
    mon.set_vcp(0xE0, 0x05)?;
    mon.set_vcp(0xE1, if on { 1 } else { 0 })
}

pub fn blue_light(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0xE7, 0x00)?;
    mon.set_vcp(0xE8, value)
}

pub fn gamma(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0xE7, 0x01)?;
    mon.set_vcp(0xE8, value)
}

pub fn color_temp(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0xE7, 0x02)?;
    mon.set_vcp(0xE8, value)
}

pub fn display_mode(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0xE2, value)
}

pub fn color_space(mon: &mut Monitor, calibration_index: u32, space_index: u32) -> Result<(), String> {
    mon.set_vcp(0xE9, calibration_index)?;
    mon.set_vcp(0xEA, space_index)
}

pub fn raw_bank(mon: &mut Monitor, bank: u8, selector: u32, value: u32) -> Result<(), String> {
    match bank {
        0xE0 => {
            mon.set_vcp(0xE0, selector)?;
            mon.set_vcp(0xE1, value)
        }
        0xE7 => {
            mon.set_vcp(0xE7, selector)?;
            mon.set_vcp(0xE8, value)
        }
        0xE9 => {
            mon.set_vcp(0xE9, selector)?;
            mon.set_vcp(0xEA, value)
        }
        _ => Err(format!("Unsupported bank 0x{bank:02X}; use e0, e7, or e9")),
    }
}
pub fn black_boost(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0xE5, value)
}

pub fn get_raw_bank(mon: &mut Monitor, bank: u8, selector: u32) -> Result<(u32, u32), String> {
    match bank {
        0xE0 => {
            mon.set_vcp(0xE0, selector)?;
            mon.get_vcp(0xE1)
        }
        0xE7 => {
            mon.set_vcp(0xE7, selector)?;
            mon.get_vcp(0xE8)
        }
        0xE9 => {
            mon.set_vcp(0xE9, selector)?;
            mon.get_vcp(0xEA)
        }
        _ => Err(format!("Unsupported bank 0x{bank:02X}; use e0, e7, or e9")),
    }
}

pub fn contrast(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0x12, value)
}

pub fn volume(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0x62, value)
}

pub fn mute(mon: &mut Monitor, on: bool) -> Result<(), String> {
    mon.set_vcp(0x8D, if on { 1 } else { 2 })
}

pub fn input(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0x60, value)
}
pub fn brightness(mon: &mut Monitor, value: u32) -> Result<(), String> {
    mon.set_vcp(0x10, value)
}

pub fn get_key_lock(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE0, 0x00)
}

pub fn get_power_key(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE0, 0x01)
}

pub fn get_power_indicator(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE0, 0x02)
}

pub fn get_overdrive(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE0, 0x04)
}

pub fn get_refresh_rate_num(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE0, 0x05)
}

pub fn get_aim_type(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE0, 0x06)
}

pub fn get_blue_light(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE7, 0x00)
}

pub fn get_gamma(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE7, 0x01)
}

pub fn get_color_temp(mon: &mut Monitor) -> Result<(u32, u32), String> {
    get_raw_bank(mon, 0xE7, 0x02)
}

pub fn get_black_boost(mon: &mut Monitor) -> Result<(u32, u32), String> {
    mon.get_vcp(0xE5)
}

pub fn fade_vcp(mon: &mut Monitor, code: u8, start_val: u32, end_val: u32, duration_ms: u64) -> Result<(), String> {
    let steps = 20u64.min(duration_ms.max(1));
    let step_delay = std::time::Duration::from_millis(duration_ms / steps);

    for i in 0..=steps {
        let val = if steps == 0 {
            end_val
        } else {
            let start = start_val as f64;
            let end = end_val as f64;
            let t = i as f64 / steps as f64;
            (start + (end - start) * t).round() as u32
        };

        mon.set_vcp(code, val)?;
        if i < steps && step_delay.as_millis() > 0 {
            std::thread::sleep(step_delay);
        }
    }
    Ok(())
}