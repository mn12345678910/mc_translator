use crate::state::app_state::AppState;

impl AppState {
    const VIEWER_OPENED_FRAMES_MAX: u32 = 150;
    const VIEWER_GUIDANCE_REPAINT_FRAMES: u32 = 65;
    const VIEWER_VISIBLE_AFTER_FRAMES: u32 = 10;
    const VIEWER_FORCE_GEOMETRY_FRAMES: u32 = 20;

    /// 若有需要則顯示建議詞管理器 Viewport
    pub fn show_viewport_if_needed(&mut self, ctx: &egui::Context) {
        if !self.show_memory_viewer || self.viewer_opening_counter > 0 {
            return;
        }

        // 1. 在主執行緒更新幀數計數器，用於隱形展現與幾何引導
        let count_val = {
            let mut count = self.viewer_shared.opened_frames.lock().unwrap();
            if *count < Self::VIEWER_OPENED_FRAMES_MAX {
                *count += 1;
            }
            *count
        };

        let opened = self.viewer_shared.opened_last_frame.lock().unwrap();
        if !*opened {
            self.is_memory_viewer_open
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        drop(opened);

        // 每個 frame 都必須呼叫 show_viewport_deferred，才能保持 Viewport 持續顯示
        self.create_viewport_deferred(ctx, count_val);

        // 開啟初期持續重繪，避免計數器卡住而導致死鎖
        if count_val < Self::VIEWER_GUIDANCE_REPAINT_FRAMES {
            ctx.request_repaint();
        }
    }

    /// 創建建議詞管理器 Viewport
    pub fn create_viewport_deferred(&mut self, ctx: &egui::Context, opened_frames: u32) {
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
            let i18n = self.i18n.clone();

            let mut opened_lock = self.viewer_shared.opened_last_frame.lock().unwrap();
            let opened_last_frame = *opened_lock;

            // 1. 先隱後顯：初始設為 invisible
            // 2. 幾何引導：前 10 幀持續強制套用座標與尺寸，壓制 OS 跳位
            let is_visible = opened_frames >= Self::VIEWER_VISIBLE_AFTER_FRAMES; // 使用者要求調至 10 幀
            let mut builder = egui::ViewportBuilder::default()
                .with_title(i18n.glossary_title.clone())
                .with_resizable(true)
                .with_maximized(false)
                .with_min_inner_size([800.0, 600.0]) // 最小尺寸限制
                .with_visible(is_visible);

            // 只有在穩定前（前 20 幀，涵蓋 10 幀亮顯期）持續強制套位，壓制 OS 隨機跳位
            if opened_frames < Self::VIEWER_FORCE_GEOMETRY_FRAMES {
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
                    // 1. 深度樣式注入：從快照重建樣式並套用到 Viewport Context
                    let is_dark = *viewer_shared.theme.read().unwrap() == "dark";
                    let style_snap = viewer_shared.style.read().unwrap();
                    
                    let mut style = (*ctx.style()).clone();
                    let mut visuals = if is_dark { egui::Visuals::dark() } else { egui::Visuals::light() };
                    
                    let bg = if is_dark { style_snap.dark_bg } else { style_snap.light_bg };
                    let text = if is_dark { style_snap.dark_text } else { style_snap.light_text };
                    let btn_bg = if is_dark { style_snap.dark_btn_bg } else { style_snap.light_btn_bg };
                    let btn_text = if is_dark { style_snap.dark_btn_text } else { style_snap.light_btn_text };
                    let input_bg = if is_dark { style_snap.dark_input_bg } else { style_snap.light_input_bg };
                    let list_bg = if is_dark { style_snap.dark_list_bg } else { style_snap.light_list_bg };
                    
                    let bg_color = egui::Color32::from_rgb(bg[0], bg[1], bg[2]);
                    let text_color = egui::Color32::from_rgb(text[0], text[1], text[2]);
                    let btn_bg_color = egui::Color32::from_rgb(btn_bg[0], btn_bg[1], btn_bg[2]);
                    let btn_text_color = egui::Color32::from_rgb(btn_text[0], btn_text[1], btn_text[2]);
                    
                    visuals.window_fill = bg_color;
                    visuals.panel_fill = bg_color;
                    visuals.override_text_color = Some(text_color);
                    
                    visuals.widgets.inactive.bg_fill = btn_bg_color;
                    visuals.widgets.inactive.weak_bg_fill = btn_bg_color;
                    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, btn_text_color);
                    visuals.widgets.hovered.bg_fill = btn_bg_color.linear_multiply(if is_dark { 1.1 } else { 0.95 });
                    
                    let r = style_snap.rounding.into();
                    visuals.widgets.inactive.rounding = r;
                    visuals.widgets.hovered.rounding = r;
                    visuals.widgets.active.rounding = r;
                    visuals.window_rounding = r;
                    
                    visuals.extreme_bg_color = egui::Color32::from_rgb(input_bg[0], input_bg[1], input_bg[2]);
                    visuals.faint_bg_color = egui::Color32::from_rgb(list_bg[0], list_bg[1], list_bg[2]);
                    
                    style.visuals = visuals;
                    ctx.set_style(style);

                    // 監聽關閉事件
                    if ctx.input(|i| i.viewport().close_requested()) {
                        is_memory_viewer_open.store(false, std::sync::atomic::Ordering::SeqCst);
                        viewer_shared
                            .close_requested
                            .store(true, std::sync::atomic::Ordering::SeqCst);

                        // 當視窗關閉時觸發一次設定檔存盤
                        viewer_shared.update_tx
                            .send(crate::state::viewer_state::ViewerUpdate::SaveConfig).ok();
                    }

                    // 2. 視窗同步：回報當前位置與大小
                    let inner_size = ctx.screen_rect().size();
                    if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                        let pos = outer_rect.min;
                        let current_count = *viewer_shared.opened_frames.lock().unwrap();

                        // (顯現指令已移至主執行緒 create_viewport_deferred 以防死鎖)

                        if (pos.x > 1.1 || pos.y > 1.1) && current_count > 20 {
                            // 幾何同步保護：加入差值檢查，並限制合理的同步上限
                            let clamped_width = inner_size.x.clamp(400.0, 1920.0);
                            let clamped_height = inner_size.y.clamp(300.0, 1080.0);
                            let clamped_size = egui::vec2(clamped_width, clamped_height);

                            let mut save_needed = false;
                            if let Ok(mut p_lock) = viewer_shared.position.write() {
                                let changed = p_lock
                                    .map(|old| (old.x - pos.x).abs() + (old.y - pos.y).abs() > 5.0)
                                    .unwrap_or(true);
                                if changed {
                                    *p_lock = Some(pos);
                                    save_needed = true;
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
                                    save_needed = true;
                                }
                            }

                            if save_needed {
                                viewer_shared.update_tx
                                    .send(crate::state::viewer_state::ViewerUpdate::SaveConfig).ok();
                            }
                        }
                    }

                    // 3. 渲染內容
                    Self::render_memory_viewer_content(
                        ctx,
                        &i18n,
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
}
