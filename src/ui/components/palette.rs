use eframe::egui;
use crate::state::app_state::{AppState, PaletteEditSlot};

impl AppState {
    pub fn render_palette_settings(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(self.i18n.header_palette.clone());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(self.i18n.btn_reset_all.clone()).on_hover_text(self.i18n.hover_reset_all.clone()).clicked() {
                        self.show_restore_default_confirm = true;
                    }
                });
            });

            ui.separator();

            // 1. 編輯模式切換
            ui.horizontal(|ui| {
                ui.label(self.i18n.label_edit_mode.clone());
                if ui.selectable_label(!self.palette_edit_dark, self.i18n.mode_light.clone()).clicked() {
                    self.palette_edit_dark = false;
                }
                if ui.selectable_label(self.palette_edit_dark, self.i18n.mode_dark.clone()).clicked() {
                    self.palette_edit_dark = true;
                }
            });

            ui.add_space(8.0);

            // 2. 編輯目標選擇 (Edit Slots)
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(self.i18n.label_palette_step_1.clone()).strong());
                let mut remove_idx = None;
                let slots_len = self.palette_edit_slots.len();
                
                for (idx, slot) in self.palette_edit_slots.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // 移除原有的 checkbox，直接下拉選單
                        ui.label(format!("#{}", idx + 1));
                        egui::ComboBox::from_id_source(format!("slot_{}", idx))
                            .selected_text(&slot.target_id)
                            .width(200.0)
                            .show_ui(ui, |ui| {
                                let target_groups = [
                                    (self.i18n.group_batch.clone(), vec![
                                        self.i18n.cat_all_buttons.clone(), self.i18n.cat_all_labels.clone(), self.i18n.cat_all_inputs.clone(), self.i18n.cat_all_logs.clone(), 
                                        self.i18n.cat_all_tabs.clone(), self.i18n.cat_all_progress.clone(), self.i18n.cat_all_bg.clone(), self.i18n.cat_nav_bar.clone()
                                    ]),
                                    (self.i18n.group_specific.clone(), vec![
                                        self.i18n.spec_btn_select_file.clone(), self.i18n.spec_btn_select_folder.clone(), 
                                        self.i18n.spec_btn_output_dir.clone(), self.i18n.spec_btn_open_output.clone(),
                                        self.i18n.spec_btn_run_trans.clone(), self.i18n.spec_btn_pause.clone(), 
                                        self.i18n.spec_btn_stop.clone(), self.i18n.spec_btn_clear_log.clone(),
                                        self.i18n.spec_btn_nav_settings.clone(), self.i18n.spec_btn_nav_dict.clone(),
                                        self.i18n.spec_btn_nav_palette.clone(), self.i18n.spec_btn_nav_theme.clone(),
                                        self.i18n.spec_btn_nav_dev.clone(), self.i18n.spec_input_search.clone(),
                                        self.i18n.spec_area_dict.clone(), self.i18n.spec_label_output.clone(),
                                        self.i18n.spec_progress_current.clone(), self.i18n.spec_progress_total.clone()
                                    ])
                                ];
                                
                                for (group_name, group_items) in target_groups {
                                    ui.label(egui::RichText::new(group_name).small().weak());
                                    for t in group_items {
                                        ui.selectable_value(&mut slot.target_id, t.to_string(), t);
                                    }
                                    ui.separator();
                                }
                            });
                        
                        if slots_len > 1 {
                            if ui.button("🗑").on_hover_text(self.i18n.hover_remove_slot.clone()).clicked() { 
                                remove_idx = Some(idx); 
                            }
                        }
                    });
                }
                if let Some(idx) = remove_idx { self.palette_edit_slots.remove(idx); }

                ui.horizontal(|ui| {
                    if ui.button(self.i18n.btn_add_target.clone()).clicked() {
                        self.palette_edit_slots.push(PaletteEditSlot { 
                            target_id: self.i18n.cat_all_buttons.clone(), 
                            is_checked: true 
                        });
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                         ui.label(egui::RichText::new(self.i18n.label_slot_count.replace("{}", &slots_len.to_string())).small().weak());
                    });
                });
            });

            ui.add_space(10.0);

            // 3. 屬性調整區
            ui.label(egui::RichText::new(self.i18n.label_palette_step_2.clone()).strong());
            egui::Frame::group(ui.style()).show(ui, |ui| {
                let is_dark = self.palette_edit_dark;
                // V6 邏輯：所有在列表中的槽位皆視為選取
                let active_targets: Vec<String> = self.palette_edit_slots.iter()
                    .map(|s| s.target_id.clone())
                    .collect();

                egui::Grid::new("prop_grid_v6").num_columns(2).spacing([20.0, 10.0]).show(ui, |ui| {
                    let mut has_bg_supported = false;
                    let mut has_text_supported = false;
                    let mut has_rounding_supported = false;

                    for target in &active_targets {
                        let (b, t, r) = self.is_prop_supported(target);
                        if b { has_bg_supported = true; }
                        if t { has_text_supported = true; }
                        if r { has_rounding_supported = true; }
                    }

                    // 背景顏色
                    if has_bg_supported {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.palette_prop_sync_bg, self.i18n.label_bg_color.clone());
                        });
                        // 這裡取第一個目標的顏色作為預覽，實際變更會同步所有
                        let mut dummy_bg = if is_dark { self.dark_btn_bg } else { self.light_btn_bg };
                        if ui.color_edit_button_srgb(&mut dummy_bg).changed() && self.palette_prop_sync_bg {
                            self.apply_batch_color(true, dummy_bg);
                        }
                        ui.end_row();
                    }

                    // 文字顏色
                    if has_text_supported {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.palette_prop_sync_text, self.i18n.label_text_color.clone());
                        });
                        let mut dummy_text = if is_dark { self.dark_btn_text } else { self.light_btn_text };
                        if ui.color_edit_button_srgb(&mut dummy_text).changed() && self.palette_prop_sync_text {
                            self.apply_batch_color(false, dummy_text);
                        }
                        ui.end_row();
                    }

                    // 自定義圓角
                    if has_rounding_supported {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.palette_prop_sync_rounding, self.i18n.label_custom_rounding.clone());
                        });
                        let mut dummy_rounding = self.btn_rounding_value;
                        if ui.add(egui::DragValue::new(&mut dummy_rounding).speed(0.5).clamp_range(0.0..=30.0)).changed() && self.palette_prop_sync_rounding {
                            self.apply_batch_rounding(dummy_rounding);
                        }
                        ui.end_row();
                    }
                });

                // 4. 組件專用屬性 (圓角/動畫)
                let has_button = active_targets.iter().any(|t| t.contains("按鈕") || t.contains("🎨") || t.contains("⚙") || t.contains("🌓") || t.contains("🔧") || t.contains("📖"));
                let has_progress = active_targets.iter().any(|t| t.contains("進度條"));

                if has_button {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.btn_rounding_enabled, self.i18n.label_force_global_rounding.clone());
                        if self.btn_rounding_enabled {
                            if ui.add(egui::Slider::new(&mut self.btn_rounding_value, 0.0..=20.0).text(self.i18n.label_rounding_value.clone())).changed() {
                                self.trigger_save();
                            }
                        }
                    });
                }

                if has_progress {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.progress_pulse_enabled, self.i18n.label_enable_pulse.clone());
                    });
                    if self.progress_pulse_enabled {
                        ui.horizontal(|ui| {
                            ui.label(self.i18n.label_anim_speed.clone());
                            if ui.add(egui::Slider::new(&mut self.progress_pulse_speed, 0.1..=5.0)).changed() {
                                self.trigger_save();
                            }
                        });
                    }
                }
            });

            ui.add_space(10.0);
            ui.label(egui::RichText::new(self.i18n.label_palette_hint.clone()).small().weak());
        });
    }

    /// 批次套用顏色至清單中所有目標
    fn apply_batch_color(&mut self, is_bg: bool, color: [u8; 3]) {
        let is_dark = self.palette_edit_dark;
        let targets: Vec<String> = self.palette_edit_slots.iter()
            .map(|s| s.target_id.clone())
            .collect();

        for t in targets {
            if t.contains("全部") || !t.contains("[特定]") {
                // 類別更新 (V6 擴展)
                if t == self.i18n.cat_all_buttons {
                    if is_bg { if is_dark { self.dark_btn_bg = color; } else { self.light_btn_bg = color; } } 
                    else { if is_dark { self.dark_btn_text = color; } else { self.light_btn_text = color; } }
                } else if t == self.i18n.cat_all_labels {
                    if is_bg { if is_dark { self.dark_bg = color; } else { self.light_bg = color; } }
                    else { if is_dark { self.dark_label = color; } else { self.light_label = color; } }
                } else if t == self.i18n.cat_all_inputs {
                    if is_bg { if is_dark { self.dark_input_bg = color; } else { self.light_input_bg = color; } } 
                    else {
                        // 針對輸入框實施覆寫，不再修改全域 dark_text/light_text
                        let key = self.get_id_from_target_name(&self.i18n.spec_input_search);
                        self.instance_overrides.entry(key).or_default().text = Some(color);
                    }
                } else if t == self.i18n.cat_all_logs {
                    if is_bg { if is_dark { self.dark_list_bg = color; } else { self.light_list_bg = color; } } 
                    else {
                        // 針對日誌區與字典列表實施覆寫
                        let key_log = "area_log".to_string();
                        self.instance_overrides.entry(key_log).or_default().text = Some(color);
                        let key_dict = self.get_id_from_target_name(&self.i18n.spec_area_dict);
                        self.instance_overrides.entry(key_dict).or_default().text = Some(color);
                    }
                } else if t == self.i18n.cat_all_tabs {
                    if is_bg { if is_dark { self.dark_tab_active = color; } else { self.light_tab_active = color; } } 
                    else { if is_dark { self.dark_btn_text = color; } else { self.light_btn_text = color; } }
                } else if t == self.i18n.cat_all_progress {
                    if is_bg { 
                        if is_dark { self.dark_bg = color; } else { self.light_bg = color; } 
                    } else {
                        // 同步至進度條文字顏色 (透過覆寫確保生效)
                        let p_current = self.instance_overrides.entry("progress_current".to_string()).or_default();
                        p_current.text = Some(color);
                        let p_total = self.instance_overrides.entry("progress_total".to_string()).or_default();
                        p_total.text = Some(color);
                        // 同時更新全域標籤顏色以供回退使用
                        if is_dark { self.dark_label = color; } else { self.light_label = color; }
                    }
                } else if t == self.i18n.cat_all_bg {
                    if is_bg { if is_dark { self.dark_bg = color; } else { self.light_bg = color; } }
                } else if t == self.i18n.cat_nav_bar {
                    if is_bg { if is_dark { self.dark_tab_inactive = color; } else { self.light_tab_inactive = color; } } 
                    else { if is_dark { self.dark_btn_text = color; } else { self.light_btn_text = color; } }
                }
            } else {
                // 特定元件覆寫 (V6 映射)
                let key = self.get_id_from_target_name(&t);
                let entry = self.instance_overrides.entry(key).or_default();
                if is_bg { entry.bg = Some(color); } else { entry.text = Some(color); }
            }
        }
        self.trigger_save();
    }

    /// 批次套用圓角至清單中所有目標 (Revision 15.30+)
    fn apply_batch_rounding(&mut self, val: f32) {
        let targets: Vec<String> = self.palette_edit_slots.iter()
            .map(|s| s.target_id.clone())
            .collect();

        for t in targets {
            if t.contains("全部") || !t.contains("[特定]") {
                // 類別更新
                if t == self.i18n.cat_all_buttons || t == self.i18n.cat_all_progress || t == self.i18n.cat_nav_bar {
                    self.btn_rounding_value = val;
                }
            } else {
                // 特定元件覆寫
                let key = self.get_id_from_target_name(&t);
                let entry = self.instance_overrides.entry(key).or_default();
                entry.rounding = Some(val);
            }
        }
        self.trigger_save();
    }

    fn get_id_from_target_name(&self, name: &str) -> String {
        match name {
            n if n == self.i18n.spec_btn_select_file => "btn_select_file",
            n if n == self.i18n.spec_btn_select_folder => "btn_select_folder",
            n if n == self.i18n.spec_btn_output_dir => "btn_output_dir",
            n if n == self.i18n.spec_btn_open_output => "btn_open_output",
            n if n == self.i18n.spec_btn_run_trans => "btn_run_trans",
            n if n == self.i18n.spec_btn_pause => "btn_pause",
            n if n == self.i18n.spec_btn_stop => "btn_stop",
            n if n == self.i18n.spec_btn_clear_log => "btn_clear_log",
            n if n == self.i18n.spec_btn_nav_settings => "btn_nav_settings",
            n if n == self.i18n.spec_btn_nav_dict => "btn_nav_dict",
            n if n == self.i18n.spec_btn_nav_palette => "btn_nav_palette",
            n if n == self.i18n.spec_btn_nav_theme => "btn_nav_theme",
            n if n == self.i18n.spec_btn_nav_dev => "btn_nav_dev",
            n if n == self.i18n.spec_input_search => "input_dict_search",
            n if n == self.i18n.spec_area_dict => "area_dict_list",
            n if n == self.i18n.spec_label_output => "label_output_path",
            n if n == self.i18n.spec_progress_current => "progress_current",
            n if n == self.i18n.spec_progress_total => "progress_total",
            _ => name,
        }.to_string()
    }

    /// 檢查目標是否支援特定屬性 (背景, 文字, 圓角)
    fn is_prop_supported(&self, target: &str) -> (bool, bool, bool) {
        match target {
            n if n == self.i18n.cat_all_buttons || n == self.i18n.cat_nav_bar || 
            n == self.i18n.spec_btn_select_file || n == self.i18n.spec_btn_select_folder || 
            n == self.i18n.spec_btn_output_dir || n == self.i18n.spec_btn_open_output || 
            n == self.i18n.spec_btn_run_trans || n == self.i18n.spec_btn_pause || 
            n == self.i18n.spec_btn_stop || n == self.i18n.spec_btn_clear_log || 
            n == self.i18n.spec_btn_nav_settings || n == self.i18n.spec_btn_nav_dict || 
            n == self.i18n.spec_btn_nav_palette || n == self.i18n.spec_btn_nav_theme || 
            n == self.i18n.spec_btn_nav_dev => (true, true, true),
            
            n if n == self.i18n.cat_all_labels || n == self.i18n.spec_label_output => (true, true, false),
            
            n if n == self.i18n.cat_all_progress || n == self.i18n.spec_progress_current || n == self.i18n.spec_progress_total => (true, true, true),
            
            n if n == self.i18n.cat_all_inputs || n == self.i18n.spec_input_search => (true, true, false),
            
            n if n == self.i18n.cat_all_logs || n == self.i18n.cat_all_tabs || n == self.i18n.spec_area_dict => (true, true, false),
            
            n if n == self.i18n.cat_all_bg => (true, false, false),
            
            _ => (true, true, true),
        }
    }
}
