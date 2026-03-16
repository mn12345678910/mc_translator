use crate::state::app_state::AppState;
use std::sync::atomic::Ordering;
// use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    /// 渲染頂部控制項 (優化佈局防止遮擋)
    pub fn render_header_controls(&mut self, ui: &mut egui::Ui, ui_enabled: bool) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // 左側按鈕區
            ui.horizontal(|ui| {
                let (bg, text, rounding) = self.get_instance_style_full("btn_select_file");
                if ui
                    .add_enabled(ui_enabled, egui::Button::new(egui::RichText::new(self.i18n.btn_select_file.clone()).color(text)).fill(bg).rounding(rounding))
                    .clicked()
                {
                    if let Some(files) = rfd::FileDialog::new()
                        .add_filter(&self.i18n.dialog_filter_jar_json_js, &["jar", "js", "json"])
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
                        self.global_total.store((self.input_paths.len() as f32).to_bits(), Ordering::SeqCst);
                        self.global_progress.store(0.0f32.to_bits(), Ordering::SeqCst);
                    }
                }
                let (bg, text, rounding) = self.get_instance_style_full("btn_select_folder");
                if ui
                    .add_enabled(ui_enabled, egui::Button::new(egui::RichText::new(self.i18n.btn_select_folder.clone()).color(text)).fill(bg).rounding(rounding))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let files = crate::file::scanner::scan_files_recursive(&path, &path);
                        self.add_log(&format!("已選擇 {} 個檔案", files.len()));
                        self.input_paths = files;
                        self.global_total.store((self.input_paths.len() as f32).to_bits(), Ordering::SeqCst);
                        self.global_progress.store(0.0f32.to_bits(), Ordering::SeqCst);
                    }
                }

                ui.separator();

                let (bg, text, rounding) = self.get_instance_style_full("btn_output_dir");
                if ui
                    .add_enabled(ui_enabled, egui::Button::new(egui::RichText::new(self.i18n.btn_output_dir.clone()).color(text)).fill(bg).rounding(rounding))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_dir = path.to_string_lossy().to_string();
                        self.trigger_save();
                        self.add_log(&format!("輸出資料夾已設定: {}", self.output_dir));
                    }
                }

                let (bg, text, rounding) = self.get_instance_style_full("btn_open_output");
                if ui.add(egui::Button::new(egui::RichText::new(self.i18n.btn_open_output.clone()).color(text)).fill(bg).rounding(rounding)).clicked() {
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
                    {
                        use std::os::windows::process::CommandExt;
                        let _ = std::process::Command::new("explorer")
                            .arg(path)
                            .creation_flags(0x08000000) // CREATE_NO_WINDOW
                            .spawn();
                    }
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
            let (_, label_color) = self.get_instance_style("label_output_path");
            let display_path = if self.output_dir.is_empty() {
                self.i18n.label_default_path.clone()
            } else {
                self.output_dir.clone()
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!("{}{}", self.i18n.label_output_path, display_path))
                        .color(label_color)
                        .strong(),
                )
                .truncate(true),
            )
            .on_hover_text(display_path.clone());
        });
    }

    /// 渲染導航按鈕 (⚙ 🌓 📖 🔧)
    pub fn render_status_navigation(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (bg_set, text_set, rounding_set) = self.get_instance_style_full("btn_nav_settings");
            if ui.add(egui::Button::new(egui::RichText::new("⚙").color(text_set)).fill(bg_set).rounding(rounding_set)).on_hover_text(self.i18n.btn_nav_settings.clone()).clicked() {
                self.show_api_settings = !self.show_api_settings;
                if self.show_api_settings {
                    self.show_developer_mode = false;
                }
                self.trigger_save();
            }
            let (bg_dict, text_dict, rounding_dict) = self.get_instance_style_full("btn_nav_dict");
            if ui.add(egui::Button::new(egui::RichText::new("📖").color(text_dict)).fill(bg_dict).rounding(rounding_dict)).on_hover_text(self.i18n.btn_nav_dict.clone()).clicked() {
                self.show_memory_viewer = !self.show_memory_viewer;
                if self.show_memory_viewer {
                    self.viewer_opening_counter = 10;
                } else {
                    let mut frames = self.viewer_shared.opened_frames.lock().unwrap();
                    *frames = 0;
                    self.is_memory_viewer_open
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    let mut opened = self.viewer_shared.opened_last_frame.lock().unwrap();
                    *opened = false;
                }
                self.trigger_save();
            }
            let (bg_pal, text_pal, rounding_pal) = self.get_instance_style_full("btn_nav_palette");
            if ui.add(egui::Button::new(egui::RichText::new("🎨").color(text_pal)).fill(bg_pal).rounding(rounding_pal)).on_hover_text(self.i18n.btn_nav_palette.clone()).clicked() {
                self.show_palette_settings = !self.show_palette_settings;
                if self.show_palette_settings {
                    self.show_api_settings = false;
                    self.show_developer_mode = false;
                }
                self.trigger_save();
            }
            let (bg_theme, text_theme, rounding_theme) = self.get_instance_style_full("btn_nav_theme");
            if ui.add(egui::Button::new(egui::RichText::new("🌓").color(text_theme)).fill(bg_theme).rounding(rounding_theme)).on_hover_text(self.i18n.btn_nav_theme.clone()).clicked() {
                self.theme = if self.theme == "dark" {
                    "light".into()
                } else {
                    "dark".into()
                };
                self.trigger_save();
            }
            let (bg_dev, text_dev, rounding_dev) = self.get_instance_style_full("btn_nav_dev");
            if ui.add(egui::Button::new(egui::RichText::new("🔧").color(text_dev)).fill(bg_dev).rounding(rounding_dev)).on_hover_text(self.i18n.btn_nav_dev.clone()).clicked() {
                self.show_developer_mode = !self.show_developer_mode;
                if self.show_developer_mode {
                    self.show_api_settings = false;
                }
                self.trigger_save();
            }
            ui.add_space(8.0);

            // UI 介面語言切換下拉選單 (放置在左側)
            let ui_langs = crate::ui::i18n::I18nLabels::get_available_ui_langs();
            egui::ComboBox::from_id_source("ui_lang_combo")
                .selected_text(&self.ui_lang)
                .show_ui(ui, |ui| {
                    for l in ui_langs {
                        if ui.selectable_value(&mut self.ui_lang, l.clone(), l.clone()).clicked() {
                            self.i18n = crate::ui::i18n::I18nLabels::load_or_default(&self.ui_lang);
                            self.trigger_save();
                        }
                    }
                });
        });
    }
}
