use crate::edid::EdidInfo;

pub fn calculate_power(brightness: u32, is_hdr: bool) -> (f64, f64, f64) {
    let edid = EdidInfo::inspect_connected();
    let (width_cm, height_cm) = edid.map(|e| (e.width_cm, e.height_cm)).unwrap_or((60, 34));

    let diag_inches = if width_cm > 0 && height_cm > 0 {
        (f64::from(width_cm).powi(2) + f64::from(height_cm).powi(2)).sqrt() / 2.54
    } else {
        27.0
    };

    // Estimated base power draw scaling with screen area & brightness
    let base_min_watts = 10.0 + (diag_inches - 24.0).max(0.0) * 0.8;
    let base_max_watts = 35.0 + (diag_inches - 24.0).max(0.0) * 1.5;

    let eff_brightness = if is_hdr { 100.0 } else { brightness as f64 };
    let b_pct = eff_brightness.min(100.0) / 100.0;
    
    let mut current_watts = base_min_watts + (base_max_watts - base_min_watts) * b_pct;
    if is_hdr {
        current_watts *= 1.25; // Peak HDR luminance backlight boost multiplier (~38-45W)
    }

    // Assuming 8 hours usage per day
    let yearly_kwh = (current_watts * 8.0 * 365.0) / 1000.0;
    let yearly_cost_usd = yearly_kwh * 0.15; // Average $0.15 per kWh

    (current_watts, yearly_kwh, yearly_cost_usd)
}

pub fn report_energy(brightness: u32, desc: &str, is_hdr: bool) -> String {
    let (watts, kwh, cost) = calculate_power(brightness, is_hdr);
    let mode_str = if is_hdr { " (⚡ HDR Peak Mode)" } else { "" };
    format!(
        "Monitor Energy Estimates for '{desc}'{mode_str}:\n  Current Brightness:  {brightness}%\n  Power Consumption:   {watts:.1} Watts\n  Estimated Energy:    {kwh:.1} kWh/year (8 hrs/day)\n  Estimated Cost:      ${cost:.2}/year (@ $0.15/kWh)"
    )
}
