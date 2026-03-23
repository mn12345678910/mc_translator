use crate::translation::api;
use crate::translation::context::TranslationContext;
use crate::utils::skip_rules::{should_skip_key, should_skip_value};
use crate::utils::text_processing::{
    detect_loop, postprocess_text, preprocess_text, sync_formatting, validate_and_cleanup,
};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

/// 根據介面提供規則過濾是否需要翻譯
pub async fn translate_json_recursive(
    source_value: &mut serde_json::Value,
    target_base: &serde_json::Value,
    key_name: Option<&str>,
    _current_path: Vec<String>,
    ctx: &mut TranslationContext<'_>,
    realtime_save_info: Option<(std::path::PathBuf, String)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut pending_items = Vec::new();
    collect_translatable_strings(source_value, target_base, key_name, &mut pending_items, ctx);

    if pending_items.is_empty() {
        ctx.total_progress.store(0.0f32.to_bits(), Ordering::SeqCst);
        ctx.progress.store(0.0f32.to_bits(), Ordering::SeqCst);
        return Ok(());
    }

    let total_to_translate = pending_items.len();
    ctx.total_progress
        .store((total_to_translate as f32).to_bits(), Ordering::SeqCst);
    ctx.progress.store(0.0f32.to_bits(), Ordering::SeqCst);
    *ctx.counter.lock().unwrap() = 0;

    let mut results = HashMap::new();

    let mut unique_texts = std::collections::HashSet::new();
    for (orig, _) in &pending_items {
        unique_texts.insert(orig.clone());
    }
    let unique_pending: Vec<String> = unique_texts.into_iter().collect();
    let total_unique_to_translate = unique_pending.len();

    ctx.total_progress.store(
        (total_unique_to_translate as f32).to_bits(),
        Ordering::SeqCst,
    );
    ctx.progress.store(0.0f32.to_bits(), Ordering::SeqCst);
    *ctx.counter.lock().unwrap() = 0;

    let current_config = ctx.config.lock().unwrap().clone();
    if (current_config.api_provider == "Ollama" || !current_config.api_key.is_empty())
        && unique_pending.len() > 1
        && current_config.batch_size > 1
    {
        let chunks: Vec<Vec<String>> = unique_pending
            .chunks(current_config.batch_size as usize)
            .map(|c| c.to_vec())
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let texts: Vec<String> = chunk.clone();

            let mut preprocessed_texts = Vec::new();
            let mut markers_list = Vec::new();
            for t in &texts {
                let (p, m) = preprocess_text(t);
                preprocessed_texts.push(p);
                markers_list.push(m);
            }

            let chunk_glossary = if !unique_pending.is_empty() {
                Some(ctx.glossary_automaton.extract(texts.as_slice()))
            } else {
                None
            };

            *ctx.status.lock().unwrap() = ctx
                .i18n
                .status_processing_batch
                .replace("{}", &(i + 1).to_string())
                .replacen("{}", &chunks.len().to_string(), 1)
                .replacen("{}", &ctx.filename, 1);

            let chunk_entries = chunk_glossary.map(|g| crate::utils::hashmap_to_entries(&g));
            let current_batch_config = ctx.config.lock().unwrap().clone();
            match api::translate_batch(
                &preprocessed_texts,
                &current_batch_config,
                &ctx.filename,
                chunk_entries.as_deref(),
            )
            .await
            {
                Ok(map) => {
                    let mut unique_resolved = 0;
                    for (p_idx, p_input_text) in preprocessed_texts.iter().enumerate() {
                        if let Some(translated_p) = map.get(p_input_text) {
                            let orig_text = &texts[p_idx];
                            let restored = postprocess_text(translated_p, &markers_list[p_idx]);
                            let finalized = validate_and_cleanup(&restored);
                            let translated = if current_batch_config.target_lang == "zh_tw" {
                                hanconv::s2tw(&finalized)
                            } else {
                                finalized
                            };

                            for (p_orig, path_key) in &pending_items {
                                if p_orig == orig_text {
                                    ctx.translations
                                        .lock()
                                        .unwrap()
                                        .entry(path_key.clone())
                                        .or_default()
                                        .push(translated.clone());
                                }
                            }
                            if results.contains_key(orig_text) {
                                continue;
                            }
                            results.insert(orig_text.clone(), translated.clone());
                            unique_resolved += 1;
                        }
                    }

                    if let Some((ref fs_path, ref content)) = realtime_save_info {
                        let translations_map = ctx.translations.lock().unwrap().clone();
                        realtime_save_file_helper(
                            fs_path.clone(),
                            content.clone(),
                            translations_map,
                        )
                        .await;
                    }

                    let current = {
                        let mut counter_guard = ctx.counter.lock().unwrap();
                        *counter_guard += unique_resolved;
                        *counter_guard
                    };
                    ctx.progress
                        .store((current as f32).to_bits(), Ordering::SeqCst);
                    *ctx.status.lock().unwrap() = ctx
                        .i18n
                        .status_processing_batch
                        .replace("{}", &ctx.filename)
                        .replacen("{}", &current.to_string(), 1)
                        .replacen("{}", &total_unique_to_translate.to_string(), 1);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.starts_with("TIMEOUT:")
                        || err_msg.starts_with("API_ERROR:")
                        || err_msg.starts_with("NETWORK_ERROR:")
                        || err_msg.starts_with("PARSE_ERROR:")
                        || err_msg.starts_with("UNSUPPORTED:")
                    {
                        return Err(e);
                    }
                    ctx.current_log
                        .lock()
                        .unwrap()
                        .push(ctx.i18n.log_batch_error.replace("{}", &err_msg));
                }
            }

            while ctx.paused.load(Ordering::SeqCst) {
                if ctx.cancelled.load(Ordering::SeqCst) {
                    return Ok(());
                }
                ctx.pause_notifier.notified().await;
            }
            if ctx.cancelled.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    for orig_text in unique_pending.iter() {
        if results.contains_key(orig_text) {
            continue;
        }

        let (preprocessed, markers) = preprocess_text(orig_text);
        let mut finalized_str = None;

        *ctx.status.lock().unwrap() = ctx
            .i18n
            .status_translating_item
            .replace("{}", &ctx.filename);
        let single_glossary = Some(
            ctx.glossary_automaton
                .extract(std::slice::from_ref(orig_text)),
        );
        let single_entries = single_glossary.map(|g| crate::utils::hashmap_to_entries(&g));
        let current_single_config = ctx.config.lock().unwrap().clone();
        match api::translate_one(
            &preprocessed,
            &current_single_config,
            &ctx.filename,
            single_entries.as_deref(),
        )
        .await
        {
            Ok(translated) => {
                if let Some(final_str) = finalize_single_translation(
                    &translated,
                    &markers,
                    &current_single_config.target_lang,
                ) {
                    finalized_str = Some(final_str);
                } else {
                    ctx.current_log
                        .lock()
                        .unwrap()
                        .push(ctx.i18n.log_loop_detected.replace("{}", &ctx.filename));
                    finalized_str = None;
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.starts_with("TIMEOUT:") {
                    return Err(e);
                }
                ctx.current_log
                    .lock()
                    .unwrap()
                    .push(ctx.i18n.log_single_failed.replace("{}", &err_msg));
            }
        }

        if let Some(val) = finalized_str {
            for (p_orig, path_key) in &pending_items {
                if p_orig == orig_text {
                    ctx.translations
                        .lock()
                        .unwrap()
                        .entry(path_key.clone())
                        .or_default()
                        .push(val.clone());
                }
            }
            results.insert(orig_text.clone(), val.clone());

            if let Some((ref fs_path, ref content)) = realtime_save_info {
                let translations_map = ctx.translations.lock().unwrap().clone();
                realtime_save_file_helper(fs_path.clone(), content.clone(), translations_map).await;
            }
        }

        let current = {
            let mut counter_guard = ctx.counter.lock().unwrap();
            *counter_guard += 1;
            *counter_guard
        };
        ctx.progress
            .store((current as f32).to_bits(), Ordering::SeqCst);
        *ctx.status.lock().unwrap() = ctx
            .i18n
            .status_processing_item
            .replace("{}", &ctx.filename)
            .replacen("{}", &current.to_string(), 1)
            .replacen("{}", &total_unique_to_translate.to_string(), 1);

        while ctx.paused.load(Ordering::SeqCst) {
            if ctx.cancelled.load(Ordering::SeqCst) {
                return Ok(());
            }
            ctx.pause_notifier.notified().await;
        }
        if ctx.cancelled.load(Ordering::SeqCst) {
            break;
        }
    }

    Ok(())
}

pub fn collect_translatable_strings(
    value: &serde_json::Value,
    target_base: &serde_json::Value,
    key_name: Option<&str>,
    pending: &mut Vec<(String, String)>,
    ctx: &TranslationContext<'_>,
) {
    match value {
        serde_json::Value::String(s) => {
            if let Some(k) = key_name {
                if should_skip_key(k) {
                    return;
                }
            }

            if should_skip_value(s) {
                return;
            }

            if ctx.skip_memory {
                pending.push((
                    s.clone(),
                    key_name.unwrap_or("__ARRAY_ELEMENT__").to_string(),
                ));
                return;
            }

            if let Some(existing) = target_base.as_str() {
                if !existing.is_empty() && existing != s {
                    let has_diff = false;

                    if !has_diff {
                        let key = key_name.unwrap_or("__ARRAY_ELEMENT__").to_string();
                        ctx.translations
                            .lock()
                            .unwrap()
                            .entry(key.clone())
                            .or_default()
                            .push(existing.to_string());
                        ctx.prefilled
                            .lock()
                            .unwrap()
                            .push((s.clone(), key, existing.to_string()));
                        return;
                    }
                }
            }

            pending.push((
                s.clone(),
                key_name.unwrap_or("__ARRAY_ELEMENT__").to_string(),
            ));
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let next_base = target_base.get(k).unwrap_or(&serde_json::Value::Null);
                collect_translatable_strings(v, next_base, Some(k), pending, ctx);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let next_base = target_base.get(i).unwrap_or(&serde_json::Value::Null);
                collect_translatable_strings(v, next_base, None, pending, ctx);
            }
        }
        _ => {}
    }
}

/// 遞迴計算一個 JSON 物件中需要翻譯的字串總數
#[allow(dead_code)]
pub fn count_strings(
    value: &serde_json::Value,
    key_name: Option<&str>,
    target_base: &serde_json::Value,
) -> usize {
    match value {
        serde_json::Value::String(_) => {
            if let Some(k) = key_name {
                if should_skip_key(k) {
                    return 0;
                }
                if let Some(existing_str) = target_base.as_str() {
                    if let Some(s_val) = value.as_str() {
                        if !existing_str.is_empty() && existing_str != s_val {
                            return 0;
                        }
                    }
                }
            }
            1
        }
        serde_json::Value::Object(map) => {
            let mut sum = 0;
            for (k, v) in map {
                let next_base = target_base.get(k).unwrap_or(&serde_json::Value::Null);
                sum += count_strings(v, Some(k), next_base);
            }
            sum
        }
        serde_json::Value::Array(arr) => {
            let mut sum = 0;
            for (i, v) in arr.iter().enumerate() {
                let next_base = target_base.get(i).unwrap_or(&serde_json::Value::Null);
                sum += count_strings(v, None, next_base);
            }
            sum
        }
        _ => 0,
    }
}

/// 項目翻譯結果清理與驗證
fn finalize_single_translation(
    translated: &str,
    markers: &[String],
    target_lang: &str,
) -> Option<String> {
    let restored = postprocess_text(translated, markers);
    let cleaned = validate_and_cleanup(&restored);

    if detect_loop(&cleaned) {
        None
    } else {
        Some(if target_lang == "zh_tw" {
            hanconv::s2tw(&cleaned)
        } else {
            cleaned
        })
    }
}

/// 即時儲存翻譯檔案
async fn realtime_save_file_helper(
    fs_path: std::path::PathBuf,
    content: String,
    translations_map: std::collections::HashMap<String, Vec<String>>,
) {
    let final_content = sync_formatting(&content, &translations_map);
    let _ = tokio::task::spawn_blocking(move || std::fs::write(fs_path, final_content)).await;
}
