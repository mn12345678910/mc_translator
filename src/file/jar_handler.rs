use crate::file::pipeline::FileTask;
use crate::translation::batching::GlobalBatchItem;
use crate::translation::context::{ContextOptions, TranslationContext};
use crate::translation::engine;
use crate::translation::job::JobSharedState;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

pub async fn collect_jar_tasks(
    start_file_id: usize,
    path: &Path,
    state: &JobSharedState,
) -> Result<(Vec<FileTask>, Vec<GlobalBatchItem>), Box<dyn std::error::Error + Send + Sync>> {
    let path_clone = path.to_path_buf();
    let (source_lang, target_lang, skip_book) = {
        let cfg = state.config.lock().unwrap();
        (
            cfg.source_lang.clone(),
            cfg.target_lang.clone(),
            cfg.skip_book,
        )
    };

    type JarTaskData = (String, serde_json::Value, String, serde_json::Value);
    let tasks_data = tokio::task::spawn_blocking(
        move || -> Result<Vec<JarTaskData>, Box<dyn std::error::Error + Send + Sync>> {
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
                            eprintln!(
                                "\x1b[31m[{}] [錯誤] 無法讀取 JAR 檔案索引 {}: {}\x1b[0m",
                                chrono::Local::now().format("%H:%M:%S"),
                                i,
                                e
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
                            eprintln!(
                                "\x1b[31m[{}] [錯誤] 無法讀取 JAR 內檔案 {}: {}\x1b[0m",
                                chrono::Local::now().format("%H:%M:%S"),
                                name,
                                e
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
                        entries.push((name, value, content, target_base));

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

    for (idx, (name, source_value, content, target_base)) in tasks_data.into_iter().enumerate() {
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
            filename: name.clone(),
        });

        engine::collect_translatable_strings(&source_value, &target_base, None, &mut pending, &ctx);
        let prefilled_count = ctx.prefilled.lock().unwrap().len();
        if !pending.is_empty() || prefilled_count > 0 {
            for (orig, key) in pending {
                global_items.push(GlobalBatchItem::new(&orig, file_id, &key));
            }
            // 加入預先填滿的項目 (修正丟失條目與進度條問題)
            for (orig, key, trans) in ctx.prefilled.lock().unwrap().iter() {
                let mut item = GlobalBatchItem::new(orig, file_id, key);
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

/// 已廢棄實體臨時目錄方案，改為純記憶體快取。此函數僅供相容性佔位，稍後將在 pipeline 中移除呼叫。
#[deprecated(note = "Use memory-based buffering instead")]
pub fn write_translated_to_temp_fs(
    _jar_path: &Path,
    _inner_path: &str,
    _content: &str,
    _config: &crate::translation::job::JobConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}

pub fn repack_jar(
    source_path: &Path,
    target_path: &Path,
    translated_files: &HashMap<String, String>, // 現在直接接收記憶體中的翻譯內容
    config: &crate::translation::job::JobConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if translated_files.is_empty() {
        return Ok(());
    }

    // 確保目標資料夾存在
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).unwrap_or(());
    }

    let temp_jar_path = target_path.with_extension("jar.tmp");

    {
        let temp_file = fs::File::create(&temp_jar_path)?;
        let mut zip_out = zip::ZipWriter::new(temp_file);

        let zip_in_file = fs::File::open(source_path)?;
        let mut zip_in = zip::ZipArchive::new(zip_in_file)?;

        let src_book_match = format!("/{}", config.source_lang);
        let tgt_book_replace = format!("/{}", config.target_lang);
        let src_suffix = format!("{}.json", config.source_lang);
        let tgt_suffix = format!("{}.json", config.target_lang);

        // 1. 建立目標檔案名稱集 (支援標準與 Patchouli 手冊路徑)
        let mut target_names = std::collections::HashSet::new();
        for name in translated_files.keys() {
            let actual_name = if name.contains("patchouli_books/") {
                name.replace(&src_book_match, &tgt_book_replace)
            } else if name.ends_with(&src_suffix) {
                name.replace(&src_suffix, &tgt_suffix)
            } else {
                name.clone()
            };
            target_names.insert(actual_name);
        }

        // 2. 串流式處理現有 Entry
        for i in 0..zip_in.len() {
            let mut entry = zip_in.by_index(i)?;
            let name = entry.name().to_string();

            if target_names.contains(&name) {
                continue; // 跳過將被替換的檔案
            }

            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            zip_out.start_file(name, options)?;
            zip_out.write_all(&buffer)?;
        }

        // 3. 寫入新的翻譯內容
        for (name, content) in translated_files {
            let actual_name = if name.contains("patchouli_books/") {
                name.replace(&src_book_match, &tgt_book_replace)
            } else if name.ends_with(&src_suffix) {
                name.replace(&src_suffix, &tgt_suffix)
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

    fs::rename(&temp_jar_path, target_path)?;
    Ok(())
}
