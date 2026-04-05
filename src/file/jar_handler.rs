use crate::file::pipeline::FileTask;
use crate::file::utils::flatten_json_values;
use crate::translation::batching::GlobalBatchItem;
use crate::translation::context::{ContextOptions, TranslationContext};
use crate::translation::engine;
use crate::translation::job::JobSharedState;
use crate::translation::LogLevel;
use crate::utils::helpers::add_log_event;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

pub async fn collect_jar_tasks(
    start_file_id: usize,
    path: &Path,
    state: &JobSharedState,
) -> Result<(Vec<FileTask>, Vec<GlobalBatchItem>), Box<dyn std::error::Error + Send + Sync>> {
    let path_clone = path.to_path_buf();
    let (source_lang, target_lang, skip_book, fast_convert) = {
        let cfg = state.config.lock().unwrap();
        (
            cfg.source_lang.clone(),
            cfg.target_lang.clone(),
            cfg.skip_book,
            cfg.fast_convert,
        )
    };

    // 計算兄弟中文方言（用於跨語言快速轉換）
    let is_target_chinese = target_lang == "zh_cn" || target_lang == "zh_tw";
    let is_source_chinese = source_lang == "zh_cn" || source_lang == "zh_tw";
    let alt_lang: Option<String> = if fast_convert && is_target_chinese && !is_source_chinese {
        if target_lang == "zh_tw" {
            Some("zh_cn".to_string())
        } else {
            Some("zh_tw".to_string())
        }
    } else {
        None
    };

    type JarTaskData = (
        String,
        serde_json::Value,
        String,
        serde_json::Value,
        HashMap<String, VecDeque<String>>,
    );
    let state_clone = state.clone();
    let log = state.log.clone();
    let enable_debug = {
        let cfg = state.config.lock().unwrap();
        cfg.enable_debug_log
    };
    let tasks_data = tokio::task::spawn_blocking(
        move || -> Result<Vec<JarTaskData>, Box<dyn std::error::Error + Send + Sync>> {
            let state = state_clone;
            let file = fs::File::open(&path_clone)?;
            let mut archive = zip::ZipArchive::new(file)?;
            let mut entries = Vec::new();

            let src_suffix = format!("{}.json", source_lang);
            let src_dir = format!("/{}/", source_lang);

            // 預先收集所有檔案名稱，用於降級判定
            let mut name_list = std::collections::HashSet::new();
            for i in 0..archive.len() {
                if let Ok(f) = archive.by_index(i) {
                    name_list.insert(f.name().to_string());
                }
            }

            for i in 0..archive.len() {
                let (is_target, name, content) = {
                    let mut zip_entry = match archive.by_index(i) {
                        Ok(e) => e,
                        Err(e) => {
                            add_log_event(
                                &log,
                                LogLevel::Error,
                                &state
                                    .i18n
                                    .error_read_jar_index
                                    .replace("{}", &i.to_string())
                                    .replace("{}", &e.to_string()),
                                &source_lang,
                                &target_lang,
                                &path_clone.to_string_lossy(),
                                enable_debug,
                            );
                            continue;
                        }
                    };
                    let name = zip_entry.name().to_string();
                    let is_book = name.contains("patchouli_books")
                        && (name.contains(&format!("/{}", source_lang))
                            || name.contains("/en_us/"));

                    let is_actual_source = (name.ends_with(&src_suffix) || name.contains(&src_dir))
                        && name.ends_with(".json");
                    let is_fallback = (name.ends_with("en_us.json") || name.contains("/en_us/"))
                        && name.ends_with(".json")
                        && source_lang != "en_us";

                    let mut is_source = false;
                    if is_actual_source {
                        is_source = true;
                    } else if is_fallback {
                        let source_name = if name.contains("patchouli_books/") {
                            name.replace("/en_us/", &format!("/{}/", source_lang))
                        } else {
                            name.replace("en_us.json", &src_suffix)
                        };
                        if !name_list.contains(&source_name) {
                            is_source = true;
                        }
                    }

                    let is_target = is_source && !(is_book && skip_book);
                    let mut content = String::new();
                    if is_target {
                        if let Err(e) = zip_entry.read_to_string(&mut content) {
                            add_log_event(
                                &log,
                                LogLevel::Error,
                                &state
                                    .i18n
                                    .error_read_jar_file
                                    .replace("{}", &name)
                                    .replace("{}", &e.to_string()),
                                &source_lang,
                                &target_lang,
                                &path_clone.to_string_lossy(),
                                enable_debug,
                            );
                        }
                    }
                    (is_target, name, content)
                };

                if is_target {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                        let target_name = if name.contains("patchouli_books/") {
                            let src_book = if name.contains("/en_us/") {
                                "/en_us/".to_string()
                            } else {
                                format!("/{}/", source_lang)
                            };
                            name.replace(&src_book, &format!("/{}/", target_lang))
                        } else {
                            let src_file = if name.ends_with("en_us.json") {
                                "en_us.json".to_string()
                            } else {
                                src_suffix.clone()
                            };
                            name.replace(&src_file, &format!("{}.json", target_lang))
                        };
                        let mut target_base = serde_json::Value::Null;
                        if let Ok(mut zh_f) = archive.by_name(&target_name) {
                            let mut zh_c = String::new();
                            if zh_f.read_to_string(&mut zh_c).is_ok() {
                                target_base =
                                    serde_json::from_str(&zh_c).unwrap_or(serde_json::Value::Null);
                            }
                        }
                        // 讀取兄弟中文方言檔案（在 ZIP 內部搜尋），支援嵌套 JSON
                        let mut alt_map: HashMap<String, VecDeque<String>> = HashMap::new();
                        if let Some(ref alt) = alt_lang {
                            let alt_name = if name.contains("patchouli_books/") {
                                let src_part = if name.contains("/en_us/") {
                                    "/en_us/"
                                } else {
                                    &src_dir
                                };
                                name.replace(src_part, &format!("/{}/", alt))
                            } else {
                                let src_part = if name.ends_with("en_us.json") {
                                    "en_us.json"
                                } else {
                                    &src_suffix
                                };
                                name.replace(src_part, &format!("{}.json", alt))
                            };
                            if let Ok(mut alt_entry) = archive.by_name(&alt_name) {
                                let mut alt_content = String::new();
                                if alt_entry.read_to_string(&mut alt_content).is_ok() {
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
                        entries.push((name, value, content, target_base, alt_map));

                        // 移除此處的多餘遞增，由 pipeline 統一計算
                    }
                }
            }
            Ok(entries)
        },
    )
    .await??;

    let mut file_tasks = Vec::new();
    let mut global_items = Vec::new();
    let glossary_automaton = Arc::new(crate::translation::glossary::GlossaryAutomaton::new_simple(
        &HashMap::new(),
        &HashMap::new(),
    ));

    for (idx, (name, source_value, content, target_base, mut alt_map)) in
        tasks_data.into_iter().enumerate()
    {
        let file_id = start_file_id + idx;
        let mut pending = Vec::new();
        let empty_map = HashMap::new();
        let empty_vec = Vec::new();
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
            translation_memory: state.translation_memory.clone(),
            skip_memory: false,
            pause_notifier: state.pause_notifier.clone(),
            i18n: &state.i18n,
            filename: crate::utils::helpers::extract_display_path(Path::new(&name)),
        });

        engine::collect_translatable_strings(&source_value, &target_base, None, &mut pending, &ctx);
        let prefilled_count = ctx.prefilled.lock().unwrap().len();
        if !pending.is_empty() || prefilled_count > 0 {
            for (orig, key) in pending {
                let mut item = GlobalBatchItem::new(&orig, file_id, &name, &key);
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
                let mut item = GlobalBatchItem::new(orig, file_id, &name, key);
                item.translated = Some(trans.clone());
                global_items.push(item);
            }
            file_tasks.push(FileTask::new_json(
                file_id,
                path,
                name.clone(),
                content,
                source_value,
                target_base,
            ));
        }
    }

    Ok((file_tasks, global_items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::job::{JobConfig, JobSharedState};
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::{Arc, Mutex};

    fn create_mock_shared_state() -> JobSharedState {
        let config = JobConfig {
            source_lang: "zh_tw".to_string(),
            target_lang: "en_us".to_string(),
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
    async fn test_collect_jar_tasks_read_fail() {
        let state = create_mock_shared_state();
        let non_existent = std::path::PathBuf::from("non_existent_file_xyz.jar");
        let res = collect_jar_tasks(1, &non_existent, &state).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_collect_jar_tasks_corrupt_jar() {
        let temp_dir = std::env::temp_dir().join("mc_translator_jar_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("corrupt.jar");
        std::fs::write(&file_path, b"not a zip file content").unwrap();

        let state = create_mock_shared_state();
        let res = collect_jar_tasks(1, &file_path, &state).await;
        assert!(res.is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_collect_jar_tasks_success() {
        let temp_dir = std::env::temp_dir().join("mc_translator_jar_test_success");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.jar");
        {
            let file = std::fs::File::create(&file_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::FileOptions::<()>::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file::<_, ()>("assets/minecraft/lang/en_us.json", options)
                .unwrap();
            zip.write_all(r#"{"menu.play": "Play", "menu.options": "Options"}"#.as_bytes())
                .unwrap();

            // 增加 patchouli_books 測試路徑
            zip.start_file::<_, ()>(
                "assets/minecraft/patchouli_books/guide/en_us/book.json",
                options,
            )
            .unwrap();
            zip.write_all(r#"{"name": "Guide"}"#.as_bytes()).unwrap();

            zip.finish().unwrap();
        }

        let state = create_mock_shared_state();
        {
            let mut cfg = state.config.lock().unwrap();
            cfg.source_lang = "zh_tw".to_string(); // Trigger fallback
            cfg.target_lang = "en_us".to_string();
        }

        // 觸發 translation_memory 預填項目 (覆蓋率提升)
        {
            let mut tm = state.translation_memory.lock().unwrap();
            tm.insert("Play".to_string(), "遊玩".to_string());
        }

        let res = collect_jar_tasks(1, &file_path, &state).await;
        assert!(res.is_ok());
        let (tasks, items) = res.unwrap();

        assert!(!tasks.is_empty());
        assert!(!items.is_empty());

        // 驗證是否包含 patchouli 路徑或標準路徑
        let has_book = tasks.iter().any(|t| t.rel_path.contains("patchouli_books"));
        assert!(has_book);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
