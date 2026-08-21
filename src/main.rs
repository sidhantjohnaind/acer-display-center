#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod acer;
mod cli;
mod ddc;
mod edid;
mod energy;
mod hdr;
mod idle;

mod gui;
mod hotkeys;
mod monitor;
mod osd;
mod pattern;
mod server;
mod solar;
mod tray;

fn main() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::{
            AttachConsole, GetStdHandle, SetStdHandle, ATTACH_PARENT_PROCESS,
            STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
        };
        use std::os::windows::io::IntoRawHandle;

        let out_handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if out_handle.is_null() || out_handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE as _ {
            if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
                if let Ok(conout) = std::fs::OpenOptions::new().write(true).open("CONOUT$") {
                    let handle = conout.into_raw_handle();
                    SetStdHandle(STD_OUTPUT_HANDLE, handle as _);
                    SetStdHandle(STD_ERROR_HANDLE, handle as _);
                }
                if let Ok(conin) = std::fs::OpenOptions::new().read(true).open("CONIN$") {
                    let handle = conin.into_raw_handle();
                    SetStdHandle(STD_INPUT_HANDLE, handle as _);
                }
            }
        }
    }

    if let Err(err) = cli::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
