use mc_translator::config::dictionary::{
    get_official_dict_path, get_user_dict_path, load_dict, save_dict,
};
use mc_translator::config::{settings::StyleConfig, AppConfig};
use mc_translator::i18n::CommonLabels;
use mc_translator::translation::job::JobStatus;
use mc_translator::translation::{LogEntry, LogLevel};
use mc_translator::utils::helpers::add_log_event;
use std::collections::HashMap;
use tauri::{Emitter, Manager};

#[tauri::command]
pub async fn get_models_from_provider(provider: String) -> Result<Vec<String>, String> {
    let api_key = mc_translator::config::encryption::get_api_key().unwrap_or_default();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

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
                        if !names.is_empty() {
                            return Ok(names);
                        }
                    }
                }
            }
            Err("err_ollama_connect".to_string())
        }
        "Gemini" => {
            if api_key.is_empty() {
                return Err("err_api_key_empty".to_string());
            }
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
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
                        if !names.is_empty() {
                            return Ok(names);
                        }
                    }
                }
            }
            Err("err_gemini_models".to_string())
        }
        "OpenAI" => {
            if api_key.is_empty() {
                return Err("err_api_key_empty".to_string());
            }
            if let Ok(resp) = client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
            {
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
                        if !names.is_empty() {
                            return Ok(names);
                        }
                    }
                }
            }
            Err("err_openai_models".to_string())
        }
        "DeepSeek" => {
            if api_key.is_empty() {
                return Err("err_api_key_empty".to_string());
            }
            if let Ok(resp) = client
                .get("https://api.deepseek.com/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = json.get("data").and_then(|m| m.as_array()) {
                        let mut names = Vec::new();
                        for m in models {
                            if let Some(n) = m.get("id").and_then(|n| n.as_str()) {
                                names.push(n.to_string());
                            }
                        }
                        if !names.is_empty() {
                            return Ok(names);
                        }
                    }
                }
            }
            Err("err_deepseek_models".to_string())
        }
        _ => Err("err_unsupported_provider".to_string()),
    }
}

#[tauri::command]
pub fn get_i18n_labels(lang: Option<String>) -> mc_translator::i18n::GuiLabels {
    if let Some(l) = lang {
        mc_translator::i18n::GuiLabels::load_or_default(&l)
    } else {
        let config = AppConfig::load();
        mc_translator::i18n::GuiLabels::load_or_default(&config.ui_lang)
    }
}

#[tauri::command]
pub fn get_config() -> AppConfig {
    AppConfig::load()
}

#[tauri::command]
pub fn save_config(mut config: AppConfig) -> Result<(), String> {
    config.save();
    Ok(())
}

#[tauri::command]
pub fn get_default_config() -> AppConfig {
    AppConfig::default()
}

#[tauri::command]
pub fn get_api_key_cmd() -> String {
    mc_translator::config::encryption::get_api_key().unwrap_or_default()
}

