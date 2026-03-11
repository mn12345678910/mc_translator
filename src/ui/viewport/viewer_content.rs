use crate::state::app_state::AppState;
use crate::state::viewer_state::ViewerSharedState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};
use crate::ui::widgets::toggle::toggle;
use std::sync::{Arc, Mutex};

impl AppState {
    pub fn render_memory_viewer_content(
        ctx: &egui::Context,
        is_processing: Arc<Mutex<bool>>,
        _is_paused: Arc<Mutex<bool>>,
        viewer_shared: Arc<ViewerSharedState>,
        translation_memory: Arc<Mutex<std::collections::HashMap<String, String>>>,
        inferred_match_map: Arc<Mutex<std::collections::HashMap<String, String>>>,
        _term_replacements: Arc<Mutex<Vec<(String, String)>>>,
        _dict_cache: Arc<Mutex<Vec<(String, String)>>>,
        dict_search: Arc<Mutex<String>>,
        dict_search_last: Arc<Mutex<(String, usize)>>,
        dict_page: Arc<Mutex<usize>>,
        dict_active_tab: Arc<Mutex<usize>>,
        dict_edit_key: Arc<Mutex<Option<String>>>,
        dict_edit_value: Arc<Mutex<String>>,
        dict_new_key: Arc<Mutex<String>>,
        dict_new_value: Arc<Mutex<String>>,
        dict_replace_target: Arc<Mutex<String>>,
        dict_replace_new: Arc<Mutex<String>>,
        dict_replace_all: Arc<Mutex<bool>>,
        show_dict_add_dialog: Arc<Mutex<bool>>,
        show_dict_replace_dialog: Arc<Mutex<bool>>,
        show_dict_clear_confirm: Arc<Mutex<bool>>,
        glossary_priority: Arc<Mutex<String>>,
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
