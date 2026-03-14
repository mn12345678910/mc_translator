use crate::state::app_state::AppState;

impl AppState {
    pub fn render_action_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, processing: bool) {
        ui.separator();
        ui.add_space(1.0);
        let is_paused = *self.is_paused.lock().unwrap();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            if !processing {
                let is_google_free = self.api_provider == "Google Free";
                let has_model = !self.selected_model.is_empty();
                let can_start = !self.input_paths.is_empty() && (is_google_free || has_model);
                
                let (bg, _text, rounding) = self.get_instance_style_full("btn_run_trans");
                let start_btn = egui::Button::new(self.i18n.btn_run_trans).min_size([120.0, 32.0].into()).fill(bg).rounding(rounding);
                let mut resp = ui.add_enabled(can_start, start_btn);
                
                if !can_start {
                    if self.input_paths.is_empty() {
                        resp = resp.on_disabled_hover_text(self.i18n.hover_select_file_first);
                    } else if !is_google_free && !has_model {
                        resp = resp.on_disabled_hover_text(self.i18n.hover_select_model_first);
                    }
                }

                if resp.clicked() {
                    self.start_translation(ctx.clone());
                }
            } else if !is_paused {
                let (bg, _text, rounding) = self.get_instance_style_full("btn_pause");
                if ui
                    .add(egui::Button::new(self.i18n.btn_pause).min_size([80.0, 32.0].into()).fill(bg).rounding(rounding))
                    .clicked()
                {
                    *self.is_paused.lock().unwrap() = true;
                    self.add_log(self.i18n.log_pause_requested);
                }
                let (bg_stop, _text, rounding_stop) = self.get_instance_style_full("btn_stop");
                if ui
                    .add(egui::Button::new(self.i18n.btn_stop).min_size([80.0, 32.0].into()).fill(bg_stop).rounding(rounding_stop))
                    .clicked()
                {
                    self.show_stop_confirm = true;
                }
            } else {
                let (bg_res, _text, rounding_res) = self.get_instance_style_full("btn_pause"); // 繼續按鈕與暫停同色
                if ui
                    .add(egui::Button::new(self.i18n.btn_resume).min_size([80.0, 32.0].into()).fill(bg_res).rounding(rounding_res))
                    .clicked()
                {
                    self.resume_translation();
                }
                let (bg_stop, _text, rounding_stop) = self.get_instance_style_full("btn_stop");
                if ui
                    .add(egui::Button::new(self.i18n.btn_stop).min_size([80.0, 32.0].into()).fill(bg_stop).rounding(rounding_stop))
                    .clicked()
                {
                    self.show_stop_confirm = true;
                }
            }
        });

        if self.show_stop_confirm {
            egui::Window::new(self.i18n.title_confirm_stop)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(self.i18n.text_confirm_stop);
                    ui.horizontal(|ui| {
                        if ui.button(self.i18n.btn_confirm_stop).clicked() {
                            let _ = std::fs::remove_file("progress_state.json");
                            *self.is_cancelled.lock().unwrap() = true;
                            *self.is_paused.lock().unwrap() = false;
                            *self.is_processing.lock().unwrap() = false;
                            self.active_job_config = None;
                            *self.status.lock().unwrap() = self.i18n.status_stopped.to_string();
                            self.add_log(self.i18n.log_stopped);
                            self.show_stop_confirm = false;
                        }
                        if ui.button(self.i18n.btn_cancel).clicked() {
                            self.show_stop_confirm = false;
                        }
                    });
                });
        }
    }
}
