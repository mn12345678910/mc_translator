//! # 檔案處理模組
//! 負責 JAR 檔案的解壓、翻譯後的檔案替換，以及重新封裝 ZIP 的輸出邏輯。

use crate::data_processing;
use crate::translation_job::{JobConfig, JobSharedState};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};

/// 檔案處理狀態枚舉
#[derive(Debug, Clone)]
pub enum FileStatus {
    /// 檔案處理完成，並提供最終路徑以便日後處理
    Completed(String),
    /// 檔案處理跳過，因為不包含翻譯條目或已被翻譯
    Skipped(String),
}

/// 檔案翻譯任務結構：儲存單一檔案在批次處理中的所有上下文
#[derive(Debug, Clone)]
pub struct FileTask {
    pub file_id: usize,
    pub path: std::path::PathBuf,
    pub rel_path: String,
    pub original_content: String,
    // JSON 專用
    pub en_us_value: Option<serde_json::Value>,
    pub zh_tw_base: Option<serde_json::Value>,
    // JS 專用
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

/// 第一階段：從 JSON 檔案中收集所有待翻譯條目
pub async fn collect_json_task(
    file_id: usize,
    path: &Path,
    rel_path: String,
    state: &JobSharedState,
) -> Result<
    Option<(FileTask, Vec<data_processing::GlobalBatchItem>)>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let path_clone = path.to_path_buf();
    let (content, en_us_value, zh_tw_value) = tokio::task::spawn_blocking(move || -> Result<(String, serde_json::Value, serde_json::Value), Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(&path_clone)?;
        let en_us_value: serde_json::Value = serde_json::from_str(&content)?;

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

    // 使用暫時的自動機進行收集，因為 ctx 只是用來過濾，我們還沒到真正翻譯階段
    let empty_map = HashMap::new();
    let empty_vec = Vec::new();
    let glossary_automaton = Arc::new(crate::utils::GlossaryAutomaton::new(&empty_map, &empty_map));
    let ctx = data_processing::TranslationContext::new(data_processing::ContextOptions {
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
    });

    data_processing::collect_translatable_strings(
        &en_us_value,
        &zh_tw_value,
        None,
        &mut pending,
        &ctx,
    );

    if pending.is_empty() {
        return Ok(None);
    }

    let mut global_items = Vec::new();
    for (orig, key) in pending {
        global_items.push(data_processing::GlobalBatchItem::new(&orig, file_id, &key));
    }

    Ok(Some((
        FileTask::new_json(file_id, path, rel_path, content, en_us_value, zh_tw_value),
        global_items,
    )))
}

/// 第二階段：將翻譯後的結果套用回 JSON 並儲存
pub async fn apply_json_task(
    task: &FileTask,
    global_items: &[data_processing::GlobalBatchItem],
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
        data_processing::sync_formatting(&task.original_content, &translations_map);

    let actual_output_dir = {
        let cfg = config.lock().unwrap();
        if cfg.output_dir.is_empty() {
            "./LLMTranslator".to_string()
        } else {
            cfg.output_dir.clone()
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

/// 第一階段：從 JS 檔案中收集所有待翻譯條目 (例如 KubeJS)
pub async fn collect_js_task(
    file_id: usize,
    path: &Path,
    rel_path: String,
) -> Result<
    Option<(FileTask, Vec<data_processing::GlobalBatchItem>)>,
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
                            if !crate::utils::should_skip_value(mi.as_str()) {
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
                            if !crate::utils::should_skip_value(mi.as_str()) {
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
                } else if !crate::utils::should_skip_value(s) {
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
        global_items.push(data_processing::GlobalBatchItem::new(text, file_id, &key));
    }

    Ok(Some((
        FileTask::new_js(file_id, path, rel_path, content, filtered_matches),
        global_items,
    )))
}

/// 第二階段：將翻譯後的結果套用回 JS 並儲存
pub async fn apply_js_task(
    task: &FileTask,
    global_items: &[data_processing::GlobalBatchItem],
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

/// 快速檢查 JAR 檔案是否包含翻譯目標 (通常是 `en_us` 語言檔案)
pub async fn check_jar_has_target(
    path: &Path,
    skip_book: bool,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let path_str = path.to_string_lossy().to_lowercase();
    if !path_str.contains("mods") {
        return Ok(false);
    }

    let path_clone = path.to_path_buf();
    let has_target = tokio::task::spawn_blocking(
        move || -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            let file = match fs::File::open(&path_clone) {
                Ok(f) => f,
                Err(_) => return Ok(false),
            };
            let mut archive = match zip::ZipArchive::new(file) {
                Ok(a) => a,
                Err(_) => return Ok(false),
            };

            for i in 0..archive.len() {
                let file_in_zip = match archive.by_index(i) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let name = file_in_zip.name().to_string();
                let is_book = name.contains("patchouli_books") && name.contains("en_us");
                let is_en_us = name.ends_with("en_us.json")
                    || (name.contains("/en_us/") && name.ends_with(".json"));

                if is_en_us && name.ends_with(".json") {
                    if is_book && skip_book {
                        continue;
                    }
                    return Ok(true);
                }
            }
            Ok(false)
        },
    )
    .await??;

    Ok(has_target)
}

/// 快速檢查 JS 檔案是否包含需要翻譯的字串
pub async fn check_js_has_target(
    path: &Path,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let content = fs::read_to_string(path)?;
    for re in JS_REGEX_LIST.iter() {
        for cap in re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let s = m.as_str();
                if re.as_str().contains(r"\[(.*?)\]") {
                    let array_str = s;
                    for inner_cap in JS_INNER_SINGLE_RE.captures_iter(array_str) {
                        if let Some(mi) = inner_cap.get(1) {
                            if !crate::utils::should_skip_value(mi.as_str()) {
                                return Ok(true);
                            }
                        }
                    }
                    for inner_cap in JS_INNER_DOUBLE_RE.captures_iter(array_str) {
                        if let Some(mi) = inner_cap.get(1) {
                            if !crate::utils::should_skip_value(mi.as_str()) {
                                return Ok(true);
                            }
                        }
                    }
                } else if !crate::utils::should_skip_value(s) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// 快速檢查 JSON 檔案是否包含需要翻譯的字串
pub async fn check_json_has_target(
    path: &Path,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let path_clone = path.to_path_buf();
    let has_target = tokio::task::spawn_blocking(
        move || -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            let content = match fs::read_to_string(&path_clone) {
                Ok(c) => c,
                Err(_) => return Ok(false),
            };
            let en_us_value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => return Ok(false),
            };

            let zh_tw_path = path_clone.with_file_name("zh_tw.json");
            let mut zh_tw_value = serde_json::Value::Null;
            if zh_tw_path.exists() {
                if let Ok(existing) = fs::read_to_string(&zh_tw_path) {
                    zh_tw_value =
                        serde_json::from_str(&existing).unwrap_or(serde_json::Value::Null);
                }
            }

            let count = crate::data_processing::count_strings(&en_us_value, None, &zh_tw_value);
            if count == 0 {
                return Ok(false);
            }

            Ok(true)
        },
    )
    .await??;

    Ok(has_target)
}

/// 全域檔案處理核心函數：負責收集、翻譯並應用結果
pub async fn process_all_files(
    paths: Vec<(std::path::PathBuf, String)>,
    state: JobSharedState,
    _mc_lang: Arc<Mutex<Option<crate::utils::McLangFiles>>>,
    _terms: Arc<Mutex<Vec<(String, String)>>>,
    exact: Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file_tasks: Vec<FileTask> = Vec::new();
    let mut global_items: Vec<crate::data_processing::GlobalBatchItem> = Vec::new();
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

        join_set.spawn(async move {
            let path_str = path.to_string_lossy().to_lowercase();
            if path_str.ends_with(".jar") {
                collect_jar_tasks(0, &path, &state_clone).await
            } else if path_str.ends_with(".json") {
                match collect_json_task(0, &path, rel_path, &state_clone).await {
                    Ok(Some((task, items))) => Ok((vec![task], items)),
                    Ok(None) => Ok((vec![], vec![])),
                    Err(e) => Err(e),
                }
            } else if path_str.ends_with(".js") {
                match collect_js_task(0, &path, rel_path).await {
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

            // 1.5 增量更新總量 (Revision 12.3)
            {
                let mut total = state.global_total.lock().unwrap();
                *total = file_id_counter as f32;
            }
        }
    }

    // 確定分析完畢，即將開始翻譯時的狀態文字切換
    {
        let mut s = state.status.lock().unwrap();
        *s = "正在翻譯中".to_string();
    }

    if global_items.is_empty() {
        return Ok(());
    }

    // 2. 翻譯階段 - 按檔案進行迴圈翻譯 (Per-file Consolidated Translation)
    let glossary_automaton = crate::utils::GlossaryAutomaton::new(
        &exact.lock().unwrap(),
        &state.translation_memory.lock().unwrap(),
    );

    let mut item_offset = 0;
    for (idx, task) in file_tasks.iter().enumerate() {
        if *cancelled_arc.lock().unwrap() {
            break;
        }

        // 計算目前檔案的條目數量
        let file_item_count = global_items[item_offset..]
            .iter()
            .take_while(|item| item.file_id == task.file_id)
            .count();

        if file_item_count == 0 {
            // 如果該檔案沒有可翻譯項目，直接增加總進度並跳過
            let mut g_prog = state.global_progress.lock().unwrap();
            *g_prog = (idx + 1) as f32;
            continue;
        }

        // 取得目前檔案的條目切片 (Slice)
        let current_file_items = &mut global_items[item_offset..item_offset + file_item_count];
        item_offset += file_item_count;

        // 執行翻譯
        crate::data_processing::translate_global_batches(
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

        // 該檔案翻譯完成後，增加總進度 (Files)
        {
            let mut g_prog = state.global_progress.lock().unwrap();
            *g_prog = (idx + 1) as f32;
        }
    }

    // 3. 應用階段
    if !*cancelled_arc.lock().unwrap() {
        let config_locked = job_config.lock().unwrap().clone();
        apply_global_results(&config_locked, &file_tasks, &global_items)?;

        // 如果有包含 mods 或 patchouli_books，則輸出資源包
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

/// 第一階段：從 JAR 檔案中收集所有待翻譯條目
#[allow(clippy::too_many_arguments)]
pub async fn collect_jar_tasks(
    start_file_id: usize,
    path: &Path,
    state: &JobSharedState,
) -> Result<
    (Vec<FileTask>, Vec<data_processing::GlobalBatchItem>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let path_clone = path.to_path_buf();
    let skip_book = state.config.lock().unwrap().skip_book;

    type JarTaskData = (String, serde_json::Value, String, serde_json::Value);
    let g_total_arc = state.global_total.clone();
    let tasks_data = tokio::task::spawn_blocking(
        move || -> Result<Vec<JarTaskData>, Box<dyn std::error::Error + Send + Sync>> {
            let file = fs::File::open(&path_clone)?;
            let mut archive = zip::ZipArchive::new(file)?;
            let mut entries = Vec::new();
            for i in 0..archive.len() {
                let (is_target, name, content) = {
                    let mut f = archive.by_index(i)?;
                    let name = f.name().to_string();
                    let is_book = name.contains("patchouli_books") && name.contains("en_us");
                    let is_en_us = (name.ends_with("en_us.json")
                        || (name.contains("/en_us/") && name.ends_with(".json")))
                        && name.ends_with(".json");
                    let is_target = is_en_us && !(is_book && skip_book);
                    let mut content = String::new();
                    if is_target {
                        f.read_to_string(&mut content)?;
                    }
                    (is_target, name, content)
                };

                if is_target {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                        let zh_tw_name = if name.contains("patchouli_books/") {
                            name.replace("/en_us/", "/zh_tw/")
                        } else {
                            name.replace("en_us.json", "zh_tw.json")
                        };
                        let mut zh_tw_value = serde_json::Value::Null;
                        if let Ok(mut zh_f) = archive.by_name(&zh_tw_name) {
                            let mut zh_c = String::new();
                            if zh_f.read_to_string(&mut zh_c).is_ok() {
                                zh_tw_value =
                                    serde_json::from_str(&zh_c).unwrap_or(serde_json::Value::Null);
                            }
                        }
                        entries.push((name, value, content, zh_tw_value));

                        // 1.5 增量更新總量 (Revision 12.3) - 在 JAR 掃描過程中讓使用者看到數字增加
                        {
                            let mut total = g_total_arc.lock().unwrap();
                            *total += 1.0;
                        }
                    }
                }
            }
            Ok(entries)
        },
    )
    .await??;

    let mut file_tasks = Vec::new();
    let mut global_items = Vec::new();
    let glossary_automaton = Arc::new(crate::utils::GlossaryAutomaton::new(
        &HashMap::new(),
        &HashMap::new(),
    ));

    for (idx, (name, en_us, content, zh_tw)) in tasks_data.into_iter().enumerate() {
        let file_id = start_file_id + idx;
        let mut pending = Vec::new();
        let empty_map = HashMap::new();
        let empty_vec = Vec::new();
        let ctx = data_processing::TranslationContext::new(data_processing::ContextOptions {
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
            filename: name.clone(),
            translation_memory: state.translation_memory.clone(),
            skip_memory: false,
            pause_notifier: state.pause_notifier.clone(),
        });

        data_processing::collect_translatable_strings(&en_us, &zh_tw, None, &mut pending, &ctx);
        if !pending.is_empty() {
            for (orig, key) in pending {
                global_items.push(data_processing::GlobalBatchItem::new(&orig, file_id, &key));
            }
            file_tasks.push(FileTask::new_json(
                file_id,
                path,
                name.clone(),
                content,
                en_us,
                zh_tw,
            ));
        }
    }

    Ok((file_tasks, global_items))
}

pub fn apply_global_results(
    config: &JobConfig,
    file_tasks: &[FileTask],
    global_items: &[data_processing::GlobalBatchItem],
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
            data_processing::sync_formatting(&task.original_content, &local_map)
        } else {
            let mut replacements = Vec::new();
            for (start, end, _, idx) in &task.js_matches {
                // JS 任務在 collect 時使用了 js_key_{idx} 作為 key
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

    // 針對 JAR 檔案執行重新封裝 (如有需要)
    for (jar_path, files) in jar_repack_map {
        repack_jar(&jar_path, &files)?;
    }

    Ok(())
}

/// 將翻譯後的檔案重新打包進指定的 JAR (直接覆寫原始檔案)
pub fn repack_jar(
    path: &Path,
    translated_files: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let temp_jar_path = path.with_extension("jar.tmp");

    // 預先計算所有會被寫入或覆寫的目標路徑
    let mut target_names = std::collections::HashSet::new();
    for name in translated_files.keys() {
        let actual_name = if name.ends_with("en_us.json") {
            name.replace("en_us.json", "zh_tw.json")
        } else {
            name.clone()
        };
        target_names.insert(actual_name);
    }

    {
        let temp_file = fs::File::create(&temp_jar_path)?;
        let mut zip_out = zip::ZipWriter::new(temp_file);

        let zip_in_file = fs::File::open(path)?;
        let mut zip_in = zip::ZipArchive::new(zip_in_file)?;

        for i in 0..zip_in.len() {
            let mut entry = zip_in.by_index(i)?;
            let name = entry.name().to_string();

            // 如果該檔案路徑在 target_names 中 (即我們即將寫入新版本)，則跳過原始檔案
            if target_names.contains(&name) {
                continue;
            }

            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            zip_out.start_file(name, options)?;
            zip_out.write_all(&buffer)?;
        }

        for (name, content) in translated_files {
            let actual_name = if name.ends_with("en_us.json") {
                name.replace("en_us.json", "zh_tw.json")
            } else {
                name.clone()
            };
            zip_out.start_file(
                actual_name,
                zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )?;
            zip_out.write_all(content.as_bytes())?;
        }
        zip_out.finish()?;
    }

    fs::rename(&temp_jar_path, path)?;
    Ok(())
}

/// 將翻譯結果寫入暫存目錄 (用於打包 Resource Pack) 或直接寫入輸出目錄
pub fn write_to_temp_or_output(
    config: &JobConfig,
    translated_files: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output_path = Path::new(&config.output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path).unwrap_or(());
    }

    let temp_dir = output_path.join("temp_translator");

    for (name, content) in translated_files {
        let mut clean_name = name.clone();

        if let Some(stripped) = name.strip_prefix("[BUNDLE]") {
            clean_name = stripped.to_string();
        }

        let name_unix = clean_name.replace('\\', "/");
        let mut final_path = if name_unix.ends_with("en_us.json") {
            name_unix.replace("en_us.json", "zh_tw.json")
        } else {
            name_unix.clone()
        };

        let is_absolute = Path::new(&clean_name).is_absolute();
        let has_dirs = name_unix.contains("/");

        if is_absolute || !has_dirs {
            let path_obj = Path::new(&clean_name);
            if let (Some(parent), Some(fname)) = (path_obj.parent(), path_obj.file_name()) {
                let fname_str = fname.to_string_lossy();
                let target_fname = if fname_str == "en_us.json" {
                    "zh_tw.json"
                } else {
                    &fname_str
                };

                if let Some(modid) = parent.file_name() {
                    final_path =
                        format!("assets/{}/lang/{}", modid.to_string_lossy(), target_fname);
                } else {
                    final_path = format!("assets/unknown/lang/{}", target_fname);
                }
            } else {
                let fname = name_unix.split('/').next_back().unwrap_or("zh_tw.json");
                let target_fname = if fname == "en_us.json" {
                    "zh_tw.json"
                } else {
                    fname
                };
                final_path = format!("assets/unknown/lang/{}", target_fname);
            }
        }

        if final_path.contains("patchouli_books/") {
            final_path = final_path.replace("/en_us/", "/zh_tw/");
        } else if !final_path.ends_with(".js") {
            if let Some(pos) = final_path.rfind('/') {
                let dir = &final_path[..=pos];
                if dir.ends_with("lang/") {
                    final_path = format!("{}zh_tw.json", dir);
                }
            }
        }

        if final_path.starts_with("assets/") {
            let fs_path = temp_dir.join(&final_path);
            if let Some(parent) = fs_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&fs_path, content);
        } else {
            let fs_path = output_path.join(&final_path);
            if let Some(parent) = fs_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&fs_path, content);
        }
    }
    Ok(())
}

/// 將暫存目錄中的內容封裝為 Minecraft 資源包 ZIP
pub async fn output_resource_pack(
    _src_path: &Path,
    _translated_files: HashMap<String, String>,
    config: JobConfig,
    log: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::task::spawn_blocking(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let output_path = Path::new(&config.output_dir);
            let temp_dir = output_path.join("temp_translator");

            if !temp_dir.exists() {
                return Ok(());
            }

            let pack_mcmeta = serde_json::json!({
                "pack": {
                    "pack_format": config.pack_format,
                    "description": "LLMTranslator 資源翻譯包"
                }
            });
            fs::write(
                temp_dir.join("pack.mcmeta"),
                serde_json::to_string_pretty(&pack_mcmeta)?,
            )?;

            let zip_filename = "LLMTranslator.zip";
            let zip_path = output_path.join(zip_filename);

            if zip_path.exists() {
                log.lock().unwrap().push(format!(
                    "警告：已存在相同的資源包檔案 {}，將會被直接覆蓋。",
                    zip_filename
                ));
            }

            let zip_file = fs::File::create(&zip_path)?;
            let mut zip_out = zip::ZipWriter::new(zip_file);
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            for entry in walkdir::WalkDir::new(&temp_dir) {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    let relative_path = path
                        .strip_prefix(&temp_dir)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    zip_out.start_file(relative_path, options)?;
                    let mut file = fs::File::open(path)?;
                    std::io::copy(&mut file, &mut zip_out)?;
                }
            }
            zip_out.finish()?;

            let _ = fs::remove_dir_all(&temp_dir);

            Ok(())
        },
    )
    .await??;

    Ok(())
}

/// JS 文本提取的正則表達式清單 (單引號)
static JS_INNER_SINGLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?s)'(.*?)'").unwrap());
/// JS 文本提取的正則表達式清單 (雙引號)
static JS_INNER_DOUBLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)"(.*?)""#).unwrap());

/// 用於匹配 JavaScript (例如 KubeJS) 中可翻譯字串的正則規則列表
static JS_REGEX_LIST: LazyLock<Vec<regex::Regex>> = LazyLock::new(|| {
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
