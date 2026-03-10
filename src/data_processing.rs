//! # 數據處理模組
//! 負責全域批次翻譯、文本預處理、術語提取以及翻譯後的後處理邏輯。

use crate::translation_job::JobConfig;
use crate::translation_service;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 不需要翻譯的鍵名列表
const SKIP_KEYS: &[&str] = &[
    "icon",
    "id",
    "type",
    "category",
    "entity",
    "recipe",
    "recipe2",
    "advancement",
    "predicate",
    "parent",
    "flag",
    "ingredient",
    "item",
    "block",
    "tag",
    "registry_name",
    "entry",
    "model",
    "texture",
    "reset to default",
    "restore defaults",
    "default settings",
    "back",
    "next",
];

/// 翻譯上下文，攜帶字典與當前任務狀態等資訊
pub struct TranslationContext<'a> {
    pub config: Arc<Mutex<JobConfig>>,
    pub inferred: &'a HashMap<String, String>,
    pub terms: &'a Vec<(String, String)>,
    pub glossary_automaton: &'a crate::utils::GlossaryAutomaton,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<Mutex<f32>>,
    pub total_progress: Arc<Mutex<f32>>,
    pub cancelled: Arc<Mutex<bool>>,
    pub paused: Arc<Mutex<bool>>,
    pub current_log: Arc<Mutex<Vec<String>>>,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub filename: String,
    pub counter: Arc<Mutex<usize>>,
    pub translations: Arc<Mutex<HashMap<String, Vec<String>>>>,
    pub translation_memory: Arc<Mutex<HashMap<String, String>>>,
    pub skip_memory: bool,
}

pub struct ContextOptions<'a> {
    pub config: Arc<Mutex<JobConfig>>,
    pub inferred: &'a HashMap<String, String>,
    pub terms: &'a Vec<(String, String)>,
    pub glossary_automaton: &'a crate::utils::GlossaryAutomaton,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<Mutex<f32>>,
    pub total_progress: Arc<Mutex<f32>>,
    pub cancelled: Arc<Mutex<bool>>,
    pub paused: Arc<Mutex<bool>>,
    pub current_log: Arc<Mutex<Vec<String>>>,
    pub filename: String,
    pub translation_memory: Arc<Mutex<HashMap<String, String>>>,
    pub skip_memory: bool,
    pub pause_notifier: Arc<tokio::sync::Notify>,
}

impl<'a> TranslationContext<'a> {
    pub fn new(opts: ContextOptions<'a>) -> Self {
        Self {
            config: opts.config,
            inferred: opts.inferred,
            terms: opts.terms,
            glossary_automaton: opts.glossary_automaton,
            status: opts.status,
            progress: opts.progress,
            total_progress: opts.total_progress,
            cancelled: opts.cancelled,
            paused: opts.paused,
            current_log: opts.current_log,
            filename: opts.filename,
            counter: Arc::new(Mutex::new(0)),
            translations: Arc::new(Mutex::new(HashMap::new())),
            translation_memory: opts.translation_memory,
            skip_memory: opts.skip_memory,
            pause_notifier: opts.pause_notifier,
        }
    }
}

/// 在全域模式下的一個翻譯項目
#[derive(Debug, Clone)]
pub struct GlobalBatchItem {
    /// 原始內容（含格式代碼）
    pub original: String,
    /// 預處理後的純文本
    pub preprocessed: String,
    /// 預處理標記
    pub markers: Vec<String>,
    /// 來源檔案 ID（對應到 FileTask）
    pub file_id: usize,
    /// 在檔案中的鍵或路徑標記
    pub key: String,
    /// 最終翻譯結果
    pub translated: Option<String>,
}

impl GlobalBatchItem {
    pub fn new(original: &str, file_id: usize, key: &str) -> Self {
        let (preprocessed, markers) = preprocess_text(original);
        Self {
            original: original.to_string(),
            preprocessed,
            markers,
            file_id,
            key: key.to_string(),
            translated: None,
        }
    }
}

