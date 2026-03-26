//! # 輔助函式
//! 包含日誌、路徑顯示等通用工具函式。

use crate::translation::{LogEntry, LogLevel};
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{GlossaryEntry, TermType};

pub fn extract_display_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(pos) = path_str.find("assets") {
        let after_assets = &path_str[pos + 7..];
        let parts: Vec<&str> = after_assets
            .split(['/', '\\'])
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() >= 2 {
            return format!("{}/{}", parts[0], parts.last().unwrap_or(&"en_us.json"));
        }
    }
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub fn get_log_dir() -> PathBuf {
    let mut path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    // 如果是在開發環境 (Cargo run)，嘗試退回專案根目錄
    if path.ends_with("target\\debug") || path.ends_with("target\\release") {
        if let Some(parent) = path.parent().and_then(|p| p.parent()) {
            path = parent.to_path_buf();
        }
    } else if path.ends_with("src-tauri") {
        if let Some(parent) = path.parent() {
            path = parent.to_path_buf();
        }
    }

    path.join("logs")
}

pub fn add_log_event(
    log_arc: &Arc<Mutex<Vec<LogEntry>>>,
    level: LogLevel,
    msg: &str,
    source_lang: &str,
    target_lang: &str,
    file_name: &str,
    enable_persistence: bool,
) {
    let now = Local::now();
    let timestamp_ms = now.timestamp_millis();
    let timestamp_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let lang_info = if source_lang.is_empty() && target_lang.is_empty() {
        String::new()
    } else {
        format!("<{}->{}> ", source_lang, target_lang)
    };

    let file_info = if file_name.is_empty() {
        String::new()
    } else {
        format!("[{}] ", file_name)
    };

    let mut log = log_arc.lock().unwrap();
    let full_msg = format!("{}{}{}", lang_info, file_info, msg);

    log.push(LogEntry {
        level: level.clone(),
        message: full_msg.clone(),
        timestamp: timestamp_ms,
    });

    if enable_persistence {
        let log_dir = get_log_dir();
        if fs::create_dir_all(&log_dir).is_ok() {
            let log_file = log_dir.join("debug.log");
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
                let level_str = match level {
                    LogLevel::Info => "INFO",
                    LogLevel::Success => "SUCCESS",
                    LogLevel::Warn => "WARN",
                    LogLevel::Error => "ERROR",
                };
                let _ = writeln!(file, "[{}] [{}] {}", timestamp_str, level_str, full_msg);
            }
        }
    }
}

pub fn add_log(
    log_arc: &Arc<Mutex<Vec<String>>>,
    msg: &str,
    source_lang: &str,
    target_lang: &str,
    file_name: &str,
) {
    let mut log = log_arc.lock().unwrap();
    let now = chrono::Local::now();
    let timestamp = now.format("%H:%M:%S").to_string();

    let lang_info = if source_lang.is_empty() && target_lang.is_empty() {
        String::new()
    } else {
        format!("<{}->{}> ", source_lang, target_lang)
    };

    let file_info = if file_name.is_empty() {
        String::new()
    } else {
        format!("[{}] ", file_name)
    };

    for line in msg.lines() {
        if !line.trim().is_empty() {
            log.push(format!(
                "[{}] {}{}{}",
                timestamp, lang_info, file_info, line
            ));
        } else {
            log.push("".to_string());
        }
    }
}

pub fn format_log_message(msg: &str) -> Vec<String> {
    let mut log_entries = Vec::new();
    let now = chrono::Local::now();
    let timestamp = now.format("%H:%M:%S").to_string();
    for line in msg.lines() {
        if !line.trim().is_empty() {
            log_entries.push(format!("[{}] {}", timestamp, line));
        } else {
            log_entries.push("".to_string());
        }
    }
    log_entries
}

pub fn hashmap_to_entries(
    map: &std::collections::HashMap<String, (String, TermType)>,
) -> Vec<GlossaryEntry> {
    let mut entries: Vec<GlossaryEntry> = map
        .iter()
        .map(|(k, (v, t))| GlossaryEntry {
            original: k.clone(),
            translated: v.clone(),
            source: t.clone(),
        })
        .collect();
    entries.sort_by(|a, b| b.original.len().cmp(&a.original.len()));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 1. 正常路徑測試（正常流程）
    #[test]
    fn test_extract_display_path_standard() {
        let path = std::path::Path::new("some/assets/minecraft/lang/en_us.json");
        assert_eq!(extract_display_path(path), "minecraft/en_us.json");
    }

    /// 2. 邊界值與 UTF-8 測試（邊界案例 / UTF-8）
    #[test]
    fn test_extract_display_path_utf8_edge() {
        // 包含中文字元與空白
        let path = std::path::Path::new("assets/範例 目錄/en_us.json");
        assert_eq!(extract_display_path(path), "範例 目錄/en_us.json");
    }

    /// 3. 強韌性與異常處理（健壯性 / 負向案例）
    #[test]
    fn test_extract_display_path_fallback() {
        // 沒有 assets 的路徑，應直接顯示檔名
        let path = std::path::Path::new("usr/local/bin/lang.json");
        assert_eq!(extract_display_path(path), "lang.json");
    }

    #[test]
    fn test_hashmap_to_entries_standard() {
        let mut map = HashMap::new();
        map.insert(
            "Apple".to_string(),
            ("蘋果".to_string(), TermType::Official),
        );
        map.insert(
            "Stone".to_string(),
            ("石頭".to_string(), TermType::Official),
        );

        let entries = hashmap_to_entries(&map);
        assert_eq!(entries.len(), 2);

        map.clear();
        map.insert("A".to_string(), ("A1".to_string(), TermType::Official));
        map.insert(
            "Apple".to_string(),
            ("蘋果".to_string(), TermType::Official),
        );
        let entries_after_clear = hashmap_to_entries(&map);
        assert_eq!(entries_after_clear[0].original, "Apple"); // 照長度排序
    }

    #[test]
    fn test_add_log_edge_cases() {
        let log = Arc::new(Mutex::new(Vec::new()));
        // 測試空語言與空檔名
        add_log(&log, "Message 1", "", "", "");
        let guard = log.lock().unwrap();
        assert_eq!(guard.len(), 1);
        assert!(guard[0].contains("Message 1"));
        assert!(!guard[0].contains("<")); // 不包含語言標籤
    }

    #[test]
    fn test_add_log_empty_lines() {
        let log = Arc::new(Mutex::new(Vec::new()));
        add_log(&log, "Line1\n\nLine3", "en", "zh", "test.json");
        let guard = log.lock().unwrap();
        assert_eq!(guard.len(), 3); // Line1, "", Line3
        assert_eq!(guard[1], ""); // 驗證空行確實是空字串
    }

    #[test]
    fn test_format_log_message_standard() {
        let msg = "Hello\n\nWorld";
        let logs = format_log_message(msg);
        assert_eq!(logs.len(), 3);
        assert!(logs[0].contains("Hello"));
        assert_eq!(logs[1], "");
        assert!(logs[2].contains("World"));
    }
}
