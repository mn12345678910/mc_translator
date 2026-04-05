use crate::file::pipeline::{FileStatus, FileTask};
use crate::file::utils::flatten_json_values;
use crate::translation::batching::GlobalBatchItem;
use crate::translation::context::{ContextOptions, TranslationContext};
use crate::translation::engine;
use crate::translation::job::{JobConfig, JobSharedState};
use crate::translation::LogLevel;
use crate::utils::helpers::add_log_event;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub async fn collect_json_task(
    file_id: usize,
    path: &Path,
    rel_path: String,
    state: &JobSharedState,
) -> Result<Option<(FileTask, Vec<GlobalBatchItem>)>, Box<dyn std::error::Error + Send + Sync>> {
    let path_clone = path.to_path_buf();
    let (target_lang, source_lang, fast_convert) = {
        let cfg = state.config.lock().unwrap();
        (
            cfg.target_lang.clone(),
            cfg.source_lang.clone(),
            cfg.fast_convert,
        )
    };

    // 計算兄弟中文方言（用於跨語言快速轉換）
    // 条件： fast_convert 開啟、目標為中文、來源非中文
    let is_target_chinese = target_lang == "zh_cn" || target_lang == "zh_tw";
    let is_source_chinese = source_lang == "zh_cn" || source_lang == "zh_tw";
    let alt_lang = if fast_convert && is_target_chinese && !is_source_chinese {
        if target_lang == "zh_tw" {
            Some("zh_cn".to_string())
        } else {
            Some("zh_tw".to_string())
        }
    } else {
        None
    };

    type JsonTaskData = (
        String,
        serde_json::Value,
        serde_json::Value,
        HashMap<String, VecDeque<String>>,
    );
    let log = state.log.clone();
    let display_path = crate::utils::helpers::extract_display_path(path);
    let enable_debug = {
        let cfg = state.config.lock().unwrap();
        cfg.enable_debug_log
    };
    let (content, source_value, target_base, alt_map) = tokio::task::spawn_blocking(
        move || -> Result<JsonTaskData, Box<dyn std::error::Error + Send + Sync>> {
            let content = match fs::read_to_string(&path_clone) {
                Ok(c) => c,
                Err(e) => {
                    add_log_event(
                        &log,
                        LogLevel::Error,
                        &format!("無法讀取 JSON 檔案 {:?}: {}", path_clone, e),
                        &source_lang,
                        &target_lang,
                        &display_path,
                        enable_debug,
                    );
                    return Err(e.into());
                }
            };
            let source_value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    add_log_event(
                        &log,
                        LogLevel::Error,
                        &format!("JSON 格式錯誤 {:?}: {}", path_clone, e),
                        &source_lang,
                        &target_lang,
                        &display_path,
                        enable_debug,
                    );
                    return Err(e.into());
                }
            };

            let target_path = path_clone.with_file_name(format!("{}.json", target_lang));
            let mut target_base = serde_json::Value::Null;
            if target_path.exists() {
                if let Ok(existing) = fs::read_to_string(&target_path) {
                    target_base =
                        serde_json::from_str(&existing).unwrap_or(serde_json::Value::Null);
                }
            }

            // 讀取兄弟中文方言檔案，建立 key → value 對照表（支援嵌套 JSON）
            let mut alt_map: HashMap<String, VecDeque<String>> = HashMap::new();
            if let Some(ref alt) = alt_lang {
                let alt_path = path_clone.with_file_name(format!("{}.json", alt));
                if alt_path.exists() {
                    if let Ok(alt_content) = fs::read_to_string(&alt_path) {
                        if let Ok(alt_value) =
                            serde_json::from_str::<serde_json::Value>(&alt_content)
                        {
                            let mut pairs = Vec::new();
                            flatten_json_values(&alt_value, None, &mut pairs);
                            for (k, v) in pairs {
                                alt_map.entry(k).or_default().push_back(v);
                            }
                        }
                    }
                }
            }

            Ok((content, source_value, target_base, alt_map))
        },
    )
    .await??;

    let mut pending = Vec::new();

    let empty_map = HashMap::new();
    let empty_vec = Vec::new();
    let glossary_automaton = Arc::new(crate::translation::glossary::GlossaryAutomaton::new_simple(
        &empty_map, &empty_map,
    ));
    let ctx = TranslationContext::new(ContextOptions {
        config: state.config.clone(),
        inferred: &empty_map,
        terms: &empty_vec,
        glossary_automaton: &glossary_automaton,
        status: state.status.clone(),
        progress: state.progress.clone(),
        total_progress: state.progress_total.clone(),
        cancelled: state.cancelled.clone(),
        paused: state.paused.clone(),
        current_log: state.log.clone(),
        filename: crate::utils::helpers::extract_display_path(path),
        translation_memory: state.translation_memory.clone(),
        skip_memory: false,
        pause_notifier: state.pause_notifier.clone(),
        i18n: &state.i18n,
    });

    engine::collect_translatable_strings(&source_value, &target_base, None, &mut pending, &ctx);

    let prefilled_count = ctx.prefilled.lock().unwrap().len();
    if pending.is_empty() && prefilled_count == 0 {
        return Ok(None);
    }

    let mut global_items = Vec::new();
    let mut alt_map = alt_map; // 取得所有權以進行消費式匹配
    for (orig, key) in pending {
        let mut item = GlobalBatchItem::new(&orig, file_id, &rel_path, &key);
        // 消費式匹配：按順序取出兄弟方言的對應值，確保同名 Key 不碰撞
        if let Some(queue) = alt_map.get_mut(&key) {
            if let Some(alt_val) = queue.pop_front() {
                item.alt_source = Some(alt_val);
            }
        }
        global_items.push(item);
    }

    // 加入預填項目，維持條目與進度一致
    for (orig, key, trans) in ctx.prefilled.lock().unwrap().iter() {
        let mut item = GlobalBatchItem::new(orig, file_id, &rel_path, key);
        item.translated = Some(trans.clone());
        global_items.push(item);
    }

    Ok(Some((
        FileTask::new_json(file_id, path, rel_path, content, source_value, target_base),
        global_items,
    )))
}

