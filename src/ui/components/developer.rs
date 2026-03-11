use crate::state::app_state::AppState;
use super::super::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};
use super::super::widgets::toggle::toggle;

impl AppState {
    /// 渲染開發人員模式面板 (還原至備份 Grid，套用隨主題變色標籤)
    pub fn render_developer_mode_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_developer_mode {
            return;
        }
        let label_color = if self.theme == "light" {
            LABEL_COLOR_LIGHT
        } else {
            LABEL_COLOR_DARK
        };

        ui.group(|ui| {
            ui.label(
                egui::RichText::new("🔧 開發人員模式")
                    .color(label_color)
                    .strong(),
            );
            egui::Grid::new("developer_grid")
                .num_columns(4)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    let json_label = if self.skip_json {
                        "跳過 .json"
                    } else {
                        "不跳過 .json"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(
                            egui::RichText::new(json_label).color(label_color).strong(),
                        ),
                    );
                    ui.add(toggle(&mut self.skip_json));

                    let jar_label = if self.skip_jar {
                        "跳過 .jar"
                    } else {
                        "不跳過 .jar"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(
                            egui::RichText::new(jar_label).color(label_color).strong(),
                        ),
                    );
                    ui.add(toggle(&mut self.skip_jar));
                    ui.end_row();

                    let js_label = if self.skip_js {
                        "跳過 .js"
                    } else {
                        "不跳過 .js"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(egui::RichText::new(js_label).color(label_color).strong()),
                    );
                    ui.add(toggle(&mut self.skip_js));

                    let log_label = if self.enable_llm_log {
                        "開啟記錄日誌"
                    } else {
                        "關閉記錄日誌"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(
                            egui::RichText::new(log_label).color(label_color).strong(),
                        ),
                    );
                    ui.add(toggle(&mut self.enable_llm_log));
                    ui.end_row();
                });
        });
    }
}
