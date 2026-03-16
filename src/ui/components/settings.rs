use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    /// 渲染 API 設定面板 (細粒度鎖定優化：僅在翻譯時鎖定必要參數)
    pub fn render_settings_panel(&mut self, ui: &mut egui::Ui, ui_enabled: bool, ctx: &egui::Context) {
        if self.show_restore_default_confirm {
            self.render_restore_default_confirm(ctx);
        }
        if !self.show_api_settings {
            return;
        }
        ui.add_space(4.0);
        ui.group(|ui| {
            egui::ScrollArea::vertical()
                .id_source("settings_scroll_area")
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
                            ui.label(egui::RichText::new(self.i18n.label_provider.clone()).color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                egui::ComboBox::from_id_source("provider_combo")
                                    .selected_text(&self.api_provider)
                                    .width(140.0)
                                    .show_ui(ui, |ui| {
                                        let mut providers = vec![
                                            self.i18n.label_provider_none.clone(),
                                            "Gemini".to_string(),
                                            "OpenAI".to_string(),
                                            "DeepSeek".to_string(),
                                            "Mistral".to_string(),
                                            "DeepL".to_string(),
                                            "Ollama".to_string(),
                                            "Google Free".to_string(),
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
                                        if ui.button(self.i18n.btn_restore_defaults.clone()).clicked() {
                                            self.show_restore_default_confirm = true;
                                        }
                                    });
                                },
                            );
                        });

                        // --- 模型選擇 (混合：ComboBox 鎖定，刷新按鈕不鎖定) ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.i18n.label_model.clone()).color(label_color).strong());

                            let models = self.available_models.lock().unwrap().clone();
                            let mut changed = false;

                            ui.add_enabled_ui(ui_enabled, |ui| {
                                egui::ComboBox::from_id_source("dynamic_model_combo")
                                    .selected_text(
                                        if self.api_key.is_empty() && self.api_provider != "Ollama"
                                        {
                                            self.i18n.prompt_enter_key.clone()
                                        } else if self.selected_model.is_empty() {
                                            self.i18n.prompt_select_model.clone()
                                        } else {
                                            self.selected_model.clone()
                                        },
                                    )
                                    .width(ui.available_width() - 40.0)
                                    .show_ui(ui, |ui| {
                                        if models.is_empty() {
                                            ui.label(self.i18n.prompt_update_list.clone());
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
                                ui.label(egui::RichText::new(format!("{}:", self.i18n.label_ollama_url)).color(label_color).strong());
                                ui.add_enabled_ui(ui_enabled, |ui| {
                                    let (input_bg, input_text) = self.get_instance_style("input_ollama_url");
                                    ui.scope(|ui| {
                                        ui.visuals_mut().extreme_bg_color = input_bg;
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut self.ollama_url)
                                                    .text_color(input_text)
                                                    .desired_width(ui.available_width()),
                                            )
                                            .changed()
                                        {
                                            self.save_config();
                                            self.refresh_models();
                                        }
                                    });
                                });
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(self.i18n.label_api_key.clone()).color(label_color).strong(),
                                );
                                ui.add_enabled_ui(ui_enabled, |ui| {
                                    let (input_bg, input_text) = self.get_instance_style("input_api_key");
                                    ui.scope(|ui| {
                                        ui.visuals_mut().extreme_bg_color = input_bg;
                                        let resp = ui.add(
                                            egui::TextEdit::singleline(&mut self.api_key)
                                                .password(true)
                                                .text_color(input_text)
                                                .desired_width(ui.available_width() - 80.0),
                                        );

                                        if resp.lost_focus() || resp.changed() {
                                            self.save_config();
                                            self.refresh_models();
                                        }
                                    });
                                });
                            });
                        }

                        // --- 參數設定 (混合：效能參數鎖定，字體大小不鎖定) ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.i18n.label_batch_size.clone()).color(label_color).strong());
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
                            ui.label(egui::RichText::new(self.i18n.label_max_chars.clone()).color(label_color).strong()); // 縮短名稱避免擁擠
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
                                egui::RichText::new(self.i18n.label_timeout.clone())
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
                            ui.label(egui::RichText::new(self.i18n.label_font_size.clone()).color(label_color).strong());
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
                            ui.label(egui::RichText::new(self.i18n.label_pack_format.clone()).color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                egui::ComboBox::from_id_source("pack_format_presets")
                                    .selected_text(self.i18n.label_presets.clone())
                                    .width(80.0)
                                    .show_ui(ui, |ui| {
                                        for (ver, fmt) in &[
                                            ("1.21.4", 46),
                                            ("1.21.2", 42),
                                            ("1.21", 34),
                                            ("1.20.4", 22),
                                            ("1.20.2", 18),
                                            ("1.20.1", 15),
                                            ("1.19.4", 13),
                                            ("1.19.2", 10),
                                            ("1.18.2", 9),
                                            ("1.16.5", 6),
                                        ] {
                                            if ui.selectable_label(self.pack_format == *fmt, *ver).clicked() {
                                                self.pack_format = *fmt;
                                                self.save_config();
                                            }
                                        }
                                    });

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
                        });

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.i18n.label_source_lang.clone()).color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                egui::ComboBox::from_id_source("source_lang_combo")
                                    .selected_text(&self.source_lang)
                                    .show_ui(ui, |ui| {
                                        let langs = ["en_us", "en_gb", "zh_tw", "zh_cn", "ja_jp", "ko_kr", "fr_fr", "de_de", "es_es", "ru_ru"];
                                        for l in langs {
                                            if ui.selectable_value(&mut self.source_lang, l.to_string(), l).clicked() {
                                                self.trigger_save();
                                            }
                                        }
                                    });
                            });

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(self.i18n.label_target_lang.clone()).color(label_color).strong());
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                egui::ComboBox::from_id_source("target_lang_combo")
                                    .selected_text(&self.target_lang)
                                    .show_ui(ui, |ui| {
                                        let langs = ["zh_tw", "zh_cn", "en_us", "ja_jp", "ko_kr", "fr_fr", "de_de", "es_es", "ru_ru"];
                                        for l in langs {
                                            if ui.selectable_value(&mut self.target_lang, l.to_string(), l).clicked() {
                                                self.trigger_save();
                                            }
                                        }
                                    });
                            });

                            ui.add_space(12.0);
                            ui.label(egui::RichText::new(self.i18n.label_fps.clone()).color(label_color).strong());
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
                                    egui::RichText::new(self.i18n.label_fps_preset_vsync.clone())
                                        .color(label_color),
                                );
                            }
                        });

                        ui.separator();
                        ui.add_space(1.0);

                        // --- 使用者翻譯提示 (鎖定) ---
                        ui.group(|ui| {
                            ui.label(
                                egui::RichText::new(self.i18n.label_user_prompt.clone())
                                    .color(label_color)
                                    .strong(),
                            );
                            ui.add_enabled_ui(ui_enabled, |ui| {
                                let (prompt_bg, input_text) = self.get_instance_style("input_user_prompt");
                                ui.visuals_mut().extreme_bg_color = prompt_bg;

                                if ui
                                    .add(
                                        egui::TextEdit::multiline(&mut self.user_prompt)
                                            .text_color(input_text)
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
                                egui::RichText::new(self.i18n.label_api_status.clone())
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
                                self.i18n.status_connected.clone()
                            } else {
                                self.i18n.status_not_ready.clone()
                            };
                            let status_color = if is_ready {
                                if self.theme == "light" {
                                    egui::Color32::from_rgb(0, 100, 0) // 深綠
                                } else {
                                    egui::Color32::GREEN
                                }
                            } else if self.theme == "light" {
                                egui::Color32::from_rgb(180, 100, 0) // 深橘
                            } else {
                                egui::Color32::from_rgb(255, 165, 0) // 亮橘
                            };
                            ui.label(egui::RichText::new(status_text).color(status_color));
                        });
                    });
                });
        });
        self.render_restore_default_confirm(ctx);
    }

    pub fn render_restore_default_confirm(&mut self, ctx: &egui::Context) {
        if !self.show_restore_default_confirm {
            return;
        }

        egui::Window::new(self.i18n.confirm_restore_title.clone())
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.label(self.i18n.confirm_restore_text.clone());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.btn_confirm_restore.clone()).clicked() {
                        // 僅重置外觀相關設定 (從調色盤觸發)
                        let style_def = crate::config::settings::StyleConfig::default();
                        
                        self.theme = style_def.theme;
                        self.font_size = style_def.font_size;
                        
                        // 外觀相關
                        self.dark_bg = style_def.dark_bg;
                        self.dark_text = style_def.dark_text;
                        self.light_bg = style_def.light_bg;
                        self.light_text = style_def.light_text;
                        self.dark_label = style_def.dark_label;
                        self.light_label = style_def.light_label;
                        self.dark_btn_bg = style_def.dark_btn_bg;
                        self.dark_btn_text = style_def.dark_btn_text;
                        self.light_btn_bg = style_def.light_btn_bg;
                        self.light_btn_text = style_def.light_btn_text;
                        self.dark_input_bg = style_def.dark_input_bg;
                        self.light_input_bg = style_def.light_input_bg;
                        self.dark_list_bg = style_def.dark_list_bg;
                        self.light_list_bg = style_def.light_list_bg;
                        self.dark_tab_active = style_def.dark_tab_active;
                        self.dark_tab_inactive = style_def.dark_tab_inactive;
                        self.light_tab_active = style_def.light_tab_active;
                        self.light_tab_inactive = style_def.light_tab_inactive;
                        self.btn_rounding_enabled = style_def.btn_rounding_enabled;
                        self.btn_rounding_value = style_def.btn_rounding_value;
                        self.progress_pulse_enabled = style_def.progress_pulse_enabled;
                        self.progress_pulse_speed = style_def.progress_pulse_speed;
                        self.instance_overrides = style_def.instance_overrides.clone();
                        
                        self.trigger_save();
                        self.show_restore_default_confirm = false;
                    }
                    if ui.button(self.i18n.btn_cancel.clone()).clicked() {
                        self.show_restore_default_confirm = false;
                    }
                });
            });
    }
}
