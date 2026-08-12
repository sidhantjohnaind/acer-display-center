#[cfg(unix)]
use std::process::Command;


pub fn show_osd_banner(label: &str, current: u32, max: u32) {
    let max = max.max(1);
    let pct = (current * 100) / max;
    let filled_blocks = ((pct as usize) * 15) / 100;
    let empty_blocks = 15usize.saturating_sub(filled_blocks);

    let bar = format!(
        "[{}{}] {}%",
        "█".repeat(filled_blocks),
        "░".repeat(empty_blocks),
        pct
    );

    let title = format!("Acer Display: {label}");

    #[cfg(unix)]
    {
        let _ = Command::new("notify-send")
            .arg("-r")
            .arg("99123")
            .arg("-t")
            .arg("1500")
            .arg("-i")
            .arg("display")
            .arg(&title)
            .arg(&bar)
            .status();
    }

    println!("{title}\n{bar}");
}
