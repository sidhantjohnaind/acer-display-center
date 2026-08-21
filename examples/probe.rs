use acer_monitor_cli::monitor::MonitorSet;

fn main() {
    let mut set = MonitorSet::enumerate().expect("Failed to initialize monitor set");
    if let Ok(mon) = set.pick_mut(Some(0)) {
        println!("Monitor: {}", mon.description);

        // 1. Standard VESA MCCS
        println!("--- VESA MCCS ---");
        for vcp in [0x14, 0x10, 0x12, 0x54, 0x60, 0x8D, 0xD6, 0xDC, 0xDF, 0xE2, 0xE5] {
            match mon.get_vcp(vcp) {
                Ok((cur, max)) => println!("VCP 0x{:02X}: cur={} (0x{:02X}), max={}", vcp, cur, cur, max),
                Err(e) => println!("VCP 0x{:02X}: Err({e})", vcp),
            }
        }

        // 2. Acer Bank 0xE7
        println!("--- Acer Bank 0xE7 ---");
        for sel in 0..=5 {
            match acer_monitor_cli::acer::get_raw_bank(mon, 0xE7, sel) {
                Ok((cur, max)) => println!("Bank 0xE7 [sel={sel}]: cur={} (0x{:02X}), max={}", cur, cur, max),
                Err(e) => println!("Bank 0xE7 [sel={sel}]: Err({e})"),
            }
        }

        // 3. Acer Bank 0xE0
        println!("--- Acer Bank 0xE0 ---");
        for sel in 0..=8 {
            match acer_monitor_cli::acer::get_raw_bank(mon, 0xE0, sel) {
                Ok((cur, max)) => println!("Bank 0xE0 [sel={sel}]: cur={} (0x{:02X}), max={}", cur, cur, max),
                Err(e) => println!("Bank 0xE0 [sel={sel}]: Err({e})"),
            }
        }

        // 4. Acer Bank 0xE9
        println!("--- Acer Bank 0xE9 ---");
        for sel in 0..=4 {
            match acer_monitor_cli::acer::get_raw_bank(mon, 0xE9, sel) {
                Ok((cur, max)) => println!("Bank 0xE9 [sel={sel}]: cur={} (0x{:02X}), max={}", cur, cur, max),
                Err(e) => println!("Bank 0xE9 [sel={sel}]: Err({e})"),
            }
        }
    } else {
        println!("No monitor found!");
    }
}
