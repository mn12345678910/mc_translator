use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    /// 渲染頂部控制項 (優化佈局防止遮擋)
    pub fn render_header_controls(&mut self, ui: &mut egui::Ui, ui_enabled: bool) {
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
                        let files = crate::file::scanner::scan_files_recursive(&path, &path);
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
    pub fn render_status_navigation(&mut self, ui: &mut egui::Ui) {
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
}
