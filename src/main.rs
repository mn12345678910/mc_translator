//! # Minecraft 模組翻譯工具
//!
//! 支援 Gemini / OpenAI / Ollama 翻譯引擎，
//! 可輸出為 Minecraft 資源包或直接修改 JAR。

#![windows_subsystem = "windows"]

use mc_translator_rs::config;
use mc_translator_rs::state::app_state::AppState;

/// 程式入口
fn main() -> Result<(), eframe::Error> {
    // 設置全域 Panic 捕捉器，確保在 Windows Subsystem 模式下也能看到錯誤訊息
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        rfd::MessageDialog::new()
            .set_title("程式崩潰 (Panic)")
            .set_description(&msg)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }));

    let config = config::AppConfig::load();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.main_width, config.main_height])
            .with_position([config.main_x, config.main_y])
            .with_min_inner_size([800.0, 600.0]),
        follow_system_theme: true,
        vsync: true,
        ..Default::default()
    };
    eframe::run_native(
        "Minecraft 模組翻譯工具",
        options,
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}
