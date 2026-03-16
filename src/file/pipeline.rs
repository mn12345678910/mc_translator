use crate::translation::batching::{GlobalBatchItem, translate_global_batches};
use crate::translation::job::JobSharedState;
use crate::file::pack_gen::{output_resource_pack, write_to_temp_or_output};
use crate::translation::glossary::GlossaryAutomaton;
use crate::utils::text_processing::sync_formatting;
use crate::file::json_handler::collect_json_task;
use crate::file::js_handler::collect_js_task;
use crate::file::jar_handler::collect_jar_tasks;
use crate::translation::glossary::mc_lang::McLangFiles;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Processing(String),
    Completed(String),
    Skipped(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct FileTask {
    pub file_id: usize,
    pub path: std::path::PathBuf,
    pub rel_path: String,
    pub original_content: String,
    pub en_us_value: Option<serde_json::Value>,
    pub zh_tw_base: Option<serde_json::Value>,
    pub js_matches: Vec<(usize, usize, String, usize)>,
}

impl FileTask {
    pub fn new_json(
        file_id: usize,
        path: &Path,
        rel_path: String,
        original_content: String,
        en_us_value: serde_json::Value,
        zh_tw_base: serde_json::Value,
    ) -> Self {
        Self {
            file_id,
            path: path.to_path_buf(),
            rel_path,
            original_content,
            en_us_value: Some(en_us_value),
            zh_tw_base: Some(zh_tw_base),
            js_matches: Vec::new(),
        }
    }

    pub fn new_js(
        file_id: usize,
        path: &Path,
        rel_path: String,
        original_content: String,
        js_matches: Vec<(usize, usize, String, usize)>,
    ) -> Self {
        Self {
            file_id,
            path: path.to_path_buf(),
            rel_path,
            original_content,
            en_us_value: None,
            zh_tw_base: None,
            js_matches,
        }
    }
}

pub async fn process_all_files(
    paths: Vec<(std::path::PathBuf, String)>,
    state: JobSharedState,
    _mc_lang_arc: Arc<Mutex<Option<McLangFiles>>>,
    _term_arc: Arc<Mutex<Vec<(String, String)>>>,
    exact_arc: Arc<Mutex<HashMap<String, String>>>,
    inferred_arc: Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let job_config = state.config.clone();
    let status_arc = state.status.clone();
    let progress_arc = state.progress.clone();
    let cancelled_arc = state.cancelled.clone();
    let paused_arc = state.paused.clone();
    let log = state.log.clone();

    // --- 階段一：掃描與條目收集 ---
    {
        let mut s = status_arc.lock().unwrap();
        *s = state.i18n.status_scanning_files.clone();
    }

    let mut file_tasks = Vec::new();
    let mut global_items = Vec::new();
    let mut current_file_id = 0;

    let (skip_json, skip_js, skip_jar) = {
        let cfg = job_config.lock().unwrap();
        (cfg.skip_json, cfg.skip_js, cfg.skip_jar)
    };

    for (path, rel_path) in paths {
        if cancelled_arc.load(Ordering::SeqCst) {
            return Ok(());
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "json" => {
                if skip_json { continue; }
                if let Ok(Some((task, items))) = collect_json_task(current_file_id, &path, rel_path, &state).await {
                    file_tasks.push(task);
                    global_items.extend(items);
                    current_file_id += 1;
                }
            }
            "js" => {
                if skip_js { continue; }
                if let Ok(Some((task, items))) = collect_js_task(current_file_id, &path, rel_path, &state).await {
                    file_tasks.push(task);
                    global_items.extend(items);
                    current_file_id += 1;
                }
            }
            "jar" => {
                if skip_jar { continue; }
                if let Ok((tasks, items)) = collect_jar_tasks(current_file_id, &path, &state).await {
                    let tasks: Vec<FileTask> = tasks;
                    let items: Vec<GlobalBatchItem> = items;
                    let task_len = tasks.len();
                    file_tasks.extend(tasks);
                    global_items.extend(items);
                    current_file_id += task_len;
                }
            }
            _ => {}
        }
    }
    // ----------------------------

    // --- 階段二：排序與分組 ---
    state.global_progress.store(0.0f32.to_bits(), Ordering::SeqCst);
    state.progress.store(0.0f32.to_bits(), Ordering::SeqCst);
    if let Ok(mut p) = state.current_processing_path.lock() {
        p.clear();
    }

    let get_group_key = |path: &std::path::Path| -> std::path::PathBuf {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.eq_ignore_ascii_case("jar") {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        }
    };

    file_tasks.sort_by(|a, b| get_group_key(&a.path).cmp(&get_group_key(&b.path)));
    
    // [Bug Fix] 重新規整 global_items，使其與已排序的 file_tasks 順序一致，防範切片失配
    let mut task_item_groups: std::collections::HashMap<usize, Vec<GlobalBatchItem>> = std::collections::HashMap::new();
    for item in global_items {
        task_item_groups.entry(item.file_id).or_default().push(item);
    }
    let mut reordered_items = Vec::new();
    for task in &file_tasks {
        if let Some(mut items) = task_item_groups.remove(&task.file_id) {
            reordered_items.append(&mut items);
        }
    }
    global_items = reordered_items;
    
    // 更新全域進度總量 (以此來源數量為準)
    let unique_files_count = {
        let mut seen = std::collections::HashSet::new();
        for t in &file_tasks {
            seen.insert(t.path.clone());
        }
        seen.len()
    };
    if unique_files_count > 0 {
        state.global_total.store((unique_files_count as f32).to_bits(), Ordering::SeqCst);
    }
    // 定錨全域條目總數
    state.progress_total.store((global_items.len() as f32).to_bits(), Ordering::SeqCst);
    // ----------------------------

    if global_items.is_empty() {
        return Ok(());
    }

    let glossary_automaton = GlossaryAutomaton::new(
        &exact_arc.lock().unwrap(),
        &state.translation_memory.lock().unwrap(),
        &inferred_arc.lock().unwrap(),
        &job_config.lock().unwrap().glossary_priority,
    );

    // --- 階段三：窗口式跨檔案翻譯迴圈 ---
    let mut task_ptr = 0;
    let mut item_ptr = 0;
    let mut global_items_offset = 0; // 新增：全域累加 Offset

    while task_ptr < file_tasks.len() {
        if cancelled_arc.load(Ordering::SeqCst) {
            break;
        }

        let source_path = file_tasks[task_ptr].path.clone();
        let group_key = get_group_key(&source_path);
        let mut group_tasks = Vec::new();
        while task_ptr < file_tasks.len() && get_group_key(&file_tasks[task_ptr].path) == group_key {
            group_tasks.push(&file_tasks[task_ptr]);
            task_ptr += 1;
        }

        let group_file_count = {
            let mut seen = std::collections::HashSet::new();
            for t in &group_tasks {
                seen.insert(t.path.clone());
            }
            seen.len() as f32
        };

        let group_file_ids: std::collections::HashSet<usize> = group_tasks.iter().map(|t| t.file_id).collect();
        let start_item_idx = item_ptr;
        while item_ptr < global_items.len() && group_file_ids.contains(&global_items[item_ptr].file_id) {
            item_ptr += 1;
        }
        let items_in_source = &mut global_items[start_item_idx..item_ptr];

        if items_in_source.is_empty() {
            let current_g = f32::from_bits(state.global_progress.load(Ordering::SeqCst));
            state.global_progress.store((current_g + group_file_count).to_bits(), Ordering::SeqCst);
            global_items_offset += items_in_source.len();
            continue;
        }

        let display_name = if source_path.extension().and_then(|e| e.to_str()).unwrap_or("").eq_ignore_ascii_case("jar") {
            source_path.file_name().unwrap_or_default().to_string_lossy().to_string()
        } else {
            let parent = source_path.parent().unwrap_or(&source_path);
            format!("{}/", parent.file_name().unwrap_or_default().to_string_lossy())
        };

        if let Ok(mut p) = state.current_processing_path.lock() {
            *p = display_name.clone();
        }

        translate_global_batches(
            items_in_source,
            job_config.clone(),
            status_arc.clone(),
            progress_arc.clone(),
            state.current_batch.clone(),
            state.total_batches.clone(),
            cancelled_arc.clone(),
            paused_arc.clone(),
            log.clone(),
            state.pause_notifier.clone(),
            &glossary_automaton,
            &state.i18n,
            &display_name,
            global_items_offset, // 新增：傳入全域偏移
        )
        .await?;

        let mut translated_results = HashMap::new();
        for task in group_tasks {
            let task_items: Vec<GlobalBatchItem> = items_in_source.iter().filter(|it| it.file_id == task.file_id).cloned().collect();
            let content = get_translated_content_for_task(task, &task_items);
            
            let is_jar = source_path.extension().is_some_and(|ext| ext.to_string_lossy().to_lowercase() == "jar");
            let key = if is_jar { format!("[BUNDLE]{}", task.rel_path) } else { task.rel_path.clone() };
            translated_results.insert(key, content);
        }

        let config_locked = job_config.lock().unwrap().clone();
        tokio::task::spawn_blocking(move || {
            write_to_temp_or_output(&config_locked, translated_results)
        }).await??;

        // [新增] 補齊檔案處理完成日誌
        let cfg = job_config.lock().unwrap().clone();
        crate::utils::add_log(
            &log,
            &state.i18n.log_processing_finished,
            &cfg.source_lang,
            &cfg.target_lang,
            &display_name,
        );

        let current_g = f32::from_bits(state.global_progress.load(Ordering::SeqCst));
        state.global_progress.store((current_g + group_file_count).to_bits(), Ordering::SeqCst);

        // 累計全域 Offset
        global_items_offset += items_in_source.len();
    }

    if !cancelled_arc.load(Ordering::SeqCst) {
        let config_locked = job_config.lock().unwrap().clone();
        output_resource_pack(&std::path::PathBuf::new(), HashMap::new(), config_locked, log.clone(), state.i18n.clone()).await?;
    }

    Ok(())
}

fn get_translated_content_for_task(task: &FileTask, items: &[GlobalBatchItem]) -> String {
    let mut local_map: HashMap<String, Vec<String>> = HashMap::new();
    for item in items {
        if let Some(ref t) = item.translated {
            local_map.entry(item.key.clone()).or_default().push(t.clone());
        }
    }

    if task.en_us_value.is_some() {
        sync_formatting(&task.original_content, &local_map)
    } else {
        let mut replacements = Vec::new();
        for (start, end, _, idx) in &task.js_matches {
            if let Some(v_list) = local_map.get(&format!("js_key_{}", idx)) {
                if let Some(t) = v_list.first() {
                    replacements.push((*start, *end, t.clone()));
                }
            }
        }
        replacements.sort_by_key(|r| r.0);
        replacements.reverse();
        let mut c = task.original_content.clone();
        for (s, e, t) in replacements {
            if s < e && e <= c.len() {
                c.replace_range(s..e, &t);
            }
        }
        c
    }
}
