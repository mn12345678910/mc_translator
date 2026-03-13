//! # 設定模組
//! 負責 AppConfig 結構體定義、`.env` 環境變數的讀寫邏輯。

use serde::{Deserialize, Serialize};
use std::fs;

use super::encryption::{decrypt_string, encrypt_string};

pub const DEFAULT_PROMPT: &str = "你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。";

/// 應用程式設定結構體
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    /// API 金鑰（Gemini / OpenAI）
    #[serde(skip)]
    pub api_key: String,

    // --- [核心 API 設定] ---
    #[serde(rename = "服務提供商", alias = "api_provider")]
    pub api_provider: String,
    #[serde(rename = "模型名稱", alias = "model")]
    pub model: String,
    #[serde(rename = "Ollama位址", alias = "ollama_url")]
    pub ollama_url: String,

    // --- [Prompt 集] ---
    #[serde(rename = "使用者翻譯提示", alias = "user_prompt")]
    pub user_prompt: String,
    #[serde(rename = "系統技術指令", alias = "system_prompt")]
    pub system_prompt: String,

    // --- [翻譯參數] ---
    #[serde(rename = "批次量", alias = "batch_size")]
    pub batch_size: u32,
    #[serde(rename = "批次字數上限", alias = "batch_max_chars")]
    pub batch_max_chars: u32,
    #[serde(rename = "API逾時秒數", alias = "ollama_timeout")]
    pub ollama_timeout: u32,
    #[serde(rename = "術語優先級", alias = "glossary_priority")]
    pub glossary_priority: String,

    // --- [輸出與介面] ---
    #[serde(rename = "輸出路徑", alias = "output_dir")]
    pub output_dir: String,
    #[serde(rename = "主題顏色", alias = "theme")]
    pub theme: String,
    #[serde(rename = "字體大小", alias = "font_size")]
    pub font_size: f32,
    #[serde(rename = "資源包版本", alias = "pack_format")]
    pub pack_format: u32,

    // --- [效能與面板狀態] ---
    #[serde(rename = "自定義FPS開關", alias = "enable_custom_fps")]
    pub enable_custom_fps: bool,
    #[serde(rename = "自定義FPS數值", alias = "custom_fps")]
    pub custom_fps: u32,
    #[serde(rename = "顯示API設定", alias = "show_api_settings")]
    pub show_api_settings: bool,
    #[serde(rename = "顯示開發者模式", alias = "show_developer_mode")]
    pub show_developer_mode: bool,

    // --- [檔案篩選] ---
    #[serde(rename = "跳過JSON", alias = "skip_json")]
    pub skip_json: bool,
    #[serde(rename = "跳過JS", alias = "skip_js")]
    pub skip_js: bool,
    #[serde(rename = "跳過JAR", alias = "skip_jar")]
    pub skip_jar: bool,
    #[serde(rename = "跳過手冊", alias = "skip_book")]
    pub skip_book: bool,
    #[serde(rename = "記錄LLM通訊", alias = "enable_llm_log")]
    pub enable_llm_log: bool,

    // --- [視窗幾何資訊] ---
    #[serde(rename = "主視窗X", alias = "main_x")]
    pub main_x: f32,
    #[serde(rename = "主視窗Y", alias = "main_y")]
    pub main_y: f32,
    #[serde(rename = "主視窗寬度", alias = "main_width")]
    pub main_width: f32,
    #[serde(rename = "主視窗高度", alias = "main_height")]
    pub main_height: f32,
    #[serde(rename = "建議詞視窗X", alias = "viewer_x")]
    pub viewer_x: f32,
    #[serde(rename = "建議詞視窗Y", alias = "viewer_y")]
    pub viewer_y: f32,
    #[serde(rename = "建議詞視窗寬度", alias = "viewer_width")]
    pub viewer_width: f32,
    #[serde(rename = "建議詞視窗高度", alias = "viewer_height")]
    pub viewer_height: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_provider: "無".to_string(),
            model: String::new(),
            ollama_url: "http://localhost:11434".to_string(),
            user_prompt: DEFAULT_PROMPT.to_string(),
            system_prompt: "\n\n[內部技術指令 - 請務必遵守]\n\
1. 僅針對 %%VAR_n%%, %%MC_n%%, %%HEX_n%% 等技術佔位符執行「保持原樣」操作（不可修改、翻譯或增刪標籤）。\n\
2. 除上述佔位符外的其餘文本內容均「必須」按要求翻譯，絕對不可將全文原樣輸出。".to_string(),
            batch_size: 150,
            batch_max_chars: 3500,
            ollama_timeout: 60,
            glossary_priority: "official".to_string(),
            output_dir: String::new(),
            theme: "dark".to_string(),
            font_size: 15.0,
            pack_format: 15,
            enable_custom_fps: false,
            custom_fps: 60,
            show_api_settings: false,
            show_developer_mode: false,
            skip_json: false,
            skip_js: false,
            skip_jar: false,
            skip_book: false,
            enable_llm_log: false,
            main_x: 50.0,
            main_y: 50.0,
            main_width: 750.0,
            main_height: 550.0,
            viewer_x: 100.0,
            viewer_y: 100.0,
            viewer_width: 750.0,
            viewer_height: 500.0,
        }
    }
}

impl AppConfig {
    /// 載入設定：從 .env 讀取 API_KEY，其餘從 config.cfg 讀取
    pub fn load() -> Self {
        dotenvy::dotenv_override().ok();

        let mut config = Self::load_from_config_cfg();

        // 覆蓋 API_KEY
        let raw_key = std::env::var("API_KEY").unwrap_or_default();
        let decrypted_key = if let Some(stripped) = raw_key.strip_prefix("DPAPI:") {
            decrypt_string(stripped).unwrap_or(raw_key)
        } else {
            raw_key
        };
        config.api_key = decrypted_key;

        config
    }

    /// 從 config.cfg 載入非敏感設定
    fn load_from_config_cfg() -> Self {
        if let Ok(content) = fs::read_to_string("config.cfg") {
            if let Ok(config) = serde_json::from_str::<Self>(&content) {
                return config;
            }
        }
        Self::default()
    }

    /// 儲存設定：API_KEY 至 .env，其餘至 config.cfg
    pub fn save(&self) {
        // 1. 儲存 API_KEY 於 .env
        let encrypted_key = if !self.api_key.is_empty() {
            match encrypt_string(&self.api_key) {
                Ok(enc) => format!("DPAPI:{}", enc),
                Err(_) => self.api_key.clone(),
            }
        } else {
            String::new()
        };
        let _ = fs::write(".env", format!("API_KEY={}", encrypted_key));

        // 2. 儲存其餘設定於 config.cfg
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write("config.cfg", json);
        }
    }
}
