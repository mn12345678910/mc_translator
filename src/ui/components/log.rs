use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    pub fn render_log_area(&mut self, ui: &mut egui::Ui) {
        let label_color = if self.theme == "light" {
            LABEL_COLOR_LIGHT
        } else {
            LABEL_COLOR_DARK
        };
        ui.separator();
        ui.label(egui::RichText::new("執行日誌:").color(label_color).strong());
        let log = self.log.lock().unwrap();
        let log_bg = if self.theme == "light" {
            egui::Color32::from_rgb(0xFA, 0xF0, 0xE6) // 亞麻色 (Log Background)
        } else {
            egui::Color32::from_rgb(25, 25, 30) // 深色模式背景
        };

        egui::Frame::none()
            .fill(log_bg)
            .rounding(4.0)
            .inner_margin(4.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_source("log_scroll_area")
                    .stick_to_bottom(true)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for line in log.iter() {
                            let mut text = egui::RichText::new(line).monospace();
                            
                            if line.contains("Err") || line.contains("錯誤") || line.contains("失敗") || line.contains("中斷") {
                                text = text.color(egui::Color32::RED);
                            } else if line.contains("成功") || line.contains("完成") || line.contains("Done") {
                                let success_color = if self.theme == "light" {
                                    egui::Color32::from_rgb(0, 100, 0) // 深綠色
                                } else {
                                    egui::Color32::GREEN
                                };
                                text = text.color(success_color);
                            } else {
                                text = text.color(label_color);
                            }
                            
                            ui.add(egui::Label::new(text));
                        }
                    });
            });
    }
}
