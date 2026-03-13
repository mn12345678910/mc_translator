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
                                        let mut providers = vec![
                                            "無", "Gemini", "OpenAI", "DeepSeek", "Mistral", "DeepL",
                                            "Ollama", "Google Free",
                                        ];
                                        // 保持 "無" 在最前面，其餘排序
                                        let none_provider = providers.remove(0);
                                        providers.sort();
                                        providers.insert(0, none_provider);

                                        for p in providers {
                                            if ui
                                                .selectable_value(
                                                    &mut self.api_provider,
                                                    p.to_string(),
                                                    p,
                                                )
                                                .changed()
                                            {
                                                self.selected_model = String::new(); // 切換服務商時清空已選模型 (Mismatched Model Fix)
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
                                            self.show_restore_default_confirm = true;
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
                                ui.label(egui::RichText::new("Ollama URL:").color(label_color).strong());
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
                            ui.label(egui::RichText::new("(1-300)").color(label_color).small());

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("上限:").color(label_color).strong()); // 縮短名稱避免擁擠
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
                            ui.label(egui::RichText::new("(1-10k)").color(label_color).small());

                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("逾時:")
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
                            ui.label(egui::RichText::new("(1-600s)").color(label_color).small());

                            ui.add_space(8.0);
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
                            ui.label(egui::RichText::new("(12-30)").color(label_color).small());
                        });

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("版本:").color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut self.pack_format)
                                            .clamp_range(1..=100)
                                            .speed(1.0),
                                    )
                                    .changed()
                                {
                                    self.save_config();
                                }
                            });
                            ui.label(egui::RichText::new("(1-100)").color(label_color).small());

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
                                ui.label(egui::RichText::new("(1-240)").color(label_color).small());
                            } else {
                                ui.label(
                                    egui::RichText::new("(預設:vsync)")
                                        .color(label_color),
                                );
                            }
                        });

                        ui.separator();
                        ui.add_space(1.0);

                        // --- 使用者翻譯提示 (鎖定) ---
                        ui.group(|ui| {
                            ui.label(
                                egui::RichText::new("📝 使用者翻譯提示:")
                                    .color(label_color)
                                    .strong(),
                            );
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                if ui
                                    .add(
                                        egui::TextEdit::multiline(&mut self.user_prompt)
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
                            let is_ready = if self.api_provider == "Ollama" {
                                !models_locked.is_empty()
                            } else {
                                !self.api_key.is_empty() && !models_locked.is_empty()
                            };
                            let status_text = if is_ready {
                                "[已連線]"
                            } else {
                                "[未就緒]"
                            };
                            let status_color = if is_ready {
                                if self.theme == "light" {
                                    egui::Color32::from_rgb(0, 100, 0) // 深綠
                                } else {
                                    egui::Color32::GREEN
                                }
                            } else {
                                if self.theme == "light" {
                                    egui::Color32::from_rgb(180, 100, 0) // 深橘
                                } else {
                                    egui::Color32::from_rgb(255, 165, 0) // 亮橘
                                }
                            };
                            ui.label(egui::RichText::new(status_text).color(status_color));
                        });
                    });
                });
        });
        self.render_restore_default_confirm(_ctx);
    }

    fn render_restore_default_confirm(&mut self, ctx: &egui::Context) {
        if !self.show_restore_default_confirm {
            return;
        }

        egui::Window::new("確認恢復預設")
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.label("您確定要將所有設定恢復為系統預設值嗎？\n這將覆蓋您目前的所有設定。");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("確定恢復").clicked() {
                        let def = crate::config::settings::AppConfig::default();
                        self.api_provider = def.api_provider;
                        self.api_key = def.api_key;
                        self.selected_model = def.model;
                        self.ollama_url = def.ollama_url;
                        self.batch_size = def.batch_size;
                        self.batch_max_chars = def.batch_max_chars;
                        self.ollama_timeout = def.ollama_timeout;
                        self.user_prompt = def.user_prompt;
                        self.system_prompt = def.system_prompt;
                        self.output_dir = def.output_dir;
                        self.theme = def.theme;
                        self.font_size = def.font_size;
                        self.pack_format = def.pack_format;
                        self.skip_json = def.skip_json;
                        self.skip_js = def.skip_js;
                        self.skip_jar = def.skip_jar;
                        self.skip_book = def.skip_book;
                        self.enable_llm_log = def.enable_llm_log;
                        self.enable_custom_fps = def.enable_custom_fps;
                        self.custom_fps = def.custom_fps;
                        self.show_api_settings = def.show_api_settings;
                        self.show_developer_mode = def.show_developer_mode;
                        // 不再從 Config 恢復 show_memory_viewer，預設關閉
                        self.show_memory_viewer = false;
                        {
                            let mut priority = self.glossary_priority.lock().unwrap();
                            *priority = def.glossary_priority;
                        }
                        self.save_config();
                        self.refresh_models();
                        self.show_restore_default_confirm = false;
                    }
                    if ui.button("取消").clicked() {
                        self.show_restore_default_confirm = false;
                    }
                });
            });
    }
}
