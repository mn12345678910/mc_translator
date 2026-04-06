//! # 設定模組
//! 負責 AppConfig 結構體定義、config.cfg 與系統憑證 (Keyring) 的讀寫邏輯。

#[allow(unused_imports)]
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[cfg(not(test))]
use super::encryption::{get_api_key, save_api_key};

pub const DEFAULT_PROMPT: &str = "你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。";

pub const DEFAULT_SYSTEM_PROMPT: &str = "\n\n[內部技術指令 - 請務必遵守]\n\
1. 僅針對 %%VAR_n%%, %%MC_n%%, %%HEX_n%% 等技術佔位符執行「保持原樣」操作（不可修改、翻譯或增刪標籤）。\n\
2. 嚴禁在此類標籤（%%...%%）之外自行臆造、增加或移動任何格式標籤。若原文無標籤，譯文亦不可有標籤。\n\
3. 其餘文本內容均「必須」按要求翻譯，絕對不可將全文原樣輸出。";

/// 核心功能設定檔 (config.cfg)
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    /// API 金鑰（Gemini / OpenAI）
    #[serde(skip)]
    pub api_key: SecretString,

    // --- [核心 API 設定] ---
    #[serde(default = "default_api_provider")]
    pub api_provider: String,
    pub model: String,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default)]
    pub api_base_url: String,

    // --- [Prompt 集] ---
    #[serde(default = "default_user_prompt")]
    pub user_prompt: String,
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    // --- [翻譯參數] ---
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    #[serde(default = "default_batch_max_chars")]
    pub batch_max_chars: u32,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    #[serde(default = "default_glossary_priority")]
    pub glossary_priority: String,

    // --- [語言設定] ---
    #[serde(default = "default_source_lang")]
    pub source_lang: String,
    #[serde(default = "default_target_lang")]
    pub target_lang: String,
    #[serde(default = "default_ui_lang")]
    pub ui_lang: String,

    // --- [輸出與路徑] ---
    pub output_dir: String,
    #[serde(default = "default_pack_format")]
    pub pack_format: u32,

    // --- [效能與面板狀態] ---
    pub show_api_settings: bool,
    pub show_developer_mode: bool,
    #[serde(default)]
    pub show_debug_tools: bool,

    // --- [檔案篩選] ---
    pub skip_json: bool,
    pub skip_js: bool,
    pub skip_jar: bool,
    pub skip_book: bool,
    pub enable_llm_log: bool,
    #[serde(default)]
    pub enable_debug_log: bool,

    // --- [檔案過濾] ---
    #[serde(default = "default_excluded_paths")]
    pub excluded_paths: Vec<String>,

    // --- [快速轉換] ---
    #[serde(default)]
    pub fast_convert: bool,

    // --- [視窗幾何資訊] ---
    pub main_x: f32,
    pub main_y: f32,
    pub main_width: f32,
    pub main_height: f32,
    pub viewer_x: f32,
    pub viewer_y: f32,
    pub viewer_width: f32,
    pub viewer_height: f32,
}

pub fn default_excluded_paths() -> Vec<String> {
    vec![
        "kubejs/data/".to_string(),
        "packmenu/".to_string(),
        "config/almostunified/".to_string(),
        "fancymenu/".to_string(),
        "journeymap/icon/theme".to_string(),
        "shaderpacks/".to_string(),
        "screenshots/".to_string(),
        "saves/".to_string(),
        "logs/".to_string(),
        "defaultconfigs/".to_string(),
        "local/".to_string(),
        ".mixin.out/".to_string(),
    ]
}

// === Default 函數 (AppConfig) ===
fn default_api_provider() -> String {
    "無".to_string()
}
fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}
fn default_user_prompt() -> String {
    DEFAULT_PROMPT.to_string()
}
fn default_system_prompt() -> String {
    DEFAULT_SYSTEM_PROMPT.to_string()
}
fn default_batch_size() -> u32 {
    150
}
fn default_batch_max_chars() -> u32 {
    3500
}
fn default_timeout() -> u32 {
    60
}
fn default_glossary_priority() -> String {
    "official".to_string()
}
fn default_source_lang() -> String {
    "en_us".to_string()
}
fn default_target_lang() -> String {
    "zh_tw".to_string()
}
fn default_pack_format() -> u32 {
    15
}
fn default_ui_lang() -> String {
    "zh_tw".to_string()
}

