use crate::translation::batching::GlobalBatchItem;
use crate::translation::job::JobConfig;
use crate::file::pipeline::{FileTask, FileStatus};
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
) -> Result<
    Option<(FileTask, Vec<GlobalBatchItem>)>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let content = fs::read_to_string(path)?;

    let mut js_matches = Vec::new();
    let mut match_counter = 0;

    for re in JS_REGEX_LIST.iter() {
        for cap in re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let s = m.as_str();
                if re.as_str().contains(r"\[(.*?)\]") {
                    let offset = m.start();
                    for inner_cap in JS_INNER_SINGLE_RE.captures_iter(s) {
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
                    for inner_cap in JS_INNER_DOUBLE_RE.captures_iter(s) {
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
                } else if !should_skip_value(s) {
                    js_matches.push((m.start(), m.end(), s.to_string(), match_counter));
                    match_counter += 1;
                }
            }
        }
    }

    if js_matches.is_empty() {
        return Ok(None);
    }

    js_matches.sort_by_key(|m| m.0);
    let mut filtered_matches = Vec::new();
    let mut last_end = 0;
    for m in js_matches {
        if m.0 >= last_end {
            filtered_matches.push(m.clone());
            last_end = m.1;
        }
    }

    let mut global_items = Vec::new();
    for (_, _, text, idx) in &filtered_matches {
        let key = format!("js_key_{}", idx);
        global_items.push(GlobalBatchItem::new(text, file_id, &key));
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
        if cfg.output_dir.is_empty() {
            "./LLMTranslator".to_string()
        } else {
            cfg.output_dir.clone()
        }
    };

    let fs_path = std::path::Path::new(&actual_output_dir).join(&task.rel_path);
    if let Some(parent) = fs_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    tokio::fs::write(&fs_path, &final_content).await?;
    Ok(FileStatus::Completed(task.rel_path.clone()))
}
