//! # 共通翻譯驅動管線 (Shared Execution Pipeline)
//!
//! 本模組負責聚合「字典載入、準備環境、建立共用狀態與呼叫檔案管線」的完整工作流。
//! 透過 Callback 機制來更新進度與日誌，解耦 GUI 與 CLI。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::config::AppConfig;
use crate::i18n::CommonLabels;
use crate::translation::job::{JobConfig, JobSharedState};
use crate::translation::{LogEntry, LogLevel};
use crate::utils;
use crate::utils::helpers::add_log_event;

/// 載入並準備字典檔 (McLang 與推論字典)
pub async fn load_and_prepare_dictionaries(
    config: &AppConfig,
    mc_lang_arc: Arc<Mutex<Option<utils::McLangFiles>>>,
    exact_arc: Arc<Mutex<HashMap<String, String>>>,
    inferred_arc: Arc<Mutex<HashMap<String, String>>>,
    term_arc: Arc<Mutex<Vec<(String, String)>>>,
    available_langs_arc: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 呼叫底層讀取
    let (files, exact, unfiltered) =
        utils::load_mc_dicts(&config.source_lang, &config.target_lang).await?;

    {
        let mut exact_map = exact_arc.lock().unwrap();
        *exact_map = exact.clone();
    }

    let inferred = utils::analyze_dictionary(&exact);
    {
        let mut inferred_map = inferred_arc.lock().unwrap();
        *inferred_map = inferred.clone();
    }

    {
        let mut term_map = term_arc.lock().unwrap();
        *term_map = unfiltered;
    }

    // 儲存推論字典
    crate::config::save_dict(
        &crate::config::get_official_dict_path(&config.ui_lang),
        &inferred,
    );

    // 更新可用語言
    let mut langs: Vec<String> = files.langs.keys().cloned().collect();
    langs.sort();
    if let Ok(mut av) = available_langs_arc.lock() {
        *av = langs;
    }

    if let Ok(mut mc) = mc_lang_arc.lock() {
        *mc = Some(files);
    }

    Ok(())
}

