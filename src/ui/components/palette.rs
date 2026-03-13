use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};
use eframe::egui;
use crate::state::app_state::{AppState, PaletteEditSlot};

impl AppState {
    pub fn render_palette_settings(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("🎨 自定義調色盤 (V5)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⟲ 全部重置").on_hover_text("將所有顏色恢復為預設值").clicked() {
                        self.show_restore_default_confirm = true;
                    }
                });
            });

            ui.separator();

            // 1. 編輯模式切換
            ui.horizontal(|ui| {
                ui.label("當前編輯模式:");
                if ui.selectable_label(!self.palette_edit_dark, "☀️ 淺色").clicked() {
                    self.palette_edit_dark = false;
                }
                if ui.selectable_label(self.palette_edit_dark, "🌙 深色").clicked() {
                    self.palette_edit_dark = true;
                }
            });

            ui.add_space(10.0);

            // 2. 編輯目標選擇 (Edit Slots)
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("【 1. 選擇編輯目標 】").strong());
                let mut remove_idx = None;
                for (idx, slot) in self.palette_edit_slots.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut slot.is_checked, "");
                        egui::ComboBox::from_id_source(format!("slot_{}", idx))
                            .selected_text(&slot.target_id)
                            .show_ui(ui, |ui| {
                                let targets = [
                                    "全部按鈕", "全部標籤", "全部輸入框", "全部日誌區域", 
                                    "全部建議詞分頁", "全部進度條",
                                    "[特定] 選擇檔案按鈕", "[特定] 執行翻譯按鈕", "[特定] 建議詞搜尋框"
                                ];
                                for t in targets {
                                    ui.selectable_value(&mut slot.target_id, t.to_string(), t);
                                }
                            });
                        if self.palette_edit_slots.len() > 1 {
                            if ui.button("🗑").clicked() { remove_idx = Some(idx); }
                        }
                    });
                }
                if let Some(idx) = remove_idx { self.palette_edit_slots.remove(idx); }

                ui.horizontal(|ui| {
                    if ui.button("+ 新增目標").clicked() {
                        self.palette_edit_slots.push(PaletteEditSlot { 
                            target_id: "全部按鈕".to_string(), 
                            is_checked: true 
                        });
                    }
                    if ui.button("全選/取消").clicked() {
                        self.palette_all_selected = !self.palette_all_selected;
                        for s in &mut self.palette_edit_slots { s.is_checked = self.palette_all_selected; }
                    }
                });
            });

            ui.add_space(10.0);

            // 3. 屬性調整區
            ui.label(egui::RichText::new("【 2. 勾選屬性進行批次調整 】").strong());
            egui::Frame::group(ui.style()).show(ui, |ui| {
                let is_dark = self.palette_edit_dark;
                let active_targets: Vec<String> = self.palette_edit_slots.iter()
                    .filter(|s| s.is_checked)
                    .map(|s| s.target_id.clone())
                    .collect();

                if active_targets.is_empty() {
                    ui.label(egui::RichText::new("請先勾選上方編輯目標").weak());
                    return;
                }

                egui::Grid::new("prop_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                    // 背景顏色
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.palette_prop_sync_bg, "背景顏色");
                    });
                    // 這裡取第一個目標的顏色作為預覽，實際變更會同步所有
                    let mut dummy_bg = if is_dark { self.dark_btn_bg } else { self.light_btn_bg };
                    if ui.color_edit_button_srgb(&mut dummy_bg).changed() && self.palette_prop_sync_bg {
                        self.apply_batch_color(true, dummy_bg);
                    }
                    ui.end_row();

                    // 文字顏色
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.palette_prop_sync_text, "文字顏色");
                    });
                    let mut dummy_text = if is_dark { self.dark_btn_text } else { self.light_btn_text };
                    if ui.color_edit_button_srgb(&mut dummy_text).changed() && self.palette_prop_sync_text {
                        self.apply_batch_color(false, dummy_text);
                    }
                    ui.end_row();
                });

                // 4. 組件專用屬性 (圓角/動畫)
                let has_button = active_targets.iter().any(|t| t.contains("按鈕"));
                let has_progress = active_targets.iter().any(|t| t.contains("進度條"));

                if has_button {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.btn_rounding_enabled, "啟用控制按鈕圓角");
                        if self.btn_rounding_enabled {
                            if ui.add(egui::Slider::new(&mut self.btn_rounding_value, 0.0..=20.0)).changed() {
                                self.trigger_save();
                            }
                        }
                    });
                }

                if has_progress {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.progress_pulse_enabled, "啟用進度條脈衝動畫");
                    });
                    if self.progress_pulse_enabled {
                        ui.horizontal(|ui| {
                            ui.label("動畫速度:");
                            if ui.add(egui::Slider::new(&mut self.progress_pulse_speed, 0.1..=5.0)).changed() {
                                self.trigger_save();
                            }
                        });
                    }
                }
            });

            ui.add_space(10.0);
            ui.label(egui::RichText::new("ℹ 變更將立即套用並透過背景任務儲存").small().weak());
        });
    }

    /// 批次套用顏色至勾選的目標
    fn apply_batch_color(&mut self, is_bg: bool, color: [u8; 3]) {
        let is_dark = self.palette_edit_dark;
        let targets: Vec<String> = self.palette_edit_slots.iter()
            .filter(|s| s.is_checked)
            .map(|s| s.target_id.clone())
            .collect();

        for t in targets {
            if t.contains("全部") || !t.contains("[特定]") {
                // 類別更新
                match t.as_str() {
                    "全部按鈕" => if is_bg { if is_dark { self.dark_btn_bg = color; } else { self.light_btn_bg = color; } } 
                                  else { if is_dark { self.dark_btn_text = color; } else { self.light_btn_text = color; } },
                    "全部標籤" => if is_bg { if is_dark { self.dark_label = color; } else { self.light_label = color; } }
                                  else { if is_dark { self.dark_text = color; } else { self.light_text = color; } },
                    "全部輸入框" => if is_bg { if is_dark { self.dark_input_bg = color; } else { self.light_input_bg = color; } } else {},
                    "全部日誌區域" => if is_bg { if is_dark { self.dark_list_bg = color; } else { self.light_list_bg = color; } } else {},
                    "全部建議詞分頁" => if is_bg { if is_dark { self.dark_tab_active = color; } else { self.light_tab_active = color; } } else {},
                    "全部進度條" => if is_bg { if is_dark { self.dark_bg = color; } else { self.light_bg = color; } } else {}, // 借用主背景
                    _ => {}
                }
            } else {
                // 特定元件覆寫
                let key = self.get_id_from_target_name(&t);
                let entry = self.instance_overrides.entry(key).or_default();
                if is_bg { entry.bg = Some(color); } else { entry.text = Some(color); }
            }
        }
        self.trigger_save();
    }

    fn get_id_from_target_name(&self, name: &str) -> String {
        match name {
            "[特定] 選擇檔案按鈕" => "btn_select_file",
            "[特定] 執行翻譯按鈕" => "btn_run_trans",
            "[特定] 建議詞搜尋框" => "input_dict_search",
            _ => name,
        }.to_string()
    }
}
