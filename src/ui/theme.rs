use crate::state::app_state::AppState;

impl AppState {
    /// 渲染並套用主題與視覺風格 (還原自備份版本，含視覺統一優化)
    pub fn render_theme_application(&mut self, ctx: &egui::Context) {
        let is_dark = self.theme == "dark";
        let current_is_dark = ctx.style().visuals.dark_mode;

        // 簡化更新偵測，總是套用以確保自定義變更即時反應
        let visuals = if is_dark {
            let mut v = egui::Visuals::dark();
            let bg = egui::Color32::from_rgb(self.dark_bg[0], self.dark_bg[1], self.dark_bg[2]);
            let text = egui::Color32::from_rgb(self.dark_text[0], self.dark_text[1], self.dark_text[2]);
            let btn_bg = egui::Color32::from_rgb(self.dark_btn_bg[0], self.dark_btn_bg[1], self.dark_btn_bg[2]);
            let btn_text = egui::Color32::from_rgb(self.dark_btn_text[0], self.dark_btn_text[1], self.dark_btn_text[2]);
            
            v.window_fill = bg;
            v.panel_fill = bg;
            v.override_text_color = Some(text);
            
            // 按鈕設定
            v.widgets.inactive.bg_fill = btn_bg;
            v.widgets.inactive.weak_bg_fill = btn_bg;
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, btn_text);
            
            // 圓角設定
            if self.btn_rounding_enabled {
                let r = self.btn_rounding_value.into();
                v.widgets.inactive.rounding = r;
                v.widgets.hovered.rounding = r;
                v.widgets.active.rounding = r;
            }

            v.extreme_bg_color = egui::Color32::from_rgb(self.dark_input_bg[0], self.dark_input_bg[1], self.dark_input_bg[2]);
            v.faint_bg_color = egui::Color32::from_rgb(self.dark_list_bg[0], self.dark_list_bg[1], self.dark_list_bg[2]);
            v
        } else {
            let mut v = egui::Visuals::light();
            let bg = egui::Color32::from_rgb(self.light_bg[0], self.light_bg[1], self.light_bg[2]);
            let text = egui::Color32::from_rgb(self.light_text[0], self.light_text[1], self.light_text[2]);
            let btn_bg = egui::Color32::from_rgb(self.light_btn_bg[0], self.light_btn_bg[1], self.light_btn_bg[2]);
            let btn_text = egui::Color32::from_rgb(self.light_btn_text[0], self.light_btn_text[1], self.light_btn_text[2]);

            v.window_fill = bg;
            v.panel_fill = bg;
            v.override_text_color = Some(text);

            v.widgets.inactive.bg_fill = btn_bg;
            v.widgets.inactive.weak_bg_fill = btn_bg;
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.2, btn_text);

            if self.btn_rounding_enabled {
                let r = self.btn_rounding_value.into();
                v.widgets.inactive.rounding = r;
                v.widgets.hovered.rounding = r;
                v.widgets.active.rounding = r;
            }

            v.extreme_bg_color = egui::Color32::from_rgb(self.light_input_bg[0], self.light_input_bg[1], self.light_input_bg[2]);
            v.faint_bg_color = egui::Color32::from_rgb(self.light_list_bg[0], self.light_list_bg[1], self.light_list_bg[2]);
            v
        };

        let mut style = (*ctx.style()).clone();
        style.visuals = visuals;
        // 更新字體大小
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(self.font_size));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(self.font_size));
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(self.font_size * 1.5));
        
        ctx.set_style(style);

        // 同步狀態至建議詞管理器
        *self.viewer_shared.theme.write().unwrap() = self.theme.clone();
        *self.viewer_shared.font_size.write().unwrap() = self.font_size;
        ctx.request_repaint_of(egui::ViewportId::from_hash_of("memory_viewer"));
    }

    /// 獲取特定元件的背景與文字顏色 (解析優先級：實例 > 類別 > 全域)
    pub fn get_instance_style(&self, id: &str) -> (egui::Color32, egui::Color32) {
        let is_dark = self.theme == "dark";
        
        // 1. 優先查詢實例覆寫
        if let Some(style) = self.instance_overrides.get(id) {
            let bg = style.bg.map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]));
            let text = style.text.map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]));
            
            if bg.is_some() || text.is_some() {
                let final_bg = bg.unwrap_or_else(|| {
                    let c = if is_dark { self.dark_bg } else { self.light_bg };
                    egui::Color32::from_rgb(c[0], c[1], c[2])
                });
                let final_text = text.unwrap_or_else(|| {
                    let c = if is_dark { self.dark_text } else { self.light_text };
                    egui::Color32::from_rgb(c[0], c[1], c[2])
                });
                return (final_bg, final_text);
            }
        }

        // 2. 類別樣式
        let (rgb_bg, rgb_text) = if id.contains("btn") {
            (if is_dark { self.dark_btn_bg } else { self.light_btn_bg }, 
             if is_dark { self.dark_btn_text } else { self.light_btn_text })
        } else if id.contains("input") {
            (if is_dark { self.dark_input_bg } else { self.light_input_bg },
             if is_dark { self.dark_text } else { self.light_text })
        } else {
            (if is_dark { self.dark_bg } else { self.light_bg }, 
             if is_dark { self.dark_text } else { self.light_text })
        };

        (egui::Color32::from_rgb(rgb_bg[0], rgb_bg[1], rgb_bg[2]),
         egui::Color32::from_rgb(rgb_text[0], rgb_text[1], rgb_text[2]))
    }
}
