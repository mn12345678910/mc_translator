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
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for line in log.iter() {
                    ui.add(egui::Label::new(egui::RichText::new(line).monospace()));
                }
            });
    }
}
