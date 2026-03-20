use mc_translator_rs::config::{AppConfig, settings::StyleConfig};
use mc_translator_rs::config::dictionary::{get_user_dict_path, get_official_dict_path, load_dict, save_dict};
use std::collections::HashMap;
use tauri::Emitter;

#[tauri::command]
pub async fn get_models_from_provider(provider: String) -> Result<Vec<String>, String> {
    let api_key = mc_translator_rs::config::encryption::get_api_key().unwrap_or_default();
    let client = reqwest::Client::new();

    match provider.as_str() {
        "Ollama" => {
            let config = AppConfig::load();
            let url = format!("{}/api/tags", config.ollama_url.trim_end_matches('/'));
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                        let mut names = Vec::new();
                        for m in models {
                            if let Some(n) = m.get("name").and_then(|n| n.as_str()) {
                                names.push(n.to_string());
                            }
                        }
                        if !names.is_empty() { return Ok(names); }
                    }
                }
            }
            Err("err_ollama_connect".to_string())
        }
        "Gemini" => {
            if api_key.is_empty() {
                return Err("err_api_key_empty".to_string());
            }
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key);
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                        let mut names = Vec::new();
                        for m in models {
                            if let Some(n) = m.get("name").and_then(|n| n.as_str()) {
                                if n.contains("gemini") {
                                    names.push(n.trim_start_matches("models/").to_string());
                                }
                            }
                        }
                        if !names.is_empty() { return Ok(names); }
                    }
                }
            }
            Err("err_gemini_models".to_string())
        }
        "OpenAI" => {
            if api_key.is_empty() {
                return Err("err_api_key_empty".to_string());
            }
            if let Ok(resp) = client.get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = json.get("data").and_then(|m| m.as_array()) {
                        let mut names = Vec::new();
                        for m in models {
                            if let Some(n) = m.get("id").and_then(|n| n.as_str()) {
                                if n.starts_with("gpt-") {
                                    names.push(n.to_string());
                                }
                            }
                        }
                        names.sort();
                        if !names.is_empty() { return Ok(names); }
                    }
                }
            }
            Err("err_openai_models".to_string())
        }
        "DeepSeek" => {
            if api_key.is_empty() {
                return Err("err_api_key_empty".to_string());
            }
            if let Ok(resp) = client.get("https://api.deepseek.com/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = json.get("data").and_then(|m| m.as_array()) {
                        let mut names = Vec::new();
                        for m in models {
                            if let Some(n) = m.get("id").and_then(|n| n.as_str()) {
                                names.push(n.to_string());
                            }
                        }
                        if !names.is_empty() { return Ok(names); }
                    }
                }
            }
            Err("err_deepseek_models".to_string())
        }
        _ => Err("err_unsupported_provider".to_string()),
    }
}

#[tauri::command]
pub fn get_i18n_labels(lang: Option<String>) -> mc_translator_rs::ui::i18n::I18nLabels {
    if let Some(l) = lang {
        mc_translator_rs::ui::i18n::I18nLabels::load_or_default(&l)
    } else {
        let config = AppConfig::load();
        mc_translator_rs::ui::i18n::I18nLabels::load_or_default(&config.ui_lang)
    }
}

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
pub fn get_default_config() -> AppConfig {
    AppConfig::default()
}

#[tauri::command]
pub fn get_api_key_cmd() -> String {
    mc_translator_rs::config::encryption::get_api_key().unwrap_or_default()
}

#[tauri::command]
pub fn save_api_key_cmd(key: String) -> Result<(), String> {
    mc_translator_rs::config::encryption::save_api_key(&key).map_err(|e| e.to_string())
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
        let _ = handle_log.emit("translation-log", msg.to_string());
    };

    // 定義進度 Callback
    let handle_progress = handle.clone();
    let progress_updater = move |current: f32, total: f32, batch_curr: f32, batch_tot: f32, status: &str| {
        let payload = serde_json::json!({
            "current": current as u32,
            "total": total as u32,
            "batch_current": batch_curr as u32,
            "batch_total": batch_tot as u32,
            "status": status.to_string()
        });
        let _ = handle_progress.emit("translation-progress", payload);
    };

    // 啟動工作流
    let res = mc_translator_rs::translation::pipeline::start_translation_workflow(
        config,
        paths,
        logger,
        progress_updater,
    )
    .await;

    // 發送完成與狀態更新
    let status_msg = match &res {
        Ok(_) => "任務完成 (Finished)",
        Err(e) => return Err(e.to_string()),
    };
    let _ = handle.emit("translation-status", status_msg.to_string());

    Ok(())
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
pub fn get_default_style_config() -> StyleConfig {
    StyleConfig::default()
}

