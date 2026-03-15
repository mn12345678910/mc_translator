use crate::state::app_state::AppState;
use crate::state::viewer_state::ViewerSharedState;
// 移除硬編碼常量引用
use crate::ui::widgets::toggle::toggle;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

impl AppState {
    pub fn render_memory_viewer_content(
        ctx: &egui::Context,
        i18n: &crate::ui::i18n::I18nLabels,
        is_processing: Arc<AtomicBool>,
        _is_paused: Arc<AtomicBool>,
        viewer_shared: Arc<ViewerSharedState>,
        translation_memory: Arc<Mutex<std::collections::HashMap<String, String>>>,
        inferred_match_map: Arc<Mutex<std::collections::HashMap<String, String>>>,
        _term_replacements: Arc<Mutex<Vec<(String, String)>>>,
        _dict_cache: Arc<Mutex<Vec<(String, String)>>>,
        dict_search: Arc<Mutex<String>>,
        dict_search_last: Arc<Mutex<(String, usize)>>,
        dict_page: Arc<AtomicUsize>,
        dict_active_tab: Arc<AtomicUsize>,
        dict_edit_key: Arc<Mutex<Option<String>>>,
        dict_edit_value: Arc<Mutex<String>>,
        dict_new_key: Arc<Mutex<String>>,
        dict_new_value: Arc<Mutex<String>>,
        dict_replace_target: Arc<Mutex<String>>,
        dict_replace_new: Arc<Mutex<String>>,
        dict_replace_all: Arc<AtomicBool>,
        show_dict_add_dialog: Arc<AtomicBool>,
        show_dict_replace_dialog: Arc<AtomicBool>,
        show_dict_clear_confirm: Arc<AtomicBool>,
        glossary_priority: Arc<Mutex<String>>,
    ) {
        let is_dark = *viewer_shared.theme.read().unwrap() == "dark";
        let font_size = *viewer_shared.font_size.read().unwrap();
        let style_snap = viewer_shared.style.read().unwrap().clone();
        
        let mut style = (*ctx.style()).clone();

        // 完整移植主視窗的視覺參數，確保按鈕、選取色與主題高度一致 (Revision 15.18)
        let visuals = if is_dark {
            let mut v = egui::Visuals::dark();
            let bg = egui::Color32::from_rgb(style_snap.dark_bg[0], style_snap.dark_bg[1], style_snap.dark_bg[2]);
            let text = egui::Color32::from_rgb(style_snap.dark_text[0], style_snap.dark_text[1], style_snap.dark_text[2]);
            let btn_bg = egui::Color32::from_rgb(style_snap.dark_btn_bg[0], style_snap.dark_btn_bg[1], style_snap.dark_btn_bg[2]);
            let btn_text = egui::Color32::from_rgb(style_snap.dark_btn_text[0], style_snap.dark_btn_text[1], style_snap.dark_btn_text[2]);
            let input_bg = egui::Color32::from_rgb(style_snap.dark_input_bg[0], style_snap.dark_input_bg[1], style_snap.dark_input_bg[2]);
            let list_bg = egui::Color32::from_rgb(style_snap.dark_list_bg[0], style_snap.dark_list_bg[1], style_snap.dark_list_bg[2]);

            v.window_fill = bg;
            v.panel_fill = bg;
            v.override_text_color = Some(text);
            v.extreme_bg_color = input_bg;
            v.faint_bg_color = list_bg;
            v.selection.bg_fill = egui::Color32::from_rgb(style_snap.dark_tab_active[0], style_snap.dark_tab_active[1], style_snap.dark_tab_active[2]);

            v.widgets.inactive.bg_fill = btn_bg;
            v.widgets.inactive.weak_bg_fill = btn_bg;
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, btn_text);
            v.widgets.hovered.bg_fill = btn_bg.linear_multiply(1.1);
            v.widgets.active.bg_fill = btn_bg.linear_multiply(0.9);
            v
        } else {
            let mut v = egui::Visuals::light();
            let bg = egui::Color32::from_rgb(style_snap.light_bg[0], style_snap.light_bg[1], style_snap.light_bg[2]);
            let text = egui::Color32::from_rgb(style_snap.light_text[0], style_snap.light_text[1], style_snap.light_text[2]);
            let btn_bg = egui::Color32::from_rgb(style_snap.light_btn_bg[0], style_snap.light_btn_bg[1], style_snap.light_btn_bg[2]);
            let btn_text = egui::Color32::from_rgb(style_snap.light_btn_text[0], style_snap.light_btn_text[1], style_snap.light_btn_text[2]);
            let input_bg = egui::Color32::from_rgb(style_snap.light_input_bg[0], style_snap.light_input_bg[1], style_snap.light_input_bg[2]);
            let list_bg = egui::Color32::from_rgb(style_snap.light_list_bg[0], style_snap.light_list_bg[1], style_snap.light_list_bg[2]);

            v.window_fill = bg;
            v.panel_fill = bg;
            v.override_text_color = Some(text);
            v.extreme_bg_color = input_bg;
            v.faint_bg_color = list_bg;
            v.selection.bg_fill = egui::Color32::from_rgb(style_snap.light_tab_active[0], style_snap.light_tab_active[1], style_snap.light_tab_active[2]);

            v.widgets.inactive.bg_fill = btn_bg;
            v.widgets.inactive.weak_bg_fill = btn_bg;
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.2, btn_text);
            v.widgets.hovered.bg_fill = btn_bg.linear_multiply(0.95);
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
            let processing = is_processing.load(Ordering::SeqCst);
            let current_tab = dict_active_tab.load(Ordering::SeqCst);
            let (_, label_color) = get_instance_style_from_snap(&style_snap, "label_viewer", is_dark);