#[tauri::command]
pub fn save_api_key_cmd(key: String) -> Result<(), String> {
    mc_translator::config::encryption::save_api_key(&key).map_err(|e| e.to_string())
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
    let logger = move |entry: LogEntry| {
        let _ = handle_log.emit("translation-log", entry);
    };

    // 定義進度 Callback
    let handle_progress = handle.clone();
    let progress_updater =
        move |current: f32, total: f32, batch_curr: f32, batch_tot: f32, status: &str| {
            let payload = serde_json::json!({
                "current": current as u32,
                "total": total as u32,
                "batch_current": batch_curr as u32,
                "batch_total": batch_tot as u32,
                "status": status.to_string(),
                "msg": status.to_string() // 👈 支援前端 msg 解讀
            });
            let _ = handle_progress.emit("translation-progress", payload);

            // 補發 translation-batch-update
            let batch_payload = serde_json::json!({
                "batch_index": batch_curr as u32,
                "total_batches": batch_tot as u32,
            });
            let _ = handle_progress.emit("translation-batch-update", batch_payload);
        };

    // 📢 發送初始運行狀態
    let _ = handle.emit("job-state-changed", JobStatus::Running);

    // 啟動工作流
    let res = mc_translator::translation::pipeline::start_translation_workflow(
        config,
        paths,
        logger,
        progress_updater,
    )
    .await;

    // 📢 結束後歸位為 Idle
    let _ = handle.emit("job-state-changed", JobStatus::Idle);

    let success = res.is_ok();
    let status_key = if success {
        "status_finished"
    } else {
        "status_failed_or_cancelled"
    };

    // 發送完成與狀態更新
    let _ = handle.emit("translation-status", status_key.to_string());

    // 補發 translation-finished 連通前端
    let finish_payload = serde_json::json!({
        "success": success,
        "msg": if success { "翻譯已完成" } else { "翻譯發生錯誤" }
    });
    let _ = handle.emit("translation-finished", finish_payload);

    if !success {
        if let Some(e_val) = res.as_ref().err() {
            let err_msg = e_val.to_string();
            // 嚴重錯誤記錄至日誌區
            if let Ok(active) = mc_translator::translation::ACTIVE_JOB.lock() {
                if let Some(job) = active.as_ref() {
                    let cfg_shared = job.config.lock().unwrap();
                    add_log_event(
                        &job.log,
                        LogLevel::Error,
                        &job.i18n.status_trans_error.replace("{}", &err_msg),
                        "",
                        "",
                        "",
                        cfg_shared.enable_debug_log,
                    );

                    // 同步發送至前端
                    let entry = LogEntry {
                        level: LogLevel::Error,
                        message: job.i18n.status_trans_error.replace("{}", &err_msg),
                        timestamp: chrono::Local::now().timestamp_millis(),
                    };
                    let _ = handle.emit("translation-log", entry);
                }
            }
        }
    }

    res.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_style_config() -> StyleConfig {
    StyleConfig::load()
}

#[tauri::command]
pub fn save_style_config(mut config: StyleConfig) -> Result<(), String> {
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
            *cache = Some(DictCache {
                path: path_str.clone(),
                items: it.clone(),
            });
            it
        }
    } else {
        let full_dict: HashMap<String, String> = load_dict(&path);
        let mut it: Vec<_> = full_dict.into_iter().collect();
        it.sort_by(|a, b| a.0.cmp(&b.0));
        *cache = Some(DictCache {
            path: path_str.clone(),
            items: it.clone(),
        });
        it
    };

    let mut filtered_items = items;

    // 2. 關鍵字過濾
    if !search_key.is_empty() {
        filtered_items.retain(|(k, v)| k.contains(&search_key) || v.contains(&search_key));
    }

    // 3. 分頁切片
    let total_count = filtered_items.len();
    let safe_page_size = if page_size == 0 { 10 } else { page_size };
    let total_pages = total_count.div_ceil(safe_page_size);

    let start = page * safe_page_size;
    if start >= total_count {
        return Ok((vec![], total_pages));
    }
    let end = (start + safe_page_size).min(total_count);

    Ok((filtered_items[start..end].to_vec(), total_pages))
}

#[tauri::command]
pub fn edit_dictionary_item(
    app: tauri::AppHandle,
    key: String,
    value: String,
    delete: bool,
) -> Result<(), String> {
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

    // 📢 發送全域通知同步各視窗
    let _ = app.emit("dictionary-changed", ());

    Ok(())
}

#[tauri::command]
pub fn pause_translation(handle: tauri::AppHandle) -> Result<(), String> {
    if let Ok(active) = mc_translator::translation::ACTIVE_JOB.lock() {
        if let Some(job) = active.as_ref() {
            job.paused.store(true, std::sync::atomic::Ordering::SeqCst);

            // 發送暫停日誌
            let config_shared = job.config.lock().unwrap();
            add_log_event(
                &job.log,
                LogLevel::Warn,
                &job.i18n.log_pause_requested,
                "",
                "",
                "",
                config_shared.enable_debug_log,
            );

            // 更新狀態並通知前端
            if let Ok(mut status) = job.current_state.lock() {
                *status = JobStatus::Paused;
            }
            let _ = handle.emit("job-state-changed", JobStatus::Paused);

            return Ok(());
        }
    }
    Err("err_no_active_job".to_string())
}

