use crate::state::app_state::AppState;
// use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};
use crate::ui::widgets::toggle::toggle;

impl AppState {
    /// 渲染開發人員模式面板 (還原至備份 Grid，套用隨主題變色標籤)
    pub fn render_developer_mode_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_developer_mode {
            return;
        }
        let (_, label_color) = self.get_instance_style("label_dev");

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
                        self.trigger_save();
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
                        self.trigger_save();
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
                        self.trigger_save();
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
                        self.trigger_save();
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
                        self.trigger_save();
                    }
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("📜 系統技術指令")
                        .color(label_color)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (bg, _) = self.get_instance_style("btn_clear_log");
                    if ui.add(egui::Button::new("🗑 清除執行日誌").fill(bg)).clicked() {
                        self.show_clear_log_confirm = true;
                    }
                });
            });
            let (prompt_bg, _) = self.get_instance_style("input_system_prompt");
            ui.visuals_mut().extreme_bg_color = prompt_bg;

            if ui
                .add(
                    egui::TextEdit::multiline(&mut self.system_prompt)
                        .desired_rows(4)
                        .desired_width(ui.available_width()),
                )
                .changed()
            {
                self.save_config();
            }
        });

        self.render_clear_log_confirm(ui.ctx());
    }

    fn render_clear_log_confirm(&mut self, ctx: &egui::Context) {
        if !self.show_clear_log_confirm {
            return;
        }

        egui::Window::new("⚠ 確認清除日誌")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("確定要清除目前所有的執行日誌嗎？\n此操作無法復原。");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("確定清除").clicked() {
                        let mut log = self.log.lock().unwrap();
                        log.clear();
                        log.push(">>> 使用者已清除執行日誌。".to_string());
                        self.show_clear_log_confirm = false;
                    }
                    if ui.button("取消").clicked() {
                        self.show_clear_log_confirm = false;
                    }
                });
            });
    }
}