// Trigger recompile to pick up new include_str! JSON files
use std::sync::{Mutex, OnceLock};

struct DictCache {
    path: String,
    items: Vec<(String, String)>,
}

fn dict_cache() -> &'static Mutex<Option<DictCache>> {
    static CACHE: OnceLock<Mutex<Option<DictCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
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

    let path_str = path.to_string_lossy().to_string();

    let mut cache = dict_cache().lock().unwrap();
    let items = if let Some(c) = cache.as_ref() {
        if c.path == path_str {
            c.items.clone()
        } else {
            let full_dict: HashMap<String, String> = load_dict(&path);
            let mut it: Vec<_> = full_dict.into_iter().collect();
            it.sort_by(|a, b| a.0.cmp(&b.0));
            *cache = Some(DictCache { path: path_str.clone(), items: it.clone() });
            it
        }
    } else {
        let full_dict: HashMap<String, String> = load_dict(&path);
        let mut it: Vec<_> = full_dict.into_iter().collect();
        it.sort_by(|a, b| a.0.cmp(&b.0));
        *cache = Some(DictCache { path: path_str.clone(), items: it.clone() });
        it
    };

    let mut filtered_items = items;

    // 2. 關鍵字過濾
    if !search_key.is_empty() {
        filtered_items.retain(|(k, v)| k.contains(&search_key) || v.contains(&search_key));
    }

    // 3. 分頁切片
    let total_count = filtered_items.len();
    let total_pages = total_count.div_ceil(page_size);
    
    let start = page * page_size;
    if start >= total_count {
        return Ok((vec![], total_pages));
    }
    let end = (start + page_size).min(total_count);
    
    Ok((filtered_items[start..end].to_vec(), total_pages))
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

    // 🔧 清除快取，迫使下次查詢重新載入
    if let Ok(mut cache) = dict_cache().lock() {
        *cache = None;
    }

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
    Err("err_no_active_job".to_string())
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
    Err("err_no_active_job".to_string())
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
    Err("err_no_active_job".to_string())
}

#[tauri::command]
pub fn open_path_dialog(diag_type: String) -> Result<Option<String>, String> {
    let builder = rfd::FileDialog::new();
    let path = if diag_type == "dir" {
        builder.pick_folder()
    } else if diag_type == "save_file" {
        builder.save_file()
    } else {
        builder.pick_file()
    };

    Ok(path.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    // 即使路徑不存在也嘗試開啟上一層或直接讓系統報錯
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(p)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let cmd = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
        std::process::Command::new(cmd)
            .arg(p)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
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

#[tauri::command]
pub fn clear_user_dictionary() -> Result<(), String> {
    use mc_translator_rs::config::settings::AppConfig;
    use std::collections::HashMap;

    let config = AppConfig::load();
    let path = get_user_dict_path(&config.ui_lang);
    save_dict(&path, &HashMap::<String, String>::new());
    if let Ok(mut cache) = dict_cache().lock() {
        *cache = None;
    }
    Ok(())
}

#[tauri::command]
pub fn import_user_dictionary(file_path: String) -> Result<(), String> {
    use mc_translator_rs::config::settings::AppConfig;
    use std::collections::HashMap;

    let config = AppConfig::load();
    let user_path = get_user_dict_path(&config.ui_lang);
    let mut current_dict: HashMap<String, String> = load_dict(&user_path);

    let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let imported: HashMap<String, String> = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    for (k, v) in imported {
        current_dict.insert(k, v);
    }
    save_dict(&user_path, &current_dict);
    if let Ok(mut cache) = dict_cache().lock() {
        *cache = None;
    }
    Ok(())
}

#[tauri::command]
pub fn export_user_dictionary(file_path: String) -> Result<(), String> {
    use mc_translator_rs::config::settings::AppConfig;

    let config = AppConfig::load();
    let path = get_user_dict_path(&config.ui_lang);
    let dict: std::collections::HashMap<String, String> = load_dict(&path);

    let json = serde_json::to_string_pretty(&dict).map_err(|e| e.to_string())?;
    std::fs::write(file_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn show_window(window: tauri::Window) {
    let _ = window.show();
}