/// 全域翻譯批次處理核心函數
#[allow(clippy::too_many_arguments)]
pub async fn translate_global_batches(
    items: &mut [GlobalBatchItem],
    config: Arc<Mutex<JobConfig>>,
    status: Arc<Mutex<String>>,
    progress: Arc<Mutex<f32>>,
    progress_total: Arc<Mutex<f32>>, // 目前檔案總條目數
    cancelled: Arc<Mutex<bool>>,
    paused: Arc<Mutex<bool>>,
    log: Arc<Mutex<Vec<String>>>,
    pause_notifier: Arc<tokio::sync::Notify>,
    glossary_automaton: &crate::utils::GlossaryAutomaton,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 首先設定目前的條目總數
    *progress_total.lock().unwrap() = items.len() as f32;

    run_translation_batch(RunBatchContext {
        items,
        config,
        status,
        progress,
        counter: Arc::new(Mutex::new(0)),
        log,
        cancelled,
        paused,
        pause_notifier,
        glossary_automaton,
    })
    .await
}

pub struct RunBatchContext<'a> {
    pub items: &'a mut [GlobalBatchItem],
    pub config: Arc<Mutex<JobConfig>>,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<Mutex<f32>>,
    pub counter: Arc<Mutex<usize>>,
    pub log: Arc<Mutex<Vec<String>>>,
    pub cancelled: Arc<Mutex<bool>>,
    pub paused: Arc<Mutex<bool>>,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub glossary_automaton: &'a crate::utils::GlossaryAutomaton,
}

/// 執行一組全域翻譯批次 (包含重試與降級邏輯)
pub async fn run_translation_batch(
    ctx: RunBatchContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let items = ctx.items;
    let config = ctx.config;
    let status = ctx.status;
    let progress = ctx.progress;
    let _counter = ctx.counter;
    let log = ctx.log;
    let cancelled = ctx.cancelled;
    let paused = ctx.paused;
    let pause_notifier = ctx.pause_notifier;
    let glossary_automaton = ctx.glossary_automaton;

    let cfg = config.lock().unwrap().clone();
    let total_items = items.len();
    if total_items == 0 {
        return Ok(());
    }
    *progress.lock().unwrap() = 0.0;

    let mut success_count = 0;
    let mut failed_indices = Vec::new();

    // 1. 初次嘗試：使用使用者設定的上限 (Adaptive: batch_size & batch_max_chars)
    let initial_batches = create_adaptive_batches(items, cfg.batch_size, cfg.batch_max_chars);

    for (batch_idx, batch_item_indices) in initial_batches.iter().enumerate() {
        if *cancelled.lock().unwrap() {
            break;
        }
        while *paused.lock().unwrap() {
            if *cancelled.lock().unwrap() {
                break;
            }
            *status.lock().unwrap() = "⏸ 已暫停".to_string();
            pause_notifier.notified().await;
        }
        if *cancelled.lock().unwrap() {
            break;
        }

        let batch_result = process_one_global_batch(BatchContext {
            all_items: items,
            batch_indices: batch_item_indices,
            config: &config,
            status_arc: &status,
            _log: &log,
            glossary_automaton,
            current_idx: batch_idx + 1,
            total_batch: initial_batches.len(),
            is_retry: false,
        })
        .await;

        match batch_result {
            Ok(_) => {
                success_count += batch_item_indices.len();
                *progress.lock().unwrap() = success_count as f32;
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.starts_with("OLLAMA_TIMEOUT:") {
                    return Err(e);
                }
                crate::utils::add_log(
                    &log,
                    &format!(
                        "⚠️ 批次 {}/{} 翻譯失敗: {}, 已加入重試佇列",
                        batch_idx + 1,
                        initial_batches.len(),
                        err_msg
                    ),
                );
                failed_indices.extend(batch_item_indices.clone());
            }
        }
    }

    // 2. 失敗重試階段：收集失敗項目，切割更細批次 (降級：批次與字元上限減半)
    if !failed_indices.is_empty() && !*cancelled.lock().unwrap() {
        crate::utils::add_log(
            &log,
            &format!(">>> 開始失敗批次重試 ({} 條)...", failed_indices.len()),
        );

        let retry_batch_size = (cfg.batch_size / 2).max(1);
        let retry_max_chars = (cfg.batch_max_chars / 2).max(100);

        let retry_batches = create_adaptive_batches_from_indices(
            items,
            &failed_indices,
            retry_batch_size,
            retry_max_chars,
        );
        let mut second_failed_indices = Vec::new();

        for (idx, batch) in retry_batches.iter().enumerate() {
            if *cancelled.lock().unwrap() {
                break;
            }
            let res = process_one_global_batch(BatchContext {
                all_items: items,
                batch_indices: batch,
                config: &config,
                status_arc: &status,
                _log: &log,
                glossary_automaton,
                current_idx: idx + 1,
                total_batch: retry_batches.len(),
                is_retry: true,
            })
            .await;

            match res {
                Ok(_) => {
                    success_count += batch.len();
                    *progress.lock().unwrap() = success_count as f32;
                }
                Err(_) => {
                    second_failed_indices.extend(batch.clone());
                }
            }
        }

        // 3. 最終回退階段：單筆翻譯
        if !second_failed_indices.is_empty() && !*cancelled.lock().unwrap() {
            crate::utils::add_log(
                &log,
                &format!(
                    ">>> 開始最終單筆重試 ({} 條)...",
                    second_failed_indices.len()
                ),
            );
            for &idx in &second_failed_indices {
                if *cancelled.lock().unwrap() {
                    break;
                }
                let item = &mut items[idx];

                let glossary = glossary_automaton.extract(std::slice::from_ref(&item.original));
                let entries = crate::utils::hashmap_to_entries(&glossary);

                let cfg_snapshot = config.lock().unwrap().clone();
                match translation_service::translate_one(
                    &item.preprocessed,
                    &cfg_snapshot,
                    Some(&entries),
                )
                .await
                {
                    Ok(translated) => {
                        let restored = postprocess_text(&translated, &item.markers);
                        let cleaned = validate_and_cleanup(&restored);
                        item.translated = Some(hanconv::s2tw(&cleaned));
                        success_count += 1;
                        *progress.lock().unwrap() = success_count as f32;
                    }
                    Err(e) => {
                        crate::utils::add_log(&log, &format!("❌ 條目翻譯最終失敗: {}", e));
                    }
                }
            }
        }
    }

    crate::utils::add_log(
        &log,
        &format!(
            "✅ 全域批次翻譯完成 (成功: {}/{})",
            success_count, total_items
        ),
    );
    Ok(())
}

