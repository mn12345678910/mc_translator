use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    /// 渲染調色盤設定介面
    pub fn render_palette_settings(&mut self, ui: &mut egui::Ui) {
        let label_color = if self.theme == "light" {
            LABEL_COLOR_LIGHT
        } else {
            LABEL_COLOR_DARK
        };

        ui.vertical(|ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🎨 自定義調色盤").strong().color(label_color));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("重置預設").on_hover_text("將目前模式的顏色回復為系統預設值").clicked() {
                        if self.palette_edit_dark {
                            self.dark_bg = [30, 30, 35];
                            self.dark_text = [200, 160, 100];
                        } else {
                            self.light_bg = [0xFF, 0xFD, 0xF0];
                            self.light_text = [34, 34, 34];
                        }
                        self.save_config();
                    }
                });
            });
            ui.separator();

            // 模式切換開關
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("編輯模式:").color(label_color));
                let mode_text = if self.palette_edit_dark { "🌙 深色設定" } else { "☀️ 淺色設定" };
                if ui.selectable_label(!self.palette_edit_dark, "☀️ 淺色").clicked() {
                    self.palette_edit_dark = false;
                }
                if ui.selectable_label(self.palette_edit_dark, "🌙 深色").clicked() {
                    self.palette_edit_dark = true;
                }
            });

            ui.add_space(5.0);

            let (bg, text) = if self.palette_edit_dark {
                (&mut self.dark_bg, &mut self.dark_text)
            } else {
                (&mut self.light_bg, &mut self.light_text)
            };

            egui::Grid::new("palette_grid").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
                ui.label(egui::RichText::new("背景顏色:").color(label_color));
                if ui.color_edit_button_srgb(bg).changed() {
                    self.save_config();
                }
                ui.end_row();

                ui.label(egui::RichText::new("文字顏色:").color(label_color));
                if ui.color_edit_button_srgb(text).changed() {
                    self.save_config();
                }
                ui.end_row();
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("💡 變更將立即儲存並套用至全域主題。")
                .small()
                .color(label_color.linear_multiply(0.7)));
        });
    }
}