            ui.label(
                egui::RichText::new(i18n.glossary_title.clone())
                    .heading()
                    .color(label_color)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(i18n.glossary_desc.clone())
                .color(label_color)
                .strong(),
            );

            ui.horizontal(|ui| {
                let current_tab_val = dict_active_tab.load(Ordering::SeqCst);
                let mut active_tab = current_tab_val;
                let theme_val = viewer_shared.theme.read().unwrap().clone();
                let is_light = theme_val == "light";
                let fill = if is_light {
                    egui::Color32::from_rgb(style_snap.light_tab_active[0], style_snap.light_tab_active[1], style_snap.light_tab_active[2])
                } else {
                    egui::Color32::from_rgb(style_snap.dark_tab_active[0], style_snap.dark_tab_active[1], style_snap.dark_tab_active[2])
                };
 
                egui::Frame::none()
                    .fill(fill)
                    .rounding(4.0)
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        if ui
                            .selectable_value(&mut active_tab, 0, i18n.glossary_tab_user.clone())
                            .clicked()
                        {
                            dict_page.store(0, Ordering::SeqCst);
                        }
                        if ui
                            .selectable_value(&mut active_tab, 1, i18n.glossary_tab_official.clone())
                            .clicked()
                        {
                            dict_page.store(0, Ordering::SeqCst);
                        }
                    });
                
