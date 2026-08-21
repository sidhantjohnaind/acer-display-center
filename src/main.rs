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
        if windows_sys::Win32::System::Console::AttachConsole(
            windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS,
        ) != 0 {
            use std::os::windows::io::IntoRawHandle;
            use windows_sys::Win32::System::Console::{
                SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
            };

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

    if let Err(err) = cli::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
