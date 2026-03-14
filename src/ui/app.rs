use crate::state::app_state::AppState;
use crate::state::viewer_state::ViewerUpdate;
use std::sync::atomic::Ordering;

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
                ViewerUpdate::Style(_) => {}
                ViewerUpdate::SaveConfig => {
                    self.trigger_save();
                }
            }
        }

        // 0.5 視窗同步 (ses_342b): 提前於儲存前同步，且不限視覺狀態 (fix_save_race)
        {
            if let Ok(pos_lock) = self.viewer_shared.position.read() {
                if let Some(pos) = *pos_lock {
                    // 提升閾值至 5.0 以防止 Windows 系統邊框微調產生的飄移 (Drift Fix)
                    if (pos.x - self.viewer_x).abs() > 5.0 || (pos.y - self.viewer_y).abs() > 5.0 {
                        self.viewer_x = pos.x;
                        self.viewer_y = pos.y;
                    }
                }
            }
            if let Ok(size_lock) = self.viewer_shared.inner_size.read() {
                if let Some(size) = *size_lock {
                    if (size.x - self.viewer_width).abs() > 5.0 || (size.y - self.viewer_height).abs() > 5.0 {
                        self.viewer_width = size.x;
                        self.viewer_height = size.y;
                    }
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
            // 重置計數器，確保下次開啟時能再次套用引導座標 (Reset Fix)
            let mut frames = self.viewer_shared.opened_frames.lock().unwrap();
            *frames = 0;
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
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 4.0);
                let processing = self.is_processing.load(Ordering::SeqCst);
                let ui_enabled = !processing || self.is_paused.load(Ordering::SeqCst);

                // 2. 標頭控制項 (檔案/資料夾/輸出路徑)
                self.render_header_controls(ui, ui_enabled);
                ui.add_space(1.0);

                // 3. 設定面板
                self.render_settings_panel(ui, ui_enabled, ctx);

                // 3.5 調色盤面板
                if self.show_palette_settings {
                    ui.add_space(4.0);
                    ui.group(|ui| {
                        self.render_palette_settings(ui);
                    });
                }

                // 4. 開發者模式面板
                self.render_developer_mode_panel(ui);
                ui.add_space(1.0);

                // 5. 進度顯示區域
                self.render_progress_section(ui, ctx, processing);

                // 6. 操作按鈕與停止對話框
                self.render_action_buttons(ui, ctx, processing);

                // 7. 日誌區域
                self.render_log_area(ui);
            });
        // 8. 建議詞管理器 (Viewport)
        if self.show_memory_viewer {
            self.show_viewport_if_needed(ctx);
        }

        // --- 同步主視窗幾幾何至 AppState (Revision 15.17) ---
        // 注意：此處需使用 inner_size (screen_rect) 以對齊 main.rs 的載入邏輯，防止視窗不斷長大 (Size Drift Fix)
        let inner_rect = ctx.screen_rect();
        if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
            let pos = outer_rect.min;
            if (pos.x - self.main_x).abs() > 5.0 || (pos.y - self.main_y).abs() > 5.0 {
                self.main_x = pos.x;
                self.main_y = pos.y;
                self.trigger_save();
            }
            if (inner_rect.width() - self.main_width).abs() > 5.0 || (inner_rect.height() - self.main_height).abs() > 5.0
            {
                self.main_width = inner_rect.width();
                self.main_height = inner_rect.height();
                self.trigger_save();
            }
        }

        // 處理中時持續重繪
        if self.is_processing.load(Ordering::SeqCst) {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.trigger_save();
    }
}
