use crate::translation::batching::{GlobalBatchItem, translate_global_batches};
use crate::translation::job::{JobConfig, JobSharedState};
use crate::file::jar_handler::repack_jar;
use crate::file::pack_gen::{output_resource_pack, write_to_temp_or_output};
use crate::translation::glossary::GlossaryAutomaton;
use crate::utils::text_processing::sync_formatting;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum FileStatus {
    Completed(String),
    Skipped(String),
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
        content: String,
        en_us: serde_json::Value,
        zh_tw: serde_json::Value,
    ) -> Self {
        Self {
            file_id,
            path: path.to_path_buf(),
            rel_path,
            original_content: content,
            en_us_value: Some(en_us),
            zh_tw_base: Some(zh_tw),
            js_matches: Vec::new(),
        }
    }

    pub fn new_js(
        file_id: usize,
        path: &Path,
        rel_path: String,
        content: String,
        matches: Vec<(usize, usize, String, usize)>,
    ) -> Self {
        Self {
            file_id,
            path: path.to_path_buf(),
            rel_path,
            original_content: content,
            en_us_value: None,
            zh_tw_base: None,
            js_matches: matches,
        }
    }
}

pub async fn process_all_files(
    paths: Vec<(std::path::PathBuf, String)>,
    state: JobSharedState,
    _mc_lang: Arc<Mutex<Option<crate::translation::glossary::McLangFiles>>>,
    _terms: Arc<Mutex<Vec<(String, String)>>>,
    exact: Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file_tasks: Vec<FileTask> = Vec::new();
    let mut global_items: Vec<GlobalBatchItem> = Vec::new();
    let mut file_id_counter = 0;

    let job_config = state.config.clone();
    let status_arc = state.status.clone();
    let progress_arc = state.progress.clone();
    let cancelled_arc = state.cancelled.clone();
    let paused_arc = state.paused.clone();
    let log = state.log.clone();

    let mut join_set = tokio::task::JoinSet::new();

    for (path, rel_path) in paths {
        if *cancelled_arc.lock().unwrap() {
            break;
        }
        let state_clone = state.clone();
        let rel_path_clone = rel_path.clone();

        join_set.spawn(async move {
            let path_str = path.to_string_lossy().to_lowercase();
            if path_str.ends_with(".jar") {
                crate::file::jar_handler::collect_jar_tasks(0, &path, &state_clone).await
            } else if path_str.ends_with(".json") {
                // 將預先檢查整合到此處，減少一次重複的檔案讀取 (整合優化)
                match crate::file::json_handler::collect_json_task(0, &path, rel_path_clone, &state_clone).await {
                    Ok(Some((task, items))) => Ok((vec![task], items)),
                    Ok(None) => Ok((vec![], vec![])),
                    Err(e) => Err(e),
                }
            } else if path_str.ends_with(".js") {
                match crate::file::js_handler::collect_js_task(0, &path, rel_path_clone, &state_clone).await {
                    Ok(Some((task, items))) => Ok((vec![task], items)),
                    Ok(None) => Ok((vec![], vec![])),
                    Err(e) => Err(e),
                }
            } else {
                Ok((vec![], vec![]))
            }
        });
    }

    let mut total_string_count = 0;
    while let Some(res) = join_set.join_next().await {
        if let Ok(Ok((tasks, items))) = res {
            let offset = file_id_counter;
            let mut tasks = tasks;
            let mut items = items;

            for t in &mut tasks {
                t.file_id += offset;
            }
            for i in &mut items {
                i.file_id += offset;
            }

            file_id_counter += tasks.len();
            total_string_count += items.len();
            file_tasks.extend(tasks);
            global_items.extend(items);

            {
                let mut total = state.global_total.lock().unwrap();
                *total = total_string_count as f32;
            }
        }
    }

    {
        let mut s = state.status.lock().unwrap();
        *s = "正在翻譯中".to_string();
    }

    if global_items.is_empty() {
        return Ok(());
    }

    let glossary_automaton = GlossaryAutomaton::new(
        &exact.lock().unwrap(),
        &state.translation_memory.lock().unwrap(),
    );

    let mut item_offset = 0;
    for (idx, task) in file_tasks.iter().enumerate() {
        if *cancelled_arc.lock().unwrap() {
            break;
        }

        let file_item_count = global_items[item_offset..]
            .iter()
            .take_while(|item| item.file_id == task.file_id)
            .count();

        if file_item_count == 0 {
            let mut g_prog = state.global_progress.lock().unwrap();
            *g_prog = (idx + 1) as f32;
            continue;
        }

        let current_file_items = &mut global_items[item_offset..item_offset + file_item_count];
        item_offset += file_item_count;

        translate_global_batches(
            current_file_items,
            job_config.clone(),
            status_arc.clone(),
            progress_arc.clone(),
            state.progress_total.clone(),
            cancelled_arc.clone(),
            paused_arc.clone(),
            log.clone(),
            state.pause_notifier.clone(),
            &glossary_automaton,
        )
        .await?;

        {
            let mut g_prog = state.global_progress.lock().unwrap();
            *g_prog = (idx + 1) as f32;
        }
    }

    if !*cancelled_arc.lock().unwrap() {
        let config_locked = job_config.lock().unwrap().clone();
        apply_global_results(&config_locked, &file_tasks, &global_items)?;

        let has_bundle = file_tasks.iter().any(|t| {
            t.rel_path.contains("patchouli_books") || t.path.to_string_lossy().contains("mods")
        });
        if has_bundle {
            output_resource_pack(
                &std::path::PathBuf::new(),
                HashMap::new(),
                config_locked,
                log.clone(),
            )
            .await?;
        }
    }

    Ok(())
}

pub fn apply_global_results(
    config: &JobConfig,
    file_tasks: &[FileTask],
    global_items: &[GlobalBatchItem],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut all_results = HashMap::new();
    let mut jar_repack_map: HashMap<std::path::PathBuf, HashMap<String, String>> = HashMap::new();

    for task in file_tasks {
        let mut local_map: HashMap<String, Vec<String>> = HashMap::new();
        for item in global_items {
            if item.file_id == task.file_id {
                if let Some(ref t) = item.translated {
                    local_map
                        .entry(item.key.clone())
                        .or_default()
                        .push(t.clone());
                }
            }
        }

        if local_map.is_empty() {
            continue;
        }

        let final_content = if task.en_us_value.is_some() {
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
        };

        if task.path.to_string_lossy().contains("mods") && !task.rel_path.ends_with(".js") {
            jar_repack_map
                .entry(task.path.clone())
                .or_default()
                .insert(task.rel_path.clone(), final_content.clone());
            all_results.insert(format!("[BUNDLE]{}", task.rel_path), final_content);
        } else {
            all_results.insert(task.rel_path.clone(), final_content);
        }
    }

    if !all_results.is_empty() {
        write_to_temp_or_output(config, all_results)?;
    }

    for (jar_path, files) in jar_repack_map {
        repack_jar(&jar_path, &files)?;
    }

    Ok(())
}
