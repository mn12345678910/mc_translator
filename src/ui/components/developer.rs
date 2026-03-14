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
                egui::RichText::new(self.i18n.header_dev_mode)
                    .color(label_color)
                    .strong(),
            );
            egui::Grid::new("developer_grid")
                .num_columns(4)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    // 第一列：JSON 與 JAR
                    let json_label = if self.skip_json {
                        self.i18n.label_skip_json
                    } else {
                        self.i18n.label_no_skip_json
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
                        self.i18n.label_skip_jar
                    } else {
                        self.i18n.label_no_skip_jar
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
                        self.i18n.label_skip_js
                    } else {
                        self.i18n.label_no_skip_js
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(egui::RichText::new(js_label).color(label_color).strong()),
                    );
                    if ui.add(toggle(&mut self.skip_js)).changed() {
                        self.trigger_save();
                    }

                    let book_label = if self.skip_book {
                        self.i18n.label_skip_book
                    } else {
                        self.i18n.label_no_skip_book
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
                        self.i18n.label_enable_log
                    } else {
                        self.i18n.label_disable_log
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
                    egui::RichText::new(self.i18n.label_system_prompt)
                        .color(label_color)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (bg, _) = self.get_instance_style("btn_clear_log");
                    if ui.add(egui::Button::new(self.i18n.btn_clear_log).fill(bg)).clicked() {
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

        egui::Window::new(self.i18n.title_confirm_clear_log)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(self.i18n.text_confirm_clear_log);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.btn_confirm_clear_log).clicked() {
                        let mut log = self.log.lock().unwrap();
                        log.clear();
                        log.push(self.i18n.log_log_cleared.to_string());
                        self.show_clear_log_confirm = false;
                    }
                    if ui.button(self.i18n.btn_cancel).clicked() {
                        self.show_clear_log_confirm = false;
                    }
                });
            });
    }
}
