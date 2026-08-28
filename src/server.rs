#[cfg(unix)]
use std::{
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};

pub const SOCKET_PATH: &str = "/tmp/acer_monitor.sock";

#[cfg(unix)]
pub fn run_server() -> Result<(), String> {
    if Path::new(SOCKET_PATH).exists() {
        let _ = fs::remove_file(SOCKET_PATH);
    }

    let listener = UnixListener::bind(SOCKET_PATH)
        .map_err(|e| format!("Failed to bind Unix socket at '{SOCKET_PATH}': {e}"))?;

    println!("Acer Monitor Daemon running on Unix socket: {SOCKET_PATH}");
    println!("Send commands using: acer_monitor_cli send <command>");
    println!("Press Ctrl+C to stop daemon.");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = vec![0u8; 1024];
                let count = match stream.read(&mut buf) {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                let cmd_line = String::from_utf8_lossy(&buf[..count]).trim().to_string();
                if cmd_line.is_empty() {
                    continue;
                }

                if cmd_line.eq_ignore_ascii_case("stop") || cmd_line.eq_ignore_ascii_case("quit") {
                    let _ = stream.write_all(b"Daemon stopping.\n");
                    break;
                }

                // Execute command via internal dispatch
                let args: Vec<String> = cmd_line.split_whitespace().map(|s| s.to_string()).collect();
                let output = crate::cli::dispatch_command(args).unwrap_or_else(|e| format!("Error: {e}"));
                let _ = stream.write_all(output.as_bytes());
            }
            Err(_) => continue,
        }
    }

    let _ = fs::remove_file(SOCKET_PATH);
    Ok(())
}

#[cfg(unix)]
pub fn send_command(cmd_args: &[String]) -> Result<String, String> {
    if !Path::new(SOCKET_PATH).exists() {
        return Err(format!(
            "Daemon socket '{SOCKET_PATH}' not found. Start the daemon first using 'acer_monitor_cli server'."
        ));
    }

    let mut stream = UnixStream::connect(SOCKET_PATH)
        .map_err(|e| format!("Failed to connect to daemon socket '{SOCKET_PATH}': {e}"))?;

    let payload = cmd_args.join(" ");
    stream
        .write_all(payload.as_bytes())
        .map_err(|e| format!("Failed to send command to daemon: {e}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response from daemon: {e}"))?;

    Ok(response)
}

#[cfg(not(unix))]
pub fn run_server() -> Result<(), String> {
    Err("Daemon socket server mode is currently supported on Unix/Linux systems.".to_string())
}

#[cfg(not(unix))]
pub fn send_command(_cmd_args: &[String]) -> Result<String, String> {
    Err("Daemon socket mode is currently supported on Unix/Linux systems.".to_string())
}
