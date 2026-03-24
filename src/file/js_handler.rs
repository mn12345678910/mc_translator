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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::pipeline::FileTask;
    use crate::translation::job::{JobConfig, JobSharedState};
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::{Arc, Mutex};

    fn create_mock_shared_state() -> JobSharedState {
        let mut config = JobConfig::default();
        config.target_lang = "zh_tw".to_string();

        JobSharedState {
            log: Arc::new(Mutex::new(Vec::new())),
            status: Arc::new(Mutex::new(String::new())),
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

    #[tokio::test]
    async fn test_apply_js_task() {
        let temp_dir = std::env::temp_dir().join("mc_translator_js_test_apply");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let task = FileTask::new_js(
            1,
            &std::path::PathBuf::from("test.js"),
            "test.js".to_string(),
            r#"Text.of('你好')"#.to_string(),
            vec![(9, 15, "你好".to_string(), 0)],
        );

        let mut config = JobConfig::default();
        config.output_dir = temp_dir.to_string_lossy().to_string();
        let config_locked = Arc::new(Mutex::new(config));

        let mut item = GlobalBatchItem::new("你好", 1, "js_key_0");
        item.translated = Some("Hello".to_string());

        let res = apply_js_task(&task, &[item], &config_locked).await;
        assert!(res.is_ok());
        match res.unwrap() {
            FileStatus::Completed(path) => assert_eq!(path, "test.js"),
            _ => panic!("Expected Completed"),
        }

        let output_file = temp_dir.join("LLMTranslator").join("test.js");
        assert!(output_file.exists());
        let content = std::fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("Hello"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_apply_js_task_skipped() {
        let task = FileTask::new_js(
            1,
            &std::path::PathBuf::from("test.js"),
            "test.js".to_string(),
            r#"Text.of('你好')"#.to_string(),
            vec![(9, 15, "你好".to_string(), 0)],
        );

        let config = Arc::new(Mutex::new(JobConfig::default()));
        let res = apply_js_task(&task, &[], &config).await;
        assert!(res.is_ok());
        match res.unwrap() {
            FileStatus::Skipped(path) => assert_eq!(path, "test.js"),
            _ => panic!("Expected Skipped"),
        }
    }
}
