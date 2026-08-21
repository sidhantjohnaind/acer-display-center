#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("gui");

    if cmd == "tray" {
        if let Err(err) = acer_monitor_cli::tray::run_tray() {
            eprintln!("Tray error: {err}");
        }
    } else {
        if let Err(err) = acer_monitor_cli::gui::run_gui() {
            eprintln!("GUI error: {err}");
        }
    }
}
