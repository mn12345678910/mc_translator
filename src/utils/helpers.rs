//! # 輔助函式
//! 包含日誌、路徑顯示等通用工具函式。

use std::path::Path;
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
            log.push(format!("[{}] {}{}{}", timestamp, lang_info, file_info, line));
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