                if active_tab != current_tab_val {
                    dict_active_tab.store(active_tab, Ordering::SeqCst);
                }
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
                    .add_enabled(!processing, egui::Button::new(i18n.btn_add.clone()))
                    .clicked()
                {
                    show_dict_add_dialog.store(true, Ordering::SeqCst);
                }
                // 取代按鈕
                if ui
                    .add_enabled(!processing, egui::Button::new(i18n.btn_replace.clone()))
                    .clicked()
                {
                    show_dict_replace_dialog.store(true, Ordering::SeqCst);
                }
                // 匯入按鈕
                if ui
                    .add_enabled(!processing, egui::Button::new(i18n.btn_import.clone()))
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
                if ui.button(i18n.btn_export.clone()).clicked() {
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
                    .on_hover_text(i18n.spec_btn_nav_dict.clone()) // 使用已有的 hover 文字
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
                        {
                            use std::os::windows::process::CommandExt;
                            let _ = std::process::Command::new("explorer")
                                .arg("/select,")
                                .arg(&abs_path)
                                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                                .spawn();

                            // 2. 以預設編輯器開啟實體檔案
                            let _ = std::process::Command::new("cmd")
                                .arg("/c")
                                .arg("start")
                                .arg("")
                                .arg(&abs_path)
                                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                                .spawn();
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(!processing, egui::Button::new(i18n.btn_clear_all.clone()))
                        .clicked()
                    {
                        show_dict_clear_confirm.store(true, Ordering::SeqCst);
                    }
                });
            });

            // --- 補回新增與取代對話框區塊 (Revision 14.1) ---
            if show_dict_add_dialog.load(Ordering::SeqCst) {
                egui::Window::new(i18n.glossary_add_title.clone())
                    .collapsible(false)
                    .resizable(false)
                    .default_pos([400.0, 300.0]) // 移除 anchor 使其可移動 (Revision 14.4)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(i18n.glossary_key.clone());
                            let (input_bg, input_text) = get_instance_style_from_snap(&style_snap, "dict_new_key", is_dark);
                            ui.visuals_mut().extreme_bg_color = input_bg;
                            ui.add(egui::TextEdit::singleline(&mut *dict_new_key.lock().unwrap()).text_color(input_text));
                        });
                        ui.horizontal(|ui| {
                            ui.label(i18n.glossary_value.clone());
                            let (input_bg, input_text) = get_instance_style_from_snap(&style_snap, "dict_new_value", is_dark);
                            ui.visuals_mut().extreme_bg_color = input_bg;
                            ui.add(egui::TextEdit::singleline(&mut *dict_new_value.lock().unwrap()).text_color(input_text));
                        });
                        ui.horizontal(|ui| {
                            let confirm_btn = ui.button(i18n.btn_confirm_add.clone());
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
                                        // 官方分頁編輯也存入使用者字典並從官方移除 (Migration)
                                        let mut mem = translation_memory.lock().unwrap();
                                        mem.insert(key.clone(), val);
                                        crate::config::save_translation_memory(&*mem);
                                        
                                        let mut inferred = inferred_match_map.lock().unwrap();
                                        if inferred.remove(&key).is_some() {
                                            crate::config::save_dict(
                                                crate::config::OFFICIAL_DICT,
                                                &*inferred,
                                            );
                                        }
                                    }
                                }
                                show_dict_add_dialog.store(false, Ordering::SeqCst);
                                *dict_new_key.lock().unwrap() = String::new();
                                *dict_new_value.lock().unwrap() = String::new();
                            }
                            if ui.button(i18n.btn_cancel.clone()).clicked() {
                                show_dict_add_dialog.store(false, Ordering::SeqCst);
                            }
                        });
                    });
            }
 
            if show_dict_replace_dialog.load(Ordering::SeqCst) {
                egui::Window::new(i18n.glossary_replace_title.clone())
                    .collapsible(false)
                    .resizable(false)
                    .default_pos([400.0, 300.0]) // 移除 anchor 使其可移動 (Revision 14.4)
                    .show(ctx, |ui| {
                        ui.label(i18n.glossary_replace_desc.clone());
                        ui.horizontal(|ui| {
                            ui.label(i18n.glossary_old_value.clone());
                            let (input_bg, input_text) = get_instance_style_from_snap(&style_snap, "dict_replace_target", is_dark);
                            ui.visuals_mut().extreme_bg_color = input_bg;
                            ui.add(egui::TextEdit::singleline(&mut *dict_replace_target.lock().unwrap()).text_color(input_text));
                        });
                        ui.horizontal(|ui| {
                            ui.label(i18n.glossary_new_value.clone());
                            let (input_bg, input_text) = get_instance_style_from_snap(&style_snap, "dict_replace_new", is_dark);
                            ui.visuals_mut().extreme_bg_color = input_bg;
                            ui.add(egui::TextEdit::singleline(&mut *dict_replace_new.lock().unwrap()).text_color(input_text));
                        });
                        
                        let current_replace_all = dict_replace_all.load(Ordering::SeqCst);
                        let mut replace_all_val = current_replace_all;
                        if ui.checkbox(&mut replace_all_val, i18n.glossary_replace_exact.clone()).changed() {
                            dict_replace_all.store(replace_all_val, Ordering::SeqCst);
                        }

                        ui.horizontal(|ui| {
                            let replace_btn = ui.button(i18n.btn_confirm_replace.clone());
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if replace_btn.clicked() || enter_pressed {
                                let target = dict_replace_target.lock().unwrap().clone();
                                let new_val = dict_replace_new.lock().unwrap().clone();
                                let is_exact = dict_replace_all.load(Ordering::SeqCst);

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
                                show_dict_replace_dialog.store(false, Ordering::SeqCst);
                            }
                            if ui.button(i18n.btn_cancel.clone()).clicked() {
                                show_dict_replace_dialog.store(false, Ordering::SeqCst);
                            }
                        });
                    });
            }
            // ----------------------------------------------

            // 顯示清空對話框
            if show_dict_clear_confirm.load(Ordering::SeqCst) {
                egui::Window::new(i18n.glossary_clear_title.clone())
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(i18n.glossary_clear_desc.clone());
                        ui.horizontal(|ui| {
                            if ui.button(i18n.btn_confirm_clear.clone()).clicked() {
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
                                show_dict_clear_confirm.store(false, Ordering::SeqCst);
                            }
                            if ui.button(i18n.btn_cancel.clone()).clicked() {
                                show_dict_clear_confirm.store(false, Ordering::SeqCst);
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
            let current_page = dict_page.load(Ordering::SeqCst);
            let mut page_val = if current_page >= total_pages {
                dict_page.store(0, Ordering::SeqCst);
                0
            } else {
                current_page
            };
            
            let start = (page_val * page_size).min(total_items);
            let end = (start + page_size).min(total_items);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(i18n.label_search.clone()).color(label_color).strong());
                let (input_bg, input_text) = get_instance_style_from_snap(&style_snap, "input_dict_search", is_dark);
                ui.visuals_mut().extreme_bg_color = input_bg;
                ui.add(
                    egui::TextEdit::singleline(&mut *dict_search.lock().unwrap())
                        .text_color(input_text)
                        .desired_width(120.0),
                );
                ui.add_space(20.0);
                if ui.button("◀").clicked() {
                    page_val = page_val.saturating_sub(1);
                    dict_page.store(page_val, Ordering::SeqCst);
                }
                ui.label(
                    egui::RichText::new(i18n.glossary_page_info.clone()
                        .replace("{}", &(page_val + 1).to_string())
                        .replacen("{}", &total_pages.to_string(), 1)
                        .replacen("{}", &(if total_items > 0 { start + 1 } else { 0 }).to_string(), 1)
                        .replacen("{}", &end.to_string(), 1)
                        .replacen("{}", &total_items.to_string(), 1)
                    )
                    .color(label_color)
                    .strong(),
                );
                if ui.button("▶").clicked() && (page_val + 1) < total_pages {
                    page_val += 1;
                    dict_page.store(page_val, Ordering::SeqCst);
                }

                // 優先級開關移到右側 (label 在左邊，toggle 在右邊)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut is_user_priority = glossary_priority.lock().unwrap().as_str() == "user";
                    if ui
                        .add(toggle(&mut is_user_priority))
                        .on_hover_text(i18n.glossary_priority_hover.clone())
                        .clicked()
                    {
                        *glossary_priority.lock().unwrap() = if is_user_priority {
                            "user".to_string()
                        } else {
                            "official".to_string()
                        };
                        viewer_shared.update_tx.send(crate::state::viewer_state::ViewerUpdate::SaveConfig).ok();
                    }
                    let priority_label = if is_user_priority {
                        i18n.glossary_priority_user.clone()
                    } else {
                        i18n.glossary_priority_official.clone()
                    };
                    ui.label(
                        egui::RichText::new(priority_label)
                            .color(label_color)
                            .strong(),
                    );
                });
            });

            ui.separator();
            let (area_bg, _) = get_instance_style_from_snap(&style_snap, "area_dict_list", is_dark);
            egui::ScrollArea::vertical().id_source("memory_viewer_dict_scroll")
                .hscroll(false) // 禁用水平捲動防止無限放大 (Revision 14.2)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // 創造條紋對比色：深色加亮、淺色變暗 (以防與 area_bg 融為一體)
                    let stripe_bg = if is_dark {
                        egui::Color32::from_rgb(
                            area_bg.r().saturating_add(10),
                            area_bg.g().saturating_add(10),
                            area_bg.b().saturating_add(15),
                        )
                    } else {
                        egui::Color32::from_rgb(
                            area_bg.r().saturating_sub(10),
                            area_bg.g().saturating_sub(10),
                            area_bg.b().saturating_sub(12),
                        )
                    };
                    ui.visuals_mut().faint_bg_color = stripe_bg;
                    egui::Frame::none().fill(area_bg).show(ui, |ui| {
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
                                            egui::RichText::new(i18n.glossary_col_actions.clone()).color(label_color).strong(),
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
                                            egui::RichText::new(i18n.glossary_empty.clone())
                                                .color(label_color)
                                                .strong(),
                                        );
                                    });
                                });
                                ui.label("");
                                ui.end_row();
                            }

                            let start = page_val * page_size;
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
                                            let save_btn = ui.button(i18n.btn_save.clone());
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
                                                        egui::Button::new(i18n.btn_delete.clone()),
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
                                                        egui::Button::new(i18n.btn_edit.clone()),
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
        });
    }
}

