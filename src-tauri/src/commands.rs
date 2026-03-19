use mc_translator_rs::config::{AppConfig, settings::StyleConfig};
use tauri::Emitter;

#[tauri::command]
pub fn get_config() -> AppConfig {
    AppConfig::load()
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    config.save();
    Ok(())
}

#[tauri::command]
pub async fn start_translation(
    handle: tauri::AppHandle,
    input_paths: Vec<String>,
) -> Result<(), String> {
    let config = AppConfig::load();

    // 轉換路徑為 Pipeline 所需格式 (PathBuf, 顯示檔名)
    let paths: Vec<(std::path::PathBuf, String)> = input_paths
        .into_iter()
        .map(|p| {
            let path = std::path::PathBuf::from(&p);
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            (path, filename)
        })
        .collect();

    // 定義日誌 Callback
    let handle_log = handle.clone();
    let logger = move |msg: &str| {
        let _ = handle_log.emit("log_event", msg.to_string());
    };

    // 定義進度 Callback
    let handle_progress = handle.clone();
    let progress_updater = move |ratio: f32, status: &str| {
        let _ = handle_progress.emit("progress_event", (ratio, status.to_string()));
    };

    // 啟動工作流
    mc_translator_rs::translation::pipeline::start_translation_workflow(
        config,
        paths,
        logger,
        progress_updater,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_style_config() -> StyleConfig {
    StyleConfig::load()
}

#[tauri::command]
pub fn save_style_config(config: StyleConfig) -> Result<(), String> {
    config.save();
    Ok(())
}
