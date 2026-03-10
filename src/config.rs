//! # 設定模組
//! 負責 `.env` 環境變數的讀寫邏輯，並使用 Windows DPAPI 加密 API 金鑰。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use base64::{engine::general_purpose, Engine as _};
#[cfg(target_os = "windows")]
use std::ptr;
#[cfg(target_os = "windows")]
use winapi::um::dpapi::{CryptProtectData, CryptUnprotectData};
#[cfg(target_os = "windows")]
use winapi::um::wincrypt::DATA_BLOB;

pub const DEFAULT_PROMPT: &str = "你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。";

/// 應用程式設定結構體
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    /// API 金鑰（Gemini / OpenAI）
    pub api_key: String,
    /// 服務商（Gemini / OpenAI / Ollama）
    pub provider: String,
    /// 所選模型名稱
    pub model: String,
    /// Ollama 服務端 URL
    pub ollama_url: String,

    /// 批量翻譯大小（1-300）
    pub batch_size: u32,
    /// 批次最大字元數 (預設 3500)
    pub batch_max_chars: u32,
    /// Ollama 逾時秒數（1-300）
    pub ollama_timeout: u32,
    /// 資源包輸出資料夾
    pub output_dir: String,
    /// 自訂翻譯提示
    pub translation_prompt: String,
    /// 介面主題 (light / dark)
    pub theme: String,
    /// 資源包版本號 (pack_format)
    pub pack_format: u32,
    /// 字體大小
    pub font_size: f32,
    /// 術語優先級 (official / user)
    pub glossary_priority: String,
    /// 跳過 .json
    pub skip_json: bool,
    /// 跳過 .js
    pub skip_js: bool,
    /// 跳過 .jar
    pub skip_jar: bool,
    /// 跳過 patchouli_books
    pub skip_book: bool,
    /// 開啟 LLM 通訊紀錄
    pub enable_llm_log: bool,
    /// 建議詞管理器視窗 X 座標
    pub viewer_x: f32,
    /// 建議詞管理器視窗 Y 座標
    pub viewer_y: f32,
    /// 建議詞管理器視窗寬度
    pub viewer_width: f32,
    /// 建議詞管理器視窗高度
    pub viewer_height: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            provider: "Gemini".to_string(),
            model: String::new(),
            ollama_url: "http://localhost:11434".to_string(),
            batch_size: 100,
            batch_max_chars: 3500,
            ollama_timeout: 60,
            output_dir: String::new(),
            translation_prompt: DEFAULT_PROMPT.to_string(),
            theme: "dark".to_string(),
            pack_format: 15,
            font_size: 15.0,
            glossary_priority: "official".to_string(),
            skip_json: false,
            skip_js: false,
            skip_jar: false,
            skip_book: false,
            enable_llm_log: false,
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
        let mut config_to_save = self.clone();
        config_to_save.api_key = String::new(); // 不要把金鑰存進 cfg
        if let Ok(json) = serde_json::to_string_pretty(&config_to_save) {
            let _ = fs::write("config.cfg", json);
        }
    }
}

pub const DICT_DIR: &str = "dicts";
pub const USER_DICT: &str = "user.json";
pub const OFFICIAL_DICT: &str = "official.json";

/// 確保辭典目錄存在
pub fn ensure_dicts_dir() {
    let _ = fs::create_dir_all(DICT_DIR);
}

/// 泛型載入辭典檔案
pub fn load_dict<T: serde::de::DeserializeOwned + Default>(filename: &str) -> T {
    ensure_dicts_dir();
    let path = std::path::Path::new(DICT_DIR).join(filename);
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        T::default()
    }
}

/// 泛型儲存辭典檔案
pub fn save_dict<T: serde::Serialize>(filename: &str, data: &T) {
    ensure_dicts_dir();
    let path = std::path::Path::new(DICT_DIR).join(filename);
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = fs::write(path, json);
    }
}

/// 載入使用者建議詞
pub fn load_translation_memory() -> HashMap<String, String> {
    load_dict(USER_DICT)
}

/// 儲存翻譯記憶體檔案 (使用者建議詞)
pub fn save_translation_memory(memory: &HashMap<String, String>) {
    save_dict(USER_DICT, memory);
}

/// 使用 Windows DPAPI 加密字串
#[cfg(target_os = "windows")]
fn encrypt_string(data: &str) -> Result<String, String> {
    let bytes = data.as_bytes();
    if bytes.is_empty() {
        return Ok(String::new());
    }
    assert!(bytes.len() <= u32::MAX as usize, "Data too large for DPAPI");

    let mut input = DATA_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = DATA_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    unsafe {
        let result = CryptProtectData(
            &mut input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut output,
        );

        if result != 0 {
            let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
            let encoded = general_purpose::STANDARD.encode(slice);
            winapi::um::winbase::LocalFree(output.pbData as *mut _);
            Ok(encoded)
        } else {
            Err("CryptProtectData failed".to_string())
        }
    }
}

/// 非 Windows 环境下，不進行 DPAPI 加密，直接回傳 Base64 對原始資料加碼
#[cfg(not(target_os = "windows"))]
fn encrypt_string(data: &str) -> Result<String, String> {
    Ok(general_purpose::STANDARD.encode(data.as_bytes()))
}

/// 使用 Windows DPAPI 解密字串
#[cfg(target_os = "windows")]
fn decrypt_string(encoded_data: &str) -> Result<String, String> {
    let decoded = general_purpose::STANDARD
        .decode(encoded_data)
        .map_err(|_| "Base64 decode failed".to_string())?;

    if decoded.is_empty() {
        return Ok(String::new());
    }
    assert!(
        decoded.len() <= u32::MAX as usize,
        "Data too large for DPAPI unprotect"
    );

    let mut input = DATA_BLOB {
        cbData: decoded.len() as u32,
        pbData: decoded.as_ptr() as *mut u8,
    };
    let mut output = DATA_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    unsafe {
        let result = CryptUnprotectData(
            &mut input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut output,
        );

        if result != 0 {
            let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
            let decoded_str = String::from_utf8_lossy(slice).into_owned();
            winapi::um::winbase::LocalFree(output.pbData as *mut _);
            Ok(decoded_str)
        } else {
            Err("CryptUnprotectData failed".to_string())
        }
    }
}

/// 非 Windows 环境下，將 Base64 解碼回原始字串
#[cfg(not(target_os = "windows"))]
fn decrypt_string(encoded_data: &str) -> Result<String, String> {
    let decoded = general_purpose::STANDARD
        .decode(encoded_data)
        .map_err(|_| "Base64 decode failed".to_string())?;
    String::from_utf8(decoded).map_err(|e| e.to_string())
}