// === Default 函數 (StyleConfig 關鍵欄位) ===
fn default_theme() -> String {
    "dark".to_string()
}
fn default_font_size() -> f32 {
    15.0
}
fn default_true() -> bool {
    true
}

// === Macro 輔助：減少樣板代碼 ===
macro_rules! default_color {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        fn $name() -> [u8; 3] {
            [$r, $g, $b]
        }
    };
}

macro_rules! default_float {
    ($name:ident, $val:expr) => {
        fn $name() -> f32 {
            $val
        }
    };
}

macro_rules! default_string {
    ($name:ident, $val:expr) => {
        fn $name() -> String {
            $val.to_string()
        }
    };
}

// 調色盤
default_color!(default_dark_bg, 30, 30, 35);
default_color!(default_dark_text, 200, 160, 100);
default_color!(default_light_bg, 252, 252, 253);
default_color!(default_light_text, 30, 30, 35);
default_color!(default_dark_label, 200, 160, 100);
default_color!(default_light_label, 30, 30, 35);
default_color!(default_dark_text_muted, 170, 170, 170);
default_color!(default_light_text_muted, 102, 102, 102);
default_color!(default_dark_btn_bg, 45, 45, 50);
default_color!(default_dark_btn_text, 220, 220, 220);
default_color!(default_light_btn_bg, 240, 240, 245);
default_color!(default_light_btn_text, 45, 45, 50);
default_color!(default_dark_input_bg, 20, 20, 25);
default_color!(default_light_input_bg, 255, 255, 255);
default_color!(default_dark_list_bg, 25, 25, 30);
default_color!(default_light_list_bg, 250, 250, 252);
default_color!(default_dark_tab_active, 60, 60, 70);
default_color!(default_dark_tab_inactive, 35, 35, 40);
default_color!(default_light_tab_active, 230, 235, 245);
default_color!(default_light_tab_inactive, 245, 245, 250);
default_color!(default_dark_header_bg, 37, 37, 43);
default_color!(default_light_header_bg, 235, 235, 240);
default_color!(default_dark_border_color, 60, 60, 66);
default_color!(default_light_border_color, 210, 210, 220);
default_color!(default_dark_hover_bg, 56, 56, 64);
default_color!(default_light_hover_bg, 225, 235, 250);
default_color!(default_dark_slider_bg, 42, 42, 48);
default_color!(default_light_slider_bg, 220, 220, 210);
default_color!(default_dark_slider_thumb, 224, 224, 224);
default_color!(default_light_slider_thumb, 80, 80, 80);
default_color!(default_dark_switch_bg, 26, 26, 31);
default_color!(default_light_switch_bg, 230, 230, 220);
default_color!(default_dark_progress_bg, 51, 51, 51);
default_color!(default_light_progress_bg, 235, 235, 240);
default_color!(default_dark_accent, 212, 175, 55);
default_color!(default_light_accent, 0, 120, 212);
default_color!(default_dark_danger, 170, 17, 17);
default_color!(default_light_danger, 170, 17, 17);

// 日誌色彩
default_color!(default_dark_log_info, 200, 200, 200);
default_color!(default_light_log_info, 30, 30, 35);
default_color!(default_dark_log_warn, 217, 119, 6);
default_color!(default_light_log_warn, 180, 100, 0);
default_color!(default_dark_log_error, 255, 85, 85);
default_color!(default_light_log_error, 170, 17, 17);
default_color!(default_dark_log_success, 60, 180, 120);
default_color!(default_light_log_success, 5, 150, 105);
default_color!(default_dark_log_dir, 212, 175, 55);
default_color!(default_light_log_dir, 150, 110, 0);
default_color!(default_dark_log_file, 85, 255, 255);
default_color!(default_light_log_file, 0, 120, 212);

