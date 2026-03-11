use crate::state::app_state::AppState;
use crate::state::viewer_state::{ViewerSharedState, ViewerUpdate};

// --- UI 顏色常量 (Revision 15.20) ---
const LABEL_COLOR_LIGHT: egui::Color32 = egui::Color32::from_rgb(30, 60, 120); // 深靛藍
const LABEL_COLOR_DARK: egui::Color32 = egui::Color32::from_rgb(200, 160, 100); // 淺沙色

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 0. 處理來自 Viewport 的非同步更新訊息 (Revision 15.12)
        while let Ok(update) = self._update_rx.try_recv() {
            match update {
                ViewerUpdate::Theme(t) => {
                    self.theme = t;
                }
                ViewerUpdate::FontSize(s) => {
                    self.font_size = s;
                }
                ViewerUpdate::SaveConfig => {
                    self.save_config();
                }
            }
        }

        // 0. 更新啟動延遲計數器 (Revision 15.13: V6 終極整合 - 補回被誤刪的遞減邏輯)
        if self.viewer_opening_counter > 0 {
            self.viewer_opening_counter -= 1;
            ctx.request_repaint(); // 確保計數器遞減期間持續重繪
        }

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

            // 視窗同步 (ses_342b): 從共享狀態同步回 AppState 並儲存
            {
                if let Ok(pos_lock) = self.viewer_shared.position.read() {
                    if let Some(pos) = *pos_lock {
                        if (pos.x - self.viewer_x).abs() > 0.1
                            || (pos.y - self.viewer_y).abs() > 0.1
                        {
                            self.viewer_x = pos.x;
                            self.viewer_y = pos.y;
                        }
                    }
                }
                if let Ok(size_lock) = self.viewer_shared.inner_size.read() {
                    if let Some(size) = *size_lock {
                        if (size.x - self.viewer_width).abs() > 0.1
                            || (size.y - self.viewer_height).abs() > 0.1
                        {
                            self.viewer_width = size.x;
                            self.viewer_height = size.y;
                        }
                    }
                }
            }

            // --- 同步主視窗幾幾何至 AppState (Revision 15.17) ---
            if let Some(rect) = ctx.input(|i| i.viewport().outer_rect) {
                if (rect.min.x - self.main_x).abs() > 0.1 || (rect.min.y - self.main_y).abs() > 0.1
                {
                    self.main_x = rect.min.x;
                    self.main_y = rect.min.y;
                }
                if (rect.width() - self.main_width).abs() > 0.1
                    || (rect.height() - self.main_height).abs() > 0.1
                {
                    self.main_width = rect.width();
                    self.main_height = rect.height();
                }
            }
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
        if !self.show_memory_viewer || self.viewer_opening_counter > 0 {
            return;
        }

        // 1. 在主執行緒更新幀數計數器，用於隱形展現與幾何引導 (Revision 15.10)
        let count_val = {
            let mut count = self.viewer_shared.opened_frames.lock().unwrap();
            if *count < 150 {
                *count += 1;
            }
            *count
        };

        let opened = self.viewer_shared.opened_last_frame.lock().unwrap();
        if !*opened {
            self.refresh_all_dictionaries();
            self.is_memory_viewer_open
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        drop(opened);

        // 每 frame 都必須調用 show_viewport_deferred 才能保持 viewport 持績顯示
        self.create_viewport_deferred(ctx, count_val);

        // 確保在開啟初期的引導與亮顯階段持續重繪，防止計數器卡住 (Deadlock Prevention)
        if count_val < 65 {
            ctx.request_repaint();
        }
    }

    /// 創建建議詞管理器 Viewport
    fn create_viewport_deferred(&mut self, ctx: &egui::Context, opened_frames: u32) {
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

            let mut opened_lock = self.viewer_shared.opened_last_frame.lock().unwrap();
            let opened_last_frame = *opened_lock;

            // 1. 隱形啟動策略 (Invisible-First): 初始設為 invisible (Revision 15.10)
            // 2. 幾何引導 (Geometry Guidance): 前 20 幀持續強制套用座標與尺寸，壓制 OS 跳位 (Feedback Fix)
            let is_visible = opened_frames >= 30; // 使用者要求調回 30 幀
            let mut builder = egui::ViewportBuilder::default()
                .with_title("📖 建議詞管理器")
                .with_resizable(true)
                .with_maximized(false)
                .with_min_inner_size([800.0, 600.0]) // [Revision 15.15] 最小尺寸限制
                .with_visible(is_visible);

            // 只有在穩定前（前 40 幀，涵蓋 30 幀亮顯期）持續強制套位，壓制 OS 隨機跳位 (Revision 15.13)
            if opened_frames < 40 {
                builder = builder
                    .with_inner_size([self.viewer_width, self.viewer_height])
                    .with_position([self.viewer_x, self.viewer_y]);
            }

            // 一旦建立了帶有初始位置的 builder，就標記為已開啟
            if !opened_last_frame {
                *opened_lock = true;
            }
            drop(opened_lock);

            // (顯現指令已透過 builder.with_visible(is_visible) 處理，不需額外發送以防 ID 註冊競爭)

            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of("memory_viewer"),
                builder,
                move |ctx, _viewport_id| {
                    // 1. 視覺初始化: 消彌白閃
                    let is_dark = *viewer_shared.theme.read().unwrap() == "dark";
                    let bg_color = if is_dark {
                        egui::Color32::from_rgb(30, 30, 35)
                    } else {
                        egui::Color32::from_rgb(0xFF, 0xDE, 0xAD)
                    };
                    ctx.style_mut(|s| s.visuals.window_fill = bg_color);

                    // 監聽關閉事件 (Revision 15.12: Deferred Save)
                    if ctx.input(|i| i.viewport().close_requested()) {
                        is_memory_viewer_open.store(false, std::sync::atomic::Ordering::SeqCst);
                        viewer_shared
                            .close_requested
                            .store(true, std::sync::atomic::Ordering::SeqCst);

                        // 當視窗關閉時觸發一次設定檔存盤 (Feedback Fix)
                        viewer_shared.update_tx
                            .send(crate::state::viewer_state::ViewerUpdate::Theme(viewer_shared.theme.read().unwrap().clone())).ok();
                    }

                    // 2. 視窗同步 (ses_342b): 回報當前位置與大小
                    let inner_size = ctx.screen_rect().size();
                    if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                        let pos = outer_rect.min;
                        let current_count = *viewer_shared.opened_frames.lock().unwrap();

                        // (顯現指令已移至主執行緒 create_viewport_deferred 以防死鎖)

                        if (pos.x > 1.1 || pos.y > 1.1) && current_count > 60 {
                            // 幾何同步保護：加入 Delta Check，並限制合理的同步上限 (Revision 15.15)
                            let clamped_width = inner_size.x.clamp(400.0, 1920.0);
                            let clamped_height = inner_size.y.clamp(300.0, 1080.0);
                            let clamped_size = egui::vec2(clamped_width, clamped_height);

                            if let Ok(mut p_lock) = viewer_shared.position.write() {
                                let changed = p_lock
                                    .map(|old| (old.x - pos.x).abs() + (old.y - pos.y).abs() > 2.0)
                                    .unwrap_or(true);
                                if changed {
                                    *p_lock = Some(pos);
                                }
                            }
                            if let Ok(mut s_lock) = viewer_shared.inner_size.write() {
                                let changed = s_lock
                                    .map(|old| {
                                        (old.x - clamped_size.x).abs()
                                            + (old.y - clamped_size.y).abs()
                                            > 10.0
                                    })
                                    .unwrap_or(true);
                                if changed {
                                    *s_lock = Some(clamped_size);
                                }
                            }
                        }
                    }

                    // 3. 渲染內容
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

    /// 渲染頂部控制項 (優化佈局防止遮擋)
    fn render_header_controls(&mut self, ui: &mut egui::Ui, ui_enabled: bool) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // 左側按鈕區
            ui.horizontal(|ui| {
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
                        *self.global_total.lock().unwrap() = self.input_paths.len() as f32;
                        *self.global_progress.lock().unwrap() = 0.0;
                    }
                }
                if ui
                    .add_enabled(ui_enabled, egui::Button::new("📂 選擇資料夾"))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let files = crate::file_handler::scan_files_recursive(&path, &path);
                        self.add_log(&format!("已選擇 {} 個檔案", files.len()));
                        self.input_paths = files;
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
            });

            // 右側導航按鈕 (固定置右)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_status_navigation(ui);
            });
        });

        ui.add_space(2.0);

        // 路徑標籤單獨一行，並具備截斷保護
        ui.horizontal(|ui| {
            let label_color = if self.theme == "light" {
                LABEL_COLOR_LIGHT
            } else {
                LABEL_COLOR_DARK
            };
            let display_path = if self.output_dir.is_empty() {
                "預設: ./LLMTranslator".into()
            } else {
                self.output_dir.clone()
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!("輸出路徑: {}", display_path))
                        .color(label_color)
                        .strong(),
                )
                .truncate(true),
            );
        });
    }

    /// 渲染導航按鈕 (⚙ 🌓 📖 🔧)
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
                if self.show_memory_viewer {
                    // 點擊開啟時發動 0.5s (30 frames) 的靜默期，等待主程式狀態穩定 (Feedback Fix)
                    self.viewer_opening_counter = 30;
                } else {
                    let mut frames = self.viewer_shared.opened_frames.lock().unwrap();
                    *frames = 0;
                    self.is_memory_viewer_open
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    // 重要：重置旗標，確保下次開啟能正確初始化 (Revision 15.13)
                    let mut opened = self.viewer_shared.opened_last_frame.lock().unwrap();
                    *opened = false;
                    // 關閉時存盤
                    self.save_config();
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

    /// 渲染開發人員模式面板 (還原至備份 Grid，套用隨主題變色標籤)
    fn render_developer_mode_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_developer_mode {
            return;
        }
        let label_color = if self.theme == "light" {
            LABEL_COLOR_LIGHT
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

    fn render_progress_section(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        processing: bool,
    ) {
        ui.separator();
        ui.add_space(1.0);

        let label_color = if self.theme == "light" {
            LABEL_COLOR_LIGHT
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
        ui.label(
            egui::RichText::new(format!("目前狀態: {}", current_status))
                .color(label_color)
                .strong(),
        );

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
            LABEL_COLOR_LIGHT
        } else {
            LABEL_COLOR_DARK
        };
        ui.separator();
        ui.label(egui::RichText::new("執行日誌:").color(label_color).strong());
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
            let theme_val = viewer_shared.theme.read().unwrap().clone();
            let label_color = if theme_val == "light" {
                LABEL_COLOR_LIGHT
            } else {
                LABEL_COLOR_DARK
            };

            ui.label(
                egui::RichText::new("📖 建議詞管理器")
                    .heading()
                    .color(label_color)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(
                    "存在裡面的文字將作為術語表建議 LLM 如何翻譯該文字（僅建議，不一定會使用）",
                )
                .color(label_color)
                .strong(),
            );

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
                ui.label(egui::RichText::new("🔍 搜尋:").color(label_color).strong());
                ui.add(
                    egui::TextEdit::singleline(&mut *dict_search.lock().unwrap())
                        .desired_width(120.0),
                );
                ui.add_space(20.0);
                if ui.button("◀").clicked() {
                    *page = page.saturating_sub(1);
                }
                ui.label(
                    egui::RichText::new(format!(
                        "第 {}/{} 頁 (顯示 {}-{}/{})",
                        current_page + 1,
                        total_pages,
                        if total_items > 0 { start + 1 } else { 0 },
                        end,
                        total_items
                    ))
                    .color(label_color)
                    .strong(),
                );
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
                    ui.label(
                        egui::RichText::new(priority_label)
                            .color(label_color)
                            .strong(),
                    );
                });
            });

            ui.separator();
            egui::ScrollArea::vertical()
                .hscroll(false) // 禁用水平捲動防止無限放大 (Revision 14.2)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // 計算欄位寬度，扣除間距與操作欄固定寬度，並實施 150px 最小寬度保護
                    let spacing = 12.0;
                    let actions_w = 120.0; // 增加空間以利按鈕置中
                    let col_w =
                        ((ui.available_width() - actions_w - spacing * 2.0) / 2.0).max(150.0);

                    egui::Grid::new("mem_grid")
                        .num_columns(3)
                        .spacing([spacing, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // 標題置中對齊且不鎖死寬度 (Revision 15.15: 使用 allocate_ui 使其可縮小)
                            ui.allocate_ui([col_w, 20.0].into(), |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        egui::RichText::new("Key").color(label_color).strong(),
                                    );
                                });
                            });
                            ui.allocate_ui([col_w, 20.0].into(), |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        egui::RichText::new("Value").color(label_color).strong(),
                                    );
                                });
                            });
                            ui.allocate_ui([actions_w, 20.0].into(), |ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new("操作").color(label_color).strong(),
                                        );
                                    },
                                );
                            });
                            ui.end_row();

                            if items.is_empty() {
                                ui.label("");
                                ui.allocate_ui([col_w, 30.0].into(), |ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            egui::RichText::new("(目前的字典分頁是空的)")
                                                .color(label_color)
                                                .strong(),
                                        );
                                    });
                                });
                                ui.label("");
                                ui.end_row();
                            }

                            let start = *page * page_size;
                            let end = (start + page_size).min(total_items);

                            for (k, v) in &items[start..end] {
                                // 使用 Layout 確保水平與垂直置中 (Revision 15.15)
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    ui.set_max_width(col_w);
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(k).color(label_color).strong(),
                                        )
                                        .wrap(true),
                                    );
                                });

                                let is_editing =
                                    dict_edit_key.lock().unwrap().as_deref() == Some(k);
                                if is_editing {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            ui.add(
                                                egui::TextEdit::singleline(
                                                    &mut *dict_edit_value.lock().unwrap(),
                                                )
                                                .desired_width(col_w - 20.0),
                                            );
                                        },
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("❌").clicked() {
                                                *dict_edit_key.lock().unwrap() = None;
                                            }
                                            let save_btn = ui.button("💾");
                                            let enter_pressed =
                                                ui.input(|i| i.key_pressed(egui::Key::Enter));
                                            if save_btn.clicked() || enter_pressed {
                                                let mut mem = translation_memory.lock().unwrap();
                                                let edit_val =
                                                    dict_edit_value.lock().unwrap().clone();
                                                mem.insert(k.clone(), edit_val);
                                                crate::config::save_translation_memory(&*mem);
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
                                        },
                                    );
                                } else {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            ui.set_max_width(col_w);
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(v)
                                                        .color(label_color)
                                                        .strong(),
                                                )
                                                .wrap(true),
                                            );
                                        },
                                    );

                                    // 確保按鈕群組在 Grid 內置右對齊且順序正確 (Revision 15.17)
                                    ui.allocate_ui([actions_w, 20.0].into(), |ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                // 右對齊模式下，先加的在右邊 -> [✏️] [🗑️]
                                                if ui
                                                    .add_enabled(
                                                        !processing,
                                                        egui::Button::new("🗑"),
                                                    )
                                                    .clicked()
                                                {
                                                    if current_tab == 0 {
                                                        let mut mem =
                                                            translation_memory.lock().unwrap();
                                                        mem.remove(k);
                                                        crate::config::save_translation_memory(
                                                            &*mem,
                                                        );
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
                                                    .add_enabled(
                                                        !processing,
                                                        egui::Button::new("✏"),
                                                    )
                                                    .clicked()
                                                {
                                                    *dict_edit_key.lock().unwrap() =
                                                        Some(k.clone());
                                                    *dict_edit_value.lock().unwrap() = v.clone();
                                                }
                                            },
                                        );
                                    });
                                }
                                ui.end_row();
                            }
                        });
                });
        });
    }
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