/// 切割自適應批次 (遵守行數與字數上限)
fn create_adaptive_batches(
    items: &[GlobalBatchItem],
    max_items: u32,
    max_chars: u32,
) -> Vec<Vec<usize>> {
    let indices: Vec<usize> = (0..items.len()).collect();
    create_adaptive_batches_from_indices(items, &indices, max_items, max_chars)
}

fn create_adaptive_batches_from_indices(
    items: &[GlobalBatchItem],
    indices: &[usize],
    max_items: u32,
    max_chars: u32,
) -> Vec<Vec<usize>> {
    let mut batches = Vec::new();
    let mut current_batch = Vec::new();
    let mut current_chars = 0;

    for &idx in indices {
        let item = &items[idx];
        let item_len = item.preprocessed.len() as u32;

        if (!current_batch.is_empty())
            && (current_batch.len() as u32 >= max_items || current_chars + item_len > max_chars)
        {
            batches.push(current_batch);
            current_batch = Vec::new();
            current_chars = 0;
        }

        current_batch.push(idx);
        current_chars += item_len;
    }

    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    batches
}

struct BatchContext<'a> {
    all_items: &'a mut [GlobalBatchItem],
    batch_indices: &'a [usize],
    config: &'a Arc<Mutex<JobConfig>>,
    status_arc: &'a Arc<Mutex<String>>,
    _log: &'a Arc<Mutex<Vec<String>>>,
    glossary_automaton: &'a crate::utils::GlossaryAutomaton,
    current_idx: usize,
    total_batch: usize,
    is_retry: bool,
}

