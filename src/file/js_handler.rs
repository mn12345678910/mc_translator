use crate::file::pipeline::{FileStatus, FileTask};
use crate::translation::batching::GlobalBatchItem;
use crate::translation::job::JobConfig;
use crate::utils::skip_rules::should_skip_value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};

pub static JS_INNER_SINGLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?s)'(.*?)'").unwrap());
pub static JS_INNER_DOUBLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)"(.*?)""#).unwrap());

pub static JS_REGEX_LIST: LazyLock<Vec<regex::Regex>> = LazyLock::new(|| {
    vec![
        regex::Regex::new(r#"(?s)Text\.of\(\s*['"](.*?)['"]"#).unwrap(),
        regex::Regex::new(r#"(?s)add(?:Item)?\(\s*['"][^'"]*['"]\s*,\s*\[(.*?)\]"#).unwrap(),
        regex::Regex::new(r#"(?s)(?:\.text|scene\.text)\(\s*(?:[^'"]*?,\s*)?'(.*?)'"#).unwrap(),
        regex::Regex::new(r#"(?s)(?:\.text|scene\.text)\(\s*(?:[^'"]*?,\s*)?"(.*?)""#).unwrap(),
        regex::Regex::new(r"(?s)text\s*:\s*'(.*?)'").unwrap(),
        regex::Regex::new(r#"(?s)text\s*:\s*"(.*?)""#).unwrap(),
        regex::Regex::new(r#"(?s)\.scene\(\s*'[^']*'\s*,\s*'(.*?)'"#).unwrap(),
        regex::Regex::new(r#"(?s)\.scene\(\s*['"][^'"]*['"]\s*,\s*"(.*?)""#).unwrap(),
        regex::Regex::new(r"(?s)\.title\(\s*'(.*?)'").unwrap(),
        regex::Regex::new(r#"(?s)\.title\(\s*"(.*?)""#).unwrap(),
    ]
});

pub async fn collect_js_task(
    file_id: usize,
    path: &Path,
    rel_path: String,
    _state: &crate::translation::job::JobSharedState,
) -> Result<Option<(FileTask, Vec<GlobalBatchItem>)>, Box<dyn std::error::Error + Send + Sync>> {
    let path_clone = path.to_path_buf();
    let content = tokio::task::spawn_blocking(
        move || -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            match fs::read_to_string(&path_clone) {
                Ok(file_content) => Ok(file_content),
                Err(e) => {
                    eprintln!(
                        "\x1b[31m[{}] [錯誤] 無法讀取 JS 檔案 {:?}: {}\x1b[0m",
                        chrono::Local::now().format("%H:%M:%S"),
                        path_clone,
                        e
                    );
                    Err(e.into())
                }
            }
        },
    )
    .await??;

    let mut js_matches = Vec::new();
    let mut match_counter = 0;

    for re in JS_REGEX_LIST.iter() {
        for cap in re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let matched_str = m.as_str();
                if re.as_str().contains(r"\[(.*?)\]") {
                    let offset = m.start();
                    for inner_cap in JS_INNER_SINGLE_RE.captures_iter(matched_str) {
                        if let Some(mi) = inner_cap.get(1) {
                            if !should_skip_value(mi.as_str()) {
                                js_matches.push((
                                    offset + mi.start(),
                                    offset + mi.end(),
                                    mi.as_str().to_string(),
                                    match_counter,
                                ));
                                match_counter += 1;
                            }
                        }
                    }
                    for inner_cap in JS_INNER_DOUBLE_RE.captures_iter(matched_str) {
                        if let Some(mi) = inner_cap.get(1) {
                            if !should_skip_value(mi.as_str()) {
                                js_matches.push((
                                    offset + mi.start(),
                                    offset + mi.end(),
                                    mi.as_str().to_string(),
                                    match_counter,
                                ));
                                match_counter += 1;
                            }
                        }
                    }
                } else if !should_skip_value(matched_str) {
                    js_matches.push((m.start(), m.end(), matched_str.to_string(), match_counter));
                    match_counter += 1;
                }
            }
        }
    }

    if js_matches.is_empty() {
        return Ok(None);
    }

    js_matches.sort_by_key(|match_item| match_item.0);
    let mut filtered_matches = Vec::new();
    let mut last_end = 0;
    for match_item in js_matches {
        if match_item.0 >= last_end {
            filtered_matches.push(match_item.clone());
            last_end = match_item.1;
        }
    }

    let mut global_items = Vec::new();
    for (_, _, text, idx) in &filtered_matches {
        let key = format!("js_key_{}", idx);
        let item = GlobalBatchItem::new(text, file_id, &key);
        global_items.push(item);
    }

    Ok(Some((
        FileTask::new_js(file_id, path, rel_path, content, filtered_matches),
        global_items,
    )))
}

pub async fn apply_js_task(
    task: &FileTask,
    global_items: &[GlobalBatchItem],
    config: &Arc<Mutex<JobConfig>>,
) -> Result<FileStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut translations_map: HashMap<String, String> = HashMap::new();

    for item in global_items {
        if item.file_id == task.file_id {
            if let Some(ref trans) = item.translated {
                translations_map.insert(item.key.clone(), trans.clone());
            }
        }
    }

    if translations_map.is_empty() {
        return Ok(FileStatus::Skipped(task.rel_path.clone()));
    }

    let mut replacements = Vec::new();
    for (start, end, _, idx) in &task.js_matches {
        if let Some(t) = translations_map.get(&format!("js_key_{}", idx)) {
            replacements.push((*start, *end, t.clone()));
        }
    }

    if replacements.is_empty() {
        return Ok(FileStatus::Skipped(task.rel_path.clone()));
    }

    replacements.sort_by_key(|r| r.0);
    replacements.reverse();

    let mut final_content = task.original_content.clone();
    for (s, e, t) in replacements {
        if s < e && e <= final_content.len() {
            final_content.replace_range(s..e, &t);
        }
    }

    let actual_output_dir = {
        let cfg = config.lock().unwrap();
        let base = if cfg.output_dir.is_empty() {
            std::path::Path::new(".")
        } else {
            std::path::Path::new(&cfg.output_dir)
        };
        base.join("LLMTranslator").to_string_lossy().to_string()
    };

    let fs_path = std::path::Path::new(&actual_output_dir).join(&task.rel_path);
    if let Some(parent) = fs_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    tokio::fs::write(&fs_path, &final_content).await?;
    Ok(FileStatus::Completed(task.rel_path.clone()))
}
