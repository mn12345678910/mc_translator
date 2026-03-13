use eframe::egui;
use crate::state::app_state::{AppState, PaletteEditSlot};

impl AppState {
    pub fn render_palette_settings(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("🎨 調色盤");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⟲ 全部重置").on_hover_text("將全程式所有設定、顏色、圓角恢復至預設值").clicked() {
                        self.show_restore_default_confirm = true;
                    }
                });
            });

            ui.separator();

            // 1. 編輯模式切換
            ui.horizontal(|ui| {
                ui.label("當前編輯模式:");
                if ui.selectable_label(!self.palette_edit_dark, "☀️ 淺色設定").clicked() {
                    self.palette_edit_dark = false;
                }
                if ui.selectable_label(self.palette_edit_dark, "🌙 深色設定").clicked() {
                    self.palette_edit_dark = true;
                }
            });

            ui.add_space(8.0);

            // 2. 編輯目標選擇 (Edit Slots)
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("【 1. 選擇編輯目標 】").strong());
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
                                    ("類別批量設定", vec![
                                        "全部按鈕", "全部標籤", "全部輸入框", "全部日誌區域", 
                                        "全部建議詞分頁", "全部進度條", "全部面板背景", "頂部導覽列"
                                    ]),
                                    ("特定元件 (精確覆寫)", vec![
                                        "[特定] 選擇檔案按鈕", "[特定] 選擇資料夾按鈕", 
                                        "[特定] 輸出資料夾按鈕", "[特定] 打開輸出按鈕",
                                        "[特定] 開始翻譯按鈕", "[特定] 暫停按鈕", 
                                        "[特定] 停止按鈕", "[特定] 清除執行日誌按鈕",
                                        "[特定] ⚙️ 設定按鈕", "[特定] 📖 字典按鈕",
                                        "[特定] 🎨 調色盤按鈕", "[特定] 🌓 主題按鈕",
                                        "[特定] 🔧 開發按鈕", "[特定] 建議詞搜尋框",
                                        "[特定] 字典列表區域", "[特定] 輸出路徑標籤",
                                        "[特定] 目前檔案進度條", "[特定] 總進度條"
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
                            if ui.button("🗑").on_hover_text("移除此編輯槽位").clicked() { 
                                remove_idx = Some(idx); 
                            }
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
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                         ui.label(egui::RichText::new(format!("共 {} 個槽位", slots_len)).small().weak());
                    });
                });
            });

            ui.add_space(10.0);

            // 3. 屬性調整區
            ui.label(egui::RichText::new("【 2. 調整屬性樣式 】").strong());
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
                            ui.checkbox(&mut self.palette_prop_sync_bg, "背景顏色");
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
                            ui.checkbox(&mut self.palette_prop_sync_text, "文字顏色");
                        });
                        let mut dummy_text = if is_dark { self.dark_btn_text } else { self.light_btn_text };
                        if ui.color_edit_button_srgb(&mut dummy_text).changed() && self.palette_prop_sync_text {
                            self.apply_batch_color(false, dummy_text);
                        }
                        ui.end_row();
                    }

                    // 自定義圓角 (Revision 15.30+)
                    if has_rounding_supported {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.palette_prop_sync_rounding, "自定義圓角");
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
                        ui.checkbox(&mut self.btn_rounding_enabled, "強制啟用全域按鈕圓角");
                        if self.btn_rounding_enabled {
                            if ui.add(egui::Slider::new(&mut self.btn_rounding_value, 0.0..=20.0).text("圓角數值")).changed() {
                                self.trigger_save();
                            }
                        }
                    });
                }

                if has_progress {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.progress_pulse_enabled, "啟用進度條呼吸脈衝動畫");
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
            ui.label(egui::RichText::new("ℹ 提示：特定元件覆寫的色彩優先級高於類別批量設定。").small().weak());
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
                match t.as_str() {
                    "全部按鈕" => if is_bg { if is_dark { self.dark_btn_bg = color; } else { self.light_btn_bg = color; } } 
                                  else { if is_dark { self.dark_btn_text = color; } else { self.light_btn_text = color; } },
                    "全部標籤" => if is_bg { if is_dark { self.dark_bg = color; } else { self.light_bg = color; } }
                                  else { if is_dark { self.dark_label = color; } else { self.light_label = color; } },
                    "全部輸入框" => if is_bg { if is_dark { self.dark_input_bg = color; } else { self.light_input_bg = color; } } 
                                   else { if is_dark { self.dark_text = color; } else { self.light_text = color; } },
                    "全部日誌區域" => if is_bg { if is_dark { self.dark_list_bg = color; } else { self.light_list_bg = color; } } 
                                    else { if is_dark { self.dark_text = color; } else { self.light_text = color; } },
                    "全部建議詞分頁" => if is_bg { if is_dark { self.dark_tab_active = color; } else { self.light_tab_active = color; } } 
                                     else { if is_dark { self.dark_btn_text = color; } else { self.light_btn_text = color; } },
                    "全部進度條" => if is_bg { 
                                       if is_dark { self.dark_bg = color; } else { self.light_bg = color; } 
                                   } else {
                                       // 同步至進度條文字顏色 (透過覆寫確保生效)
                                       let p_current = self.instance_overrides.entry("progress_current".to_string()).or_default();
                                       p_current.text = Some(color);
                                       let p_total = self.instance_overrides.entry("progress_total".to_string()).or_default();
                                       p_total.text = Some(color);
                                       // 同時更新全域標籤顏色以供回退使用 (Revision 15.32)
                                       if is_dark { self.dark_label = color; } else { self.light_label = color; }
                                   },
                    "全部面板背景" => if is_bg { if is_dark { self.dark_bg = color; } else { self.light_bg = color; } } else {},
                    "頂部導覽列" => if is_bg { if is_dark { self.dark_tab_inactive = color; } else { self.light_tab_inactive = color; } } 
                                   else { if is_dark { self.dark_btn_text = color; } else { self.light_btn_text = color; } },
                    _ => {}
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
                if t == "全部按鈕" || t == "全部進度條" || t == "頂部導覽列" {
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
            "[特定] 選擇檔案按鈕" => "btn_select_file",
            "[特定] 選擇資料夾按鈕" => "btn_select_folder",
            "[特定] 輸出資料夾按鈕" => "btn_output_dir",
            "[特定] 打開輸出按鈕" => "btn_open_output",
            "[特定] 開始翻譯按鈕" => "btn_run_trans",
            "[特定] 暫停按鈕" => "btn_pause",
            "[特定] 停止按鈕" => "btn_stop",
            "[特定] 清除執行日誌按鈕" => "btn_clear_log",
            "[特定] ⚙️ 設定按鈕" => "btn_nav_settings",
            "[特定] 📖 字典按鈕" => "btn_nav_dict",
            "[特定] 🎨 調色盤按鈕" => "btn_nav_palette",
            "[特定] 🌓 主題按鈕" => "btn_nav_theme",
            "[特定] 🔧 開發按鈕" => "btn_nav_dev",
            "[特定] 建議詞搜尋框" => "input_dict_search",
            "[特定] 字典列表區域" => "area_dict_list",
            "[特定] 輸出路徑標籤" => "label_output_path",
            "[特定] 目前檔案進度條" => "progress_current",
            "[特定] 總進度條" => "progress_total",
            _ => name,
        }.to_string()
    }

    /// 檢查目標是否支援特定屬性 (背景, 文字, 圓角)
    fn is_prop_supported(&self, target: &str) -> (bool, bool, bool) {
        match target {
            "全部按鈕" | "頂部導覽列" | "[特定] 選擇檔案按鈕" | "[特定] 選擇資料夾按鈕" | 
            "[特定] 輸出資料夾按鈕" | "[特定] 打開輸出按鈕" | "[特定] 開始翻譯按鈕" | 
            "[特定] 暫停按鈕" | "[特定] 停止按鈕" | "[特定] 清除執行日誌按鈕" | 
            "[特定] ⚙️ 設定按鈕" | "[特定] 📖 字典按鈕" | "[特定] 🎨 調色盤按鈕" | 
            "[特定] 🌓 主題按鈕" | "[特定] 🔧 開發按鈕" => (true, true, true),
            
            "全部標籤" | "[特定] 輸出路徑標籤" => (true, true, false),
            
            "全部進度條" | "[特定] 目前檔案進度條" | "[特定] 總進度條" => (true, true, true),
            
            "全部輸入框" | "[特定] 建議詞搜尋框" => (true, true, false),
            
            "全部日誌區域" | "全部建議詞分頁" | "[特定] 字典列表區域" => (true, true, false),
            
            "全部面板背景" => (true, false, false),
            
            _ => (true, true, true),
        }
    }
}
