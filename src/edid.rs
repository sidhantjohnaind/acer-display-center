use std::fs;

#[derive(Debug, Default, Clone)]
pub struct EdidInfo {
    pub manufacturer: String,
    pub product_code: u16,
    pub serial_number_u32: u32,
    pub serial_string: String,
    pub model_name: String,
    pub week_of_manufacture: u8,
    pub year_of_manufacture: u16,
    pub edid_version: String,
    pub is_digital: bool,
    pub color_depth_bits: u8,
    pub width_cm: u8,
    pub height_cm: u8,
    pub native_resolution: String,
    pub raw_hex: String,
}

impl EdidInfo {
    pub fn report(&self) -> String {
        let mut out = String::new();
        use std::fmt::Write as _;

        let _ = writeln!(out, "Hardware EDID Information:");
        let _ = writeln!(out, "  Model Name:       {}", if self.model_name.is_empty() { "Unknown" } else { &self.model_name });
        let _ = writeln!(out, "  Manufacturer ID:  {}", self.manufacturer);
        let _ = writeln!(out, "  Product Code:     0x{:04X}", self.product_code);
        let serial_disp = if !self.serial_string.is_empty() { self.serial_string.clone() } else { self.serial_number_u32.to_string() };
        let _ = writeln!(out, "  Serial Number:    {}", serial_disp);
        let _ = writeln!(out, "  Manufactured:     Week {}, Year {}", self.week_of_manufacture, self.year_of_manufacture);
        let _ = writeln!(out, "  EDID Version:     {}", self.edid_version);
        let _ = writeln!(out, "  Signal Type:      {}", if self.is_digital { "Digital" } else { "Analog" });
        if self.color_depth_bits > 0 {
            let _ = writeln!(out, "  Color Depth:      {}-bit per channel", self.color_depth_bits);
        }
        if self.width_cm > 0 && self.height_cm > 0 {
            let diag_inches = (f64::from(self.width_cm).powi(2) + f64::from(self.height_cm).powi(2)).sqrt() / 2.54;
            let _ = writeln!(out, "  Physical Size:    {} cm x {} cm ({:.1}\")", self.width_cm, self.height_cm, diag_inches);
        }
        if !self.native_resolution.is_empty() {
            let _ = writeln!(out, "  Native Timing:    {}", self.native_resolution);
        }

        out
    }

    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 128 {
            return None;
        }

        // Verify EDID Header: 00 FF FF FF FF FF FF 00
        if bytes[0..8] != [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00] {
            return None;
        }

        let mut info = EdidInfo::default();

        // Manufacturer ID (bytes 8, 9 - 3 compressed 5-bit uppercase letters)
        let m_code = u16::from(bytes[8]) << 8 | u16::from(bytes[9]);
        let c1 = (((m_code >> 10) & 0x1F) as u8 + b'A' - 1) as char;
        let c2 = (((m_code >> 5) & 0x1F) as u8 + b'A' - 1) as char;
        let c3 = ((m_code & 0x1F) as u8 + b'A' - 1) as char;
        info.manufacturer = format!("{c1}{c2}{c3}");

        // Product Code (bytes 10, 11 - little endian)
        info.product_code = u16::from(bytes[10]) | (u16::from(bytes[11]) << 8);

        // Serial Number (bytes 12..16 - little endian 32-bit uint)
        info.serial_number_u32 = u32::from(bytes[12])
            | (u32::from(bytes[13]) << 8)
            | (u32::from(bytes[14]) << 16)
            | (u32::from(bytes[15]) << 24);

        info.week_of_manufacture = bytes[16];
        info.year_of_manufacture = 1990 + u16::from(bytes[17]);

        info.edid_version = format!("{}.{}", bytes[18], bytes[19]);

        // Video Input Definition (byte 20)
        let input_byte = bytes[20];
        info.is_digital = (input_byte & 0x80) != 0;
        if info.is_digital {
            info.color_depth_bits = match (input_byte >> 4) & 0x07 {
                1 => 6,
                2 => 8,
                3 => 10,
                4 => 12,
                5 => 14,
                6 => 16,
                _ => 0,
            };
        }

        info.width_cm = bytes[21];
        info.height_cm = bytes[22];

        // Read Detailed Timings / Descriptor Blocks (bytes 54..126 in 18-byte chunks)
        for chunk in bytes[54..126].chunks(18) {
            if chunk.len() < 18 {
                continue;
            }

            if chunk[0] == 0x00 && chunk[1] == 0x00 && chunk[2] == 0x00 {
                let tag = chunk[3];
                let text = String::from_utf8_lossy(&chunk[5..18]);
                let cleaned = text.trim_matches(char::from(0)).trim().to_string();

                match tag {
                    0xFC => info.model_name = cleaned,       // Monitor Name
                    0xFF => info.serial_string = cleaned,     // Serial String
                    _ => {}
                }
            } else if info.native_resolution.is_empty() {
                // Parse Detailed Timing Descriptor (Pixel clock in 10kHz units)
                let pixel_clock_khz = (u32::from(chunk[0]) | (u32::from(chunk[1]) << 8)) * 10;
                if pixel_clock_khz > 0 {
                    let hactive = u32::from(chunk[2]) | (u32::from(chunk[4] & 0xF0) << 4);
                    let vactive = u32::from(chunk[5]) | (u32::from(chunk[7] & 0xF0) << 4);
                    let htotal = hactive + (u32::from(chunk[3]) | (u32::from(chunk[4] & 0x0F) << 8));
                    let vtotal = vactive + (u32::from(chunk[6]) | (u32::from(chunk[7] & 0x0F) << 8));

                    if htotal > 0 && vtotal > 0 {
                        let refresh_hz = (pixel_clock_khz as f64 * 1000.0) / (htotal as f64 * vtotal as f64);
                        info.native_resolution = format!("{}x{} @ {:.2} Hz", hactive, vactive, refresh_hz);
                    }
                }
            }
        }

        info.raw_hex = bytes.iter().take(128).map(|b| format!("{b:02X}")).collect::<Vec<_>>().join("");
        Some(info)
    }

    pub fn inspect_connected() -> Option<Self> {
        let entries = fs::read_dir("/sys/class/drm").ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            let status_file = path.join("status");
            let edid_file = path.join("edid");

            if status_file.exists() && edid_file.exists() {
                if let Ok(status) = fs::read_to_string(&status_file) {
                    if status.trim() == "connected" {
                        if let Ok(bytes) = fs::read(&edid_file) {
                            if let Some(parsed) = Self::parse(&bytes) {
                                return Some(parsed);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
