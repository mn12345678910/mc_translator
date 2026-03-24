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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::context::{ContextOptions, TranslationContext};
    use crate::translation::glossary::GlossaryAutomaton;
    use crate::translation::job::JobConfig;
    use std::collections::HashMap;
    use std::sync::{atomic::AtomicBool, atomic::AtomicU32, Arc, Mutex};

    fn setup_test_context<'a>(
        config: JobConfig,
        glossary: &'a GlossaryAutomaton,
        i18n: &'a crate::i18n::CommonLabels,
        inferred: &'a HashMap<String, String>,
        terms: &'a Vec<(String, String)>,
    ) -> TranslationContext<'a> {
        TranslationContext::new(ContextOptions {
            config: Arc::new(Mutex::new(config)),
            inferred,
            terms,
            glossary_automaton: glossary,
            status: Arc::new(Mutex::new("".to_string())),
            progress: Arc::new(AtomicU32::new(0)),
            total_progress: Arc::new(AtomicU32::new(0)),
            cancelled: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            current_log: Arc::new(Mutex::new(Vec::new())),
            filename: "test.json".to_string(),
            translation_memory: Arc::new(Mutex::new(HashMap::new())),
            skip_memory: false,
            pause_notifier: Arc::new(tokio::sync::Notify::new()),
            i18n,
        })
    }

    #[test]
    fn test_collect_translatable_strings_skips() {
        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();
        let config = JobConfig::default();

        let ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);
        let mut pending = Vec::new();

        // 1. 測試 should_skip_key
        let val_skip_key = serde_json::json!("should_translate_me");
        collect_translatable_strings(
            &val_skip_key,
            &serde_json::Value::Null,
            Some("id"),
            &mut pending,
            &ctx,
        );
        assert!(pending.is_empty(), "應該跳過 'id' key");

        // 2. 測試 should_skip_value
        let val_skip_value = serde_json::json!("123.45"); // 測試數值字串或日期應被跳過
        collect_translatable_strings(
            &val_skip_value,
            &serde_json::Value::Null,
            Some("text"),
            &mut pending,
            &ctx,
        );
        assert!(pending.is_empty(), "應該跳過純數字/日期 value");

        // 3. 正常收集
        let val_ok = serde_json::json!("Hello World");
        collect_translatable_strings(
            &val_ok,
            &serde_json::Value::Null,
            Some("text"),
            &mut pending,
            &ctx,
        );
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, "Hello World");
    }

    #[test]
    fn test_collect_translatable_strings_nested() {
        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();
        let config = JobConfig::default();

        let ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);
        let mut pending = Vec::new();

        let nested_json = serde_json::json!({
            "title": "Welcome",
            "list": [
                "Item 1",
                "Item 2",
                { "nested_key": "Nested Value" }
            ],
            "details": {
                "desc": "Detail info"
            }
        });

        collect_translatable_strings(
            &nested_json,
            &serde_json::Value::Null,
            None,
            &mut pending,
            &ctx,
        );

        // 應收集到 5 個字串:
        // "Welcome" (title)
        // "Item 1" (__ARRAY_ELEMENT__)
        // "Item 2" (__ARRAY_ELEMENT__)
        // "Nested Value" (nested_key)
        // "Detail info" (desc)
        assert_eq!(pending.len(), 5);
        let collected_values: Vec<String> = pending.iter().map(|(s, _)| s.clone()).collect();
        assert!(collected_values.contains(&"Welcome".to_string()));
        assert!(collected_values.contains(&"Item 1".to_string()));
        assert!(collected_values.contains(&"Item 2".to_string()));
        assert!(collected_values.contains(&"Nested Value".to_string()));
        assert!(collected_values.contains(&"Detail info".to_string()));
    }

    #[test]
    fn test_collect_translatable_strings_prefilled() {
        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();
        let config = JobConfig::default();

        let ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);
        let mut pending = Vec::new();

        let value = serde_json::json!("Original Text");
        let target_base = serde_json::json!("已翻譯文字");

        collect_translatable_strings(&value, &target_base, Some("title"), &mut pending, &ctx);

        // 因為 target_base 有值且不等於 value，這會計入 ctx.prefilled 並且 early return，pending 應維持為空
        assert!(pending.is_empty(), "Prefilled 應該跳過 pending 填充");
        let prefilled = ctx.prefilled.lock().unwrap();
        assert_eq!(prefilled.len(), 1);
        assert_eq!(prefilled[0].0, "Original Text");
        assert_eq!(prefilled[0].1, "title");
        assert_eq!(prefilled[0].2, "已翻譯文字");
    }

    #[tokio::test]
    async fn test_translate_json_recursive_batch_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // 模擬 Ollama API: 批次回應
        let mock_response = serde_json::json!({
            "response": "{\"translated\": \"[1] translated-A [2] translated-B\"}"
        });

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();

        let mut config = JobConfig::default();
        config.api_provider = "Ollama".to_string();
        config.ollama_url = server.uri();
        config.selected_model = "llama3".to_string();
        config.batch_size = 5; // 大於 1
        config.timeout = 30;
        config.target_lang = "zh_tw".to_string(); // 觸發 hanconv::s2tw

        let mut ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);

        let mut source_json = serde_json::json!({
            "itemA": "text-A",
            "itemB": "text-B"
        });

        let res = translate_json_recursive(
            &mut source_json,
            &serde_json::Value::Null,
            None,
            vec![],
            &mut ctx,
            None,
        )
        .await;

        assert!(res.is_ok());
        let translations = ctx.translations.lock().unwrap();
        assert!(translations.contains_key("itemA") || translations.contains_key("itemB"));
    }

    #[tokio::test]
    async fn test_translate_json_recursive_single_item() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // 模擬單位回應 (單筆與批次的處理路徑不同)
        let mock_response = serde_json::json!({
            "response": "{\"translated\": \"single-translated-text\"}"
        });

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();

        let mut config = JobConfig::default();
        config.api_provider = "Ollama".to_string();
        config.ollama_url = server.uri();
        config.selected_model = "llama3".to_string();
        config.batch_size = 1; // 1 會跳過批次直接進入單筆
        config.timeout = 30;
        config.target_lang = "zh_tw".to_string(); // 觸發 hanconv::s2tw

        let mut ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);

        let mut source_json = serde_json::json!({
            "itemA": "text-A"
        });

        let res = translate_json_recursive(
            &mut source_json,
            &serde_json::Value::Null,
            None,
            vec![],
            &mut ctx,
            None,
        )
        .await;

        assert!(res.is_ok());
        let translations = ctx.translations.lock().unwrap();
        assert!(translations.contains_key("itemA"));
    }

    #[test]
    fn test_count_strings_nested() {
        let value = serde_json::json!({
            "title": "Welcome",
            "list": ["Item 1", "Item 2"],
            "id": 123 // 跳過 key
        });
        let count = count_strings(&value, None, &serde_json::Value::Null);
        // title: 1, list: 2, id: skipped. Total 3
        assert_eq!(count, 3);
    }

    #[test]
    fn test_collect_translatable_strings_skip_memory() {
        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();
        let config = JobConfig::default();

        let mut ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);
        ctx.skip_memory = true; // 觸發 skip_memory 分支
        let mut pending = Vec::new();

        collect_translatable_strings(
            &serde_json::json!("Test text"),
            &serde_json::Value::Null,
            None,
            &mut pending,
            &ctx,
        );
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, "Test text");
    }

    #[tokio::test]
    async fn test_translate_json_recursive_empty_json() {
        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();
        let config = JobConfig::default();

        let mut ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);
        let mut source_json = serde_json::json!({
            "id": "123" // 測試 skip_key 或 skip_value
        });

        // 導致 pending_items 為空，觸發 early return
        let res = translate_json_recursive(
            &mut source_json,
            &serde_json::Value::Null,
            None,
            vec![],
            &mut ctx,
            None,
        )
        .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_realtime_save_file_helper_works() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("realtime_test.json");
        let content = "{\"items\": []}".to_string();
        let mut map = HashMap::new();
        map.insert("items".to_string(), vec![]);

        realtime_save_file_helper(path.clone(), content, map).await;
        assert!(path.exists());
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_translate_json_recursive_batch_error_logging() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // 模擬 Ollama API 報錯 (例如 500)
        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();

        let mut config = JobConfig::default();
        config.api_provider = "Ollama".to_string();
        config.ollama_url = server.uri();
        config.selected_model = "llama3".to_string();
        config.batch_size = 5;
        config.timeout = 30;

        let mut ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);

        let mut source_json = serde_json::json!({
            "itemA": "text-A",
            "itemB": "text-B"
        });

        let res = translate_json_recursive(
            &mut source_json,
            &serde_json::Value::Null,
            None,
            vec![],
            &mut ctx,
            None,
        )
        .await;

        // 批次報錯如果為 API_ERROR 等重大錯誤，將會 early return Err
        assert!(res.is_err(), "批次重大錯誤應該提早退出");
    }

    #[tokio::test]
    async fn test_translate_json_recursive_single_error_logging() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // 模擬 Ollama API 報錯
        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let i18n = crate::i18n::CommonLabels::default();
        let glossary = GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        );
        let inferred = HashMap::new();
        let terms = Vec::new();

        let mut config = JobConfig::default();
        config.api_provider = "Ollama".to_string();
        config.ollama_url = server.uri();
        config.selected_model = "llama3".to_string();
        config.batch_size = 1; // 單筆
        config.timeout = 30;

        let mut ctx = setup_test_context(config, &glossary, &i18n, &inferred, &terms);

        let mut source_json = serde_json::json!({
            "itemA": "text-A"
        });

        let res = translate_json_recursive(
            &mut source_json,
            &serde_json::Value::Null,
            None,
            vec![],
            &mut ctx,
            None,
        )
        .await;

        assert!(res.is_ok());
        let logs = ctx.current_log.lock().unwrap();
        assert!(!logs.is_empty(), "應該有單筆錯誤日誌記錄");
    }
}
