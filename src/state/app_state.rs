use crate::translation::job::JobConfig;
use crate::utils;
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync::Notify;
use crate::state::viewer_state::{ViewerSharedState, ViewerUpdate};
use std::sync::RwLock;

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
    // --- 主視窗位置/大小 ---
    pub main_x: f32,
    pub main_y: f32,
    pub main_width: f32,
    pub main_height: f32,

    // --- UI 控制標記 ---
    pub show_api_settings: bool,
    pub show_developer_mode: bool,
    pub show_memory_viewer: bool,
    pub is_memory_viewer_open: Arc<std::sync::atomic::AtomicBool>,
    pub show_stop_confirm: bool,
    pub dict_active_tab: Arc<Mutex<usize>>,
    pub _update_rx: tokio::sync::mpsc::UnboundedReceiver<ViewerUpdate>,

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
    /// 建議詞管理器開啟延遲計數器 (0.5s 延遲防閃爍)
    pub viewer_opening_counter: u32,
    // --- 視窗同步 (ses_342b) ---
    pub viewer_shared: Arc<ViewerSharedState>,
    /// 辭典檔案監控器 (保持生命週期)
    pub _dict_watcher: Option<Box<dyn std::any::Any>>,
}

impl AppState {
    /// 初始化 AppState，載入先前儲存的設定或預設值
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let visual = egui::Visuals::default();
        cc.egui_ctx.set_visuals(visual);
        let _ = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx);

        // 載入設定檔
        let config = crate::config::AppConfig::load();

        let (update_tx, update_rx) = tokio::sync::mpsc::unbounded_channel();
        let close_requested = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let viewer_shared = Arc::new(ViewerSharedState {
            theme: Arc::new(RwLock::new(config.theme.clone())),
            font_size: Arc::new(RwLock::new(config.font_size)),
            close_requested,
            update_tx,
            opened_last_frame: Arc::new(Mutex::new(false)),
            opened_frames: Arc::new(Mutex::new(0)),
            position: Arc::new(RwLock::new(None)),
            inner_size: Arc::new(RwLock::new(None)),
        });

        let mut state = Self {
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
            main_x: config.main_x,
            main_y: config.main_y,
            main_width: config.main_width,
            main_height: config.main_height,
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
            viewer_opening_counter: 0,
            viewer_shared,
            _update_rx: update_rx,
            _dict_watcher: None,
        };

        state.refresh_all_dictionaries();
        state.refresh_models();
        state.refresh_mc_versions();

        // 啟動辭典監控 (feat/dict-watcher)
        let _ = state.start_dict_watcher();

        state
    }

    pub fn add_log(&self, msg: &str) {
        let mut log = self.log.lock().unwrap();
        log.push(msg.to_string());
    }

    pub fn is_processing_active(&self) -> bool {
        *self.is_processing.lock().unwrap()
    }
}
