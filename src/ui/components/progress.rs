use crate::state::app_state::AppState;
use crate::ui::constants::{LABEL_COLOR_DARK, LABEL_COLOR_LIGHT};

impl AppState {
    pub fn render_progress_section(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        processing: bool,
    ) {
        ui.separator();
        ui.add_space(1.0);

        let label_color = if self.theme == "light" {
            LABEL_COLOR_LIGHT
        } else {
            LABEL_COLOR_DARK
        };

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
            egui::RichText::new(format!("目前狀態: {}", current_status))
                .color(label_color)
                .strong(),
        );

        // 如果不在處理中，清空進度顯示 (除非是在暫停中)
        let (prog, total, g_prog, g_total) = {
            (
                *self.progress.lock().unwrap(),
                *self.progress_total.lock().unwrap(),
                *self.global_progress.lock().unwrap(),
                *self.global_total.lock().unwrap(),
            )
        };

        // 目前檔案 (顯示條目進度)
        let ratio = if total > 0.0 { prog / total } else { 0.0 };
        ui.add(egui::ProgressBar::new(ratio).show_percentage().text(
            egui::RichText::new(format!("目前檔案: ({}/{})", prog as u32, total as u32)).strong(),
        ));

        // 總進度 (顯示檔案進度)
        let g_ratio = if g_total > 0.0 { g_prog / g_total } else { 0.0 };
        ui.add(
            egui::ProgressBar::new(g_ratio).text(
                egui::RichText::new(format!(
                    "總進度: ({}/{}) {}%",
                    g_prog as u32,
                    g_total as u32,
                    (g_ratio * 100.0) as u32
                ))
                .strong(),
            ),
        );
        ui.add_space(1.0);
    }
}