pub async fn apply_json_task(
    task: &FileTask,
    global_items: &[GlobalBatchItem],
    config: &Arc<Mutex<JobConfig>>,
) -> Result<FileStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut translations_map: HashMap<String, Vec<String>> = HashMap::new();

    for item in global_items {
        if item.file_id == task.file_id {
            if let Some(ref trans) = item.translated {
                translations_map
                    .entry(item.key.clone())
                    .or_default()
                    .push(trans.clone());
            }
        }
    }

    if translations_map.is_empty() {
        return Ok(FileStatus::Skipped(task.rel_path.clone()));
    }

    let final_content =
        crate::utils::text_processing::sync_formatting(&task.original_content, &translations_map);

    let (source_lang, target_lang, actual_output_dir) = {
        let cfg = config.lock().unwrap();
        let base = if cfg.output_dir.is_empty() {
            std::path::Path::new(".")
        } else {
            std::path::Path::new(&cfg.output_dir)
        };
        let dir = base.join("LLMTranslator").to_string_lossy().to_string();
        (cfg.source_lang.clone(), cfg.target_lang.clone(), dir)
    };

    let src_suffix = format!("{}.json", source_lang);
    let tgt_suffix = format!("{}.json", target_lang);
    let final_rel_path = if task.rel_path.ends_with(&src_suffix) {
        task.rel_path.replace(&src_suffix, &tgt_suffix)
    } else {
        task.rel_path.clone()
    };

    let fs_path = std::path::Path::new(&actual_output_dir).join(&final_rel_path);
    if let Some(parent) = fs_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    tokio::fs::write(&fs_path, &final_content).await?;
    Ok(FileStatus::Completed(final_rel_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::pipeline::FileTask;
    use crate::translation::job::JobConfig;
    use std::sync::atomic::{AtomicBool, AtomicU32};

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
            translation_memory: Arc::new(Mutex::new(std::collections::HashMap::new())),
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
    async fn test_collect_json_task_read_fail() {
        let state = create_mock_shared_state();
        let non_existent = std::path::PathBuf::from("non_existent_file_xyz.json");
        let res = collect_json_task(1, &non_existent, "xyz.json".to_string(), &state).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_collect_json_task_parse_fail() {
        let temp_dir = std::env::temp_dir().join("mc_translator_json_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("broken.json");
        std::fs::write(&file_path, r#"{ "broken": "#).unwrap();

        let state = create_mock_shared_state();
        let res = collect_json_task(1, &file_path, "broken.json".to_string(), &state).await;
        assert!(res.is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_collect_json_task_empty_pending() {
        let temp_dir = std::env::temp_dir().join("mc_translator_json_test_empty");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("empty.json");
        std::fs::write(&file_path, r#"{}"#).unwrap();

        let state = create_mock_shared_state();
        let res = collect_json_task(1, &file_path, "empty.json".to_string(), &state).await;
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_apply_json_task_skipped() {
        let task = FileTask::new_json(
            1,
            &std::path::PathBuf::from("test.json"),
            "test.json".to_string(),
            "{}".to_string(),
            serde_json::Value::Null,
            serde_json::Value::Null,
        );

        let config = Arc::new(Mutex::new(JobConfig::default()));
        let res = apply_json_task(&task, &[], &config).await;
        assert!(res.is_ok());
        match res.unwrap() {
            FileStatus::Skipped(path) => assert_eq!(path, "test.json"),
            _ => panic!("Expected Skipped"),
        }
    }
}