#[tauri::command]
pub fn resume_translation(handle: tauri::AppHandle) -> Result<(), String> {
    if let Ok(active) = mc_translator::translation::ACTIVE_JOB.lock() {
        if let Some(job) = active.as_ref() {
            job.paused.store(false, std::sync::atomic::Ordering::SeqCst);
            job.pause_notifier.notify_one();

            // 發送恢復日誌
            let config_shared = job.config.lock().unwrap();
            add_log_event(
                &job.log,
                LogLevel::Info,
                &job.i18n.log_resuming,
                "",
                "",
                "",
                config_shared.enable_debug_log,
            );

            // 更新狀態並通知前端
            if let Ok(mut status) = job.current_state.lock() {
                *status = JobStatus::Running;
            }
            let _ = handle.emit("job-state-changed", JobStatus::Running);

            return Ok(());
        }
    }
    Err("err_no_active_job".to_string())
}

#[tauri::command]
pub fn stop_translation(handle: tauri::AppHandle) -> Result<(), String> {
    if let Ok(active) = mc_translator::translation::ACTIVE_JOB.lock() {
        if let Some(job) = active.as_ref() {
            job.cancelled
                .store(true, std::sync::atomic::Ordering::SeqCst);
            job.pause_notifier.notify_one();

            // 發送中止日誌
            let config_shared = job.config.lock().unwrap();
            add_log_event(
                &job.log,
                LogLevel::Error,
                &job.i18n.log_stopped,
                "",
                "",
                "",
                config_shared.enable_debug_log,
            );

            // 更新狀態並通知前端
            if let Ok(mut status) = job.current_state.lock() {
                *status = JobStatus::Idle;
            }
            let _ = handle.emit("job-state-changed", JobStatus::Idle);

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
    let mut os_path = std::path::PathBuf::from(&path);

    // 如果是預設值或空值，根據開發與生產環境重新定錨（同步 pack_gen 邏輯）
    if path.is_empty() || path == "./LLMTranslator" {
        let cur = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        if cur.ends_with("src-tauri") && std::env::var("CARGO_MANIFEST_DIR").is_ok() {
            os_path = std::path::PathBuf::from("../LLMTranslator");
        } else {
            os_path = std::path::PathBuf::from("LLMTranslator");
        }
    }

    // 如果資料夾不存在，自動產出來
    if !os_path.exists() {
        std::fs::create_dir_all(&os_path).map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(os_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let cmd = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        std::process::Command::new(cmd)
            .arg(os_path)
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
pub fn open_dictionary_location(dict_type: String) -> Result<(), String> {
    use mc_translator::config::settings::AppConfig;
    let config = AppConfig::load();
    let path = if dict_type == "user" {
        get_user_dict_path(&config.ui_lang)
    } else {
        get_official_dict_path(&config.ui_lang)
    };

    // 確保檔案與資料夾存在，使 canonicalize 運作且 explorer 能選取
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, "{}");
    }

    // 轉為絕對路徑且安全處理 Windows UNC 前綴
    let abs_path = std::fs::canonicalize(&path).unwrap_or(path.clone());
    let path_str = abs_path
        .to_string_lossy()
        .to_string()
        .replace("\\\\?\\", "");

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path_str))
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        open_folder(path_str)
    }
}

#[tauri::command]
pub fn clear_user_dictionary(app: tauri::AppHandle) -> Result<(), String> {
    use mc_translator::config::settings::AppConfig;
    use std::collections::HashMap;

    let config = AppConfig::load();
    let path = get_user_dict_path(&config.ui_lang);
    save_dict(&path, &HashMap::<String, String>::new());
    if let Ok(mut cache) = dict_cache().lock() {
        *cache = None;
    }

    // 📢 發送全域通知同步各視窗
    let _ = app.emit("dictionary-changed", ());

    Ok(())
}

#[tauri::command]
pub fn import_user_dictionary(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
    use mc_translator::config::settings::AppConfig;
    use std::collections::HashMap;

    let config = AppConfig::load();
    let user_path = get_user_dict_path(&config.ui_lang);
    let mut current_dict: HashMap<String, String> = load_dict(&user_path);

    let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let imported: HashMap<String, String> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    for (k, v) in imported {
        current_dict.insert(k, v);
    }
    save_dict(&user_path, &current_dict);
    if let Ok(mut cache) = dict_cache().lock() {
        *cache = None;
    }

    // 📢 發送全域通知同步各視窗
    let _ = app.emit("dictionary-changed", ());

    Ok(())
}

