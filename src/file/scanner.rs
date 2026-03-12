use std::fs;
use std::path::Path;
use crate::translation::engine::count_strings;
use crate::file::js_handler::{JS_REGEX_LIST, JS_INNER_SINGLE_RE, JS_INNER_DOUBLE_RE};
use crate::utils::skip_rules::should_skip_value;

pub fn scan_files_recursive(
    dir: &std::path::Path,
    base_dir: &std::path::Path,
) -> Vec<(std::path::PathBuf, String)> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        // 規範化基準路徑以避免 Windows 磁碟機代號大小寫造成的 strip_prefix 失敗 (Scan Fix)
        let base_norm = base_dir.canonicalize().unwrap_or(base_dir.to_path_buf());
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(scan_files_recursive(&path, base_dir));
            } else if path
                .extension()
                .is_some_and(|ext| ext == "jar" || ext == "json" || ext == "js")
            {
                let path_norm = path.canonicalize().unwrap_or(path.clone());
                let rel = match path_norm.strip_prefix(&base_norm) {
                    Ok(p) => p.to_string_lossy().to_string(),
                    Err(_) => {
                        // 跨磁碟機或規範化失敗時的安全回退：僅保留檔名，防止絕對路徑造成輸出漂移
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown_file".to_string())
                    }
                };
                files.push((path, rel.replace('\\', "/")));
            }
        }
    }
    files
}

pub async fn check_jar_has_target(
    path: &Path,
    skip_book: bool,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let path_str = path.to_string_lossy().to_lowercase();
    if !path_str.contains("mods") {
        return Ok(false);
    }

    let path_clone = path.to_path_buf();
    let has_target = tokio::task::spawn_blocking(
        move || -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            let file = match fs::File::open(&path_clone) {
                Ok(f) => f,
                Err(_) => return Ok(false),
            };
            let mut archive = match zip::ZipArchive::new(file) {
                Ok(a) => a,
                Err(_) => return Ok(false),
            };

            for i in 0..archive.len() {
                let file_in_zip = match archive.by_index(i) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let name = file_in_zip.name().to_string();
                let is_book = name.contains("patchouli_books") && name.contains("en_us");
                let is_en_us = name.ends_with("en_us.json")
                    || (name.contains("/en_us/") && name.ends_with(".json"));

                if is_en_us && name.ends_with(".json") {
                    if is_book && skip_book {
                        continue;
                    }
                    return Ok(true);
                }
            }
            Ok(false)
        },
    )
    .await??;

    Ok(has_target)
}

pub async fn check_js_has_target(
    path: &Path,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let content = fs::read_to_string(path)?;
    for re in JS_REGEX_LIST.iter() {
        for cap in re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let s = m.as_str();
                if re.as_str().contains(r"\[(.*?)\]") {
                    let array_str = s;
                    for inner_cap in JS_INNER_SINGLE_RE.captures_iter(array_str) {
                        if let Some(mi) = inner_cap.get(1) {
                            if !should_skip_value(mi.as_str()) {
                                return Ok(true);
                            }
                        }
                    }
                    for inner_cap in JS_INNER_DOUBLE_RE.captures_iter(array_str) {
                        if let Some(mi) = inner_cap.get(1) {
                            if !should_skip_value(mi.as_str()) {
                                return Ok(true);
                            }
                        }
                    }
                } else if !should_skip_value(s) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub async fn check_json_has_target(
    path: &Path,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let path_clone = path.to_path_buf();
    let has_target = tokio::task::spawn_blocking(
        move || -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            let content = match fs::read_to_string(&path_clone) {
                Ok(c) => c,
                Err(_) => return Ok(false),
            };
            let en_us_value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => return Ok(false),
            };

            let zh_tw_path = path_clone.with_file_name("zh_tw.json");
            let mut zh_tw_value = serde_json::Value::Null;
            if zh_tw_path.exists() {
                if let Ok(existing) = fs::read_to_string(&zh_tw_path) {
                    zh_tw_value =
                        serde_json::from_str(&existing).unwrap_or(serde_json::Value::Null);
                }
            }

            let count = count_strings(&en_us_value, None, &zh_tw_value);
            if count == 0 {
                return Ok(false);
            }

            Ok(true)
        },
    )
    .await??;

    Ok(has_target)
}