/// 執行單一批次的 LLM 翻譯請求
async fn process_one_global_batch(
    ctx: BatchContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mode_str = if ctx.is_retry { "重試" } else { "翻譯" };
    *ctx.status_arc.lock().unwrap() = format!(
        "{}中 (批次 {}/{})",
        mode_str, ctx.current_idx, ctx.total_batch
    );

    // 1. 準備批次文本
    let mut texts_to_translate = Vec::new();
    let mut current_file_id = usize::MAX;

    let mut tagged_texts = Vec::new();
    for &idx in ctx.batch_indices {
        let item = &ctx.all_items[idx];
        if item.file_id != current_file_id {
            current_file_id = item.file_id;
            tagged_texts.push(format!("[f{}]", current_file_id));
        }
        tagged_texts.push(format!("[i{}]{}", idx, item.preprocessed));
        texts_to_translate.push(item.preprocessed.clone());
    }

    // 2. 提取術語
    let glossary = ctx
        .glossary_automaton
        .extract(texts_to_translate.as_slice());
    let entries = crate::utils::hashmap_to_entries(&glossary);

    // 3. 呼叫翻譯服務
    let cfg = ctx.config.lock().unwrap().clone();
    match translation_service::translate_batch(&tagged_texts, &cfg, Some(&entries)).await {
        Ok(results_map) => {
            let mut resolved_any = false;
            // 4. 解析結果
            for &idx in ctx.batch_indices {
                let tag = format!("[i{}]", idx);
                let mut translated_val = None;
                for (orig_tagged, trans_tagged) in &results_map {
                    if orig_tagged.contains(&tag) {
                        let clean = trans_tagged.replace(&tag, "").trim().to_string();
                        translated_val = Some(clean);
                        break;
                    }
                }

                if let Some(trans) = translated_val {
                    let item = &mut ctx.all_items[idx];
                    let restored = postprocess_text(&trans, &item.markers);
                    let cleaned = validate_and_cleanup(&restored);
                    item.translated = Some(hanconv::s2tw(&cleaned));
                    resolved_any = true;
                }
            }

            if resolved_any {
                Ok(())
            } else {
                Err("批次翻譯結果無效，無法解析任何條目".into())
            }
        }
        Err(e) => Err(e),
    }
}

fn should_skip_key(key: &str) -> bool {
    if SKIP_KEYS.iter().any(|&s| key.eq_ignore_ascii_case(s)) {
        return true;
    }

    let bytes = key.as_bytes();
    if bytes.len() >= 3 {
        let end = &bytes[bytes.len() - 3..];
        if end.eq_ignore_ascii_case(b"_id") {
            return true;
        }
        let start = &bytes[..3];
        if start.eq_ignore_ascii_case(b"id_") {
            return true;
        }
    }
    false
}

pub fn should_skip_value(val: &str) -> bool {
    if val.is_empty() {
        return true;
    }
    let s = val.trim();
    if s.is_empty() {
        return false;
    } // 只有空格的字串不應該被跳過

    let bytes = s.as_bytes();

    // 布林值
    if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
        return true;
    }

    // 正則表達式模式 (常見於 KubeJS)
    if bytes.len() > 1 && bytes[0] == b'/' && bytes[bytes.len() - 1] == b'/' {
        return true;
    }

    // 純數字、點、負號
    if bytes
        .iter()
        .all(|&c| c.is_ascii_digit() || c == b'.' || c == b'-')
    {
        return true;
    }

    // 命名空間 ID，例如 "tconstruct:broad_axe"
    let contains_space = s.contains(' ');
    if !contains_space && s.contains(':') {
        return true;
    }

    // 以 # 或 @ 開頭的標記
    if !bytes.is_empty() && (bytes[0] == b'#' || bytes[0] == b'@') {
        return true;
    }

    // snake_case ID
    if !contains_space
        && s.contains('_')
        && bytes.iter().all(|&c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'/' || c == b'.'
        })
    {
        return true;
    }

    false
}