#[tauri::command]
pub fn export_user_dictionary(file_path: String) -> Result<(), String> {
    use mc_translator::config::settings::AppConfig;

    let config = AppConfig::load();
    let path = get_user_dict_path(&config.ui_lang);
    let dict: std::collections::HashMap<String, String> = load_dict(&path);

    let json = serde_json::to_string_pretty(&dict).map_err(|e| e.to_string())?;
    std::fs::write(file_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_dict_window(app: tauri::AppHandle) -> Result<(), String> {
    let app_c = app.clone();
    let _ = app.run_on_main_thread(move || {
        let dict_window = tauri::WebviewWindowBuilder::new(
            &app_c,
            "dict_manager",
            tauri::WebviewUrl::App("dict.html".into()),
        )
        .title("建議詞管理器")
        .inner_size(800.0, 600.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .devtools(true)
        .build();

        match dict_window {
            Ok(window) => {
                let _ = window.set_focus();
            }
            Err(_) => {
                if let Some(window) = app_c.get_webview_window("dict_manager") {
                    let _ = window.set_focus();
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn update_active_job_config(config: AppConfig) -> Result<(), String> {
    if let Ok(active) = mc_translator::translation::ACTIVE_JOB.lock() {
        if let Some(job) = active.as_ref() {
            if let Ok(mut job_cfg) = job.config.lock() {
                job_cfg.api_key =
                    mc_translator::config::encryption::get_api_key().unwrap_or_default();
                job_cfg.api_provider = config.api_provider;
                job_cfg.selected_model = config.model;
                job_cfg.ollama_url = config.ollama_url;
                job_cfg.api_base_url = config.api_base_url;
                job_cfg.user_prompt = config.user_prompt;
                job_cfg.system_prompt = config.system_prompt;
                job_cfg.timeout = config.timeout as u64;
                job_cfg.batch_size = config.batch_size;
                job_cfg.batch_max_chars = config.batch_max_chars;
                job_cfg.output_dir = config.output_dir;
                job_cfg.pack_format = config.pack_format;
                job_cfg.glossary_priority = config.glossary_priority;
                job_cfg.skip_json = config.skip_json;
                job_cfg.skip_js = config.skip_js;
                job_cfg.skip_jar = config.skip_jar;
                job_cfg.skip_book = config.skip_book;
                job_cfg.enable_llm_log = config.enable_llm_log;
                job_cfg.source_lang = config.source_lang;

                // 如果目標語言改變，需要同步更新遺留的前綴與包含規則
                if job_cfg.target_lang != config.target_lang {
                    job_cfg.target_lang = config.target_lang;
                    let target_i18n = CommonLabels::load_or_default(&job_cfg.target_lang);
                    job_cfg.cleanup_prefixes = target_i18n.cleanup_prefixes;
                    job_cfg.cleanup_contains = target_i18n.cleanup_contains;

                    // 同步清空翻譯記憶，防止舊語言殘留干擾新語言
                    if let Ok(mut tm) = job.translation_memory.lock() {
                        tm.clear();
                    }
                }

                job_cfg.enable_debug_log = config.enable_debug_log;
                return Ok(());
            }
        }
    }
    Err("err_no_active_job".to_string())
}

#[tauri::command]
pub fn show_window(window: tauri::Window) {
    let _ = window.show();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_config() {
        let config = get_default_config();
        assert_eq!(config.api_provider, "無"); // 預設值
        assert_eq!(config.source_lang, "en_us");
    }

    #[test]
    fn test_get_default_style_config() {
        let style = get_default_style_config();
        assert_eq!(style.font_size, 15.0); // 預設值
        assert!(style.btn_rounding_enabled);
    }

    #[test]
    fn test_get_i18n_labels_none() {
        let labels = get_i18n_labels(None);
        // 預設應該會帶入 get_config().ui_lang，通常是 zh_tw
        assert!(!labels.common.btn_save.is_empty());
    }

    #[test]
    fn test_get_i18n_labels_specific() {
        let labels = get_i18n_labels(Some("en_us".to_string()));
        assert!(!labels.common.btn_save.is_empty());
    }
}