// 透明度與間距
default_float!(default_alpha_border, 0.15);
default_float!(default_alpha_panel, 0.03);
default_float!(default_alpha_backdrop, 0.6);
default_float!(default_space_sm, 10.0);
default_float!(default_space_md, 15.0);
default_float!(default_space_lg, 20.0);
default_float!(default_rounding, 4.0);
default_float!(default_pulse_speed, 1.0);

// 特效
default_color!(default_aurora_1, 255, 0, 127);
default_color!(default_aurora_2, 127, 0, 255);
default_color!(default_aurora_3, 0, 255, 255);
default_color!(default_neon_color, 0, 255, 204);

default_string!(default_progress_style, "default");

/// 外觀與視覺設定檔 (style.cfg)
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct StyleConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,

    // --- [自定義調色盤] ---
    #[serde(default = "default_dark_bg")]
    pub dark_bg: [u8; 3],
    #[serde(default = "default_dark_text")]
    pub dark_text: [u8; 3],
    #[serde(default = "default_light_bg")]
    pub light_bg: [u8; 3],
    #[serde(default = "default_light_text")]
    pub light_text: [u8; 3],

    // --- [細部色彩自定義] ---
    #[serde(default = "default_dark_label")]
    pub dark_label: [u8; 3],
    #[serde(default = "default_light_label")]
    pub light_label: [u8; 3],

    #[serde(default = "default_dark_text_muted")]
    pub dark_text_muted: [u8; 3],
    #[serde(default = "default_light_text_muted")]
    pub light_text_muted: [u8; 3],

    #[serde(default = "default_dark_btn_bg")]
    pub dark_btn_bg: [u8; 3],
    #[serde(default = "default_dark_btn_text")]
    pub dark_btn_text: [u8; 3],
    #[serde(default = "default_light_btn_bg")]
    pub light_btn_bg: [u8; 3],
    #[serde(default = "default_light_btn_text")]
    pub light_btn_text: [u8; 3],

    #[serde(default = "default_dark_input_bg")]
    pub dark_input_bg: [u8; 3],
    #[serde(default = "default_light_input_bg")]
    pub light_input_bg: [u8; 3],

    #[serde(default = "default_dark_list_bg")]
    pub dark_list_bg: [u8; 3],
    #[serde(default = "default_light_list_bg")]
    pub light_list_bg: [u8; 3],

    #[serde(default = "default_dark_tab_active")]
    pub dark_tab_active: [u8; 3],
    #[serde(default = "default_dark_tab_inactive")]
    pub dark_tab_inactive: [u8; 3],
    #[serde(default = "default_light_tab_active")]
    pub light_tab_active: [u8; 3],
    #[serde(default = "default_light_tab_inactive")]
    pub light_tab_inactive: [u8; 3],

    #[serde(default = "default_dark_header_bg")]
    pub dark_header_bg: [u8; 3],
    #[serde(default = "default_light_header_bg")]
    pub light_header_bg: [u8; 3],

    #[serde(default = "default_dark_border_color")]
    pub dark_border_color: [u8; 3],
    #[serde(default = "default_light_border_color")]
    pub light_border_color: [u8; 3],

    #[serde(default = "default_dark_hover_bg")]
    pub dark_hover_bg: [u8; 3],
    #[serde(default = "default_light_hover_bg")]
    pub light_hover_bg: [u8; 3],

    #[serde(default = "default_dark_slider_bg")]
    pub dark_slider_bg: [u8; 3],
    #[serde(default = "default_light_slider_bg")]
    pub light_slider_bg: [u8; 3],

    #[serde(default = "default_dark_slider_thumb")]
    pub dark_slider_thumb: [u8; 3],
    #[serde(default = "default_light_slider_thumb")]
    pub light_slider_thumb: [u8; 3],

    #[serde(default = "default_dark_switch_bg")]
    pub dark_switch_bg: [u8; 3],
    #[serde(default = "default_light_switch_bg")]
    pub light_switch_bg: [u8; 3],

    #[serde(default = "default_dark_progress_bg")]
    pub dark_progress_bg: [u8; 3],
    #[serde(default = "default_light_progress_bg")]
    pub light_progress_bg: [u8; 3],

    // --- [強調色與警告色] ---
    #[serde(default = "default_dark_accent")]
    pub dark_accent: [u8; 3],
    #[serde(default = "default_light_accent")]
    pub light_accent: [u8; 3],
    #[serde(default = "default_dark_danger")]
    pub dark_danger: [u8; 3],
    #[serde(default = "default_light_danger")]
    pub light_danger: [u8; 3],

    // --- [日誌色彩] ---
    #[serde(default = "default_dark_log_info")]
    pub dark_log_info: [u8; 3],
    #[serde(default = "default_light_log_info")]
    pub light_log_info: [u8; 3],

    #[serde(default = "default_dark_log_warn")]
    pub dark_log_warn: [u8; 3],
    #[serde(default = "default_light_log_warn")]
    pub light_log_warn: [u8; 3],

    #[serde(default = "default_dark_log_error")]
    pub dark_log_error: [u8; 3],
    #[serde(default = "default_light_log_error")]
    pub light_log_error: [u8; 3],

    #[serde(default = "default_dark_log_success")]
    pub dark_log_success: [u8; 3],
    #[serde(default = "default_light_log_success")]
    pub light_log_success: [u8; 3],

    #[serde(default = "default_dark_log_dir")]
    pub dark_log_dir: [u8; 3],
    #[serde(default = "default_light_log_dir")]
    pub light_log_dir: [u8; 3],

    #[serde(default = "default_dark_log_file")]
    pub dark_log_file: [u8; 3],
    #[serde(default = "default_light_log_file")]
    pub light_log_file: [u8; 3],

    // --- [透明度設定 (0.0 - 1.0)] ---
    #[serde(default = "default_alpha_border")]
    pub border_alpha: f32,
    #[serde(default = "default_alpha_panel")]
    pub panel_alpha: f32,
    #[serde(default = "default_alpha_backdrop")]
    pub backdrop_alpha: f32,

    // --- [佈局間距] ---
    #[serde(default = "default_space_sm")]
    pub space_sm: f32,
    #[serde(default = "default_space_md")]
    pub space_md: f32,
    #[serde(default = "default_space_lg")]
    pub space_lg: f32,

    // --- [造型與動畫] ---
    #[serde(default = "default_true")]
    pub btn_rounding_enabled: bool,
    #[serde(default = "default_rounding")]
    pub btn_rounding_value: f32,

    #[serde(default = "default_true")]
    pub progress_pulse_enabled: bool,
    #[serde(default = "default_pulse_speed")]
    pub progress_pulse_speed: f32,

    #[serde(default = "default_progress_style")]
    pub progress_style: String,

    // --- [特效定色] ---
    #[serde(default = "default_aurora_1")]
    pub aurora_1: [u8; 3],
    #[serde(default = "default_aurora_2")]
    pub aurora_2: [u8; 3],
    #[serde(default = "default_aurora_3")]
    pub aurora_3: [u8; 3],
    #[serde(default = "default_neon_color")]
    pub neon_color: [u8; 3],

    // --- [特定元件覆寫] ---
    #[serde(default)]
    pub instance_overrides: HashMap<String, ComponentStyle>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ComponentStyle {
    #[serde(default)]
    pub dark_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub dark_text: Option<[u8; 3]>,
    #[serde(default)]
    pub light_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub light_text: Option<[u8; 3]>,
    #[serde(default)]
    pub rounding: Option<f32>,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font_size: default_font_size(),
            dark_bg: default_dark_bg(),
            dark_text: default_dark_text(),
            light_bg: default_light_bg(),
            light_text: default_light_text(),
            dark_label: default_dark_label(),
            light_label: default_light_label(),
            dark_text_muted: default_dark_text_muted(),
            light_text_muted: default_light_text_muted(),
            dark_btn_bg: default_dark_btn_bg(),
            dark_btn_text: default_dark_btn_text(),
            light_btn_bg: default_light_btn_bg(),
            light_btn_text: default_light_btn_text(),
            dark_input_bg: default_dark_input_bg(),
            light_input_bg: default_light_input_bg(),
            dark_list_bg: default_dark_list_bg(),
            light_list_bg: default_light_list_bg(),
            dark_tab_active: default_dark_tab_active(),
            dark_tab_inactive: default_dark_tab_inactive(),
            light_tab_active: default_light_tab_active(),
            light_tab_inactive: default_light_tab_inactive(),
            dark_header_bg: default_dark_header_bg(),
            light_header_bg: default_light_header_bg(),
            dark_border_color: default_dark_border_color(),
            light_border_color: default_light_border_color(),
            dark_hover_bg: default_dark_hover_bg(),
            light_hover_bg: default_light_hover_bg(),
            dark_slider_bg: default_dark_slider_bg(),
            light_slider_bg: default_light_slider_bg(),
            dark_slider_thumb: default_dark_slider_thumb(),
            light_slider_thumb: default_light_slider_thumb(),
            dark_switch_bg: default_dark_switch_bg(),
            light_switch_bg: default_light_switch_bg(),
            dark_progress_bg: default_dark_progress_bg(),
            light_progress_bg: default_light_progress_bg(),
            dark_accent: default_dark_accent(),
            light_accent: default_light_accent(),
            dark_danger: default_dark_danger(),
            light_danger: default_light_danger(),
            dark_log_info: default_dark_log_info(),
            light_log_info: default_light_log_info(),
            dark_log_warn: default_dark_log_warn(),
            light_log_warn: default_light_log_warn(),
            dark_log_error: default_dark_log_error(),
            light_log_error: default_light_log_error(),
            dark_log_success: default_dark_log_success(),
            light_log_success: default_light_log_success(),
            dark_log_dir: default_dark_log_dir(),
            light_log_dir: default_light_log_dir(),
            dark_log_file: default_dark_log_file(),
            light_log_file: default_light_log_file(),
            aurora_1: default_aurora_1(),
            aurora_2: default_aurora_2(),
            aurora_3: default_aurora_3(),
            neon_color: default_neon_color(),
            border_alpha: default_alpha_border(),
            panel_alpha: default_alpha_panel(),
            backdrop_alpha: default_alpha_backdrop(),
            space_sm: default_space_sm(),
            space_md: default_space_md(),
            space_lg: default_space_lg(),
            btn_rounding_enabled: default_true(),
            btn_rounding_value: default_rounding(),
            progress_pulse_enabled: default_true(),
            progress_pulse_speed: default_pulse_speed(),
            progress_style: default_progress_style(),
            instance_overrides: HashMap::new(),
        }
    }
}

