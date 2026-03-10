//! # 全域狀態與日誌管理
//! 本模組定義了 AppState 以及相關的日誌與非同步處理邏輯。

use crate::config::AppConfig;
use crate::file_handler;
use crate::translation_job::{JobConfig, JobSharedState};
use crate::translation_service;
use crate::utils;
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::runtime::Runtime;
use tokio::sync::Notify;

/// 子視窗更新訊息類型
pub enum ViewerUpdate {
    Theme(String),
    FontSize(f32),
}

/// 視窗共享狀態 (用於主子視窗同步)
pub struct ViewerSharedState {
    pub theme: Arc<RwLock<String>>,
    pub font_size: Arc<RwLock<f32>>,
    pub close_requested: Arc<std::sync::atomic::AtomicBool>,
    pub update_tx: tokio::sync::mpsc::UnboundedSender<ViewerUpdate>,
    pub opened_last_frame: Arc<Mutex<bool>>,
}

/// 應用程式的主要全域狀態
pub struct AppState {
    // --- 輸入/輸出路徑 ---
    pub input_paths: Vec<(std::path::PathBuf, String)>,
    pub output_dir: String,

    // --- 非同步翻譯狀態 (共用 Arc) ---
    pub log: Arc<Mutex<Vec<String>>>,
    pub is_processing: Arc<Mutex<bool>>,
    pub is_cancelled: Arc<Mutex<bool>>,
    pub is_paused: Arc<Mutex<bool>>,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<Mutex<f32>>,
    pub progress_total: Arc<Mutex<f32>>,
    pub global_progress: Arc<Mutex<f32>>,
    pub global_total: Arc<Mutex<f32>>,

    // --- 語言與模型數據 ---
    pub mc_lang: Arc<Mutex<Option<utils::McLangFiles>>>,
    pub term_replacements: Arc<Mutex<Vec<(String, String)>>>,
    pub exact_match_map: Arc<Mutex<HashMap<String, String>>>,
    pub inferred_match_map: Arc<Mutex<HashMap<String, String>>>,
    // --- 翻譯快取記憶體 ---
    pub translation_memory: Arc<Mutex<HashMap<String, String>>>,

    // --- API 金鑰與模型設定 ---
    pub api_key: String,
    pub api_provider: String,
    pub selected_model: String,
    pub ollama_url: String,
    pub available_models: Arc<Mutex<Vec<String>>>,
    pub batch_size: u32,
    pub batch_max_chars: u32,
    pub ollama_timeout: u32,
    pub translation_prompt: String,
    pub theme: String,
    pub pack_format: u32,
    pub font_size: f32,
    pub enable_custom_fps: bool,
    pub custom_fps: u32,
    pub mc_versions: Arc<Mutex<Vec<(String, u32)>>>,
    pub glossary_priority: Arc<Mutex<String>>,
    pub skip_json: bool,
    pub skip_js: bool,
    pub skip_jar: bool,
    pub skip_book: bool,
    pub enable_llm_log: bool,

    // --- 建議詞管理器視窗位置/大小 ---
    pub viewer_x: f32,
    pub viewer_y: f32,
    pub viewer_width: f32,
    pub viewer_height: f32,

    // --- UI 控制標記 ---
    pub show_api_settings: bool,
    pub show_developer_mode: bool,
    pub show_memory_viewer: bool,
    pub is_memory_viewer_open: Arc<std::sync::atomic::AtomicBool>,
    pub show_stop_confirm: bool,
    pub dict_active_tab: Arc<Mutex<usize>>,

    // --- 建議詞管理器 UI 狀態 ---
    pub dict_search: Arc<Mutex<String>>,
    pub dict_page: Arc<Mutex<usize>>,
    pub dict_edit_key: Arc<Mutex<Option<String>>>,
    pub dict_edit_value: Arc<Mutex<String>>,
    pub show_dict_add_dialog: Arc<Mutex<bool>>,
    pub dict_new_key: Arc<Mutex<String>>,
    pub dict_new_value: Arc<Mutex<String>>,
    pub show_dict_replace_dialog: Arc<Mutex<bool>>,
    pub dict_replace_target: Arc<Mutex<String>>,
    pub dict_replace_new: Arc<Mutex<String>>,
    pub dict_replace_all: Arc<Mutex<bool>>,
    pub show_dict_clear_confirm: Arc<Mutex<bool>>,
    pub dict_cache: Arc<Mutex<Vec<(String, String)>>>,
    pub dict_search_last: Arc<Mutex<(String, usize)>>, // (搜尋詞, 總結果數)

