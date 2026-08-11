mod acer;
mod cli;
mod ddc;
mod edid;
mod energy;
mod idle;
mod monitor;
mod osd;
mod pattern;
mod server;
mod solar;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