/// 從 StyleSnapshot 獲取精確樣式 (Revision 15.18)
fn get_instance_style_from_snap(snap: &crate::state::viewer_state::StyleSnapshot, id: &str, is_dark: bool) -> (egui::Color32, egui::Color32) {
    let (bg, text, _) = get_instance_style_from_snap_full(snap, id, is_dark);
    (bg, text)
}

/// 獲取完整樣式 (含圓角) (Revision 15.30+)
fn get_instance_style_from_snap_full(
    snap: &crate::state::viewer_state::StyleSnapshot,
    id: &str,
    is_dark: bool,
) -> (egui::Color32, egui::Color32, f32) {

    let default_rounding = snap.rounding;

    if let Some(style) = snap.instance_overrides.get(id) {
        let bg = style.bg.map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]));
        let text = style.text.map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]));
        let rounding = style.rounding.unwrap_or(default_rounding);

        if bg.is_some() || text.is_some() || style.rounding.is_some() {
            let final_bg = bg.unwrap_or_else(|| {
                if id.contains("btn") || id.contains("nav") {
                    if is_dark {
                        egui::Color32::from_rgb(snap.dark_btn_bg[0], snap.dark_btn_bg[1], snap.dark_btn_bg[2])
                    } else {
                        egui::Color32::from_rgb(snap.light_btn_bg[0], snap.light_btn_bg[1], snap.light_btn_bg[2])
                    }
                } else if id.contains("list") || id.contains("area") {
                    if is_dark { egui::Color32::from_rgb(snap.dark_list_bg[0], snap.dark_list_bg[1], snap.dark_list_bg[2]) } else { egui::Color32::from_rgb(snap.light_list_bg[0], snap.light_list_bg[1], snap.light_list_bg[2]) }
                } else {
                    if is_dark {
                        egui::Color32::from_rgb(snap.dark_bg[0], snap.dark_bg[1], snap.dark_bg[2])
                    } else {
                        egui::Color32::from_rgb(snap.light_bg[0], snap.light_bg[1], snap.light_bg[2])
                    }
                }
            });
            let final_text = text.unwrap_or_else(|| {
                if id.contains("btn") || id.contains("nav") {
                    if is_dark {
                        egui::Color32::from_rgb(snap.dark_btn_text[0], snap.dark_btn_text[1], snap.dark_btn_text[2])
                    } else {
                        egui::Color32::from_rgb(snap.light_btn_text[0], snap.light_btn_text[1], snap.light_btn_text[2])
                    }
                } else {
                    if is_dark {
                        egui::Color32::from_rgb(snap.dark_text[0], snap.dark_text[1], snap.dark_text[2])
                    } else {
                        egui::Color32::from_rgb(snap.light_text[0], snap.light_text[1], snap.light_text[2])
                    }
                }
            });
            return (final_bg, final_text, rounding);
        }
    }
    
    // 預設解析
    let (rgb_bg, rgb_text) = if id.contains("input") {
        (if is_dark { snap.dark_input_bg } else { snap.light_input_bg }, if is_dark { snap.dark_text } else { snap.light_text })
    } else if id.contains("list") || id.contains("area") {
        (if is_dark { snap.dark_list_bg } else { snap.light_list_bg }, if is_dark { snap.dark_text } else { snap.light_text })
    } else {
        (if is_dark { snap.dark_bg } else { snap.light_bg }, if is_dark { snap.dark_text } else { snap.light_text })
    };
    
    (egui::Color32::from_rgb(rgb_bg[0], rgb_bg[1], rgb_bg[2]),
     egui::Color32::from_rgb(rgb_text[0], rgb_text[1], rgb_text[2]),
     default_rounding)
}
