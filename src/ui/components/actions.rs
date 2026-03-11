use crate::state::app_state::AppState;

impl AppState {
    pub fn render_action_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, processing: bool) {
        ui.separator();
        ui.add_space(1.0);
        let is_paused = *self.is_paused.lock().unwrap();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            if !processing {
                let can_start = !self.input_paths.is_empty();
                if ui
                    .add_enabled(
                        can_start,
                        egui::Button::new("▶ 開始翻譯").min_size([120.0, 32.0].into()),
                    )
                    .on_disabled_hover_text("請先選取檔案或資料夾")
                    .clicked()
                {
                    self.start_translation(ctx.clone());
                }
            } else if !is_paused {
                if ui
                    .add(egui::Button::new("⏸ 暫停").min_size([80.0, 32.0].into()))
                    .clicked()
                {
                    *self.is_paused.lock().unwrap() = true;
                    self.add_log(">>> 使用者請求暫停...");
                }
                if ui
                    .add(egui::Button::new("■ 停止").min_size([80.0, 32.0].into()))
                    .clicked()
                {
                    self.show_stop_confirm = true;
                }
            } else {
                if ui
                    .add(egui::Button::new("▶ 繼續").min_size([80.0, 32.0].into()))
                    .clicked()
                {
                    self.resume_translation();
                }
                if ui
                    .add(egui::Button::new("■ 停止").min_size([80.0, 32.0].into()))
                    .clicked()
                {
                    self.show_stop_confirm = true;
                }
            }
        });

        if self.show_stop_confirm {
            egui::Window::new("⚠ 確認停止翻譯")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("確定要停止翻譯嗎？此操作無法復原。");
                    ui.horizontal(|ui| {
                        if ui.button("確定停止").clicked() {
                            let _ = std::fs::remove_file("progress_state.json");
                            *self.is_cancelled.lock().unwrap() = true;
                            *self.is_paused.lock().unwrap() = false;
                            *self.is_processing.lock().unwrap() = false;
                            self.active_job_config = None;
                            *self.status.lock().unwrap() = "已中止".to_string();
                            self.add_log(">>> 翻譯已中斷。");
                            self.show_stop_confirm = false;
                        }
                        if ui.button("取消").clicked() {
                            self.show_stop_confirm = false;
                        }
                    });
                });
        }
    }
}
