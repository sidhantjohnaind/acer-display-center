use eframe::egui::{self, Color32, Margin, Pos2, Rect, Rounding, Stroke, Vec2};

struct CalibrationPatternApp {
    mode: String,
}

impl eframe::App for CalibrationPatternApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.pointer.any_pressed()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::BLACK).inner_margin(Margin::ZERO))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter();

                match self.mode.to_ascii_lowercase().as_str() {
                    "grid" => {
                        let step = 40.0;
                        let mut x = rect.left();
                        while x <= rect.right() {
                            painter.line_segment(
                                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                                Stroke::new(1.0_f32, Color32::from_rgb(55, 65, 80)),
                            );
                            x += step;
                        }
                        let mut y = rect.top();
                        while y <= rect.bottom() {
                            painter.line_segment(
                                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                                Stroke::new(1.0_f32, Color32::from_rgb(55, 65, 80)),
                            );
                            y += step;
                        }
                        // Center crosshair and alignment circle
                        let center = rect.center();
                        painter.circle_stroke(center, 120.0, Stroke::new(2.0_f32, Color32::WHITE));
                        painter.circle_stroke(center, 240.0, Stroke::new(1.5_f32, Color32::from_rgb(100, 180, 255)));
                        painter.line_segment(
                            [Pos2::new(center.x, rect.top()), Pos2::new(center.x, rect.bottom())],
                            Stroke::new(2.0_f32, Color32::from_rgb(239, 68, 68)),
                        );
                        painter.line_segment(
                            [Pos2::new(rect.left(), center.y), Pos2::new(rect.right(), center.y)],
                            Stroke::new(2.0_f32, Color32::from_rgb(239, 68, 68)),
                        );
                    }
                    "rgb" => {
                        let n = 256;
                        let w = rect.width() / n as f32;
                        for i in 0..n {
                            let hue = i as f32 / n as f32;
                            let r = ((hue * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0);
                            let g = (2.0 - (hue * 6.0 - 2.0).abs()).clamp(0.0, 1.0);
                            let b = (2.0 - (hue * 6.0 - 4.0).abs()).clamp(0.0, 1.0);
                            let col = Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
                            let slice = Rect::from_min_size(Pos2::new(rect.left() + i as f32 * w, rect.top()), Vec2::new(w + 1.0, rect.height()));
                            painter.rect_filled(slice, Rounding::ZERO, col);
                        }
                    }
                    "white" => {
                        painter.rect_filled(rect, Rounding::ZERO, Color32::WHITE);
                    }
                    "black" => {
                        painter.rect_filled(rect, Rounding::ZERO, Color32::BLACK);
                    }
                    "red" => {
                        painter.rect_filled(rect, Rounding::ZERO, Color32::RED);
                    }
                    "green" => {
                        painter.rect_filled(rect, Rounding::ZERO, Color32::from_rgb(0, 255, 0));
                    }
                    "blue" => {
                        painter.rect_filled(rect, Rounding::ZERO, Color32::from_rgb(0, 0, 255));
                    }
                    _ => {
                        painter.rect_filled(rect, Rounding::ZERO, Color32::WHITE);
                    }
                }
            });
    }
}

pub fn run_pattern_gui(name: &str) -> Result<(), String> {
    let mode = name.to_string();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_title(format!("Display Calibration: {}", name)),
        ..Default::default()
    };
    eframe::run_native(
        "Display Calibration",
        native_options,
        Box::new(move |_cc| Ok(Box::new(CalibrationPatternApp { mode }))),
    ).map_err(|e| e.to_string())
}

pub fn render_pattern(name: &str) -> String {
    match run_pattern_gui(name) {
        Ok(_) => "Display pattern closed.".to_string(),
        Err(e) => format!("Error launching pattern: {e}"),
    }
}