    // --- 執行中任務控制 ---
    pub active_job_config: Option<Arc<Mutex<JobConfig>>>,
    pub runtime: Runtime,
    pub pause_notifier: Arc<Notify>,
    pub last_frame_time: std::time::Instant,

    // --- 視窗同步 (ses_342b) ---
    pub viewer_shared: Arc<ViewerSharedState>,
}

impl AppState {
    /// 初始化 AppState，載入先前儲存的設定或預設值
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let visual = egui::Visuals::default();
        cc.egui_ctx.set_visuals(visual);
        let _ = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx);

        // 載入設定檔
        let config = AppConfig::load();

        let (update_tx, _update_rx) = tokio::sync::mpsc::unbounded_channel();
        let close_requested = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let viewer_shared = Arc::new(ViewerSharedState {
            theme: Arc::new(RwLock::new(config.theme.clone())),
            font_size: Arc::new(RwLock::new(config.font_size)),
            close_requested,
            update_tx,
            opened_last_frame: Arc::new(Mutex::new(false)),
        });

        let state = Self {
            input_paths: Vec::new(),
            output_dir: config.output_dir.clone(),
            log: Arc::new(Mutex::new(Vec::new())),
            is_processing: Arc::new(Mutex::new(false)),
            is_cancelled: Arc::new(Mutex::new(false)),
            is_paused: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("待機中".to_string())),
            progress: Arc::new(Mutex::new(0.0)),
            progress_total: Arc::new(Mutex::new(0.0)),
            global_progress: Arc::new(Mutex::new(0.0)),
            global_total: Arc::new(Mutex::new(0.0)),
            mc_lang: Arc::new(Mutex::new(None)),
            term_replacements: {
                let loaded: HashMap<String, String> =
                    crate::config::load_dict(crate::config::OFFICIAL_DICT);
                let mut terms: Vec<(String, String)> = loaded.into_iter().collect();
                terms.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
                Arc::new(Mutex::new(terms))
            },
            exact_match_map: Arc::new(Mutex::new(HashMap::new())),
            inferred_match_map: Arc::new(Mutex::new(crate::config::load_dict(
                crate::config::OFFICIAL_DICT,
            ))),
            translation_memory: Arc::new(Mutex::new(crate::config::load_translation_memory())),
            api_key: config.api_key.clone(),
            api_provider: config.provider.clone(),
            selected_model: config.model.clone(),
            ollama_url: config.ollama_url.clone(),
            available_models: Arc::new(Mutex::new(Vec::new())),
            batch_size: config.batch_size,
            batch_max_chars: config.batch_max_chars,
            ollama_timeout: config.ollama_timeout,
            translation_prompt: config.translation_prompt.clone(),
            theme: config.theme.clone(),
            pack_format: config.pack_format,
            font_size: config.font_size,
            enable_custom_fps: false,
            custom_fps: 60,
            mc_versions: Arc::new(Mutex::new(Vec::new())),
            glossary_priority: Arc::new(Mutex::new(config.glossary_priority.clone())),
            skip_json: config.skip_json,
            skip_js: config.skip_js,
            skip_jar: config.skip_jar,
            skip_book: config.skip_book,
            enable_llm_log: config.enable_llm_log,
            viewer_x: config.viewer_x,
            viewer_y: config.viewer_y,
            viewer_width: config.viewer_width,
            viewer_height: config.viewer_height,
            show_api_settings: false,
            show_developer_mode: false,
            show_memory_viewer: false,
            is_memory_viewer_open: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            show_stop_confirm: false,
            dict_active_tab: Arc::new(Mutex::new(0)),
            dict_search: Arc::new(Mutex::new(String::new())),
            dict_page: Arc::new(Mutex::new(0)),
            dict_edit_key: Arc::new(Mutex::new(None)),
            dict_edit_value: Arc::new(Mutex::new(String::new())),
            show_dict_add_dialog: Arc::new(Mutex::new(false)),
            dict_new_key: Arc::new(Mutex::new(String::new())),
            dict_new_value: Arc::new(Mutex::new(String::new())),
            show_dict_replace_dialog: Arc::new(Mutex::new(false)),
            dict_replace_target: Arc::new(Mutex::new(String::new())),
            dict_replace_new: Arc::new(Mutex::new(String::new())),
            dict_replace_all: Arc::new(Mutex::new(true)),
            show_dict_clear_confirm: Arc::new(Mutex::new(false)),
            dict_cache: Arc::new(Mutex::new(Vec::new())),
            dict_search_last: Arc::new(Mutex::new((String::new(), 0usize))),
            active_job_config: None,
            runtime: Runtime::new().unwrap(),
            pause_notifier: Arc::new(Notify::new()),
            last_frame_time: std::time::Instant::now(),
            viewer_shared,
        };

        state.refresh_all_dictionaries();
        state.refresh_models();
        state.refresh_mc_versions();

        state
    }

    pub fn refresh_all_dictionaries(&self) {
        Self::refresh_dictionaries_core(
            self.mc_lang.clone(),
            self.exact_match_map.clone(),
            self.inferred_match_map.clone(),
            self.term_replacements.clone(),
            self.status.clone(),
            self.runtime.handle().clone(),
            self.viewer_shared.update_tx.clone(),
            self.theme.clone(),
            self.show_memory_viewer,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn refresh_dictionaries_core(
        mc_lang_arc: Arc<Mutex<Option<utils::McLangFiles>>>,
        exact_arc: Arc<Mutex<HashMap<String, String>>>,
        inferred_arc: Arc<Mutex<HashMap<String, String>>>,
        term_arc: Arc<Mutex<Vec<(String, String)>>>,
        status_arc: Arc<Mutex<String>>,
        runtime_handle: tokio::runtime::Handle,
        viewer_update_tx: tokio::sync::mpsc::UnboundedSender<ViewerUpdate>,
        theme_clone: String,
        show_viewer_clone: bool,
    ) {
        *status_arc.lock().unwrap() = "正在分析辭典...".to_string();

        runtime_handle.spawn(async move {
            if let Ok((files, exact, unfiltered)) = crate::utils::load_mc_dicts().await {
                {
                    let mut exact_map = exact_arc.lock().unwrap();
                    *exact_map = exact.clone();
                }

                let inferred = crate::utils::analyze_dictionary(&exact);
                {
                    let mut inferred_map = inferred_arc.lock().unwrap();
                    *inferred_map = inferred.clone();
                }
                if show_viewer_clone {
                    let _ = viewer_update_tx
                        .send(crate::state_and_log::ViewerUpdate::Theme(theme_clone));
                }

                {
                    let mut term_map = term_arc.lock().unwrap();
                    *term_map = unfiltered;
                }
                crate::config::save_dict(crate::config::OFFICIAL_DICT, &inferred);

                if let Ok(mut mc) = mc_lang_arc.lock() {
                    *mc = Some(files);
                }
            }
            *status_arc.lock().unwrap() = "就緒".to_string();
        });
    }

    pub fn add_log(&self, msg: &str) {
        let mut log = self.log.lock().unwrap();
        log.push(msg.to_string());
    }

    pub fn resume_translation(&mut self) {
        if let Some(ref active_cfg) = self.active_job_config {
            let mut cfg = active_cfg.lock().unwrap();
            cfg.api_key = self.api_key.clone();
            cfg.api_provider = self.api_provider.clone();
            cfg.selected_model = self.selected_model.clone();
            cfg.ollama_url = self.ollama_url.clone();
            cfg.prompt = self.translation_prompt.clone();
            cfg.ollama_timeout = self.ollama_timeout as u64;
            cfg.batch_size = self.batch_size;
            cfg.batch_max_chars = self.batch_max_chars;
            cfg.pack_format = self.pack_format;
            cfg.glossary_priority = self.glossary_priority.lock().unwrap().clone();
            cfg.skip_json = self.skip_json;
            cfg.skip_js = self.skip_js;
            cfg.skip_jar = self.skip_jar;
            cfg.skip_book = self.skip_book;
            cfg.enable_llm_log = self.enable_llm_log;
        }

        *self.is_paused.lock().unwrap() = false;
        self.pause_notifier.notify_waiters();
        self.add_log(">>> 使用者恢復翻譯任務中...");
    }

    pub fn save_config(&self) {
        let mut config = AppConfig::load();
        config.output_dir = self.output_dir.clone();
        config.provider = self.api_provider.clone();
        config.model = self.selected_model.clone();
        config.ollama_url = self.ollama_url.clone();
        config.batch_size = self.batch_size;
        config.batch_max_chars = self.batch_max_chars;
        config.ollama_timeout = self.ollama_timeout;
        config.translation_prompt = self.translation_prompt.clone();
        config.theme = self.theme.clone();
        config.pack_format = self.pack_format;
        config.font_size = self.font_size;
        config.glossary_priority = self.glossary_priority.lock().unwrap().clone();
        config.skip_json = self.skip_json;
        config.skip_js = self.skip_js;
        config.skip_jar = self.skip_jar;
        config.skip_book = self.skip_book;
        config.enable_llm_log = self.enable_llm_log;

        if self.api_provider == "Ollama" {
            config.api_key = String::new();
        } else if self.api_provider != "DeepL" || !self.api_key.is_empty() {
            config.api_key = self.api_key.clone();
        }
        config.save();
    }

    pub fn refresh_models(&self) {
        let provider = self.api_provider.clone();
        let api_key = self.api_key.clone();
        let ollama_url = self.ollama_url.clone();
        let available_models = self.available_models.clone();

        self.runtime.spawn(async move {
            if let Ok(models) =
                translation_service::fetch_dynamic_models(&provider, &api_key, &ollama_url).await
            {
                let mut m = available_models.lock().unwrap();
                *m = models;
            }
        });
    }

    pub fn refresh_mc_versions(&self) {
        let mc_versions = self.mc_versions.clone();
        self.runtime.spawn(async move {
            let versions = translation_service::fetch_mc_versions().await;
            let mut v = mc_versions.lock().unwrap();
            *v = versions;
        });
    }

    pub fn start_translation(&mut self, _ctx: egui::Context) {
        if self.is_processing_active() {
            return;
        }

        *self.progress.lock().unwrap() = 0.0;
        *self.progress_total.lock().unwrap() = 0.0;
        *self.global_progress.lock().unwrap() = 0.0;
        *self.global_total.lock().unwrap() = 0.0;
        *self.status.lock().unwrap() = "正在分析檔案".to_string();

        self.refresh_all_dictionaries();

        self.add_log(">>> 開始翻譯任務...");

        let log = self.log.clone();
        let paths = self.input_paths.clone();
        let mc_lang_arc = self.mc_lang.clone();
        let term_arc = self.term_replacements.clone();
        let exact_arc = self.exact_match_map.clone();
        let processing_arc = self.is_processing.clone();
        let cancelled_arc = self.is_cancelled.clone();
        let paused_arc = self.is_paused.clone();
        let status_arc = self.status.clone();
        let translation_memory_arc = self.translation_memory.clone();

        let api_key = self.api_key.clone();
        let api_provider = self.api_provider.clone();
        let selected_model = self.selected_model.clone();
        let ollama_url = self.ollama_url.clone();
        let batch_size = self.batch_size;
        let batch_max_chars = self.batch_max_chars;
        let ollama_timeout = self.ollama_timeout;
        let translation_prompt = self.translation_prompt.clone();
        let output_dir_val = self.output_dir.clone();
        let pack_format_val = self.pack_format;

        let job_config = Arc::new(Mutex::new(JobConfig::new(
            api_key,
            api_provider,
            selected_model,
            ollama_url,
            translation_prompt,
            ollama_timeout,
            batch_size,
            batch_max_chars,
            output_dir_val,
            pack_format_val,
            self.glossary_priority.lock().unwrap().clone(),
            self.skip_json,
            self.skip_js,
            self.skip_jar,
            self.skip_book,
            self.enable_llm_log,
        )));
        self.active_job_config = Some(job_config.clone());

        let job_state = JobSharedState {
            log: log.clone(),
            status: status_arc.clone(),
            progress: self.progress.clone(),
            progress_total: self.progress_total.clone(),
            global_progress: self.global_progress.clone(),
            global_total: self.global_total.clone(),
            cancelled: self.is_cancelled.clone(),
            paused: paused_arc.clone(),
            translation_memory: translation_memory_arc.clone(),
            pause_notifier: self.pause_notifier.clone(),
            config: job_config.clone(),
        };

        *processing_arc.lock().unwrap() = true;
        *cancelled_arc.lock().unwrap() = false;
        *paused_arc.lock().unwrap() = false;

        let self_runtime = self.runtime.handle().clone();

        self_runtime.spawn(async move {
            let res =
                file_handler::process_all_files(paths, job_state, mc_lang_arc, term_arc, exact_arc)
                    .await;

            *processing_arc.lock().unwrap() = false;
            let mut s = status_arc.lock().unwrap();
            if *cancelled_arc.lock().unwrap() {
                *s = "任務已取消".to_string();
                let mut l = log.lock().unwrap();
                l.push(">>> 任務已由使用者手動取消。".to_string());
            } else if let Err(e) = res {
                *s = format!("發生錯誤: {}", e);
                let mut l = log.lock().unwrap();
                l.push(format!(">>> 錯誤: {}", e));
            } else {
                *s = "任務完成".to_string();
                let mut l = log.lock().unwrap();
                l.push(">>> 所有翻譯任務已完成！".to_string());
            }
        });
    }

    pub fn is_processing_active(&self) -> bool {
        *self.is_processing.lock().unwrap()
    }

    pub fn stop_translation(&self) {
        *self.is_cancelled.lock().unwrap() = true;
        *self.is_paused.lock().unwrap() = false;
        self.pause_notifier.notify_waiters();
    }

    pub fn pause_translation(&self) {
        *self.is_paused.lock().unwrap() = true;
    }
}