/// 根據介面提供規則過濾是否需要翻譯
pub async fn translate_json_recursive(
    en_us: &mut serde_json::Value,
    zh_tw_base: &serde_json::Value,
    key_name: Option<&str>,
    _current_path: Vec<String>,
    ctx: &mut TranslationContext<'_>,
    realtime_save_info: Option<(std::path::PathBuf, String)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut pending_items = Vec::new();
    collect_translatable_strings(en_us, zh_tw_base, key_name, &mut pending_items, ctx);

    if pending_items.is_empty() {
        *ctx.total_progress.lock().unwrap() = 0.0;
        *ctx.progress.lock().unwrap() = 0.0;
        return Ok(());
    }

    let total_to_translate = pending_items.len();
    *ctx.total_progress.lock().unwrap() = total_to_translate as f32;
    *ctx.progress.lock().unwrap() = 0.0;
    *ctx.counter.lock().unwrap() = 0;

    let mut results = HashMap::new();

    let mut unique_texts = std::collections::HashSet::new();
    for (orig, _) in &pending_items {
        unique_texts.insert(orig.clone());
    }
    let unique_pending: Vec<String> = unique_texts.into_iter().collect();
    let total_unique_to_translate = unique_pending.len();

    *ctx.total_progress.lock().unwrap() = total_unique_to_translate as f32;
    *ctx.progress.lock().unwrap() = 0.0;
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

            *ctx.status.lock().unwrap() = format!(
                "正在翻譯批次 ({}/{})，檔案 ({})",
                i + 1,
                chunks.len(),
                ctx.filename
            );

            let chunk_entries = chunk_glossary.map(|g| crate::utils::hashmap_to_entries(&g));
            let current_batch_config = ctx.config.lock().unwrap().clone();
            match translation_service::translate_batch(
                &preprocessed_texts,
                &current_batch_config,
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
                            let translated = hanconv::s2tw(&finalized);

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
                        let final_zh_tw_content = sync_formatting(content, &translations_map);

                        let fs_path_clone = fs_path.clone();
                        let _ =
                            tokio::task::spawn_blocking(move || -> Result<(), std::io::Error> {
                                std::fs::write(&fs_path_clone, &final_zh_tw_content)
                            })
                            .await;
                    }

                    let current = {
                        let mut c = ctx.counter.lock().unwrap();
                        *c += unique_resolved;
                        *c
                    };
                    *ctx.progress.lock().unwrap() = current as f32;
                    *ctx.status.lock().unwrap() = format!(
                        "正在處理 {} ({}/{})",
                        ctx.filename, current, total_unique_to_translate
                    );
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.starts_with("OLLAMA_TIMEOUT:")
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
                        .push(format!("批次翻譯出錯: {}, 將改用單筆重試", err_msg));
                }
            }

            while *ctx.paused.lock().unwrap() {
                if *ctx.cancelled.lock().unwrap() {
                    break;
                }
                ctx.pause_notifier.notified().await;
            }
            if *ctx.cancelled.lock().unwrap() {
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

        *ctx.status.lock().unwrap() = format!("正在翻譯條目 ({})", ctx.filename);
        let single_glossary = Some(
            ctx.glossary_automaton
                .extract(std::slice::from_ref(orig_text)),
        );
        let single_entries = single_glossary.map(|g| crate::utils::hashmap_to_entries(&g));
        let current_single_config = ctx.config.lock().unwrap().clone();
        match translation_service::translate_one(
            &preprocessed,
            &current_single_config,
            single_entries.as_deref(),
        )
        .await
        {
            Ok(translated) => {
                let restored = postprocess_text(&translated, &markers);
                let cleaned = validate_and_cleanup(&restored);

                // [新增] 循環產出防護
                if detect_loop(&cleaned) {
                    ctx.current_log.lock().unwrap().push(format!(
                        "⚠️ 偵測到條目 ({}) 陷入翻譯循環，已跳過",
                        ctx.filename
                    ));
                    finalized_str = None;
                } else {
                    finalized_str = Some(hanconv::s2tw(&cleaned));
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.starts_with("OLLAMA_TIMEOUT:") {
                    return Err(e);
                }
                ctx.current_log
                    .lock()
                    .unwrap()
                    .push(format!("單筆翻譯失敗: {}", err_msg));
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
                let final_zh_tw_content = sync_formatting(content, &translations_map);

                let fs_path_clone = fs_path.clone();
                let _ = tokio::task::spawn_blocking(move || -> Result<(), std::io::Error> {
                    std::fs::write(&fs_path_clone, &final_zh_tw_content)
                        .map_err(std::io::Error::other)
                })
                .await;
            }
        }

        let current = {
            let mut c = ctx.counter.lock().unwrap();
            *c += 1;
            *c
        };
        *ctx.progress.lock().unwrap() = current as f32;
        *ctx.status.lock().unwrap() = format!(
            "正在處理 {} ({}/{})",
            ctx.filename, current, total_unique_to_translate
        );

        while *ctx.paused.lock().unwrap() {
            if *ctx.cancelled.lock().unwrap() {
                break;
            }
            ctx.pause_notifier.notified().await;
        }
        if *ctx.cancelled.lock().unwrap() {
            break;
        }
    }

    Ok(())
}