impl StyleConfig {
    pub fn load() -> Self {
        Self::load_with_path(std::path::Path::new("settings"))
    }

    pub fn load_with_path(dir: &std::path::Path) -> Self {
        let _ = fs::create_dir_all(dir);
        let path = dir.join("style.cfg");
        let mut config = if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str::<Self>(&content).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        };
        config.validate();
        config
    }

    pub fn save(&mut self) {
        self.save_with_path(std::path::Path::new("settings"))
    }

    pub fn validate(&mut self) {
        // --- [基礎範圍驗證] ---
        self.font_size = self.font_size.clamp(12.0, 30.0);
        self.btn_rounding_value = self.btn_rounding_value.clamp(0.0, 100.0);
        self.progress_pulse_speed = self.progress_pulse_speed.clamp(0.1, 10.0);

        // --- [欄位透明度驗證] ---
        self.border_alpha = self.border_alpha.clamp(0.01, 1.0);
        self.panel_alpha = self.panel_alpha.clamp(0.01, 1.0);
        self.backdrop_alpha = self.backdrop_alpha.clamp(0.01, 1.0);

        // --- [佈局間距驗證] ---
        self.space_sm = self.space_sm.clamp(0.0, 100.0);
        self.space_md = self.space_md.clamp(0.0, 100.0);
        self.space_lg = self.space_lg.clamp(0.0, 100.0);

        // --- [字串欄位防護] ---
        if self.theme.is_empty() {
            self.theme = default_theme();
        }
        if self.progress_style.is_empty() {
            self.progress_style = default_progress_style();
        }
    }

    pub fn save_with_path(&mut self, dir: &std::path::Path) {
        self.validate();
        let _ = fs::create_dir_all(dir);
        let path = dir.join("style.cfg");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: SecretString::default(),
            api_provider: default_api_provider(),
            model: String::new(),
            ollama_url: default_ollama_url(),
            api_base_url: String::new(),

            user_prompt: default_user_prompt(),
            system_prompt: default_system_prompt(),
            batch_size: default_batch_size(),
            batch_max_chars: default_batch_max_chars(),
            timeout: default_timeout(),
            glossary_priority: default_glossary_priority(),
            source_lang: default_source_lang(),
            target_lang: default_target_lang(),
            ui_lang: default_ui_lang(),
            output_dir: String::new(),
            pack_format: default_pack_format(),
            show_api_settings: false,
            show_developer_mode: false,
            show_debug_tools: false,
            skip_json: false,
            skip_js: false,
            skip_jar: false,
            skip_book: false,
            enable_llm_log: false,
            enable_debug_log: false,
            main_x: 50.0,
            main_y: 50.0,
            main_width: 800.0,
            main_height: 600.0,
            viewer_x: 100.0,
            viewer_y: 100.0,
            viewer_width: 800.0,
            viewer_height: 600.0,
            excluded_paths: default_excluded_paths(),
            fast_convert: false,
        }
    }
}

