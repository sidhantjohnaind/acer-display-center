pub fn render_pattern(name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "red" => format!("{}\x1b[41m{}\x1b[0m\nRed Full Screen Test Pattern (Press Enter to exit)", "\x1b[2J\x1b[1;1H", " ".repeat(2000)),
        "green" => format!("{}\x1b[42m{}\x1b[0m\nGreen Full Screen Test Pattern (Press Enter to exit)", "\x1b[2J\x1b[1;1H", " ".repeat(2000)),
        "blue" => format!("{}\x1b[44m{}\x1b[0m\nBlue Full Screen Test Pattern (Press Enter to exit)", "\x1b[2J\x1b[1;1H", " ".repeat(2000)),
        "white" => format!("{}\x1b[47m{}\x1b[0m\nWhite Uniformity Test Pattern (Press Enter to exit)", "\x1b[2J\x1b[1;1H", " ".repeat(2000)),
        "black" => format!("{}\x1b[40m{}\x1b[0m\nBlack Backlight Bleed Test Pattern (Press Enter to exit)", "\x1b[2J\x1b[1;1H", " ".repeat(2000)),
        "gradient" => {
            let mut out = String::from("Grayscale Gradient Step Test:\n");
            for i in (0..=100).step_by(5) {
                let blocks = "█".repeat(3);
                out.push_str(&format!("{i:3}%: {blocks}\n"));
            }
            out
        }
        "grid" => {
            let mut out = String::from("Alignment & Uniformity Grid Pattern:\n");
            for r in 0..10 {
                for c in 0..20 {
                    if r % 2 == 0 || c % 4 == 0 {
                        out.push('+');
                    } else {
                        out.push(' ');
                    }
                }
                out.push('\n');
            }
            out
        }
        _ => "Invalid test pattern. Available patterns: red, green, blue, white, black, grid, gradient".to_string(),
    }
}