pub fn collect_translatable_strings(
    value: &serde_json::Value,
    zh_tw_base: &serde_json::Value,
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

            if let Some(existing) = zh_tw_base.as_str() {
                if !existing.is_empty() && existing != s {
                    let has_diff = false;

                    if !has_diff {
                        ctx.translations
                            .lock()
                            .unwrap()
                            .entry(key_name.unwrap_or("__ARRAY_ELEMENT__").to_string())
                            .or_default()
                            .push(existing.to_string());
                        return;
                    }
                }
            }

            // [還原] 次高優先級：官方建議詞精確匹配 (inferred)
            if let Some(matched) = ctx.inferred.get(&s.to_lowercase()) {
                ctx.translations
                    .lock()
                    .unwrap()
                    .entry(key_name.unwrap_or("__ARRAY_ELEMENT__").to_string())
                    .or_default()
                    .push(matched.clone());
                return;
            }

            // [還原] 次高優先級：翻譯記憶體 (TM)
            if let Some(cached) = ctx.translation_memory.lock().unwrap().get(s) {
                let has_diff = false;
                if !has_diff {
                    ctx.translations
                        .lock()
                        .unwrap()
                        .entry(key_name.unwrap_or("__ARRAY_ELEMENT__").to_string())
                        .or_default()
                        .push(cached.clone());
                    return;
                }
            }

            pending.push((
                s.clone(),
                key_name.unwrap_or("__ARRAY_ELEMENT__").to_string(),
            ));
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let next_base = zh_tw_base.get(k).unwrap_or(&serde_json::Value::Null);
                collect_translatable_strings(v, next_base, Some(k), pending, ctx);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let next_base = zh_tw_base.get(i).unwrap_or(&serde_json::Value::Null);
                collect_translatable_strings(v, next_base, None, pending, ctx);
            }
        }
        _ => {}
    }
}

