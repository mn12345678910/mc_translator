use crate::translation::batching::{GlobalBatchItem, translate_global_batches};
use crate::translation::job::JobSharedState;
use crate::file::pack_gen::{output_resource_pack, write_to_temp_or_output};
use crate::translation::glossary::GlossaryAutomaton;
use crate::utils::text_processing::sync_formatting;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

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
    let job_config = state.config.clone();
    let status_arc = state.status.clone();
    let progress_arc = state.progress.clone();
    let cancelled_arc = state.cancelled.clone();
    let paused_arc = state.paused.clone();
    let log = state.log.clone();

    let mut join_set = tokio::task::JoinSet::new();


    for (path, rel_path) in paths {
        if cancelled_arc.load(Ordering::SeqCst) {
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
            file_tasks.extend(tasks);
            global_items.extend(items);
        }
    }

    // 重新校分配 file_id，確保連續性 (避免不同執行緒間的 ID 衝突)
    let mut current_id = 0;
    let mut id_map = HashMap::new();
    for task in &mut file_tasks {
        let old_id = task.file_id;
        task.file_id = current_id;
        id_map.insert(old_id, current_id);
        current_id += 1;
    }
    for item in &mut global_items {
        if let Some(&new_id) = id_map.get(&item.file_id) {
            item.file_id = new_id;
        }
    }

    // --- JAR 分組追蹤優化 (Immediate Repack) ---
    let mut jar_group_map: HashMap<std::path::PathBuf, std::collections::HashSet<usize>> = HashMap::new();
    for task in &file_tasks {
        let is_jar_member = task.path.to_string_lossy().contains("mods") && !task.rel_path.ends_with(".js");
        if is_jar_member {
            jar_group_map.entry(task.path.clone()).or_default().insert(task.file_id);
        }
    }
    let mut finished_ids = std::collections::HashSet::new();
    let mut repack_buffer: HashMap<std::path::PathBuf, HashMap<String, String>> = HashMap::new();
    // ------------------------------------------

    // 更新全域進度總量 (Revision 15.40+: 使用 HashSet 獲取精確的不重複來源檔案數)
    let unique_files: std::collections::HashSet<std::path::PathBuf> = file_tasks.iter().map(|t| t.path.clone()).collect();
    if !unique_files.is_empty() {
        state.global_total.store((unique_files.len() as f32).to_bits(), Ordering::SeqCst);
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
    progress_arc.store(0.0f32.to_bits(), Ordering::SeqCst);
    

    for (idx, task) in file_tasks.iter().enumerate() {
        if cancelled_arc.load(Ordering::SeqCst) {
            break;
        }

        let file_item_count = global_items[item_offset..]
            .iter()
            .take_while(|item| item.file_id == task.file_id)
            .count();

        if file_item_count == 0 {
            // 無翻譯項目，顯示略過日誌
            crate::utils::add_log(&log, &format!("(略過條目) 略過處理: {}", task.rel_path));
            finished_ids.insert(task.file_id);
            
            let next_path = file_tasks.get(idx + 1).map(|t| &t.path);
            if next_path != Some(&task.path) {
                let current_g = f32::from_bits(state.global_progress.load(Ordering::SeqCst));
                state.global_progress.store((current_g + 1.0).to_bits(), Ordering::SeqCst);
            }
            item_offset += file_item_count;
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
            &state.i18n,
        )
        .await?;

        // --- 即時收集並檢查是否可執行 Repack ---
        finished_ids.insert(task.file_id);
        let translated_content = get_translated_content_for_task(task, current_file_items);
        
        let is_jar_member = task.path.to_string_lossy().contains("mods") && !task.rel_path.ends_with(".js");
        if is_jar_member {
            repack_buffer.entry(task.path.clone()).or_default().insert(task.rel_path.clone(), translated_content);
            
            // 檢查該 JAR 是否已全部翻譯完成
            if let Some(member_ids) = jar_group_map.get(&task.path) {
                if member_ids.iter().all(|id| finished_ids.contains(id)) {
                    // 全數完成，立即觸發 Repack
                    if let Some(files_to_repack) = repack_buffer.remove(&task.path) {
                        crate::utils::add_log(&log, &format!("(即時寫入) 正在封裝 JAR: {}", task.path.file_name().unwrap_or_default().to_string_lossy()));
                        crate::file::jar_handler::repack_jar(&task.path, &files_to_repack, &job_config.lock().unwrap())?;
                        // 此處緩存已被 repack_buffer.remove 釋放 (Memory Recovery)
                    }
                }
            }
        } else {
            // 普通檔案：即時寫入磁碟 (同樣節省記憶體)
            let mut results = HashMap::new();
            results.insert(task.rel_path.clone(), translated_content);
            write_to_temp_or_output(&job_config.lock().unwrap(), results)?;
        }
        // --------------------------------------

        item_offset += file_item_count;

        let next_path = file_tasks.get(idx + 1).map(|t| &t.path);
        if next_path != Some(&task.path) {
            let current_g = f32::from_bits(state.global_progress.load(Ordering::SeqCst));
            state.global_progress.store((current_g + 1.0).to_bits(), Ordering::SeqCst);
        }
    }

    // 處理尚未觸發的（例如取消時的掃尾或 BUNDLE 資源包，在此專案目前邏輯下，大部份已在循環中處理）
    if !cancelled_arc.load(Ordering::SeqCst) {
        let config_locked = job_config.lock().unwrap().clone();
        
        // 還原起始狀態對於 BUNDLE 資源包的最後處理 (如有未處理完的非 JAR BUNDLE)
        let has_bundle = file_tasks.iter().any(|t| {
            t.rel_path.contains("patchouli_books") && !t.path.to_string_lossy().contains("mods")
        });

        if has_bundle {
            output_resource_pack(&std::path::PathBuf::new(), HashMap::new(), config_locked, log.clone()).await?;
        }
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::batching::GlobalBatchItem;
    use std::path::PathBuf;

    #[test]
    fn test_file_id_recalibration() {
        let mut file_tasks = vec![
            FileTask {
                file_id: 99,
                path: PathBuf::from("a.json"),
                rel_path: "a.json".to_string(),
                original_content: "".to_string(),
                en_us_value: None,
                zh_tw_base: None,
                js_matches: vec![],
            },
            FileTask {
                file_id: 50,
                path: PathBuf::from("b.json"),
                rel_path: "b.json".to_string(),
                original_content: "".to_string(),
                en_us_value: None,
                zh_tw_base: None,
                js_matches: vec![],
            },
        ];

        let mut global_items = vec![
            GlobalBatchItem::new("text1", 99, "key1"),
            GlobalBatchItem::new("text2", 50, "key2"),
        ];

        let mut current_id = 0;
        let mut id_map = HashMap::new();
        for task in &mut file_tasks {
            let old_id = task.file_id;
            task.file_id = current_id;
            id_map.insert(old_id, current_id);
            current_id += 1;
        }
        for item in &mut global_items {
            if let Some(&new_id) = id_map.get(&item.file_id) {
                item.file_id = new_id;
            }
        }

        assert_eq!(file_tasks[0].file_id, 0);
        assert_eq!(file_tasks[1].file_id, 1);
        assert_eq!(global_items[0].file_id, 0);
        assert_eq!(global_items[1].file_id, 1);
    }

    #[test]
    fn test_jar_grouping_logic() {
        let file_tasks = vec![
            FileTask {
                file_id: 0,
                path: PathBuf::from("mods/test.jar"),
                rel_path: "assets/a.json".to_string(),
                original_content: "".to_string(),
                en_us_value: None,
                zh_tw_base: None,
                js_matches: vec![],
            },
            FileTask {
                file_id: 1,
                path: PathBuf::from("mods/test.jar"),
                rel_path: "assets/b.json".to_string(),
                original_content: "".to_string(),
                en_us_value: None,
                zh_tw_base: None,
                js_matches: vec![],
            },
            FileTask {
                file_id: 2,
                path: PathBuf::from("standalone.json"),
                rel_path: "standalone.json".to_string(),
                original_content: "".to_string(),
                en_us_value: None,
                zh_tw_base: None,
                js_matches: vec![],
            },
        ];

        let mut jar_group_map: HashMap<PathBuf, std::collections::HashSet<usize>> = HashMap::new();
        for task in &file_tasks {
            let is_jar_member = task.path.to_string_lossy().contains("mods") && !task.rel_path.ends_with(".js");
            if is_jar_member {
                jar_group_map.entry(task.path.clone()).or_default().insert(task.file_id);
            }
        }

        assert!(jar_group_map.contains_key(&PathBuf::from("mods/test.jar")));
        assert_eq!(jar_group_map.get(&PathBuf::from("mods/test.jar")).unwrap().len(), 2);
        assert!(jar_group_map.get(&PathBuf::from("mods/test.jar")).unwrap().contains(&0));
        assert!(jar_group_map.get(&PathBuf::from("mods/test.jar")).unwrap().contains(&1));
        assert!(!jar_group_map.contains_key(&PathBuf::from("standalone.json")));
    }
}
