use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    /// 渲染 API 設定面板 (細粒度鎖定優化：僅在翻譯時鎖定必要參數)
    pub fn render_settings_panel(&mut self, ui: &mut egui::Ui, ui_enabled: bool, _ctx: &egui::Context) {
        if !self.show_api_settings {
            return;
        }
        ui.add_space(4.0);
        ui.group(|ui| {
            egui::ScrollArea::vertical()
                .max_height(350.0)
                .auto_shrink([true; 2])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let label_color = if self.theme == "light" {
                            LABEL_COLOR_LIGHT
                        } else {
                            LABEL_COLOR_DARK
                        };

                        // --- 服務商與恢復預設 (鎖定) ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("服務商:").color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                egui::ComboBox::from_id_source("provider_combo")
                                    .selected_text(&self.api_provider)
                                    .width(140.0)
                                    .show_ui(ui, |ui| {
                                        for p in &[
                                            "Gemini", "OpenAI", "DeepSeek", "Mistral", "DeepL",
                                            "Ollama",
                                        ] {
                                            if ui
                                                .selectable_value(
                                                    &mut self.api_provider,
                                                    p.to_string(),
                                                    *p,
                                                )
                                                .changed()
                                            {
                                                self.refresh_models();
                                                self.save_config();
                                            }
                                        }
                                    });
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_enabled_ui(ui_enabled, |ui| {
                                        if ui.button("⟲ 恢復預設").clicked() {
                                            let def = crate::config::settings::AppConfig::default();
                                            self.api_provider = def.provider;
                                            self.api_key = def.api_key;
                                            self.selected_model = def.model;
                                            self.ollama_url = def.ollama_url;
                                            self.batch_size = def.batch_size;
                                            self.batch_max_chars = def.batch_max_chars;
                                            self.ollama_timeout = def.ollama_timeout;
                                            self.translation_prompt = def.translation_prompt;
                                            self.theme = def.theme;
                                            self.font_size = def.font_size;
                                            self.pack_format = def.pack_format;
                                            self.skip_json = def.skip_json;
                                            self.skip_js = def.skip_js;
                                            self.skip_jar = def.skip_jar;
                                            self.skip_book = def.skip_book;
                                            self.enable_llm_log = def.enable_llm_log;
                                            self.technical_constraints = def.technical_constraints;
                                            {
                                                let mut priority = self.glossary_priority.lock().unwrap();
                                                *priority = def.glossary_priority;
                                            }
                                            self.save_config();
                                            self.refresh_models();
                                        }
                                    });
                                },
                            );
                        });

                        // --- 模型選擇 (混合：ComboBox 鎖定，刷新按鈕不鎖定) ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("選擇模型:").color(label_color).strong());

                            let models = self.available_models.lock().unwrap().clone();
                            let mut changed = false;

                            ui.add_enabled_ui(ui_enabled, |ui| {
                                egui::ComboBox::from_id_source("dynamic_model_combo")
                                    .selected_text(
                                        if self.api_key.is_empty() && self.api_provider != "Ollama"
                                        {
                                            "請輸入API金鑰".to_string()
                                        } else if self.selected_model.is_empty() {
                                            "請選取模型".to_string()
                                        } else {
                                            self.selected_model.clone()
                                        },
                                    )
                                    .width(ui.available_width() - 40.0)
                                    .show_ui(ui, |ui| {
                                        if models.is_empty() {
                                            ui.label("請先更新列表...");
                                        }
                                        for m in &models {
                                            if ui
                                                .selectable_value(
                                                    &mut self.selected_model,
                                                    m.clone(),
                                                    m.to_string(),
                                                )
                                                .changed()
                                            {
                                                changed = true;
                                            }
                                        }
                                    });
                            });

                            if ui.button("🔄").clicked() {
                                self.refresh_models();
                            }

                            if changed {
                                self.save_config();
                            }
                        });

                        if self.api_provider == "Ollama" {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Ollama URL:").color(label_color));
                                ui.add_enabled_ui(ui_enabled, |ui| {
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut self.ollama_url)
                                                .desired_width(ui.available_width()),
                                        )
                                        .changed()
                                    {
                                        self.save_config();
                                        self.refresh_models();
                                    }
                                });
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("API Key:").color(label_color).strong(),
                                );
                                ui.add_enabled_ui(ui_enabled, |ui| {
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut self.api_key)
                                            .password(true)
                                            .desired_width(ui.available_width() - 80.0),
                                    );

                                    if resp.lost_focus() || resp.changed() {
                                        self.save_config();
                                        self.refresh_models();
                                    }
                                });
                            });
                        }

                        // --- 參數設定 (混合：效能參數鎖定，字體大小不鎖定) ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("批次量:").color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                let mut bs = self.batch_size as i32;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut bs)
                                            .clamp_range(1..=300)
                                            .speed(1.0),
                                    )
                                    .changed()
                                {
                                    self.batch_size = bs as u32;
                                    self.save_config();
                                }
                            });

                            ui.add_space(12.0);
                            ui.label(egui::RichText::new("字數上限:").color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut self.batch_max_chars)
                                            .clamp_range(1..=10000)
                                            .speed(10.0),
                                    )
                                    .changed()
                                {
                                    self.save_config();
                                }
                            });

                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new("逾時 (秒):")
                                    .color(label_color)
                                    .strong(),
                            );
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut self.ollama_timeout)
                                            .clamp_range(1..=600)
                                            .speed(1.0),
                                    )
                                    .changed()
                                {
                                    self.save_config();
                                }
                            });

                            ui.add_space(12.0);
                            ui.label(egui::RichText::new("字體:").color(label_color).strong());
                            if ui
                                .add(
                                    egui::DragValue::new(&mut self.font_size)
                                        .clamp_range(12.0..=30.0)
                                        .suffix("pt")
                                        .speed(0.5),
                                )
                                .changed()
                            {
                                self.save_config();
                            }

                            ui.add_space(12.0);
                            ui.label(egui::RichText::new("FPS:").color(label_color).strong());
                            ui.checkbox(&mut self.enable_custom_fps, "");
                            if self.enable_custom_fps {
                                let fps_input = ui.add(
                                    egui::DragValue::new(&mut self.custom_fps)
                                        .clamp_range(1..=240)
                                        .speed(1)
                                        .suffix(" FPS"),
                                );
                                if fps_input.changed() {
                                    self.save_config();
                                }
                            } else {
                                ui.label(
                                    egui::RichText::new("(預設:vsync)")
                                        .color(label_color)
                                        .strong(),
                                );
                            }
                        });

                        ui.separator();
                        ui.add_space(1.0);

                        // --- 翻譯提示 (鎖定) ---
                        ui.group(|ui| {
                            ui.label(
                                egui::RichText::new("📝 翻譯提示:")
                                    .color(label_color)
                                    .strong(),
                            );
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                if ui
                                    .add(
                                        egui::TextEdit::multiline(&mut self.translation_prompt)
                                            .desired_rows(2)
                                            .desired_width(ui.available_width()),
                                    )
                                    .changed()
                                {
                                    self.save_config();
                                }
                            });
                        });

                        // API 連線狀態指示燈 (不鎖定，僅供視圖偵測)
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("🔍 API 連線狀態:")
                                    .color(label_color)
                                    .strong(),
                            );
                            let models_locked = self.available_models.lock().unwrap();
                            let is_ollama = self.api_provider == "Ollama";
                            let is_ready = if is_ollama {
                                !models_locked.is_empty()
                            } else {
                                !self.api_key.is_empty() && !models_locked.is_empty()
                            };
                            let is_light = self.theme == "light";
                            let status_text = if is_ready {
                                "[已連線]"
                            } else {
                                "[未就緒]"
                            };
                            let status_color = if is_ready {
                                if is_light {
                                    egui::Color32::from_rgb(0, 130, 0)
                                } else {
                                    egui::Color32::GREEN
                                }
                            } else {
                                if is_light {
                                    egui::Color32::from_rgb(160, 80, 0)
                                } else {
                                    egui::Color32::from_rgb(255, 165, 0)
                                }
                            };
                            ui.label(egui::RichText::new(status_text).color(status_color));
                        });
                    });
                });
        });
    }
}
