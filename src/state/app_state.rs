use crate::translation::job::JobConfig;
use crate::utils;
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU32, Ordering};
use tokio::runtime::Runtime;
use tokio::sync::Notify;
use crate::state::viewer_state::{ViewerSharedState, ViewerUpdate};
use crate::ui::i18n::I18nLabels;
use std::sync::RwLock;

/// 同步存檔用的包裹
pub struct ConfigPacket {
    pub app: crate::config::AppConfig,
    pub style: crate::config::StyleConfig,
}

/// 應用程式的主要全域狀態
pub struct AppState {
    // --- 輸入/輸出路徑 ---
    pub input_paths: Vec<(std::path::PathBuf, String)>,
    pub output_dir: String,

    // --- 非同步翻譯狀態 (共用 Arc) ---
    pub log: Arc<Mutex<Vec<String>>>,
    pub is_processing: Arc<AtomicBool>,
    pub is_cancelled: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<AtomicU32>,
    pub progress_total: Arc<AtomicU32>,
    pub global_progress: Arc<AtomicU32>,
    pub global_total: Arc<AtomicU32>,
    pub current_processing_path: Arc<Mutex<String>>,
    pub current_batch: Arc<AtomicU32>,
    pub total_batches: Arc<AtomicU32>,

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
    pub timeout: u32,
    pub user_prompt: String,
    pub system_prompt: String,
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
    pub source_lang: String,
    pub target_lang: String,

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
    pub show_palette_settings: bool,
    pub palette_edit_dark: bool,
    pub show_memory_viewer: bool,
    pub is_memory_viewer_open: Arc<AtomicBool>,
    pub show_stop_confirm: bool,
    pub show_clear_log_confirm: bool,
    pub show_restore_default_confirm: bool,
    pub dict_active_tab: Arc<AtomicUsize>,
    pub _update_rx: tokio::sync::mpsc::UnboundedReceiver<ViewerUpdate>,

    // --- 建議詞管理器 UI 狀態 ---
    pub dict_search: Arc<Mutex<String>>,
    pub dict_page: Arc<AtomicUsize>,
    pub dict_edit_key: Arc<Mutex<Option<String>>>,
    pub dict_edit_value: Arc<Mutex<String>>,
    pub show_dict_add_dialog: Arc<AtomicBool>,
    pub dict_new_key: Arc<Mutex<String>>,
    pub dict_new_value: Arc<Mutex<String>>,
    pub show_dict_replace_dialog: Arc<AtomicBool>,
    pub dict_replace_target: Arc<Mutex<String>>,
    pub dict_replace_new: Arc<Mutex<String>>,
    pub dict_replace_all: Arc<AtomicBool>,
    pub show_dict_clear_confirm: Arc<AtomicBool>,
    pub dict_cache: Arc<Mutex<Vec<(String, String)>>>,
    pub dict_search_last: Arc<Mutex<(String, usize)>>, // (搜尋詞, 總結果數)

    // --- 執行中任務控制 ---
    pub active_job_config: Option<Arc<Mutex<JobConfig>>>,
    pub runtime: Runtime,
    pub pause_notifier: Arc<Notify>,
    pub last_frame_time: std::time::Instant,
    /// 建議詞管理器開啟延遲計數器 (0.5s 延遲防閃爍)
    pub viewer_opening_counter: u32,
    // --- 視窗同步 ---
    pub viewer_shared: Arc<ViewerSharedState>,
    /// 辭典檔案監控器 (保持生命週期)
    pub _dict_watcher: Option<Box<dyn std::any::Any>>,

    // --- [自定義顏色狀態] ---
    pub dark_bg: [u8; 3],
    pub dark_text: [u8; 3],
    pub light_bg: [u8; 3],
    pub light_text: [u8; 3],

    pub dark_label: [u8; 3],
    pub light_label: [u8; 3],
    pub dark_btn_bg: [u8; 3],
    pub dark_btn_text: [u8; 3],
    pub light_btn_bg: [u8; 3],
    pub light_btn_text: [u8; 3],
    pub dark_input_bg: [u8; 3],
    pub light_input_bg: [u8; 3],
    pub dark_list_bg: [u8; 3],
    pub light_list_bg: [u8; 3],
    pub dark_tab_active: [u8; 3],
    pub dark_tab_inactive: [u8; 3],
    pub light_tab_active: [u8; 3],
    pub light_tab_inactive: [u8; 3],

    pub btn_rounding_enabled: bool,
    pub btn_rounding_value: f32,
    pub progress_pulse_enabled: bool,
    pub progress_pulse_speed: f32,

