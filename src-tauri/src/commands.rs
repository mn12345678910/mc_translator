use mc_translator_rs::config::{AppConfig, settings::StyleConfig};
use mc_translator_rs::config::dictionary::{get_user_dict_path, get_official_dict_path, load_dict, save_dict};
use std::collections::HashMap;
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
    config: AppConfig,
) -> Result<(), String> {

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

#[tauri::command]
pub fn query_dictionary(
    dict_type: String,
    page: usize,
    page_size: usize,
    search_key: String,
) -> Result<(Vec<(String, String)>, usize), String> {
    let config = AppConfig::load();
    let lang = config.ui_lang.as_str();

    let path = if dict_type == "user" {
        get_user_dict_path(lang)
    } else {
        get_official_dict_path(lang)
    };

    let full_dict: HashMap<String, String> = load_dict(&path);
    let mut items: Vec<(String, String)> = full_dict.into_iter().collect();

    // 1. 字典序排列穩定化
    items.sort_by(|a, b| a.0.cmp(&b.0));

    // 2. 關鍵字過濾
    if !search_key.is_empty() {
        items.retain(|(k, v)| k.contains(&search_key) || v.contains(&search_key));
    }

    // 3. 分頁切片
    let total_count = items.len();
    let total_pages = total_count.div_ceil(page_size);
    
    let start = page * page_size;
    if start >= total_count {
        return Ok((vec![], total_pages));
    }
    let end = (start + page_size).min(total_count);
    
    Ok((items[start..end].to_vec(), total_pages))
}

#[tauri::command]
pub fn edit_dictionary_item(key: String, value: String, delete: bool) -> Result<(), String> {
    let config = AppConfig::load();
    let path = get_user_dict_path(&config.ui_lang);

    let mut dict: HashMap<String, String> = load_dict(&path);

    if delete {
        dict.remove(&key);
    } else {
        dict.insert(key, value);
    }

    save_dict(&path, &dict);
    Ok(())
}

#[tauri::command]
pub fn pause_translation() -> Result<(), String> {
    if let Ok(active) = mc_translator_rs::translation::ACTIVE_JOB.lock() {
        if let Some(job) = active.as_ref() {
            job.paused.store(true, std::sync::atomic::Ordering::SeqCst);
            return Ok(());
        }
    }
    Err("無正在執行的翻譯任務".to_string())
}

#[tauri::command]
pub fn resume_translation() -> Result<(), String> {
    if let Ok(active) = mc_translator_rs::translation::ACTIVE_JOB.lock() {
        if let Some(job) = active.as_ref() {
            job.paused.store(false, std::sync::atomic::Ordering::SeqCst);
            job.pause_notifier.notify_one();
            return Ok(());
        }
    }
    Err("無正在執行的翻譯任務".to_string())
}

#[tauri::command]
pub fn stop_translation() -> Result<(), String> {
    if let Ok(active) = mc_translator_rs::translation::ACTIVE_JOB.lock() {
        if let Some(job) = active.as_ref() {
            job.cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
            job.pause_notifier.notify_one();
            return Ok(());
        }
    }
    Err("無正在執行的翻譯任務".to_string())
}

#[tauri::command]
pub fn open_path_dialog(diag_type: String) -> Result<Option<String>, String> {
    let builder = rfd::FileDialog::new();
    let path = if diag_type == "dir" {
        builder.pick_folder()
    } else {
        builder.pick_file()
    };

    Ok(path.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn get_available_langs() -> Result<Vec<String>, String> {
    let dir = std::path::Path::new("langs");
    if !dir.exists() {
        return Ok(vec!["zh_tw".to_string(), "en_us".to_string()]);
    }

    let mut langs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    langs.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    langs.sort();
    Ok(langs)
}