/// 解析翻譯結果，移除 Markdown 代碼塊或修復損壞的標籤
pub fn validate_and_cleanup(text: &str) -> String {
    let mut s = text.trim().to_string();

    if s.contains("```") {
        if let Some(start) = s.find("```") {
            let sub = &s[start + 3..];
            if let Some(end) = sub.find("```") {
                let content = sub[..end].trim();
                let lines: Vec<&str> = content.lines().collect();
                if !lines.is_empty()
                    && (lines[0] == "json"
                        || lines[0] == "text"
                        || lines[0] == "javascript"
                        || lines[0] == "js")
                {
                    s = lines[1..].join("\n").trim().to_string();
                } else {
                    s = content.to_string();
                }
            }
        }
    }

    if s.contains('\n') {
        let lines: Vec<&str> = s
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        for l in &lines {
            if l.contains("翻譯：")
                || l.contains("譯文：")
                || l.contains("Translation:")
                || l.contains("Translated:")
                || l.contains("Result:")
            {
                if let Some(pos) = l.find('：') {
                    let candidate = l[pos + '：'.len_utf8()..]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('「')
                        .trim_matches('」')
                        .trim_matches('『')
                        .trim_matches('』')
                        .to_string();
                    if !candidate.is_empty() && candidate != "{}" && candidate != "{ }" {
                        return candidate;
                    }
                }
                if let Some(pos) = l.find(':') {
                    let candidate = l[pos + 1..]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('「')
                        .trim_matches('」')
                        .trim_matches('『')
                        .trim_matches('』')
                        .to_string();
                    if !candidate.is_empty() && candidate != "{}" && candidate != "{ }" {
                        return candidate;
                    }
                }
            }
        }
    }

    if (s.starts_with('{') && s.ends_with('}')) || (s.starts_with('[') && s.ends_with(']')) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            if let Some(obj) = v.as_object() {
                if let Some(val) = obj.values().next().and_then(|v| v.as_str()) {
                    s = val.to_string();
                } else if obj.is_empty() {
                    s = String::new();
                }
            } else if let Some(arr) = v.as_array() {
                if arr.is_empty() {
                    s = String::new();
                }
            }
        }
    }

    let prefixes = [
        "Translation:",
        "Translated:",
        "翻譯：",
        "譯文：",
        "Note:",
        "註：",
        "結果：",
        "Output:",
        "Result:",
    ];
    for p in prefixes {
        if s.to_lowercase().starts_with(&p.to_lowercase()) {
            s = s[p.len()..].trim().to_string();
        }
    }

    if s.contains("我們已將") || s.contains("以下是翻譯") || s.contains("JSON 格式") {
        let lines: Vec<&str> = s
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .filter(|l| {
                !l.contains("我們已將")
                    && !l.contains("以下是翻譯")
                    && !l.contains("JSON 格式")
                    && !l.contains("請確認")
            })
            .collect();
        s = lines.join("\n");
    }

    s = s.trim().to_string();

    let mut chars: Vec<char> = s.chars().collect();
    if chars.len() >= 2 {
        let first = chars[0];
        let last = chars[chars.len() - 1];
        if (first == '"' && last == '"')
            || (first == '\'' && last == '\'')
            || (first == '「' && last == '」')
            || (first == '『' && last == '』')
        {
            chars.pop();
            chars.remove(0);
            s = chars.into_iter().collect::<String>().trim().to_string();
        }
    }

    if s == "{}" || s == "{ }" || s == "[]" || s == "[ ]" {
        return String::new();
    }

    s
}

/// 偵測翻譯文字是否陷入無限循環
pub fn detect_loop(text: &str) -> bool {
    if text.len() > 2000 {
        return true;
    } // 基本長度防護 (Minecraft 條目通常不會這麼長)

    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 10 {
        return false;
    }

    // 1. 簡單重複偵測 (例如 "文字文字文字...")
    for chunk_size in 2..=10 {
        if chars.len() < chunk_size * 4 {
            continue;
        }
        for i in 0..(chars.len() - chunk_size * 3) {
            let chunk = &chars[i..i + chunk_size];
            let next1 = &chars[i + chunk_size..i + chunk_size * 2];
            let next2 = &chars[i + chunk_size * 2..i + chunk_size * 3];
            if chunk == next1 && chunk == next2 {
                return true;
            }
        }
    }
    false
}

/// 使用 Aho-Corasick 進行術語替換，防止遞迴倍增
use std::sync::LazyLock;

/// 匹配需要保留的格式代碼或預留位置 (如 &, #Hex, %s, %1$s, {0}, \n)
static PLACEHOLDER_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?x)
    (§[0-9a-fk-orA-FK-OR]) |
    (&[0-9a-fk-orA-FK-OR]) |
    (\#[0-9a-fA-F]{6}) |
    (%\d+\$[sd]|%[sd]) |
    (\{\d+\}) |
    (\\n)
"#,
    )
    .unwrap()
});

/// 將文本中的格式化標記替換為臨時預留位置，以防止 LLM 破壞格式
pub fn preprocess_text(text: &str) -> (String, Vec<String>) {
    let mut markers = Vec::new();
    let processed = PLACEHOLDER_RE
        .replace_all(text, |caps: &regex::Captures| {
            let matched = caps.get(0).unwrap().as_str();
            let idx = markers.len();
            markers.push(matched.to_string());

            if matched.starts_with('§') || matched.starts_with('&') {
                format!("%%MC_{}%%", idx)
            } else if matched.starts_with('#') {
                format!("%%HEX_{}%%", idx)
            } else {
                format!("%%VAR_{}%%", idx)
            }
        })
        .to_string();

    (processed, markers)
}

