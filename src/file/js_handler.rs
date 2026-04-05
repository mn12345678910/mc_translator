use crate::file::pipeline::FileTask;
use crate::translation::batching::GlobalBatchItem;
use crate::translation::LogLevel;
use crate::utils::helpers::add_log_event;
use crate::utils::skip_rules::should_skip_value;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

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
    state: &crate::translation::job::JobSharedState,
) -> Result<Option<(FileTask, Vec<GlobalBatchItem>)>, Box<dyn std::error::Error + Send + Sync>> {
    let path_clone = path.to_path_buf();
    let log = state.log.clone();
    let enable_debug = state.config.lock().unwrap().enable_debug_log;
    let content = tokio::task::spawn_blocking(
        move || -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            match fs::read_to_string(&path_clone) {
                Ok(file_content) => Ok(file_content),
                Err(e) => {
                    add_log_event(
                        &log,
                        LogLevel::Error,
                        &format!("無法讀取 JS 檔案 {:?}: {}", path_clone, e),
                        "",
                        "",
                        "",
                        enable_debug,
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
        let item = GlobalBatchItem::new(text, file_id, &rel_path, &key);
        global_items.push(item);
    }

    Ok(Some((
        FileTask::new_js(file_id, path, rel_path, content, filtered_matches),
        global_items,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::job::{JobConfig, JobSharedState};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::{Arc, Mutex};

    fn create_mock_shared_state() -> JobSharedState {
        let config = JobConfig {
            target_lang: "zh_tw".to_string(),
            ..JobConfig::default()
        };

        JobSharedState {
            log: Arc::new(Mutex::new(Vec::new())),
            status: Arc::new(Mutex::new(String::new())),
            current_state: Arc::new(Mutex::new(crate::translation::job::JobStatus::Idle)),
            progress: Arc::new(AtomicU32::new(0)),
            progress_total: Arc::new(AtomicU32::new(0)),
            cancelled: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            translation_memory: Arc::new(Mutex::new(HashMap::new())),
            global_progress: Arc::new(AtomicU32::new(0)),
            global_total: Arc::new(AtomicU32::new(0)),
            current_processing_path: Arc::new(Mutex::new(String::new())),
            current_batch: Arc::new(AtomicU32::new(0)),
            total_batches: Arc::new(AtomicU32::new(0)),
            pause_notifier: Arc::new(tokio::sync::Notify::new()),
            config: Arc::new(Mutex::new(config)),
            i18n: crate::i18n::CommonLabels::default(),
        }
    }

    #[tokio::test]
    async fn test_collect_js_task_read_fail() {
        let state = create_mock_shared_state();
        let non_existent = std::path::PathBuf::from("non_existent_file_xyz.js");
        let res = collect_js_task(1, &non_existent, "xyz.js".to_string(), &state).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_collect_js_task_matches() {
        let temp_dir = std::env::temp_dir().join("mc_translator_js_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.js");
        std::fs::write(
            &file_path,
            r#"Text.of('你好')
.title("世界")
addItem('something', ['新增', '編輯'])"#,
        )
        .unwrap();

        let state = create_mock_shared_state();
        let res = collect_js_task(1, &file_path, "test.js".to_string(), &state).await;
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some());
        let (_task, items) = opt.unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].preprocessed, "你好");
        assert_eq!(items[1].preprocessed, "世界");
        assert_eq!(items[2].preprocessed, "新增");
        assert_eq!(items[3].preprocessed, "編輯");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_collect_js_task_no_matches() {
        let temp_dir = std::env::temp_dir().join("mc_translator_js_test_no_matches");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("empty.js");
        std::fs::write(&file_path, r#"// no matches here"#).unwrap();

        let state = create_mock_shared_state();
        let res = collect_js_task(1, &file_path, "empty.js".to_string(), &state).await;
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
