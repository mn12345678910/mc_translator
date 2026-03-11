use crate::state::app_state::AppState;

impl AppState {
    /// 渲染並套用主題與視覺風格 (還原自備份版本，含視覺統一優化)
    pub fn render_theme_application(&mut self, ctx: &egui::Context) {
        let is_dark = self.theme == "dark";
        let current_is_dark = ctx.style().visuals.dark_mode;

        let current_font_size = ctx
            .style()
            .text_styles
            .get(&egui::TextStyle::Body)
            .map(|f| f.size)
            .unwrap_or(0.0);
        let font_size_changed = (current_font_size - self.font_size).abs() > 0.1;

        let needs_update = if is_dark {
            !current_is_dark || font_size_changed
        } else {
            current_is_dark
                || ctx.style().visuals.window_fill != egui::Color32::from_rgb(0xFF, 0xDE, 0xAD)
                || font_size_changed
        };

        if needs_update {
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

            let mut style = (*ctx.style()).clone();
            style.visuals = visuals;
            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::proportional(self.font_size * 0.8),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(self.font_size),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(self.font_size),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(self.font_size * 1.5),
            );
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::monospace(self.font_size),
            );
            ctx.set_style(style);

            // 同步主題至子視窗 (Revision 14.1/14.2)
            *self.viewer_shared.theme.write().unwrap() = self.theme.clone();
            *self.viewer_shared.font_size.write().unwrap() = self.font_size;
            // 立即請求重繪子視窗 (Revision 14.2)
            ctx.request_repaint_of(egui::ViewportId::from_hash_of("memory_viewer"));
        }
    }
}
