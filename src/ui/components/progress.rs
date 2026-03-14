use crate::state::app_state::AppState;
use std::sync::atomic::Ordering;
// use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    pub fn render_progress_section(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        processing: bool,
    ) {
        let (_prog_bg, label_color) = self.get_instance_style("label_status");
        ui.separator();
        ui.add_space(1.0);

        let mut current_status = self.status.lock().unwrap().clone();
        if processing {
            let time = ctx.input(|i| i.time);
            let dots = match (time * 2.0) as i32 % 4 {
                1 => ".",
                2 => "..",
                3 => "...",
                _ => "",
            };
            current_status.push_str(dots);
        }
        ui.label(
            egui::RichText::new(format!("{}{}", self.i18n.label_current_status, current_status))
                .color(label_color)
                .strong(),
        );

        // 如果不在處理中，清空進度顯示 (除非是在暫停中)
        let (prog, total, g_prog, g_total) = {
            (
                f32::from_bits(self.progress.load(Ordering::SeqCst)),
                f32::from_bits(self.progress_total.load(Ordering::SeqCst)),
                f32::from_bits(self.global_progress.load(Ordering::SeqCst)),
                f32::from_bits(self.global_total.load(Ordering::SeqCst)),
            )
        };

        // 目前檔案 (顯示條目進度)
        let ratio = if total > 0.0 { prog / total } else { 0.0 };
        let (c_bar_color_raw, c_text_color, c_rounding) = self.get_instance_style_full("progress_current");
        // 脈衝動畫邏輯優化
        let bar_color = if processing && self.progress_pulse_enabled {
            let speed = self.progress_pulse_speed as f64 * 4.0;
            let shimmer_val = ((ctx.input(|i| i.time) * speed).sin() * 0.15 + 1.1) as f32;
            egui::Color32::from_rgb(
                (c_bar_color_raw.r() as f32 * shimmer_val).min(255.0) as u8,
                (c_bar_color_raw.g() as f32 * shimmer_val).min(255.0) as u8,
                (c_bar_color_raw.b() as f32 * shimmer_val).min(255.0) as u8,
            )
        } else {
            c_bar_color_raw
        };

        ui.add(egui::ProgressBar::new(ratio).fill(bar_color).rounding(c_rounding).show_percentage().text(
            egui::RichText::new(format!("{} ({}/{})", self.i18n.label_current_file, prog as u32, total as u32))
                .color(c_text_color)
                .strong(),
        ));

        // 總進度 (顯示檔案進度)
        let g_ratio = if g_total > 0.0 { g_prog / g_total } else { 0.0 };
        let (t_bar_color_raw, t_text_color, t_rounding) = self.get_instance_style_full("progress_total");
        let t_bar_color = if processing && self.progress_pulse_enabled {
            let speed = self.progress_pulse_speed as f64 * 4.0;
            let shimmer_val = ((ctx.input(|i| i.time) * speed).sin() * 0.15 + 1.1) as f32;
            egui::Color32::from_rgb(
                (t_bar_color_raw.r() as f32 * shimmer_val).min(255.0) as u8,
                (t_bar_color_raw.g() as f32 * shimmer_val).min(255.0) as u8,
                (t_bar_color_raw.b() as f32 * shimmer_val).min(255.0) as u8,
            )
        } else {
            t_bar_color_raw
        };

        ui.add(
            egui::ProgressBar::new(g_ratio)
                .fill(t_bar_color)
                .rounding(t_rounding)
                .text(
                egui::RichText::new(format!(
                    "{} ({}/{}) {}%",
                    self.i18n.label_global_progress,
                    g_prog as u32,
                    g_total as u32,
                    (g_ratio * 100.0) as u32
                ))
                .color(t_text_color)
                .strong(),
            ),
        );
        ui.add_space(1.0);
    }
}
