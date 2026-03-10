//! # 介面模組
//! 負責所有 egui GUI 渲染邏輯。

use crate::state_and_log::{AppState, ViewerSharedState};
use eframe::egui;
use std::fs;

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self
            .viewer_shared
            .close_requested
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            self.viewer_shared
                .close_requested
                .store(false, std::sync::atomic::Ordering::SeqCst);

            self.show_memory_viewer = false;
            // 重要：重置旗標以利下次開啟時重新整理辭典
            let mut opened = self.viewer_shared.opened_last_frame.lock().unwrap();
            *opened = false;
        }

        // 幀數控制
        if self.enable_custom_fps && self.custom_fps > 0 {
            let target_fps = self.custom_fps;
            let min_frame_time = std::time::Duration::from_millis((1000 / target_fps) as u64);

            let now = std::time::Instant::now();
            let elapsed = now.duration_since(self.last_frame_time);

            if elapsed < min_frame_time {
                let sleep_time = min_frame_time - elapsed;
                std::thread::sleep(sleep_time);
            }

            self.last_frame_time = std::time::Instant::now();
        }

        // 1. 套用主題與視覺風格
        self.render_theme_application(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 4.0);
                let processing = *self.is_processing.lock().unwrap();
                let ui_enabled = !processing || *self.is_paused.lock().unwrap();

                // 2. 標頭控制項 (檔案/資料夾/輸出路徑)
                self.render_header_controls(ui, ui_enabled);
                ui.add_space(1.0);

                // 3. 設定面板 (還原至備份版本，包含進階參數)
                self.render_settings_panel(ui, ui_enabled, ctx);

                // 4. 開發者模式面板 (還原至備份版本，包含 Toggle 開關)
                self.render_developer_mode_panel(ui);
                ui.add_space(1.0);

                // 5. 進度顯示區域 (還原雙進度條)
                self.render_progress_section(ui, ctx, processing);

                // 6. 操作按鈕與停止對話框 (還原獨立尺寸)
                self.render_action_buttons(ui, ctx, processing);

                // 7. 日誌區域
                self.render_log_area(ui);
            });

        // 8. 建議詞管理器 (Viewport)
        if self.show_memory_viewer {
            self.show_viewport_if_needed(ctx);
        }

        // 處理中時持續重繪
        if *self.is_processing.lock().unwrap() {
            ctx.request_repaint();
        }
    }
}

impl AppState {
    /// 若有需要則顯示建議詞管理器 Viewport
    fn show_viewport_if_needed(&mut self, ctx: &egui::Context) {
        if !self.show_memory_viewer {
            return;
        }

        let mut opened = self.viewer_shared.opened_last_frame.lock().unwrap();

        if !*opened {
            // 第一次打開：初始化
            *opened = true;
            drop(opened);
            self.refresh_all_dictionaries();
            self.is_memory_viewer_open
                .store(true, std::sync::atomic::Ordering::SeqCst);
        } else {
            drop(opened);
        }

        // 每 frame 都必須調用 show_viewport_deferred 才能保持 viewport 持續顯示
        self.create_viewport_deferred(ctx);
    }