/// 將預留位置還原為原始格式標記
pub fn postprocess_text(text: &str, markers: &[String]) -> String {
    let mut result = text.to_string();
    for (i, marker) in markers.iter().enumerate() {
        let mc_placeholder = format!("%%MC_{}%%", i);
        let hex_placeholder = format!("%%HEX_{}%%", i);
        let var_placeholder = format!("%%VAR_{}%%", i);

        result = result.replace(&mc_placeholder, marker);
        result = result.replace(&hex_placeholder, marker);
        result = result.replace(&var_placeholder, marker);
    }
    result
}

/// 針對原始 JSON 檔案內容進行增量更新，儘量保留原始縮排與格式
pub fn sync_formatting(original: &str, translations: &HashMap<String, Vec<String>>) -> String {
    let mut result = String::with_capacity(original.len() + 2048);
    let mut counters = HashMap::<String, usize>::new();

    static KV_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#"^(\s*)"([^"]+)"\s*:\s*"[^"]*"(,?\s*)$"#).unwrap());
    static VAL_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#"^(\s*)"([^"]+)"(,?\s*)$"#).unwrap());

    for line in original.lines() {
        if let Some(caps) = KV_RE.captures(line) {
            let indent = &caps[1];
            let key = &caps[2];
            let suffix = &caps[3];

            let idx = counters.entry(key.to_string()).or_insert(0);
            if let Some(list) = translations.get(key) {
                if let Some(translated) = list.get(*idx) {
                    *idx += 1;
                    let json_str = serde_json::to_string(translated)
                        .unwrap_or_else(|_| format!(r#""{}""#, translated));
                    let escaped = &json_str[1..json_str.len() - 1];
                    result.push_str(&format!(r#"{}"{}" : "{}"{}"#, indent, key, escaped, suffix));
                    result.push('\n');
                    continue;
                }
            }
        } else if let Some(caps) = VAL_RE.captures(line) {
            let indent = &caps[1];
            let val = &caps[2];
            let suffix = &caps[3];

            if val != "[" && val != "{" {
                let key = "__ARRAY_ELEMENT__";
                let idx = counters.entry(key.to_string()).or_insert(0);
                if let Some(list) = translations.get(key) {
                    if let Some(translated) = list.get(*idx) {
                        *idx += 1;
                        let json_str = serde_json::to_string(translated)
                            .unwrap_or_else(|_| format!(r#""{}""#, translated));
                        let escaped = &json_str[1..json_str.len() - 1];
                        result.push_str(&format!(r#"{}"{}"{}"#, indent, escaped, suffix));
                        result.push('\n');
                        continue;
                    }
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// 遞迴計算一個 JSON 物件中需要翻譯的字串總數
#[allow(dead_code)]
pub fn count_strings(
    value: &serde_json::Value,
    key_name: Option<&str>,
    zh_tw_base: &serde_json::Value,
) -> usize {
    match value {
        serde_json::Value::String(_) => {
            if let Some(k) = key_name {
                if should_skip_key(k) {
                    return 0;
                }
                if let Some(existing_str) = zh_tw_base.as_str() {
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
                let next_base = zh_tw_base.get(k).unwrap_or(&serde_json::Value::Null);
                sum += count_strings(v, Some(k), next_base);
            }
            sum
        }
        serde_json::Value::Array(arr) => {
            let mut sum = 0;
            for (i, v) in arr.iter().enumerate() {
                let next_base = zh_tw_base.get(i).unwrap_or(&serde_json::Value::Null);
                sum += count_strings(v, None, next_base);
            }
            sum
        }
        _ => 0,
    }
}

pub fn hashmap_to_entries(map: &HashMap<String, String>) -> Vec<crate::utils::GlossaryEntry> {
    map.iter()
        .map(|(k, v)| crate::utils::GlossaryEntry {
            original: k.clone(),
            translated: v.clone(),
            source: crate::utils::TermType::Official,
        })
        .collect()
}