impl AppConfig {
    /// 載入設定：從系統憑證 (Keyring) 讀取 API_KEY，其餘從 config.cfg 讀取
    pub fn load() -> Self {
        Self::load_with_path(std::path::Path::new("settings"))
    }

    pub fn load_with_path(dir: &std::path::Path) -> Self {
        let _ = fs::create_dir_all(dir);

        #[allow(unused_mut)]
        let mut config = Self::load_from_config_cfg_path(dir);

        // 避免測試過程中修改真實 Keyring
        #[cfg(not(test))]
        if let Ok(key) = get_api_key() {
            config.api_key = key.into();
        }

        config
    }

    fn load_from_config_cfg_path(dir: &std::path::Path) -> Self {
        let path = dir.join("config.cfg");
        let mut config = if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str::<Self>(&content).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        };
        config.validate();
        config
    }

    pub fn save(&mut self) {
        self.save_with_path(std::path::Path::new("settings"))
    }

    pub fn validate(&mut self) {
        // --- [核心參數校正] ---
        if self.batch_size == 0 {
            self.batch_size = default_batch_size();
        }
        self.batch_size = self.batch_size.clamp(1, 500);

        if self.batch_max_chars == 0 {
            self.batch_max_chars = default_batch_max_chars();
        }
        self.batch_max_chars = self.batch_max_chars.clamp(1, 20000);

        if self.timeout == 0 {
            self.timeout = default_timeout();
        }
        self.timeout = self.timeout.clamp(1, 300);

        if self.pack_format == 0 {
            self.pack_format = default_pack_format();
        }
        self.pack_format = self.pack_format.clamp(1, 128);

        // --- [字串欄位防護] ---
        if self.api_provider.is_empty() {
            self.api_provider = default_api_provider();
        }
        if self.ollama_url.is_empty() {
            self.ollama_url = default_ollama_url();
        }
        if self.source_lang.is_empty() {
            self.source_lang = default_source_lang();
        }
        if self.target_lang.is_empty() {
            self.target_lang = default_target_lang();
        }
        if self.user_prompt.is_empty() {
            self.user_prompt = default_user_prompt();
        }
        if self.system_prompt.is_empty() {
            self.system_prompt = default_system_prompt();
        }
        if self.glossary_priority.is_empty() {
            self.glossary_priority = default_glossary_priority();
        }
    }

    pub fn save_with_path(&mut self, dir: &std::path::Path) {
        self.validate();
        let _ = fs::create_dir_all(dir);

        // 避免測試過程中修改真實 Keyring
        #[cfg(not(test))]
        let _ = save_api_key(self.api_key.expose_secret());

        let path = dir.join("config.cfg");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.api_provider, "無");
        assert_eq!(config.main_width, 800.0);
        assert_eq!(config.main_height, 600.0);
        assert_eq!(config.batch_size, 150);
    }

    #[test]
    fn test_style_config_defaults() {
        let config = StyleConfig::default();
        assert_eq!(config.theme, "dark");
        assert_eq!(config.font_size, 15.0);
        assert_eq!(config.dark_bg, [30, 30, 35]);
    }

    #[test]
    fn test_config_load_save_cycles() {
        let temp_dir = std::env::temp_dir().join("mc_translator_settings_test_cycle");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 1. StyleConfig
        let mut style = StyleConfig {
            font_size: 20.0,
            ..StyleConfig::default()
        };
        style.save_with_path(&temp_dir);

        let loaded_style = StyleConfig::load_with_path(&temp_dir);
        assert_eq!(loaded_style.font_size, 20.0);

        // 2. AppConfig
        let mut app = AppConfig {
            batch_size: 300,
            ..AppConfig::default()
        };
        app.save_with_path(&temp_dir);

        let loaded_app = AppConfig::load_with_path(&temp_dir);
        assert_eq!(loaded_app.batch_size, 300);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_config_deserialization_defaults() {
        // AppConfig 缺 ui_lang
        let app_json = r#"{"api_provider": "Gemini", "model": "m", "ollama_url": "u", "user_prompt": "p", "system_prompt": "s", "batch_size": 1, "batch_max_chars": 1, "timeout": 1, "glossary_priority": "p", "source_lang": "s", "target_lang": "t", "output_dir": "o", "pack_format": 1, "show_api_settings": false, "show_developer_mode": false, "skip_json": false, "skip_js": false, "skip_jar": false, "skip_book": false, "enable_llm_log": false, "main_x": 0.0, "main_y": 0.0, "main_width": 0.0, "main_height": 0.0, "viewer_x": 0.0, "viewer_y": 0.0, "viewer_width": 0.0, "viewer_height": 0.0}"#;
        let app: AppConfig = serde_json::from_str(app_json).unwrap();
        assert_eq!(app.ui_lang, "zh_tw");

        // StyleConfig 缺 progress_pulse_enabled 以及其它
        let style_json = r#"{"theme": "dark", "font_size": 12.0, "dark_bg": [0,0,0], "dark_text":[0,0,0], "light_bg":[0,0,0], "light_text":[0,0,0], "dark_label":[0,0,0], "light_label":[0,0,0], "dark_btn_bg":[0,0,0], "dark_btn_text":[0,0,0], "light_btn_bg":[0,0,0], "light_btn_text":[0,0,0], "dark_input_bg":[0,0,0], "light_input_bg":[0,0,0], "dark_list_bg":[0,0,0], "light_list_bg":[0,0,0], "dark_tab_active":[0,0,0], "dark_tab_inactive":[0,0,0], "light_tab_active":[0,0,0], "light_tab_inactive":[0,0,0], "btn_rounding_enabled": true, "btn_rounding_value": 0.0, "progress_pulse_speed": 0.0, "instance_overrides": {}}"#;
        let style: StyleConfig = serde_json::from_str(style_json).unwrap();
        assert!(style.progress_pulse_enabled);
        assert_eq!(style.aurora_1, [255, 0, 127]);
        assert_eq!(style.aurora_2, [127, 0, 255]);
        assert_eq!(style.aurora_3, [0, 255, 255]);
        assert_eq!(style.neon_color, [0, 255, 204]);
    }

    #[test]
    fn test_config_validation() {
        // 1. AppConfig 驗證
        let mut app = AppConfig {
            batch_size: 999,
            timeout: 0,
            batch_max_chars: 50000,
            pack_format: 200,
            ..AppConfig::default()
        };
        app.validate();
        assert_eq!(app.batch_size, 500);
        assert_eq!(app.timeout, 60);
        assert_eq!(app.batch_max_chars, 20000);
        assert_eq!(app.pack_format, 128);

        // 2. StyleConfig 驗證
        let mut style = StyleConfig {
            font_size: 5.0,
            btn_rounding_value: 200.0,
            progress_pulse_speed: 50.0,
            border_alpha: 0.0,
            space_sm: -10.0,
            ..StyleConfig::default()
        };
        style.validate();
        assert_eq!(style.font_size, 12.0);
        assert_eq!(style.btn_rounding_value, 100.0);
        assert_eq!(style.progress_pulse_speed, 10.0);
        assert_eq!(style.border_alpha, 0.01);
        assert_eq!(style.space_sm, 0.0);
    }

    #[test]
    fn test_app_config_partial_deserialization() {
        // 缺少 api_provider, ollama_url, source_lang, target_lang 等關鍵欄位
        let json = r#"{"model": "test", "batch_size": 50}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_provider, "無");
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.source_lang, "en_us");
        assert_eq!(config.target_lang, "zh_tw");
        assert_eq!(config.user_prompt, DEFAULT_PROMPT);
        assert_eq!(config.system_prompt, DEFAULT_SYSTEM_PROMPT);
        assert_eq!(config.batch_size, 50);
        assert_eq!(config.glossary_priority, "official");
        assert_eq!(config.pack_format, 15);
    }

    #[test]
    fn test_style_config_partial_deserialization() {
        // 缺少 theme, font_size, btn_rounding_enabled
        let json = r#"{"dark_bg": [10, 10, 10]}"#;
        let config: StyleConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.theme, "dark");
        assert_eq!(config.font_size, 15.0);
        assert!(config.btn_rounding_enabled);
        assert_eq!(config.dark_bg, [10, 10, 10]);
    }

    #[test]
    fn test_validate_corrects_empty_strings() {
        let mut app = AppConfig {
            api_provider: String::new(),
            ollama_url: String::new(),
            source_lang: String::new(),
            target_lang: String::new(),
            user_prompt: String::new(),
            system_prompt: String::new(),
            glossary_priority: String::new(),
            ..AppConfig::default()
        };
        app.validate();
        assert_eq!(app.api_provider, "無");
        assert_eq!(app.ollama_url, "http://localhost:11434");
        assert_eq!(app.source_lang, "en_us");
        assert_eq!(app.target_lang, "zh_tw");
        assert_eq!(app.user_prompt, DEFAULT_PROMPT);
        assert_eq!(app.system_prompt, DEFAULT_SYSTEM_PROMPT);
        assert_eq!(app.glossary_priority, "official");
    }

    #[test]
    fn test_style_validate_corrects_empty_theme() {
        let mut style = StyleConfig {
            theme: String::new(),
            ..StyleConfig::default()
        };
        style.validate();
        assert_eq!(style.theme, "dark");
    }

    #[test]
    fn test_load_validates_on_load() {
        let temp_dir = std::env::temp_dir().join("mc_translator_settings_validate_on_load");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 寫入一個缺少關鍵欄位的設定檔
        let json = r#"{"batch_size": 0, "api_provider": ""}"#;
        fs::write(temp_dir.join("config.cfg"), json).unwrap();

        let config = AppConfig::load_with_path(&temp_dir);
        // validate() 應該在 load 時被呼叫，校正空字串與 0 值
        assert_eq!(config.api_provider, "無");
        assert_eq!(config.batch_size, 150);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