    /// 創建建議詞管理器 Viewport
    fn create_viewport_deferred(&mut self, ctx: &egui::Context) {
        // 收集所需的所有 Arc 變數
        if self.show_memory_viewer {
            let is_processing = self.is_processing.clone();
            let is_paused = self.is_paused.clone();
            let viewer_shared = self.viewer_shared.clone();
            let translation_memory = self.translation_memory.clone();
            let inferred_match_map = self.inferred_match_map.clone();
            let term_replacements = self.term_replacements.clone();
            let dict_cache = self.dict_cache.clone();
            let dict_search = self.dict_search.clone();
            let dict_search_last = self.dict_search_last.clone();
            let dict_page = self.dict_page.clone();
            let dict_active_tab = self.dict_active_tab.clone();
            let dict_edit_key = self.dict_edit_key.clone();
            let dict_edit_value = self.dict_edit_value.clone();
            let dict_new_key = self.dict_new_key.clone();
            let dict_new_value = self.dict_new_value.clone();
            let dict_replace_target = self.dict_replace_target.clone();
            let dict_replace_new = self.dict_replace_new.clone();
            let dict_replace_all = self.dict_replace_all.clone();
            let show_dict_add_dialog = self.show_dict_add_dialog.clone();
            let show_dict_replace_dialog = self.show_dict_replace_dialog.clone();
            let show_dict_clear_confirm = self.show_dict_clear_confirm.clone();
            let glossary_priority = self.glossary_priority.clone();
            let is_memory_viewer_open = self.is_memory_viewer_open.clone();

            let opened_last_frame = self.viewer_shared.opened_last_frame.lock().unwrap().clone();

            // 只有在剛開啟的一幀或特定條件下才動態計算尺寸，避免每幀 Builder 造成的尺寸閃動
            let mut builder = egui::ViewportBuilder::default()
                .with_title("📖 建議詞管理器")
                .with_resizable(true);

            if !opened_last_frame {
                builder = builder
                    .with_inner_size([self.viewer_width, self.viewer_height])
                    .with_position([self.viewer_x, self.viewer_y]);
            }

            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of("memory_viewer"),
                builder,
                move |ctx, _viewport_id| {
                    // 監聽關閉事件
                    if ctx.input(|i| i.viewport().close_requested()) {
                        is_memory_viewer_open.store(false, std::sync::atomic::Ordering::SeqCst);
                        viewer_shared
                            .close_requested
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }

                    // 直接調用渲染內容，內部已有 CentralPanel，移除此處的 egui::CentralPanel 以解決閃爍與重複邊距
                    Self::render_memory_viewer_content(
                        ctx,
                        is_processing.clone(),
                        is_paused.clone(),
                        viewer_shared.clone(),
                        translation_memory.clone(),
                        inferred_match_map.clone(),
                        term_replacements.clone(),
                        dict_cache.clone(),
                        dict_search.clone(),
                        dict_search_last.clone(),
                        dict_page.clone(),
                        dict_active_tab.clone(),
                        dict_edit_key.clone(),
                        dict_edit_value.clone(),
                        dict_new_key.clone(),
                        dict_new_value.clone(),
                        dict_replace_target.clone(),
                        dict_replace_new.clone(),
                        dict_replace_all.clone(),
                        show_dict_add_dialog.clone(),
                        show_dict_replace_dialog.clone(),
                        show_dict_clear_confirm.clone(),
                        glossary_priority.clone(),
                    );
                },
            );
        }
    }

    /// 渲染並套用主題與視覺風格 (還原自備份版本，含視覺統一優化)
    fn render_theme_application(&mut self, ctx: &egui::Context) {
        let is_dark = self.theme == "dark";
        let current_is_dark = ctx.style().visuals.dark_mode;

        let current_font_size = ctx
            .style()
            .text_styles
            .get(&egui::TextStyle::Body)
            .map(|f| f.size)
            .unwrap_or(0.0);
        let font_size_changed = (current_font_size - self.font_size).abs() > 0.1;

        let needs_update = if is_dark {
            !current_is_dark || font_size_changed
        } else {
            current_is_dark
                || ctx.style().visuals.window_fill != egui::Color32::from_rgb(0xFF, 0xDE, 0xAD)
                || font_size_changed
        };

        if needs_update {
            let visuals = if is_dark {
                let mut v = egui::Visuals::dark();
                v.window_fill = egui::Color32::from_rgb(30, 30, 35);
                v.panel_fill = egui::Color32::from_rgb(30, 30, 35);
                v.extreme_bg_color = egui::Color32::from_rgb(20, 20, 25);
                v.selection.bg_fill = egui::Color32::from_rgb(60, 100, 150);

                let btn_bg = egui::Color32::from_rgb(60, 60, 70);
                v.widgets.inactive.bg_fill = btn_bg;
                v.widgets.inactive.weak_bg_fill = btn_bg;
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(75, 75, 90);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(90, 90, 110);
                v.faint_bg_color = egui::Color32::from_rgb(40, 40, 45);
                v
            } else {
                let mut v = egui::Visuals::light();
                let bg_color = egui::Color32::from_rgb(0xFF, 0xDE, 0xAD);
                v.window_fill = bg_color;
                v.panel_fill = bg_color;

                let btn_bg = egui::Color32::from_rgb(0xE3, 0xC3, 0x95);
                let btn_stroke_color = egui::Color32::from_rgb(30, 30, 30);

                v.widgets.inactive.bg_fill = btn_bg;
                v.widgets.inactive.weak_bg_fill = btn_bg;
                v.widgets.inactive.fg_stroke = egui::Stroke::new(1.2, btn_stroke_color);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0xCD, 0xAA, 0x7D);
                v.widgets.hovered.fg_stroke = egui::Stroke::new(1.8, btn_stroke_color);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(0xA0, 0x7B, 0x7B);

                v.extreme_bg_color = egui::Color32::WHITE;
                v.selection.bg_fill = egui::Color32::from_rgb(0xCD, 0x85, 0x3F);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0xD2, 0xB4, 0x8C);
                v.widgets.noninteractive.fg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::from_gray(100));
                v.faint_bg_color = egui::Color32::from_rgb(0xEF, 0xD0, 0x9E);
                v.override_text_color = Some(egui::Color32::from_rgb(50, 40, 30));
                v
            };

            let mut style = (*ctx.style()).clone();
            style.visuals = visuals;
            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::proportional(self.font_size * 0.8),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(self.font_size),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(self.font_size),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(self.font_size * 1.5),
            );
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::monospace(self.font_size),
            );
            ctx.set_style(style);

            // 同步主題至子視窗 (Revision 14.1/14.2)
            *self.viewer_shared.theme.write().unwrap() = self.theme.clone();
            *self.viewer_shared.font_size.write().unwrap() = self.font_size;
            // 立即請求重繪子視窗 (Revision 14.2)
            ctx.request_repaint_of(egui::ViewportId::from_hash_of("memory_viewer"));
        }
    }

    /// 渲染頂部控制項 (還原自備份版本佈局)
    fn render_header_controls(&mut self, ui: &mut egui::Ui, ui_enabled: bool) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if ui
                .add_enabled(ui_enabled, egui::Button::new("📁 選擇檔案"))
                .clicked()
            {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("JAR, JS & JSON 檔案", &["jar", "js", "json"])
                    .pick_files()
                {
                    self.input_paths = files
                        .into_iter()
                        .map(|p| {
                            let rel = p
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            (p, rel.replace('\\', "/"))
                        })
                        .collect();
                    self.add_log(&format!("已選擇 {} 個檔案", self.input_paths.len()));
                    // [Rev 13] 選取時立即更新總數
                    *self.global_total.lock().unwrap() = self.input_paths.len() as f32;
                    *self.global_progress.lock().unwrap() = 0.0;
                }
            }
            if ui
                .add_enabled(ui_enabled, egui::Button::new("📂 選擇資料夾"))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let files = scan_files_recursive(&path, &path);
                    self.add_log(&format!("已選擇 {} 個檔案", files.len()));
                    self.input_paths = files;
                    // [Rev 13] 選取時立即更新總數
                    *self.global_total.lock().unwrap() = self.input_paths.len() as f32;
                    *self.global_progress.lock().unwrap() = 0.0;
                }
            }

            ui.separator();

            if ui
                .add_enabled(ui_enabled, egui::Button::new("📤 輸出資料夾"))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.output_dir = path.to_string_lossy().to_string();
                    self.save_config();
                    self.add_log(&format!("輸出資料夾已設定: {}", self.output_dir));
                }
            }

            if ui.button("📂 打開輸出").clicked() {
                let target = if self.output_dir.is_empty() {
                    "LLMTranslator"
                } else {
                    &self.output_dir
                };
                let path = std::path::Path::new(target);
                if !path.exists() {
                    let _ = std::fs::create_dir_all(path);
                }
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("explorer").arg(path).spawn();
            }

            let display_path = if self.output_dir.is_empty() {
                "預設: ./LLMTranslator".into()
            } else {
                self.output_dir.clone()
            };
            ui.add(egui::Label::new(display_path).truncate(true));

            self.render_status_navigation(ui);
        });
    }

    /// 渲染導航按鈕 (⚙ 🌓 📖 🔧) 並修復建議詞開啟邏輯
    fn render_status_navigation(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("⚙").on_hover_text("API 翻譯設定").clicked() {
                self.show_api_settings = !self.show_api_settings;
                if self.show_api_settings {
                    self.show_developer_mode = false;
                }
            }
            if ui.button("📖").on_hover_text("建議詞管理器").clicked() {
                self.show_memory_viewer = !self.show_memory_viewer;
                if !self.show_memory_viewer {
                    let mut opened = self.viewer_shared.opened_last_frame.lock().unwrap();
                    *opened = false;
                }
            }
            if ui.button("🌓").on_hover_text("切換主題").clicked() {
                self.theme = if self.theme == "dark" {
                    "light".into()
                } else {
                    "dark".into()
                };
                self.save_config();
            }
            if ui.button("🔧").on_hover_text("開發人員模式").clicked() {
                self.show_developer_mode = !self.show_developer_mode;
                if self.show_developer_mode {
                    self.show_api_settings = false;
                }
            }
            ui.add_space(8.0);
        });
    }

    /// 渲染 API 設定面板 (細粒度鎖定優化：僅在翻譯時鎖定必要參數)
    fn render_settings_panel(&mut self, ui: &mut egui::Ui, ui_enabled: bool, _ctx: &egui::Context) {
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
                            egui::Color32::from_rgb(100, 50, 0)
                        } else {
                            egui::Color32::from_rgb(200, 160, 100)
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
                                            let def = crate::config::AppConfig::default();
                                            self.api_provider = def.provider;
                                            self.api_key = def.api_key;
                                            self.selected_model = def.model;
                                            self.ollama_url = def.ollama_url;
                                            self.batch_size = def.batch_size;
                                            self.batch_max_chars = def.batch_max_chars;
                                            self.ollama_timeout = def.ollama_timeout;
                                            self.translation_prompt = def.translation_prompt;
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
                            ui.label(egui::RichText::new("批次量:").color(label_color));
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
                            ui.label(egui::RichText::new("字數上限:").color(label_color));
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
                            ui.label(egui::RichText::new("逾時 (秒):").color(label_color));
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
                            ui.label(egui::RichText::new("字體:").color(label_color));
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
                        });

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("FPS:").color(label_color));
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
                                ui.label(egui::RichText::new("(預設:vsync)").small());
                            }
                        });

                        ui.separator();
                        ui.add_space(1.0);

                        // --- 翻譯提示 (鎖定) ---
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("📝 翻譯提示:").color(label_color));
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

    /// 渲染開發人員模式面板 (還原至備份 Grid，套用隨主題變色標籤)
    fn render_developer_mode_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_developer_mode {
            return;
        }
        let label_color = if self.theme == "light" {
            egui::Color32::from_rgb(100, 50, 0)
        } else {
            egui::Color32::from_rgb(200, 160, 100)
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
                        egui::Label::new(egui::RichText::new(json_label).color(label_color)),
                    );
                    ui.add(toggle(&mut self.skip_json));

                    let jar_label = if self.skip_jar {
                        "跳過 .jar"
                    } else {
                        "不跳過 .jar"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(egui::RichText::new(jar_label).color(label_color)),
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
                        egui::Label::new(egui::RichText::new(js_label).color(label_color)),
                    );
                    ui.add(toggle(&mut self.skip_js));

                    let log_label = if self.enable_llm_log {
                        "開啟記錄日誌"
                    } else {
                        "關閉記錄日誌"
                    };
                    ui.add_sized(
                        [105.0, 20.0],
                        egui::Label::new(egui::RichText::new(log_label).color(label_color)),
                    );
                    ui.add(toggle(&mut self.enable_llm_log));
                    ui.end_row();
                });
        });
    }

    fn render_progress_section(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        processing: bool,
    ) {
        ui.separator();
        ui.add_space(1.0);

        let label_color = if self.theme == "light" {
            egui::Color32::from_rgb(100, 50, 0)
        } else {
            egui::Color32::from_rgb(200, 160, 100)
        };

        let mut current_status = self.status.lock().unwrap().clone();
        if processing {
            let time = ctx.input(|i| i.time);
            let dots = match (time * 2.0) as i32 % 4 {
                1 => ".",
                2 => "..",
                3 => "...",
                _ => "",
            };
            current_status.push_str(dots);
        }
        ui.label(egui::RichText::new(format!("目前狀態: {}", current_status)).color(label_color));

        // 如果不在處理中，清空進度顯示 (除非是在暫停中)
        let (prog, total, g_prog, g_total) = {
            (
                *self.progress.lock().unwrap(),
                *self.progress_total.lock().unwrap(),
                *self.global_progress.lock().unwrap(),
                *self.global_total.lock().unwrap(),
            )
        };

        // 目前檔案 (顯示條目進度)
        let ratio = if total > 0.0 { prog / total } else { 0.0 };
        ui.add(egui::ProgressBar::new(ratio).show_percentage().text(
            egui::RichText::new(format!("目前檔案: ({}/{})", prog as u32, total as u32)).strong(),
        ));

        // 總進度 (顯示檔案進度)
        let g_ratio = if g_total > 0.0 { g_prog / g_total } else { 0.0 };
        ui.add(
            egui::ProgressBar::new(g_ratio).text(
                egui::RichText::new(format!(
                    "總進度: ({}/{}) {}%",
                    g_prog as u32,
                    g_total as u32,
                    (g_ratio * 100.0) as u32
                ))
                .strong(),
            ),
        );
        ui.add_space(1.0);
    }

    fn render_action_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, processing: bool) {
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

    fn render_log_area(&mut self, ui: &mut egui::Ui) {
        let label_color = if self.theme == "light" {
            egui::Color32::from_rgb(100, 50, 0)
        } else {
            egui::Color32::from_rgb(200, 160, 100)
        };
        ui.separator();
        ui.label(egui::RichText::new("執行日誌:").color(label_color));
        let log = self.log.lock().unwrap();
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for line in log.iter() {
                    ui.add(egui::Label::new(egui::RichText::new(line).monospace()));
                }
            });
    }

    fn render_memory_viewer_content(
        ctx: &egui::Context,
        is_processing: std::sync::Arc<std::sync::Mutex<bool>>,
        _is_paused: std::sync::Arc<std::sync::Mutex<bool>>,
        viewer_shared: std::sync::Arc<ViewerSharedState>,
        translation_memory: std::sync::Arc<
            std::sync::Mutex<std::collections::HashMap<String, String>>,
        >,
        inferred_match_map: std::sync::Arc<
            std::sync::Mutex<std::collections::HashMap<String, String>>,
        >,
        _term_replacements: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>,
        _dict_cache: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>,
        dict_search: std::sync::Arc<std::sync::Mutex<String>>,
        dict_search_last: std::sync::Arc<std::sync::Mutex<(String, usize)>>,
        dict_page: std::sync::Arc<std::sync::Mutex<usize>>,
        dict_active_tab: std::sync::Arc<std::sync::Mutex<usize>>,
        dict_edit_key: std::sync::Arc<std::sync::Mutex<Option<String>>>,
        dict_edit_value: std::sync::Arc<std::sync::Mutex<String>>,
        dict_new_key: std::sync::Arc<std::sync::Mutex<String>>,
        dict_new_value: std::sync::Arc<std::sync::Mutex<String>>,
        dict_replace_target: std::sync::Arc<std::sync::Mutex<String>>,
        dict_replace_new: std::sync::Arc<std::sync::Mutex<String>>,
        dict_replace_all: std::sync::Arc<std::sync::Mutex<bool>>,
        show_dict_add_dialog: std::sync::Arc<std::sync::Mutex<bool>>,
        show_dict_replace_dialog: std::sync::Arc<std::sync::Mutex<bool>>,
        show_dict_clear_confirm: std::sync::Arc<std::sync::Mutex<bool>>,
        glossary_priority: std::sync::Arc<std::sync::Mutex<String>>,
    ) {
        let is_dark = *viewer_shared.theme.read().unwrap() == "dark";
        let font_size = *viewer_shared.font_size.read().unwrap();
        let mut style = (*ctx.style()).clone();

        // 完整移植主視窗的視覺參數，確保按鈕、選取色與主題高度一致
        let visuals = if is_dark {
            let mut v = egui::Visuals::dark();
            v.window_fill = egui::Color32::from_rgb(30, 30, 35);
            v.panel_fill = egui::Color32::from_rgb(30, 30, 35);
            v.extreme_bg_color = egui::Color32::from_rgb(20, 20, 25);
            v.selection.bg_fill = egui::Color32::from_rgb(60, 100, 150);

            let btn_bg = egui::Color32::from_rgb(60, 60, 70);
            v.widgets.inactive.bg_fill = btn_bg;
            v.widgets.inactive.weak_bg_fill = btn_bg;
            v.widgets.hovered.bg_fill = egui::Color32::from_rgb(75, 75, 90);
            v.widgets.active.bg_fill = egui::Color32::from_rgb(90, 90, 110);
            v.faint_bg_color = egui::Color32::from_rgb(40, 40, 45);
            v
        } else {
            let mut v = egui::Visuals::light();
            let bg_color = egui::Color32::from_rgb(0xFF, 0xDE, 0xAD);
            v.window_fill = bg_color;
            v.panel_fill = bg_color;

            let btn_bg = egui::Color32::from_rgb(0xE3, 0xC3, 0x95);
            let btn_stroke_color = egui::Color32::from_rgb(30, 30, 30);

            v.widgets.inactive.bg_fill = btn_bg;
            v.widgets.inactive.weak_bg_fill = btn_bg;
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.2, btn_stroke_color);
            v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0xCD, 0xAA, 0x7D);
            v.widgets.hovered.fg_stroke = egui::Stroke::new(1.8, btn_stroke_color);
            v.widgets.active.bg_fill = egui::Color32::from_rgb(0xA0, 0x7B, 0x7B);

            v.extreme_bg_color = egui::Color32::WHITE;
            v.selection.bg_fill = egui::Color32::from_rgb(0xCD, 0x85, 0x3F);
            v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0xD2, 0xB4, 0x8C);
            v.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_gray(100));
            v.faint_bg_color = egui::Color32::from_rgb(0xEF, 0xD0, 0x9E);
            v.override_text_color = Some(egui::Color32::from_rgb(50, 40, 30));
            v
        };

        style.visuals = visuals;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(font_size));
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(font_size),
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            // 在此處套用 style，限制樣式僅影響子視窗 ui，不影響全域 ctx (解決主畫面主題消失)
            ui.set_style(style);
            let processing = *is_processing.lock().unwrap();
            let current_tab = *dict_active_tab.lock().unwrap();

            ui.heading("📖 建議詞管理器");
            ui.label("存在裡面的文字將作為術語表建議 LLM 如何翻譯該文字（僅建議，不一定會使用）");

            ui.horizontal(|ui| {
                let mut active_tab = dict_active_tab.lock().unwrap();
                let theme_val = viewer_shared.theme.read().unwrap().clone();
                let is_light = theme_val == "light";
                let fill = if is_light {
                    egui::Color32::from_rgb(0xE3, 0xC3, 0x95)
                } else {
                    egui::Color32::from_rgb(60, 60, 70)
                };

                egui::Frame::none()
                    .fill(fill)
                    .rounding(4.0)
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        if ui
                            .selectable_value(&mut *active_tab, 0, "📝 使用者建議詞")
                            .clicked()
                        {
                            *dict_page.lock().unwrap() = 0;
                        }
                        if ui
                            .selectable_value(&mut *active_tab, 1, "📚 官方建議詞")
                            .clicked()
                        {
                            *dict_page.lock().unwrap() = 0;
                        }
                    });
            });

            // 優先級開關與搜尋框
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
            });

            ui.separator();

            // 功能按鈕列
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

                // 新增按鈕
                if ui
                    .add_enabled(!processing, egui::Button::new("➕ 新增"))
                    .clicked()
                {
                    *show_dict_add_dialog.lock().unwrap() = true;
                }
                // 取代按鈕
                if ui
                    .add_enabled(!processing, egui::Button::new("🔄 取代"))
                    .clicked()
                {
                    *show_dict_replace_dialog.lock().unwrap() = true;
                }
                // 匯入按鈕
                if ui
                    .add_enabled(!processing, egui::Button::new("📥 匯入"))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file()
                    {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(imported) = serde_json::from_str::<
                                std::collections::HashMap<String, String>,
                            >(&content)
                            {
                                if current_tab == 0 {
                                    let mut memory = translation_memory.lock().unwrap();
                                    memory.extend(imported);
                                    crate::config::save_translation_memory(&*memory);
                                } else if current_tab == 1 {
                                    let mut inferred = inferred_match_map.lock().unwrap();
                                    inferred.extend(imported);
                                    crate::config::save_dict(
                                        crate::config::OFFICIAL_DICT,
                                        &*inferred,
                                    );
                                }
                                *dict_search_last.lock().unwrap() = (String::new(), usize::MAX);
                            }
                        }
                    }
                }
                // 匯出按鈕
                if ui.button("📤 匯出").clicked() {
                    let default_name = if current_tab == 0 {
                        crate::config::USER_DICT
                    } else {
                        crate::config::OFFICIAL_DICT
                    };
                    let dialog = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_file_name(default_name);
                    if let Some(path) = dialog.save_file() {
                        let json_data = if current_tab == 0 {
                            serde_json::to_string_pretty(&*translation_memory.lock().unwrap())
                        } else {
                            serde_json::to_string_pretty(&*inferred_match_map.lock().unwrap())
                        };
                        if let Ok(json) = json_data {
                            let _ = std::fs::write(&path, json);
                        }
                    }
                }
                // .json 按鈕 (Revision 14.1 強化雙開)
                if ui
                    .button(".json")
                    .on_hover_text("開啟編輯字典檔案並瀏覽存放資料夾")
                    .clicked()
                {
                    let filename = if current_tab == 0 {
                        crate::config::USER_DICT
                    } else {
                        crate::config::OFFICIAL_DICT
                    };
                    let path = std::path::Path::new(crate::config::DICT_DIR).join(filename);
                    if let Ok(abs_path) = std::fs::canonicalize(&path) {
                        // 1. 以檔案總管開啟並選中
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("explorer")
                            .arg("/select,")
                            .arg(&abs_path)
                            .spawn();

                        // 2. 以預設編輯器開啟實體檔案
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd")
                            .arg("/c")
                            .arg("start")
                            .arg("")
                            .arg(&abs_path)
                            .spawn();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(!processing, egui::Button::new("🗑 清空全部"))
                        .clicked()
                    {
                        *show_dict_clear_confirm.lock().unwrap() = true;
                    }
                });
            });

            // --- 補回新增與取代對話框區塊 (Revision 14.1) ---
            if *show_dict_add_dialog.lock().unwrap() {
                egui::Window::new("➕ 新增建議詞")
                    .collapsible(false)
                    .resizable(false)
                    .default_pos([400.0, 300.0]) // 移除 anchor 使其可移動 (Revision 14.4)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("原文 (Key):");
                            ui.text_edit_singleline(&mut *dict_new_key.lock().unwrap());
                        });
                        ui.horizontal(|ui| {
                            ui.label("翻譯 (Value):");
                            ui.text_edit_singleline(&mut *dict_new_value.lock().unwrap());
                        });
                        ui.horizontal(|ui| {
                            let confirm_btn = ui.button("確定新增");
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if confirm_btn.clicked() || enter_pressed {
                                let key = dict_new_key.lock().unwrap().clone();
                                let val = dict_new_value.lock().unwrap().clone();
                                if !key.is_empty() {
                                    if current_tab == 0 {
                                        let mut mem = translation_memory.lock().unwrap();
                                        mem.insert(key, val);
                                        crate::config::save_translation_memory(&*mem);
                                    } else {
                                        // 官方分頁編輯也存入使用者字典 (Revision 14.1/14.4/14.5)
                                        let mut mem = translation_memory.lock().unwrap();
                                        mem.insert(key, val);
                                        crate::config::save_translation_memory(&*mem);
                                    }
                                }
                                *show_dict_add_dialog.lock().unwrap() = false;
                                *dict_new_key.lock().unwrap() = String::new();
                                *dict_new_value.lock().unwrap() = String::new();
                            }
                            if ui.button("取消").clicked() {
                                *show_dict_add_dialog.lock().unwrap() = false;
                            }
                        });
                    });
            }

            if *show_dict_replace_dialog.lock().unwrap() {
                egui::Window::new("🔄 批量取代翻譯")
                    .collapsible(false)
                    .resizable(false)
                    .default_pos([400.0, 300.0]) // 移除 anchor 使其可移動 (Revision 14.4)
                    .show(ctx, |ui| {
                        ui.label("將目前分頁中所有符合的翻譯內容進行取代。");
                        ui.horizontal(|ui| {
                            ui.label("原Value:");
                            ui.text_edit_singleline(&mut *dict_replace_target.lock().unwrap());
                        });
                        ui.horizontal(|ui| {
                            ui.label("新Value:");
                            ui.text_edit_singleline(&mut *dict_replace_new.lock().unwrap());
                        });
                        ui.checkbox(&mut *dict_replace_all.lock().unwrap(), "全部符合才取代");
                        ui.horizontal(|ui| {
                            let replace_btn = ui.button("執行取代");
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if replace_btn.clicked() || enter_pressed {
                                let target = dict_replace_target.lock().unwrap().clone();
                                let new_val = dict_replace_new.lock().unwrap().clone();
                                let is_exact = *dict_replace_all.lock().unwrap();

                                // Revision 14.5: 空值保護與計數
                                if !target.is_empty() {
                                    let mut count = 0;
                                    let mut mem = translation_memory.lock().unwrap();

                                    if current_tab == 0 {
                                        // 使用者建議詞取代
                                        for v in mem.values_mut() {
                                            if is_exact {
                                                if v == &target {
                                                    *v = new_val.clone();
                                                    count += 1;
                                                }
                                            } else if v.contains(&target) {
                                                *v = v.replace(&target, &new_val);
                                                count += 1;
                                            }
                                        }
                                    } else {
                                        // 官方建議詞取代：同時更新內存與存入 user.json
                                        // Revision 14.6: 對官方字典操作後移入使用者分頁
                                        let mut inferred = inferred_match_map.lock().unwrap();
                                        let mut keys_to_remove = Vec::new();
                                        for (k, v) in inferred.iter_mut() {
                                            let mut changed = false;
                                            if is_exact {
                                                if v == &target {
                                                    *v = new_val.clone();
                                                    changed = true;
                                                }
                                            } else if v.contains(&target) {
                                                *v = v.replace(&target, &new_val);
                                                changed = true;
                                            }
                                            if changed {
                                                mem.insert(k.clone(), v.clone());
                                                keys_to_remove.push(k.clone());
                                                count += 1;
                                            }
                                        }
                                        for k in keys_to_remove {
                                            inferred.remove(&k);
                                        }
                                        if count > 0 {
                                            crate::config::save_dict(
                                                crate::config::OFFICIAL_DICT,
                                                &*inferred,
                                            );
                                        }
                                    }

                                    if count > 0 {
                                        crate::config::save_translation_memory(&*mem);
                                        // 確保 UI 立即反應 (針對搜尋快取等可能的延遲)
                                        ctx.request_repaint();
                                    }
                                }
                                *show_dict_replace_dialog.lock().unwrap() = false;
                            }
                            if ui.button("取消").clicked() {
                                *show_dict_replace_dialog.lock().unwrap() = false;
                            }
                        });
                    });
            }
            // ----------------------------------------------

            // 顯示清空對話框
            if *show_dict_clear_confirm.lock().unwrap() {
                egui::Window::new("⚠ 確認清空字典")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("確定刪除全部內容？此操作無法復原。");
                        ui.horizontal(|ui| {
                            if ui.button("確定清空").clicked() {
                                if current_tab == 0 {
                                    translation_memory.lock().unwrap().clear();
                                    crate::config::save_translation_memory(
                                        &*translation_memory.lock().unwrap(),
                                    );
                                } else if current_tab == 1 {
                                    inferred_match_map.lock().unwrap().clear();
                                    crate::config::save_dict(
                                        crate::config::OFFICIAL_DICT,
                                        &*inferred_match_map.lock().unwrap(),
                                    );
                                }
                                *dict_search_last.lock().unwrap() = (String::new(), usize::MAX);
                                *show_dict_clear_confirm.lock().unwrap() = false;
                            }
                            if ui.button("取消").clicked() {
                                *show_dict_clear_confirm.lock().unwrap() = false;
                            }
                        });
                    });
            }

            ui.separator();

            let search_text = dict_search.lock().unwrap().to_lowercase();
            let mut items: Vec<(String, String)> = if current_tab == 0 {
                translation_memory
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect()
            } else if current_tab == 1 {
                inferred_match_map
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            };
            items.retain(|(k, v)| {
                search_text.is_empty()
                    || k.to_lowercase().contains(&search_text)
                    || v.to_lowercase().contains(&search_text)
            });
            items.sort_by(|a, b| a.0.cmp(&b.0));

            let total_items = items.len();
            let page_size = 50;
            let total_pages = total_items.div_ceil(page_size).max(1);
            let mut page = dict_page.lock().unwrap();
            if *page >= total_pages {
                *page = 0;
            }
            let current_page = *page;
            let start = (current_page * page_size).min(total_items);
            let end = (start + page_size).min(total_items);

            ui.horizontal(|ui| {
                ui.label("🔍 搜尋:");
                ui.add(
                    egui::TextEdit::singleline(&mut *dict_search.lock().unwrap())
                        .desired_width(120.0),
                );
                ui.add_space(20.0);
                if ui.button("◀").clicked() {
                    *page = page.saturating_sub(1);
                }
                ui.label(format!(
                    "第 {}/{} 頁 (顯示 {}-{}/{})",
                    current_page + 1,
                    total_pages,
                    if total_items > 0 { start + 1 } else { 0 },
                    end,
                    total_items
                ));
                if ui.button("▶").clicked() && (*page + 1) < total_pages {
                    *page += 1;
                }

                // 優先級開關移到右側 (label 在左邊，toggle 在右邊)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut is_user_priority = glossary_priority.lock().unwrap().as_str() == "user";
                    if ui
                        .add(toggle(&mut is_user_priority))
                        .on_hover_text("切換 官方優先 (關) 或 使用者優先 (開)")
                        .clicked()
                    {
                        *glossary_priority.lock().unwrap() = if is_user_priority {
                            "user".to_string()
                        } else {
                            "official".to_string()
                        };
                    }
                    let priority_label = if is_user_priority {
                        "使用者優先"
                    } else {
                        "官方優先"
                    };
                    ui.label(priority_label);
                });
            });

            ui.separator();
            egui::ScrollArea::vertical()
                .hscroll(false) // 禁用水平捲動防止無限放大 (Revision 14.2)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // 計算欄位寬度，扣除間距與操作欄固定寬度，並實施 150px 最小寬度保護
                    let spacing = 12.0;
                    let actions_w = 80.0;
                    let col_w =
                        ((ui.available_width() - actions_w - spacing * 2.0) / 2.0).max(150.0);

                    egui::Grid::new("mem_grid")
                        .num_columns(3)
                        .spacing([spacing, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // 標題置中對齊 (Revision 14.3: 改用 add_sized 避免撐開寬度)
                            ui.add_sized(
                                [col_w, 20.0],
                                egui::Label::new(egui::RichText::new("Key").strong()),
                            );
                            ui.add_sized(
                                [col_w, 20.0],
                                egui::Label::new(egui::RichText::new("Value").strong()),
                            );
                            ui.add_sized(
                                [actions_w, 20.0],
                                egui::Label::new(egui::RichText::new("操作").strong()),
                            );
                            ui.end_row();

                            if items.is_empty() {
                                ui.label("");
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("(目前的字典分頁是空的)").italics(),
                                    );
                                });
                                ui.label("");
                                ui.end_row();
                            }

                            let start = *page * page_size;
                            let end = (start + page_size).min(total_items);

                            for (k, v) in &items[start..end] {
                                // 使用 Layout 確保垂直置左對齊 (Align::Min 為左/頂)
                                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                    ui.add_sized([col_w, 0.0], egui::Label::new(k).wrap(true));
                                });

                                let is_editing =
                                    dict_edit_key.lock().unwrap().as_deref() == Some(k);
                                if is_editing {
                                    ui.add(
                                        egui::TextEdit::singleline(
                                            &mut *dict_edit_value.lock().unwrap(),
                                        )
                                        .desired_width(col_w - 20.0),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // 順序優化 (Revision 14.5)：先加 💾 使得在 right_to_left 下位於最右面
                                            // 使用者 feedback：[❌] [💾] 是相反的 -> 代表用戶希望 💾 在右
                                            // 在 right_to_left 中，第一個加的會在最右邊
                                            let save_btn = ui.button("💾");
                                            let enter_pressed =
                                                ui.input(|i| i.key_pressed(egui::Key::Enter));
                                            if save_btn.clicked() || enter_pressed {
                                                let mut mem = translation_memory.lock().unwrap();
                                                let edit_val =
                                                    dict_edit_value.lock().unwrap().clone();
                                                mem.insert(k.clone(), edit_val);
                                                crate::config::save_translation_memory(&*mem);

                                                // Revision 14.6: 對官方字典操作後移入使用者分頁 (從官方移除)
                                                if current_tab == 1 {
                                                    let mut inferred =
                                                        inferred_match_map.lock().unwrap();
                                                    inferred.remove(k);
                                                    crate::config::save_dict(
                                                        crate::config::OFFICIAL_DICT,
                                                        &*inferred,
                                                    );
                                                }

                                                *dict_edit_key.lock().unwrap() = None;
                                            }
                                            if ui.button("❌").clicked() {
                                                *dict_edit_key.lock().unwrap() = None;
                                            }
                                        },
                                    );
                                } else {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.add_sized(
                                                [col_w, 0.0],
                                                egui::Label::new(v).wrap(true),
                                            );
                                        },
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // 順序調整 (Revision 14.4)：先加 🗑 使其在右
                                            if ui
                                                .add_enabled(!processing, egui::Button::new("🗑"))
                                                .clicked()
                                            {
                                                // 刪除邏輯修正 (Revision 14.1)：根據分頁刪除對應字典
                                                if current_tab == 0 {
                                                    let mut mem =
                                                        translation_memory.lock().unwrap();
                                                    mem.remove(k);
                                                    crate::config::save_translation_memory(&*mem);
                                                } else {
                                                    let mut inferred =
                                                        inferred_match_map.lock().unwrap();
                                                    inferred.remove(k);
                                                    crate::config::save_dict(
                                                        crate::config::OFFICIAL_DICT,
                                                        &*inferred,
                                                    );
                                                }
                                            }
                                            if ui
                                                .add_enabled(!processing, egui::Button::new("✏"))
                                                .clicked()
                                            {
                                                *dict_edit_key.lock().unwrap() = Some(k.clone());
                                                *dict_edit_value.lock().unwrap() = v.clone();
                                            }
                                        },
                                    );
                                }
                                ui.end_row();
                            }
                        });
                });
        });
    }
}

fn scan_files_recursive(
    dir: &std::path::Path,
    base_dir: &std::path::Path,
) -> Vec<(std::path::PathBuf, String)> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(scan_files_recursive(&path, base_dir));
            } else if path
                .extension()
                .is_some_and(|ext| ext == "jar" || ext == "json" || ext == "js")
            {
                let rel = path
                    .strip_prefix(base_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((path, rel));
            }
        }
    }
    files
}

fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| {
        let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool(response.id, *on);
            let visuals = ui.style().interact_selectable(&response, *on);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            ui.painter().circle(
                egui::pos2(circle_x, rect.center().y),
                0.75 * radius,
                visuals.bg_stroke.color,
                visuals.fg_stroke,
            );
        }
        response
    }
}