    pub instance_overrides: HashMap<String, crate::config::settings::ComponentStyle>,
    
    // --- [持久化與 UI 內部狀態] ---
    pub save_tx: tokio::sync::mpsc::UnboundedSender<ConfigPacket>,
    pub palette_edit_slots: Vec<PaletteEditSlot>,
    pub palette_all_selected: bool,
    /// 屬性勾選狀態 (批次變更用)
    pub palette_prop_sync_bg: bool,
    pub palette_prop_sync_text: bool,
    pub palette_prop_sync_rounding: bool,

    // --- [i18n] ---
    pub i18n: I18nLabels,

    /// 暫存的 Hsva 色彩狀態 (解決 HUE 無法調整問題)
    pub palette_hsva_bg: Option<egui::ecolor::Hsva>,
    pub palette_hsva_text: Option<egui::ecolor::Hsva>,
    pub palette_hsva_target: String,
}

#[derive(Clone, PartialEq)]
pub struct PaletteEditSlot {
    pub target_id: String,
    pub is_checked: bool,
}

impl AppState {
    /// 初始化 AppState，載入先前儲存的設定或預設值
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let visual = egui::Visuals::default();
        cc.egui_ctx.set_visuals(visual);
        let _ = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx);

        // 載入設定檔
        let config = crate::config::AppConfig::load();
        let style_cfg = crate::config::StyleConfig::load();

        // [i18n] 確保目錄並載入語系
        let _ = I18nLabels::ensure_langs_exists();
        let i18n = I18nLabels::load_or_default(&config.target_lang);

        let (update_tx, update_rx) = tokio::sync::mpsc::unbounded_channel();
        let close_requested = Arc::new(AtomicBool::new(false));

        let viewer_shared = Arc::new(ViewerSharedState {
            theme: Arc::new(RwLock::new(style_cfg.theme.clone())),
            font_size: Arc::new(RwLock::new(style_cfg.font_size)),
            style: Arc::new(RwLock::new(crate::state::viewer_state::StyleSnapshot {
                dark_bg: style_cfg.dark_bg,
                dark_text: style_cfg.dark_text,
                light_bg: style_cfg.light_bg,
                light_text: style_cfg.light_text,
                dark_label: style_cfg.dark_label,
                light_label: style_cfg.light_label,
                dark_btn_bg: style_cfg.dark_btn_bg,
                dark_btn_text: style_cfg.dark_btn_text,
                light_btn_bg: style_cfg.light_btn_bg,
                light_btn_text: style_cfg.light_btn_text,
                dark_input_bg: style_cfg.dark_input_bg,
                light_input_bg: style_cfg.light_input_bg,
                dark_list_bg: style_cfg.dark_list_bg,
                light_list_bg: style_cfg.light_list_bg,
                dark_tab_active: style_cfg.dark_tab_active,
                light_tab_active: style_cfg.light_tab_active,
                rounding: style_cfg.btn_rounding_value,
                instance_overrides: style_cfg.instance_overrides.clone(),
            })),
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
            is_processing: Arc::new(AtomicBool::new(false)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            status: Arc::new(Mutex::new("待機中".to_string())),
            progress: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            progress_total: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            global_progress: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            global_total: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            current_processing_path: Arc::new(Mutex::new(String::new())),
            current_batch: Arc::new(AtomicU32::new(0)),
            total_batches: Arc::new(AtomicU32::new(0)),
            mc_lang: Arc::new(Mutex::new(None)),
            term_replacements: Arc::new(Mutex::new(Vec::new())),
            exact_match_map: Arc::new(Mutex::new(HashMap::new())),
            inferred_match_map: Arc::new(Mutex::new(HashMap::new())),
            translation_memory: Arc::new(Mutex::new(HashMap::new())),
            api_key: config.api_key.clone(),
            api_provider: config.api_provider.clone(),
            selected_model: config.model.clone(),
            ollama_url: config.ollama_url.clone(),
            available_models: Arc::new(Mutex::new(Vec::new())),
            batch_size: config.batch_size,
            batch_max_chars: config.batch_max_chars,
            timeout: config.timeout,
            user_prompt: config.user_prompt.clone(),
            system_prompt: config.system_prompt.clone(),
            theme: style_cfg.theme.clone(),
            pack_format: config.pack_format,
            font_size: style_cfg.font_size,
            enable_custom_fps: config.enable_custom_fps,
            custom_fps: config.custom_fps,
            mc_versions: Arc::new(Mutex::new(Vec::new())),
            glossary_priority: Arc::new(Mutex::new(config.glossary_priority.clone())),
            skip_json: config.skip_json,
            skip_js: config.skip_js,
            skip_jar: config.skip_jar,
            skip_book: config.skip_book,
            enable_llm_log: config.enable_llm_log,
            source_lang: config.source_lang,
            target_lang: config.target_lang,
            viewer_x: config.viewer_x,
            viewer_y: config.viewer_y,
            viewer_width: config.viewer_width,
            viewer_height: config.viewer_height,
            main_x: config.main_x,
            main_y: config.main_y,
            main_width: config.main_width,
            main_height: config.main_height,
            show_api_settings: config.show_api_settings,
            show_developer_mode: config.show_developer_mode,
            show_palette_settings: false,
            palette_edit_dark: style_cfg.theme == "dark",
            show_memory_viewer: false,
            is_memory_viewer_open: Arc::new(AtomicBool::new(false)),
            show_stop_confirm: false,
            show_clear_log_confirm: false,
            show_restore_default_confirm: false,
            dict_active_tab: Arc::new(AtomicUsize::new(0)),
            dict_search: Arc::new(Mutex::new(String::new())),
            dict_page: Arc::new(AtomicUsize::new(0)),
            dict_edit_key: Arc::new(Mutex::new(None)),
            dict_edit_value: Arc::new(Mutex::new(String::new())),
            show_dict_add_dialog: Arc::new(AtomicBool::new(false)),
            dict_new_key: Arc::new(Mutex::new(String::new())),
            dict_new_value: Arc::new(Mutex::new(String::new())),
            show_dict_replace_dialog: Arc::new(AtomicBool::new(false)),
            dict_replace_target: Arc::new(Mutex::new(String::new())),
            dict_replace_new: Arc::new(Mutex::new(String::new())),
            dict_replace_all: Arc::new(AtomicBool::new(true)),
            show_dict_clear_confirm: Arc::new(AtomicBool::new(false)),
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
            dark_bg: style_cfg.dark_bg,
            dark_text: style_cfg.dark_text,
            light_bg: style_cfg.light_bg,
            light_text: style_cfg.light_text,
            dark_label: style_cfg.dark_label,
            light_label: style_cfg.light_label,
            dark_btn_bg: style_cfg.dark_btn_bg,
            dark_btn_text: style_cfg.dark_btn_text,
            light_btn_bg: style_cfg.light_btn_bg,
            light_btn_text: style_cfg.light_btn_text,
            dark_input_bg: style_cfg.dark_input_bg,
            light_input_bg: style_cfg.light_input_bg,
            dark_list_bg: style_cfg.dark_list_bg,
            light_list_bg: style_cfg.light_list_bg,
            dark_tab_active: style_cfg.dark_tab_active,
            dark_tab_inactive: style_cfg.dark_tab_inactive,
            light_tab_active: style_cfg.light_tab_active,
            light_tab_inactive: style_cfg.light_tab_inactive,
            btn_rounding_enabled: style_cfg.btn_rounding_enabled,
            btn_rounding_value: style_cfg.btn_rounding_value,
            progress_pulse_enabled: style_cfg.progress_pulse_enabled,
            progress_pulse_speed: style_cfg.progress_pulse_speed,
            instance_overrides: style_cfg.instance_overrides.clone(),
            save_tx: {
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ConfigPacket>();
                cc.egui_ctx.request_repaint(); // 確保背景任務啟動
                let _rt = Runtime::new().unwrap(); // 這裡我們需要一個獨立的 runtime 或使用現有的
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    rt.block_on(async move {
                        while let Some(packet) = rx.recv().await {
                            packet.app.save();
                            packet.style.save();
                        }
                    });
                });
                tx
            },
            palette_edit_slots: vec![
                PaletteEditSlot { target_id: "全部按鈕".to_string(), is_checked: true },
            ],
            palette_all_selected: false,
            palette_prop_sync_bg: true,
            palette_prop_sync_text: true,
            palette_prop_sync_rounding: true,
            i18n,
            palette_hsva_bg: None,
            palette_hsva_text: None,
            palette_hsva_target: String::new(),
        };

        // 啟動背景持久化任務已在上述 thread spawn 中處理

        state.refresh_all_dictionaries();
        state.refresh_models();
        state.refresh_mc_versions();

        // 啟動辭典監控 (feat/dict-watcher)
        let _ = state.start_dict_watcher();

        state
    }

    pub fn add_log(&self, msg: &str) {
        let mut log = self.log.lock().unwrap();
        let now = chrono::Local::now().format("[%H:%M:%S]").to_string();
        log.push(format!("{} <{}->{}> {}", now, self.source_lang, self.target_lang, msg));
    }

    pub fn is_processing_active(&self) -> bool {
        self.is_processing.load(Ordering::SeqCst)
    }

    /// 觸發非同步存檔
    pub fn trigger_save(&self) {
        // 同步樣式至 Viewport
        if let Ok(mut style_lock) = self.viewer_shared.style.write() {
            *style_lock = crate::state::viewer_state::StyleSnapshot {
                dark_bg: self.dark_bg,
                dark_text: self.dark_text,
                light_bg: self.light_bg,
                light_text: self.light_text,
                dark_label: self.dark_label,
                light_label: self.light_label,
                dark_btn_bg: self.dark_btn_bg,
                dark_btn_text: self.dark_btn_text,
                light_btn_bg: self.light_btn_bg,
                light_btn_text: self.light_btn_text,
                dark_input_bg: self.dark_input_bg,
                light_input_bg: self.light_input_bg,
                dark_list_bg: self.dark_list_bg,
                light_list_bg: self.light_list_bg,
                dark_tab_active: self.dark_tab_active,
                light_tab_active: self.light_tab_active,
                rounding: self.btn_rounding_value,
                instance_overrides: self.instance_overrides.clone(),
            };
        }

        let packet = self.to_config_packet();
        let _ = self.save_tx.send(packet);
    }

    /// 產生 AppConfig 快照
    pub fn make_app_config(&self) -> crate::config::AppConfig {
        crate::config::AppConfig {
            api_key: self.api_key.clone(),
            api_provider: self.api_provider.clone(),
            model: self.selected_model.clone(),
            ollama_url: self.ollama_url.clone(),
            user_prompt: self.user_prompt.clone(),
            system_prompt: self.system_prompt.clone(),
            batch_size: self.batch_size,
            batch_max_chars: self.batch_max_chars,
            timeout: self.timeout,
            glossary_priority: self.glossary_priority.lock().unwrap().clone(),
            output_dir: self.output_dir.clone(),
            pack_format: self.pack_format,
            enable_custom_fps: self.enable_custom_fps,
            custom_fps: self.custom_fps,
            show_api_settings: self.show_api_settings,
            show_developer_mode: self.show_developer_mode,
            skip_json: self.skip_json,
            skip_js: self.skip_js,
            skip_jar: self.skip_jar,
            skip_book: self.skip_book,
            enable_llm_log: self.enable_llm_log,
            source_lang: self.source_lang.clone(),
            target_lang: self.target_lang.clone(),
            main_x: self.main_x,
            main_y: self.main_y,
            main_width: self.main_width,
            main_height: self.main_height,
            viewer_x: self.viewer_x,
            viewer_y: self.viewer_y,
            viewer_width: self.viewer_width,
            viewer_height: self.viewer_height,
        }
    }

    /// 產生 StyleConfig 快照
    pub fn make_style_config(&self) -> crate::config::StyleConfig {
        crate::config::StyleConfig {
            theme: self.theme.clone(),
            font_size: self.font_size,
            dark_bg: self.dark_bg,
            dark_text: self.dark_text,
            light_bg: self.light_bg,
            light_text: self.light_text,
            dark_label: self.dark_label,
            light_label: self.light_label,
            dark_btn_bg: self.dark_btn_bg,
            dark_btn_text: self.dark_btn_text,
            light_btn_bg: self.light_btn_bg,
            light_btn_text: self.light_btn_text,
            dark_input_bg: self.dark_input_bg,
            light_input_bg: self.light_input_bg,
            dark_list_bg: self.dark_list_bg,
            light_list_bg: self.light_list_bg,
            dark_tab_active: self.dark_tab_active,
            dark_tab_inactive: self.dark_tab_inactive,
            light_tab_active: self.light_tab_active,
            light_tab_inactive: self.light_tab_inactive,
            btn_rounding_enabled: self.btn_rounding_enabled,
            btn_rounding_value: self.btn_rounding_value,
            progress_pulse_enabled: self.progress_pulse_enabled,
            progress_pulse_speed: self.progress_pulse_speed,
            instance_overrides: self.instance_overrides.clone(),
        }
    }

    /// 將目前狀態轉換為 ConfigPacket 快照
    pub fn to_config_packet(&self) -> ConfigPacket {
        ConfigPacket {
            app: self.make_app_config(),
            style: self.make_style_config(),
        }
    }
}
