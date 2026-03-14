use crate::translation::batching::GlobalBatchItem;
use crate::translation::job::{JobConfig, JobSharedState};
use crate::file::pipeline::{FileTask, FileStatus};
use crate::translation::context::{TranslationContext, ContextOptions};
use crate::translation::engine;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub async fn collect_json_task(
    file_id: usize,
    path: &Path,
    rel_path: String,
    state: &JobSharedState,
) -> Result<
    Option<(FileTask, Vec<GlobalBatchItem>)>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let path_clone = path.to_path_buf();
    let (content, en_us_value, zh_tw_value) = tokio::task::spawn_blocking(move || -> Result<(String, serde_json::Value, serde_json::Value), Box<dyn std::error::Error + Send + Sync>> {
        let content = match fs::read_to_string(&path_clone) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("\x1b[31m[{}] [錯誤] 無法讀取 JSON 檔案 {:?}: {}\x1b[0m", 
                    chrono::Local::now().format("%H:%M:%S"), path_clone, e);
                return Err(e.into());
            }
        };
        let en_us_value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("\x1b[31m[{}] [錯誤] JSON 格式錯誤 {:?}: {}\x1b[0m", 
                    chrono::Local::now().format("%H:%M:%S"), path_clone, e);
                return Err(e.into());
            }
        };

        let zh_tw_path = path_clone.with_file_name("zh_tw.json");
        let mut zh_tw_value = serde_json::Value::Null;
        if zh_tw_path.exists() {
            if let Ok(existing) = fs::read_to_string(&zh_tw_path) {
                zh_tw_value = serde_json::from_str(&existing).unwrap_or(serde_json::Value::Null);
            }
        }
        Ok((content, en_us_value, zh_tw_value))
    }).await??;

    let mut pending = Vec::new();

    let empty_map = HashMap::new();
    let empty_vec = Vec::new();
    let glossary_automaton = Arc::new(crate::translation::glossary::GlossaryAutomaton::new_simple(&empty_map, &empty_map));
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
        filename: rel_path.clone(),
        translation_memory: state.translation_memory.clone(),
        skip_memory: false,
        pause_notifier: state.pause_notifier.clone(),
        i18n: &state.i18n,
    });

    engine::collect_translatable_strings(&en_us_value, &zh_tw_value, None, &mut pending, &ctx);

    let prefilled_count = ctx.prefilled.lock().unwrap().len();
    if pending.is_empty() && prefilled_count == 0 {
        return Ok(None);
    }

    let mut global_items = Vec::new();
    for (orig, key) in pending {
        global_items.push(GlobalBatchItem::new(&orig, file_id, &key));
    }

    // 加入預先填滿的項目 (修正丟失條目與進度條問題)
    for (orig, key, trans) in ctx.prefilled.lock().unwrap().iter() {
        let mut item = GlobalBatchItem::new(orig, file_id, key);
        item.translated = Some(trans.clone());
        global_items.push(item);
    }

    Ok(Some((
        FileTask::new_json(file_id, path, rel_path, content, en_us_value, zh_tw_value),
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

    let final_zh_tw_content =
        crate::utils::text_processing::sync_formatting(&task.original_content, &translations_map);

    let actual_output_dir = {
        let cfg = config.lock().unwrap();
        if cfg.output_dir.is_empty() {
            "./LLMTranslator".to_string()
        } else {
            let p = std::path::Path::new(&cfg.output_dir).join("LLMTranslator");
            let _ = std::fs::create_dir_all(&p);
            p.to_string_lossy().to_string()
        }
    };

    let final_rel_path = if task.rel_path.ends_with("en_us.json") {
        task.rel_path.replace("en_us.json", "zh_tw.json")
    } else {
        task.rel_path.clone()
    };

    let fs_path = std::path::Path::new(&actual_output_dir).join(&final_rel_path);
    if let Some(parent) = fs_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    tokio::fs::write(&fs_path, &final_zh_tw_content).await?;
    Ok(FileStatus::Completed(final_rel_path))
}