/// 啟動背景翻譯工作流
pub async fn start_translation_workflow(
    config: crate::config::AppConfig,
    input_paths: Vec<(PathBuf, String)>,
    logger: impl Fn(LogEntry) + Send + Sync + 'static,
    progress_updater: impl Fn(f32, f32, f32, f32, &str) + Send + Sync + 'static,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let logger_arc = Arc::new(logger);
    let progress_arc = Arc::new(progress_updater);

    // --- 自動資料夾展開 ---
    let mut expanded_input_paths = Vec::new();
    for (path, rel_path) in input_paths {
        if path.is_dir() {
            expanded_input_paths.extend(crate::file::scanner::scan_files_recursive(&path, &path));
        } else {
            expanded_input_paths.push((path, rel_path));
        }
    }

    // 1. 初始化空狀態容器 (Pipeline 需要)
    let i18n = CommonLabels::load_or_default(&config.ui_lang);
    let log_arc = Arc::new(Mutex::new(Vec::new()));
    let status_arc = Arc::new(Mutex::new(i18n.status_analyzing_files.clone()));

    let mc_lang = Arc::new(Mutex::new(None));
    let exact_match_map = Arc::new(Mutex::new(HashMap::new()));
    let inferred_match_map = Arc::new(Mutex::new(HashMap::new()));
    let term_replacements = Arc::new(Mutex::new(Vec::new()));
    let available_langs = Arc::new(Mutex::new(Vec::new()));

    // 早期日誌：使用 add_log_event 以確保進入 debug.log (如果開啟)
    add_log_event(
        &log_arc,
        LogLevel::Info,
        &i18n.status_analyzing_dict,
        "",
        "",
        "",
        config.enable_debug_log,
    );

    // 2. 準備字典檔
    load_and_prepare_dictionaries(
        &config,
        mc_lang.clone(),
        exact_match_map.clone(),
        inferred_match_map.clone(),
        term_replacements.clone(),
        available_langs.clone(),
    )
    .await?;

    add_log_event(
        &log_arc,
        LogLevel::Info,
        &i18n.status_analyzing_files,
        "",
        "",
        "",
        config.enable_debug_log,
    );

    let target_i18n = crate::i18n::CommonLabels::load_or_default(&config.target_lang);

    // 3. 組裝 JobConfig
    let job_config = Arc::new(Mutex::new(JobConfig::new(
        config.api_key.clone(),
        config.api_provider.clone(),
        config.model.clone(),
        config.ollama_url.clone(),
        config.api_base_url.clone(),
        target_i18n.cleanup_prefixes.clone(),
        target_i18n.cleanup_contains.clone(),
        config.user_prompt.clone(),
        config.system_prompt.clone(),
        config.timeout,
        config.batch_size,
        config.batch_max_chars,
        config.output_dir.clone(),
        config.pack_format,
        config.glossary_priority.clone(),
        config.skip_json,
        config.skip_js,
        config.skip_jar,
        config.skip_book,
        config.enable_llm_log,
        config.source_lang.clone(),
        config.target_lang.clone(),
        config.enable_debug_log,
    )));

    // 4. 建立 JobSharedState

    let job_state = JobSharedState {
        log: log_arc.clone(),
        status: status_arc.clone(),
        progress: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        progress_total: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        paused: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        translation_memory: Arc::new(Mutex::new(HashMap::new())),
        global_progress: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        global_total: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        current_processing_path: Arc::new(Mutex::new(String::new())),
        current_batch: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        total_batches: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        pause_notifier: Arc::new(tokio::sync::Notify::new()),
        config: job_config.clone(),
        i18n: i18n.clone(),
    };

    // 保存至全域，方便暫停/中止連鎖控制
    if let Ok(mut active) = super::ACTIVE_JOB.lock() {
        *active = Some(job_state.clone());
    }

    // 5. 宣告一個背景連動監控 Logger/Progress 的任務
    let log_monitor = log_arc.clone();
    let status_monitor = status_arc.clone();
    let global_progress_monitor = job_state.global_progress.clone();
    let global_total_monitor = job_state.global_total.clone();

    let monitor_abort = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let abort_clone = monitor_abort.clone();

    let logger_clone = logger_arc.clone();
    let progress_clone = progress_arc.clone();

    let log_monitor_c = log_monitor.clone();
    let status_monitor_c = status_monitor.clone();
    let global_progress_monitor_c = global_progress_monitor.clone();
    let global_total_monitor_c = global_total_monitor.clone();
    let batch_current_monitor_c = job_state.current_batch.clone();
    let batch_total_monitor_c = job_state.total_batches.clone();

    let monitor_handle = tokio::spawn(async move {
        let mut last_len = 0;
        while !abort_clone.load(Ordering::SeqCst) {
            {
                let log = log_monitor_c.lock().unwrap();
                if log.len() > last_len {
                    for entry in &log[last_len..] {
                        logger_clone(entry.clone());
                    }
                    last_len = log.len();
                }
            }

            let current_g = f32::from_bits(global_progress_monitor_c.load(Ordering::SeqCst));
            let total_g = f32::from_bits(global_total_monitor_c.load(Ordering::SeqCst));
            let current_b = batch_current_monitor_c.load(Ordering::SeqCst) as f32;
            let total_b = batch_total_monitor_c.load(Ordering::SeqCst) as f32;
            if total_g >= 0.0 {
                let status_str = status_monitor_c.lock().unwrap().clone();
                progress_clone(current_g, total_g, current_b, total_b, &status_str);
            }

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    });

    // 6. 喚起檔案管線
    let res = crate::file::pipeline::process_all_files(
        expanded_input_paths,
        job_state.clone(),
        mc_lang,
        term_replacements,
        exact_match_map,
        inferred_match_map,
    )
    .await;

    // 7. 停止監控任務
    monitor_abort.store(true, Ordering::SeqCst);
    let _ = monitor_handle.await;

    // 最終完成訊息由 commands.rs 的 translation-finished 事件負責，不再推送狀態字串到日誌

    // 翻譯結束或出錯，釋放全域指針
    if let Ok(mut active) = super::ACTIVE_JOB.lock() {
        *active = None;
    }

    let i18n = crate::i18n::CommonLabels::load_or_default(&config.ui_lang);
    res.map_err(|e| format!("{}: {}", i18n.error_pipeline_failed, e).into())
}
