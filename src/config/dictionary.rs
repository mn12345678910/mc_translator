//! # 辭典模組
//! 負責辭典檔案的載入與儲存邏輯。

use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

pub const DICT_DIR: &str = "dicts";

pub fn get_user_dict_path(lang: &str) -> std::path::PathBuf {
    std::path::Path::new(DICT_DIR)
        .join("user")
        .join(format!("{}.json", lang))
}

pub fn get_official_dict_path(lang: &str) -> std::path::PathBuf {
    std::path::Path::new(DICT_DIR)
        .join("official")
        .join(format!("{}.json", lang))
}

/// 確保辭典目錄存在
pub fn ensure_dicts_dir() {
    let _ = fs::create_dir_all(DICT_DIR);
}

static DICT_LOCK: RwLock<()> = RwLock::new(());

/// 泛型載入辭典檔案（受讀鎖保護，防止與 save_dict 並發衝突）
pub fn load_dict<T: serde::de::DeserializeOwned + Default>(path: &std::path::Path) -> T {
    let _read_guard = DICT_LOCK.read().unwrap_or_else(|e| e.into_inner());
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        T::default()
    }
}

/// 泛型儲存辭典檔案（受寫鎖保護）
pub fn save_dict<T: serde::Serialize>(path: &std::path::Path, data: &T) {
    let _write_guard = DICT_LOCK.write().unwrap_or_else(|e| e.into_inner());

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

/// 獲取所有可用的辭典語言
/// 掃描 dicts/ 根目錄（向後相容）與 official/、user/ 子目錄
pub fn get_available_dict_langs() -> Vec<String> {
    let mut langs = std::collections::HashSet::new();

    let base = std::path::Path::new(DICT_DIR);

    // 1. 掃描根目錄（向後相容舊格式）
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    langs.insert(stem.to_string_lossy().to_string());
                }
            }
        }
    }

    // 2. 掃描子目錄（official/ 與 user/）
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let dir_path = entry.path();
            if dir_path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&dir_path) {
                    for sub_entry in sub_entries.flatten() {
                        let path = sub_entry.path();
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                            if let Some(stem) = path.file_stem() {
                                langs.insert(stem.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if langs.is_empty() {
        langs.insert("zh_tw".to_string());
        langs.insert("zh_cn".to_string());
        langs.insert("ja_jp".to_string());
        langs.insert("en_us".to_string());
    }

    let mut langs_vec: Vec<String> = langs.into_iter().collect();
    langs_vec.sort();
    langs_vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn setup_test_dir() {
        let _ = fs::remove_dir_all(DICT_DIR);
        ensure_dicts_dir();
    }

    fn teardown_test_dir() {
        let _ = fs::remove_dir_all(DICT_DIR);
    }

    /// 1. 正常路徑測試（正常流程）
    #[test]
    fn test_save_load_dict_standard() {
        let _lock = TEST_LOCK.lock().unwrap();
        setup_test_dir();
        const TEST_DICT: &str = "test_temp_dictionary_standard.json";
        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        let mut data = HashMap::new();
        data.insert("Apple".to_string(), "蘋果".to_string());

        save_dict(&path, &data);

        let loaded: HashMap<String, String> = load_dict(&path);
        assert_eq!(loaded.get("Apple").unwrap(), "蘋果");

        teardown_test_dir();
    }

    /// 2. 邊界值與 UTF-8 測試（邊界案例 / UTF-8）
    #[test]
    fn test_save_load_dict_utf8_edge() {
        let _lock = TEST_LOCK.lock().unwrap();
        setup_test_dir();
        const TEST_DICT: &str = "test_temp_dictionary_utf8.json";
        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        let mut data = HashMap::new();
        data.insert("❄️ Ice".to_string(), "冰塊".to_string());

        save_dict(&path, &data);

        let loaded: HashMap<String, String> = load_dict(&path);
        assert_eq!(loaded.get("❄️ Ice").unwrap(), "冰塊");

        teardown_test_dir();
    }

    /// 3. 強韌性與異常處理（健壯性 / 負向案例）
    #[test]
    fn test_load_corrupt_dict_fallback() {
        let _lock = TEST_LOCK.lock().unwrap();
        setup_test_dir();
        const TEST_DICT: &str = "test_temp_dictionary_corrupt.json";
        let path = std::path::Path::new(DICT_DIR).join(TEST_DICT);
        let _ = std::fs::write(&path, "{ invalid_json: ");

        let loaded: HashMap<String, String> = load_dict(&path);
        assert!(loaded.is_empty());

        teardown_test_dir();
    }

    #[test]
    fn test_dictionary_paths_and_dirs() {
        let lang = "en_us";
        let path = get_user_dict_path(lang);
        assert!(path.to_string_lossy().contains("dicts"));
        assert!(path.to_string_lossy().contains("user"));
    }

    #[test]
    fn test_load_dict_file_not_found_fallback() {
        let path = std::path::Path::new("non_existent_file_xyz_123.json");
        let loaded: HashMap<String, String> = load_dict(path);
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_translation_memory_load_save_cycle() {
        let _lock = TEST_LOCK.lock().unwrap();
        setup_test_dir();
        let lang = "test_memory_cycle";
        let mut memory = HashMap::new();
        memory.insert("Door".to_string(), "門".to_string());

        save_translation_memory(lang, &memory);

        let loaded = load_translation_memory(lang);
        assert_eq!(loaded.get("Door").unwrap(), "門");

        teardown_test_dir();
    }

    #[test]
    fn test_get_available_dict_langs_scans_subdirs() {
        let _lock = TEST_LOCK.lock().unwrap();
        setup_test_dir();

        // 根目錄放一個（向後相容）
        fs::write(std::path::Path::new(DICT_DIR).join("zh_cn.json"), "{}").unwrap();
        // official 子目錄
        let official = std::path::Path::new(DICT_DIR).join("official");
        fs::create_dir_all(&official).unwrap();
        fs::write(official.join("zh_tw.json"), "{}").unwrap();
        // user 子目錄
        let user = std::path::Path::new(DICT_DIR).join("user");
        fs::create_dir_all(&user).unwrap();
        fs::write(user.join("en_us.json"), "{}").unwrap();

        let langs = get_available_dict_langs();
        assert!(langs.contains(&"zh_cn".to_string()));
        assert!(langs.contains(&"zh_tw".to_string()));
        assert!(langs.contains(&"en_us".to_string()));
        assert_eq!(langs.len(), 3);

        teardown_test_dir();
    }

    #[test]
    fn test_get_available_dict_langs_fallback() {
        let _lock = TEST_LOCK.lock().unwrap();
        teardown_test_dir();
        assert!(!std::path::Path::new(DICT_DIR).exists());

        let langs = get_available_dict_langs();
        assert!(langs.contains(&"zh_tw".to_string()));
        assert!(langs.contains(&"zh_cn".to_string()));
        assert!(langs.contains(&"ja_jp".to_string()));
        assert!(langs.contains(&"en_us".to_string()));
    }
}
