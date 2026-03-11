//! # 辭典模組
//! 負責辭典檔案的載入與儲存邏輯。

use std::collections::HashMap;
use std::fs;

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
