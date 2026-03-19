//! # 辭典模組
//! 負責辭典檔案的載入與儲存邏輯。

use std::collections::HashMap;
use std::fs;

pub const DICT_DIR: &str = "dicts";

pub fn get_user_dict_path(lang: &str) -> std::path::PathBuf {
    std::path::Path::new(DICT_DIR).join("user").join(format!("{}.json", lang))
}

pub fn get_official_dict_path(lang: &str) -> std::path::PathBuf {
    std::path::Path::new(DICT_DIR).join("official").join(format!("{}.json", lang))
}

/// 確保辭典目錄存在
pub fn ensure_dicts_dir() {
    let _ = fs::create_dir_all(DICT_DIR);
}

/// 泛型載入辭典檔案
pub fn load_dict<T: serde::de::DeserializeOwned + Default>(path: &std::path::Path) -> T {
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        T::default()
    }
}

static DICT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 泛型儲存辭典檔案
pub fn save_dict<T: serde::Serialize>(path: &std::path::Path, data: &T) {
    let _guard = DICT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = fs::write(path, json);
    }
}

/// 載入使用者建議詞
pub fn load_translation_memory(lang: &str) -> HashMap<String, String> {
    load_dict(&get_user_dict_path(lang))
}

/// 儲存翻譯記憶體檔案 (使用者建議詞)
pub fn save_translation_memory(lang: &str, memory: &HashMap<String, String>) {
    save_dict(&get_user_dict_path(lang), memory);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 1. 正常路徑測試（正常流程）
    #[test]
    fn test_save_load_dict_standard() {
        const TEST_DICT: &str = "test_temp_dictionary_standard.json";
        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
        let mut data = HashMap::new();
        data.insert("Apple".to_string(), "蘋果".to_string());
        
        save_dict(&path, &data);
        
        let loaded: HashMap<String, String> = load_dict(&path);
        assert_eq!(loaded.get("Apple").unwrap(), "蘋果");

        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        let _ = std::fs::remove_file(path);
    }

    /// 2. 邊界值與 UTF-8 測試（邊界案例 / UTF-8）
    #[test]
    fn test_save_load_dict_utf8_edge() {
        const TEST_DICT: &str = "test_temp_dictionary_utf8.json";
        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
        let mut data = HashMap::new();
        data.insert("❄️ Ice".to_string(), "冰塊".to_string());
        
        save_dict(&path, &data);
        
        let loaded: HashMap<String, String> = load_dict(&path);
        assert_eq!(loaded.get("❄️ Ice").unwrap(), "冰塊");

        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        let _ = std::fs::remove_file(path);
    }

    /// 3. 強韌性與異常處理（健壯性 / 負向案例）
    #[test]
    fn test_load_corrupt_dict_fallback() {
        const TEST_DICT: &str = "test_temp_dictionary_corrupt.json";
        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
        let _ = std::fs::write(&path, "{ invalid_json: ");
        
        let loaded: HashMap<String, String> = load_dict(&path);
        assert!(loaded.is_empty());

        let _ = std::fs::remove_file(path);
    }
}
