use crate::translation::batching::{GlobalBatchItem, translate_global_batches};
use crate::translation::job::{JobConfig, JobSharedState};
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
    inferred: Arc<Mutex<HashMap<String, String>>>,
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
            file_tasks.extend(tasks);
            global_items.extend(items);

            {
                let mut total = state.global_total.lock().unwrap();
                *total = file_id_counter as f32;
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
        &inferred.lock().unwrap(),
        &job_config.lock().unwrap().glossary_priority,
    );

    let mut item_offset = 0;
    
    // 用於記錄 JAR 檔案是否有被修改，以便最後決定是否 Repack
    let mut modified_jars = std::collections::HashSet::new();
    let mut has_non_jar_files = false;

    for (idx, task) in file_tasks.iter().enumerate() {
        if *cancelled_arc.lock().unwrap() {
            break;
        }

        let file_item_count = global_items[item_offset..]
            .iter()
            .take_while(|item| item.file_id == task.file_id)
            .count();

        if file_item_count == 0 {
            // 無翻譯項目，顯示略過日誌 (info 級別)
            crate::utils::add_log(&log, &format!("(略過條目) 略過處理: {}", task.rel_path));
            
            let mut g_prog = state.global_progress.lock().unwrap();
            *g_prog = (idx + 1) as f32;
            item_offset += file_item_count; // 雖然是 0 但保持邏輯完整
            continue;
        }

        let current_file_items = &mut global_items[item_offset..item_offset + file_item_count];
        
        // 執行翻譯
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

        // 立即套用結果並寫入磁碟以釋放記憶體
        let config_locked = job_config.lock().unwrap().clone();
        let is_jar_member = task.path.to_string_lossy().contains("mods") && !task.rel_path.ends_with(".js");
        
        apply_single_file_results(&config_locked, task, current_file_items)?;

        if is_jar_member {
            modified_jars.insert(task.path.clone());
        } else {
            has_non_jar_files = true;
        }

        // 翻譯完成日誌 (需求格式: /路徑 翻譯完成 條目數)
        let display_path = crate::utils::extract_display_path(Path::new(&task.rel_path));
        let display_path = if display_path.starts_with('/') {
            display_path
        } else {
            format!("/{}", display_path)
        };
        crate::utils::add_log(&log, &format!("({}) 翻譯完成 ({})", display_path, file_item_count));

        // 釋放已翻譯項目的記憶體內容 (如果有必要，這裡可以將 translated 設為 None，
        // 但目前 GlobalBatchItem 已經完成了任務，其生命週期在迴圈結束後就會消失)

        {
            let mut g_prog = state.global_progress.lock().unwrap();
            *g_prog = (idx + 1) as f32;
        }
        
        item_offset += file_item_count;
    }

    if !*cancelled_arc.lock().unwrap() {
        let config_locked = job_config.lock().unwrap().clone();
        
        // JAR Repack 階段：遍歷所有被修改過的 JAR 進行重新打包
        for jar_path in modified_jars {
            crate::file::jar_handler::repack_jar(&jar_path, &HashMap::new(), &config_locked)?; // HashMap 為空代表從 temp 目錄讀取
        }

        let has_patchouli = file_tasks.iter().any(|t| t.rel_path.contains("patchouli_books"));
        if has_non_jar_files || has_patchouli {
            crate::utils::add_log(&log, "正在生成資源包 (LLMTranslator.zip)...");
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

pub fn apply_single_file_results(
    config: &JobConfig,
    task: &FileTask,
    items: &[GlobalBatchItem],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut local_map: HashMap<String, Vec<String>> = HashMap::new();
    for item in items {
        if let Some(ref t) = item.translated {
            local_map
                .entry(item.key.clone())
                .or_default()
                .push(t.clone());
        }
    }

    if local_map.is_empty() {
        return Ok(());
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

    let is_jar_member = task.path.to_string_lossy().contains("mods") && !task.rel_path.ends_with(".js");
    
    if is_jar_member {
        // JAR 內檔案：先寫入暫存盤釋放記憶體
        crate::file::jar_handler::write_inner_temp(config, &task.path, &task.rel_path, &final_content)?;
    } else {
        // 普通外部檔案：直接寫入輸出目錄
        let mut results = HashMap::new();
        results.insert(task.rel_path.clone(), final_content);
        write_to_temp_or_output(config, results)?;
    }

    Ok(())
}
