use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};
use crate::ui::widgets::toggle::toggle;

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
                    // 第一列：JSON 與 JAR
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
                    if ui.add(toggle(&mut self.skip_json)).changed() {
                        self.save_config();
                    }

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
                    if ui.add(toggle(&mut self.skip_jar)).changed() {
                        self.save_config();
                    }
                    ui.end_row();

                    // 第二列：JS 與 Patchouli Book
                    let js_label = if self.skip_js {
                        "跳過 .js"
                    } else {
                        "不跳過 .js"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(egui::RichText::new(js_label).color(label_color).strong()),
                    );
                    if ui.add(toggle(&mut self.skip_js)).changed() {
                        self.save_config();
                    }

                    let book_label = if self.skip_book {
                        "跳過手冊"
                    } else {
                        "不跳過手冊"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(
                            egui::RichText::new(book_label).color(label_color).strong(),
                        ),
                    );
                    if ui.add(toggle(&mut self.skip_book)).changed() {
                        self.save_config();
                    }
                    ui.end_row();

                    // 第三列：LLM Log (其他空間保留)
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
                    if ui.add(toggle(&mut self.enable_llm_log)).changed() {
                        self.save_config();
                    }
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("📜 系統技術指令")
                    .color(label_color)
                    .strong(),
            );
            let mut text_edit = egui::TextEdit::multiline(&mut self.system_prompt)
                .desired_rows(4)
                .desired_width(ui.available_width());
            
            if self.theme == "light" {
                text_edit = text_edit.fill(egui::Color32::from_rgb(0xE3, 0xC3, 0x95));
            }

            if ui.add(text_edit).changed() {
                self.save_config();
            }
        });
    }
}
