use crate::file::jar_handler::collect_jar_tasks;
use crate::file::js_handler::collect_js_task;
use crate::file::json_handler::collect_json_task;
use crate::file::pack_gen::{output_resource_pack, write_to_temp_or_output};
use crate::translation::batching::{translate_global_batches, GlobalBatchItem};
use crate::translation::glossary::mc_lang::McLangFiles;
use crate::translation::glossary::GlossaryAutomaton;
use crate::translation::job::JobSharedState;
use crate::translation::LogLevel;
use crate::utils::helpers::add_log_event;
use crate::utils::text_processing::sync_formatting;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

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
    pub source_value: Option<serde_json::Value>,
    pub target_base: Option<serde_json::Value>,
    pub js_matches: Vec<(usize, usize, String, usize)>,
}

impl FileTask {
    pub fn new_json(
        file_id: usize,
        path: &Path,
        rel_path: String,
        original_content: String,
        source_value: serde_json::Value,
        target_base: serde_json::Value,
    ) -> Self {
        Self {
            file_id,
            path: path.to_path_buf(),
            rel_path,
            original_content,
            source_value: Some(source_value),
            target_base: Some(target_base),
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
            source_value: None,
            target_base: None,
            js_matches,
        }
    }
}

pub async fn process_all_files(
    paths: Vec<(std::path::PathBuf, String)>,
    state: JobSharedState,
    _mc_lang_arc: Arc<Mutex<Option<McLangFiles>>>,
    term_arc: Arc<Mutex<Vec<(String, String)>>>,
    exact_arc: Arc<Mutex<HashMap<String, String>>>,
    inferred_arc: Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let job_config = state.config.clone();
    let cancelled_arc = state.cancelled.clone();
    let log = state.log.clone();

    // --- 階段一：掃描與條目收集 ---
    {
        let mut status = state.status.lock().unwrap();
        *status = state.i18n.status_scanning_files.clone();
    }

    let mut file_tasks = Vec::new();
    let mut global_items = Vec::new();
    let mut current_file_id = 0;

    let (skip_json, skip_js, skip_jar, source_lang, target_lang) = {
        let cfg = job_config.lock().unwrap();
        (
            cfg.skip_json,
            cfg.skip_js,
            cfg.skip_jar,
            cfg.source_lang.clone(),
            cfg.target_lang.clone(),
        )
    };

    let source_lang_file = format!("{}.json", source_lang);

    for (path, rel_path) in paths {
        if cancelled_arc.load(Ordering::SeqCst) {
            return Ok(());
        }

        // --- 全域排除過濾 (Global Blacklist) ---
        // 1. 正規化路徑（小寫 + 統一正斜線）以利匹配
        let rel_path_norm = rel_path.to_lowercase().replace('\\', "/");

        // 2. 動態排除清單 (包含核心技術目錄與使用者自定義)
        let excluded = {
            let cfg = job_config.lock().unwrap();
            cfg.excluded_paths.clone()
        };

        if excluded.iter().any(|p: &String| {
            let p_norm = p.to_lowercase().replace('\\', "/");
            !p_norm.is_empty() && rel_path_norm.contains(&p_norm)
        }) {
            continue;
        }

        // 3. 針對 JourneyMap 擴展名檢查（雙重保險）
        if rel_path_norm.ends_with(".theme2.json") {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "json" => {
                if skip_json {
                    continue;
                }

                // 語言檔過濾：如果父目錄是 lang，且不是來源語言，則略過。
                let parent_name = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if parent_name.eq_ignore_ascii_case("lang") {
                    if !file_name.eq_ignore_ascii_case(&source_lang_file) {
                        continue;
                    }
                } else if rel_path.contains("patchouli_books") {
                    // Patchouli 書籍過濾：避免掃描到目標語言目錄
                    let tgt_dir = format!("/{}/", target_lang);
                    let tgt_dir_win = format!("\\{}\\", target_lang);
                    if rel_path.contains(&tgt_dir) || rel_path.contains(&tgt_dir_win) {
                        continue;
                    }
                }

                if let Ok(Some((task, items))) =
                    collect_json_task(current_file_id, &path, rel_path.clone(), &state).await
                {
                    file_tasks.push(task);
                    global_items.extend(items);
                    current_file_id += 1;
                }
            }
            "js" => {
                if skip_js {
                    continue;
                }
                if let Ok(Some((task, items))) =
                    collect_js_task(current_file_id, &path, rel_path, &state).await
                {
                    file_tasks.push(task);
                    global_items.extend(items);
                    current_file_id += 1;
                }
            }
            "jar" => {
                if skip_jar {
                    continue;
                }
                if let Ok((tasks, items)) = collect_jar_tasks(current_file_id, &path, &state).await
                {
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
    state
        .global_progress
        .store(0.0f32.to_bits(), Ordering::SeqCst);
    state.progress.store(0.0f32.to_bits(), Ordering::SeqCst);
    if let Ok(mut current_path) = state.current_processing_path.lock() {
        current_path.clear();
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

    // 重新規整 global_items，使其與已排序的 file_tasks 順序一致，避免切片失配
    let mut task_item_groups: std::collections::HashMap<usize, Vec<GlobalBatchItem>> =
        std::collections::HashMap::new();
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
    let total_tasks_count = file_tasks.len();
    if total_tasks_count > 0 {
        state
            .global_total
            .store((total_tasks_count as f32).to_bits(), Ordering::SeqCst);
    }
    // 定錨全域條目總數
    state
        .progress_total
        .store((global_items.len() as f32).to_bits(), Ordering::SeqCst);
    // ----------------------------

    if global_items.is_empty() {
        return Ok(());
    }

    let mut merged_exact = exact_arc.lock().unwrap().clone();
    let (is_fast_convert, target_lang) = {
        let cfg = job_config.lock().unwrap();
        (cfg.fast_convert, cfg.target_lang.clone())
    };
    // 雙向簡繁轉換（zh_cn↔zh_tw）時，將術語差異表合併進術語自動機
    // term_arc 在 mc_lang.rs 中已依 target_lang 方向預先建立好對應的差異表
    let is_cjk_target = target_lang == "zh_tw" || target_lang == "zh_cn";
    if is_fast_convert || is_cjk_target {
        for (src_val, tgt_val) in term_arc.lock().unwrap().iter() {
            merged_exact.insert(src_val.clone(), tgt_val.clone());
        }
    }

    let glossary_automaton = GlossaryAutomaton::new(
        &merged_exact,
        &state.translation_memory.lock().unwrap(),
        &inferred_arc.lock().unwrap(),
        &job_config.lock().unwrap().glossary_priority,
    );

    // --- 階段三：窗口式跨檔案翻譯迴圈 ---
    let mut task_ptr = 0;
    let mut item_ptr = 0;
    let mut global_items_offset = 0; // 全域累加 Offset

    while task_ptr < file_tasks.len() {
        if cancelled_arc.load(Ordering::SeqCst) {
            break;
        }

        let source_path = file_tasks[task_ptr].path.clone();
        let group_key = get_group_key(&source_path);
        let mut group_tasks = Vec::new();
        while task_ptr < file_tasks.len() && get_group_key(&file_tasks[task_ptr].path) == group_key
        {
            group_tasks.push(&file_tasks[task_ptr]);
            task_ptr += 1;
        }

        let group_tasks_count = group_tasks.len() as f32;

        let group_file_ids: std::collections::HashSet<usize> =
            group_tasks.iter().map(|t| t.file_id).collect();
        let start_item_idx = item_ptr;
        while item_ptr < global_items.len()
            && group_file_ids.contains(&global_items[item_ptr].file_id)
        {
            item_ptr += 1;
        }
        let items_in_source = &mut global_items[start_item_idx..item_ptr];

        if items_in_source.is_empty() {
            let current_g = f32::from_bits(state.global_progress.load(Ordering::SeqCst));
            state
                .global_progress
                .store((current_g + group_tasks_count).to_bits(), Ordering::SeqCst);
            global_items_offset += items_in_source.len();
            continue;
        }

        let is_jar = source_path
            .extension()
            .is_some_and(|ext| ext.to_string_lossy().to_lowercase() == "jar");

        let display_name = {
            let first_path = if is_jar {
                group_tasks
                    .first()
                    .map(|t| Path::new(&t.rel_path))
                    .unwrap_or(&source_path)
            } else {
                &source_path
            };
            extract_group_label(first_path)
        };

        if let Ok(mut p) = state.current_processing_path.lock() {
            *p = display_name.clone();
        }

        let rel_paths: Vec<String> = group_tasks.iter().map(|t| t.rel_path.clone()).collect();
        let log_file_name = shorten_rel_paths(&rel_paths);

        translate_global_batches(
            items_in_source,
            job_config.clone(),
            state.status.clone(),
            state.progress.clone(),
            state.current_batch.clone(),
            state.total_batches.clone(),
            cancelled_arc.clone(),
            state.paused.clone(),
            log.clone(),
            state.pause_notifier.clone(),
            &glossary_automaton,
            &state.i18n,
            &log_file_name,
            &display_name,
            group_tasks_count as usize,
            global_items_offset,
        )
        .await?;

        let mut translated_results = HashMap::new();
        for task in group_tasks {
            let task_items: Vec<GlobalBatchItem> = items_in_source
                .iter()
                .filter(|it| it.file_id == task.file_id)
                .cloned()
                .collect();
            let content = get_translated_content_for_task(task, &task_items);

            let key = if is_jar {
                format!("[BUNDLE]{}", task.rel_path)
            } else {
                task.rel_path.clone()
            };
            translated_results.insert(key, content);

            // 每處理完一個檔案，即時更新全域進度
            let current_g = f32::from_bits(state.global_progress.load(Ordering::SeqCst));
            state
                .global_progress
                .store((current_g + 1.0).to_bits(), Ordering::SeqCst);
        }

        let config_locked = job_config.lock().unwrap().clone();
        tokio::task::spawn_blocking(move || {
            write_to_temp_or_output(&config_locked, translated_results)
        })
        .await??;

        // 補齊檔案處理完成日誌
        let cfg = job_config.lock().unwrap().clone();
        add_log_event(
            &log,
            LogLevel::Success,
            &state.i18n.log_processing_finished,
            &cfg.source_lang,
            &cfg.target_lang,
            &display_name,
            cfg.enable_debug_log,
        );

        // 累計全域 Offset
        global_items_offset += items_in_source.len();
    }

    if !cancelled_arc.load(Ordering::SeqCst) {
        let config_locked = job_config.lock().unwrap().clone();
        output_resource_pack(config_locked, log.clone(), state.i18n.clone()).await?;
    }

    Ok(())
}

fn get_translated_content_for_task(task: &FileTask, items: &[GlobalBatchItem]) -> String {
    let mut local_map: HashMap<String, Vec<String>> = HashMap::new();
    for item in items {
        if let Some(ref t) = item.translated {
            local_map
                .entry(item.key.clone())
                .or_default()
                .push(t.clone());
        }
    }

    if task.source_value.is_some() {
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
        let mut content = task.original_content.clone();
        for (s, e, t) in replacements {
            if s < e && e <= content.len() {
                content.replace_range(s..e, &t);
            }
        }
        content
    }
}

pub fn shorten_rel_paths(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }

    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for path in paths {
        let p = Path::new(path);
        let parent = p
            .parent()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_default();
        let file = p
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        groups
            .entry(parent.replace('\\', "/"))
            .or_default()
            .push(file);
    }

    let mut result = Vec::new();
    for (_dir, files) in groups {
        // 目錄行 (含縮排一個空格)
        if !_dir.is_empty() {
            let mut dir_line = _dir.clone();
            if !dir_line.ends_with('/') {
                dir_line.push('/');
            }
            result.push(format!(" <dir>{}</dir>", dir_line));
        }

        // 檔案行 (含縮排一個空格)
        for chunk in files.chunks(5) {
            let files_str = chunk
                .iter()
                .map(|f| format!("<file>{}</file>", f))
                .collect::<Vec<_>>()
                .join(", ");
            result.push(format!(" {}", files_str));
        }
    }

    // 以換行符開頭，使 log_processing_file_mask 後立即換行
    let mut final_str = result.join("\n");
    if !final_str.is_empty() {
        final_str = format!("\n{}", final_str);
    }
    final_str
}

/// 從路徑中提取 ModID 或資料夾名稱，用於日誌標籤
fn extract_group_label(path: &Path) -> String {
    let path_str = path.to_string_lossy().replace('\\', "/");
    // 優先從資產結構提取 modid
    if let Some(pos) = path_str.find("assets/") {
        let after_assets = &path_str[pos + 7..];
        if let Some(modid) = after_assets.split('/').next() {
            if !modid.is_empty() {
                return modid.to_string();
            }
        }
    }
    if let Some(pos) = path_str.find("data/") {
        let after_data = &path_str[pos + 5..];
        if let Some(modid) = after_data.split('/').next() {
            if !modid.is_empty() {
                return modid.to_string();
            }
        }
    }
    // 回退到父目錄名，若無父目錄則使用檔名
    if let Some(parent) = path.parent() {
        if let Some(name) = parent.file_name() {
            let name_str = name.to_string_lossy().to_string();
            if !name_str.is_empty() && name_str != "." {
                return name_str;
            }
        }
    }

    path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_group_label() {
        assert_eq!(
            extract_group_label(Path::new("assets/bloodmagic/lang/en_us.json")),
            "bloodmagic"
        );
        assert_eq!(
            extract_group_label(Path::new("data/myapp/tags/items/1.json")),
            "myapp"
        );
        assert_eq!(
            extract_group_label(Path::new("some_folder/file.json")),
            "some_folder"
        );
        assert_eq!(
            extract_group_label(Path::new("root_file.json")),
            "root_file.json"
        );
    }

    #[test]
    fn test_path_filtering_logic() {
        let test_cases = vec![
            ("kubejs/data/worldgen/structure.json", true),
            ("journeymap/icon/theme/flat/DesertTemple.theme2.json", true),
            ("packmenu/resources/assets/atm/buttons/play.json", true),
            ("config/almostunified/unify.json", true),
            ("mods/my_awesome_mod.jar", false),
            ("config/minecraft/server.properties", false),
            ("patchouli_books/manual/en_us/chapters/intro.json", false),
            ("kubejs/server_scripts/script.js", false),
        ];

        for (rel_path, should_skip) in test_cases {
            let rel_path_norm = rel_path.to_lowercase().replace('\\', "/");
            let is_skipped = rel_path_norm.contains("kubejs/data/")
                || rel_path_norm.contains("journeymap/icon/theme")
                || rel_path_norm.contains("packmenu/")
                || rel_path_norm.contains("config/almostunified/")
                || rel_path_norm.contains("fancymenu/")
                || rel_path_norm.contains("shaderpacks/")
                || rel_path_norm.contains("screenshots/")
                || rel_path_norm.contains("saves/")
                || rel_path_norm.contains(".mixin.out/")
                || rel_path_norm.ends_with(".theme2.json");

            assert_eq!(
                is_skipped, should_skip,
                "Path '{}' filtering mismatch",
                rel_path
            );
        }
    }

    #[test]
    fn test_shorten_rel_paths_styling() {
        let paths = vec![
            "a/1.json".into(),
            "a/2.json".into(),
            "b/3.json".into(),
            "4.json".into(),
        ];
        let result = shorten_rel_paths(&paths);
        assert!(result.starts_with('\n'));
        assert!(result.contains(" <dir>a/</dir>"));
        assert!(result.contains(" <file>1.json</file>, <file>2.json</file>"));
        assert!(result.contains(" <dir>b/</dir>"));
        assert!(result.contains(" <file>3.json</file>"));
        assert!(result.contains(" <file>4.json</file>"));
    }

    #[test]
    fn test_shorten_rel_paths_line_split() {
        let paths = vec![
            "a/b/c/1.json".to_string(),
            "a/b/c/2.json".to_string(),
            "a/b/c/3.json".to_string(),
            "a/b/c/4.json".to_string(),
            "a/b/c/5.json".to_string(),
            "a/b/c/6.json".to_string(),
        ];
        let result = shorten_rel_paths(&paths);
        assert!(result.contains(" <dir>a/b/c/</dir>"));
        assert!(result.contains(" <file>1.json</file>, <file>2.json</file>, <file>3.json</file>, <file>4.json</file>, <file>5.json</file>"));
        // 驗證行末沒有額外的逗號
        assert!(!result.contains("5.json</file>,"));
        assert!(result.contains(" <file>6.json</file>"));
    }
}
