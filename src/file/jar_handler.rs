use crate::translation::batching::GlobalBatchItem;
use crate::translation::job::JobSharedState;
use crate::file::pipeline::FileTask;
use crate::translation::context::{TranslationContext, ContextOptions};
use crate::translation::engine;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

pub async fn collect_jar_tasks(
    start_file_id: usize,
    path: &Path,
    state: &JobSharedState,
) -> Result<
    (Vec<FileTask>, Vec<GlobalBatchItem>),
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
                    let mut f = match archive.by_index(i) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("\x1b[31m[{}] [錯誤] 無法讀取 JAR 檔案索引 {}: {}\x1b[0m", 
                                chrono::Local::now().format("%H:%M:%S"), i, e);
                            continue;
                        }
                    };
                    let name = f.name().to_string();
                    let is_book = name.contains("patchouli_books") && name.contains("en_us");
                    let is_en_us = (name.ends_with("en_us.json")
                        || (name.contains("/en_us/") && name.ends_with(".json")))
                        && name.ends_with(".json");
                    let is_target = is_en_us && !(is_book && skip_book);
                    let mut content = String::new();
                    if is_target {
                        if let Err(e) = f.read_to_string(&mut content) {
                            eprintln!("\x1b[31m[{}] [錯誤] 無法讀取 JAR 內檔案 {}: {}\x1b[0m", 
                                chrono::Local::now().format("%H:%M:%S"), name, e);
                        }
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
    let glossary_automaton = Arc::new(crate::translation::glossary::GlossaryAutomaton::new_simple(
        &HashMap::new(),
        &HashMap::new(),
    ));

    for (idx, (name, en_us, content, zh_tw)) in tasks_data.into_iter().enumerate() {
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
            filename: name.clone(),
            translation_memory: state.translation_memory.clone(),
            skip_memory: false,
            pause_notifier: state.pause_notifier.clone(),
        });

        engine::collect_translatable_strings(&en_us, &zh_tw, None, &mut pending, &ctx);
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
                en_us,
                zh_tw,
            ));
        }
    }

    Ok((file_tasks, global_items))
}

pub fn write_inner_temp(
    config: &crate::translation::job::JobConfig,
    jar_path: &Path,
    inner_path: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base_output = if config.output_dir.is_empty() {
        Path::new("LLMTranslator")
    } else {
        Path::new(&config.output_dir)
    };
    
    let output_path = if config.output_dir.is_empty() {
        base_output.to_path_buf()
    } else {
        base_output.join("LLMTranslator")
    };

    let jar_name = jar_path.file_name().unwrap_or_default().to_string_lossy();
    let temp_repack_dir = output_path.join("temp_repack").join(&*jar_name);
    
    let fs_path = temp_repack_dir.join(inner_path);
    if let Some(parent) = fs_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    fs::write(&fs_path, content)?;
    Ok(())
}

pub fn repack_jar(
    path: &Path,
    _translated_files: &HashMap<String, String>,
    config: &crate::translation::job::JobConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base_output = if config.output_dir.is_empty() {
        Path::new("LLMTranslator")
    } else {
        Path::new(&config.output_dir)
    };
    
    let output_path = if config.output_dir.is_empty() {
        base_output.to_path_buf()
    } else {
        base_output.join("LLMTranslator")
    };

    let jar_name = path.file_name().unwrap_or_default().to_string_lossy();
    let temp_repack_dir = output_path.join("temp_repack").join(&*jar_name);
    
    if !temp_repack_dir.exists() {
        return Ok(());
    }

    let temp_jar_path = path.with_extension("jar.tmp");

    // 建立待替換檔案列表 (從磁碟掃描)
    let mut disk_files = HashMap::new();
    for entry in walkdir::WalkDir::new(&temp_repack_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(&temp_repack_dir)?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let content = fs::read_to_string(entry.path())?;
            disk_files.insert(rel_str, content);
        }
    }

    if disk_files.is_empty() {
        return Ok(());
    }

    let mut target_names = std::collections::HashSet::new();
    for name in disk_files.keys() {
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

        for (name, content) in disk_files {
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
    
    // 清理暫存目錄
    let _ = fs::remove_dir_all(&temp_repack_dir);

    Ok(())
}
