use crate::translation::api;
use crate::translation::job::JobConfig;
use crate::utils::text_processing::{postprocess_text, preprocess_text, validate_and_cleanup};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

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
    progress: Arc<AtomicU32>,
    current_batch: Arc<AtomicU32>, // 當前批次
    total_batches: Arc<AtomicU32>, // 總批次
    cancelled: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    log: Arc<Mutex<Vec<String>>>,
    pause_notifier: Arc<tokio::sync::Notify>,
    glossary_automaton: &crate::translation::glossary::GlossaryAutomaton,
    i18n: &crate::i18n::CommonLabels,
    file_name: &str,
    global_items_offset: usize, // 新增：全域 offset
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_translation_batch(RunBatchContext {
        items,
        config,
        status,
        progress,
        current_batch,
        total_batches,
        counter: Arc::new(Mutex::new(0)),
        log,
        cancelled,
        paused,
        pause_notifier,
        glossary_automaton,
        i18n,
        file_name: file_name.to_string(),
        global_items_offset, // 傳遞 offset
    })
    .await
}

pub struct RunBatchContext<'a> {
    pub items: &'a mut [GlobalBatchItem],
    pub config: Arc<Mutex<JobConfig>>,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<AtomicU32>,
    pub current_batch: Arc<AtomicU32>,
    pub total_batches: Arc<AtomicU32>,
    pub counter: Arc<Mutex<usize>>,
    pub log: Arc<Mutex<Vec<String>>>,
    pub cancelled: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub glossary_automaton: &'a crate::translation::glossary::GlossaryAutomaton,
    pub i18n: &'a crate::i18n::CommonLabels,
    pub file_name: String,
    pub global_items_offset: usize, // 新增
}

/// 執行一組全域翻譯批次 (包含重試與降級邏輯)
pub async fn run_translation_batch(
    ctx: RunBatchContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let items = ctx.items;
    let config = ctx.config;
    let status = ctx.status;
    let progress = ctx.progress;
    let current_batch = ctx.current_batch;
    let total_batches = ctx.total_batches;
    let _counter = ctx.counter;
    let log = ctx.log;
    let cancelled = ctx.cancelled;
    let paused = ctx.paused;
    let pause_notifier = ctx.pause_notifier;
    let glossary_automaton = ctx.glossary_automaton;
    let i18n = ctx.i18n;

    let cfg = config.lock().unwrap().clone();
    let total_items = items.len();
    if total_items == 0 {
        return Ok(());
    }
    let mut success_count = 0;
    let mut failed_indices = Vec::new();

    // 1. 初次嘗試：僅處理尚未翻譯的項目
    let pending_indices: Vec<usize> = (0..items.len())
        .filter(|&i| items[i].translated.is_none())
        .collect();

    // 初始化本批次進度（包含全域 Offset 與原本就已翻譯的項目）
    let already_done = total_items - pending_indices.len();
    progress.store(
        ((ctx.global_items_offset + already_done) as f32).to_bits(),
        Ordering::SeqCst,
    );

    if pending_indices.is_empty() {
        progress.store(
            ((ctx.global_items_offset + total_items) as f32).to_bits(),
            Ordering::SeqCst,
        );
        return Ok(());
    }

    let initial_batches = create_adaptive_batches_from_indices(
        items,
        &pending_indices,
        cfg.batch_size,
        cfg.batch_max_chars,
    );
    total_batches.store(initial_batches.len() as u32, Ordering::SeqCst);

    for (batch_idx, batch_item_indices) in initial_batches.iter().enumerate() {
        if cancelled.load(Ordering::SeqCst) {
            break;
        }
        current_batch.store((batch_idx + 1) as u32, Ordering::SeqCst);

        while paused.load(Ordering::SeqCst) {
            if cancelled.load(Ordering::SeqCst) {
                break;
            }
            pause_notifier.notified().await;
        }
        if cancelled.load(Ordering::SeqCst) {
            break;
        }

        let batch_result = process_one_global_batch(BatchContext {
            all_items: items,
            batch_indices: batch_item_indices,
            config: &config,
            status_arc: &status,
            _log: &log,
            glossary_automaton,
            is_retry: false,
            i18n,
            file_name: &ctx.file_name,
        })
        .await;

        match batch_result {
            Ok(_) => {
                let mut batch_success = 0;
                for &idx in batch_item_indices {
                    if items[idx].translated.is_none() {
                        failed_indices.push(idx);
                    } else {
                        batch_success += 1;
                    }
                }
                success_count += batch_success;
                // 進度應包含原本就已翻譯的項目與 Offset
                let already_done = total_items - pending_indices.len();
                let current_progress = ctx.global_items_offset + already_done + success_count;
                progress.store((current_progress as f32).to_bits(), Ordering::SeqCst);
            }
            Err(e) => {
                let err_msg = e.to_string();
                crate::utils::add_log(
                    &log,
                    &i18n
                        .log_batch_failed_retry
                        .replace("{}", &(batch_idx + 1).to_string())
                        .replacen("{}", &initial_batches.len().to_string(), 1)
                        .replacen("{}", &err_msg, 1),
                    &cfg.source_lang,
                    &cfg.target_lang,
                    &ctx.file_name,
                );
                failed_indices.extend(batch_item_indices.clone());
            }
        }
    }

    // 2. 失敗重試階段：收集失敗項目，切割更細批次 (降級：批次與字元上限減半)
    if !failed_indices.is_empty() && !cancelled.load(Ordering::SeqCst) {
        crate::utils::add_log(
            &log,
            &i18n
                .log_retry_start
                .replace("{}", &failed_indices.len().to_string()),
            &cfg.source_lang,
            &cfg.target_lang,
            &ctx.file_name,
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

        for batch in retry_batches.iter() {
            if cancelled.load(Ordering::SeqCst) {
                break;
            }
            let res = process_one_global_batch(BatchContext {
                all_items: items,
                batch_indices: batch,
                config: &config,
                status_arc: &status,
                _log: &log,
                glossary_automaton,
                is_retry: true,
                i18n,
                file_name: &ctx.file_name,
            })
            .await;

            match res {
                Ok(_) => {
                    let mut batch_success = 0;
                    for &idx in batch {
                        if items[idx].translated.is_none() {
                            second_failed_indices.push(idx);
                        } else {
                            batch_success += 1;
                        }
                    }
                    success_count += batch_success;
                    let already_done = total_items - pending_indices.len();
                    progress.store(
                        ((ctx.global_items_offset + already_done + success_count) as f32).to_bits(),
                        Ordering::SeqCst,
                    );
                }
                Err(_) => {
                    second_failed_indices.extend(batch.clone());
                }
            }
        }

        // 3. 最終回退階段：單筆翻譯
        if !second_failed_indices.is_empty() && !cancelled.load(Ordering::SeqCst) {
            crate::utils::add_log(
                &log,
                &i18n
                    .log_single_retry_start
                    .replace("{}", &second_failed_indices.len().to_string()),
                &cfg.source_lang,
                &cfg.target_lang,
                &ctx.file_name,
            );
            for &idx in &second_failed_indices {
                if cancelled.load(Ordering::SeqCst) {
                    break;
                }
                let item = &mut items[idx];

                let glossary = glossary_automaton.extract(std::slice::from_ref(&item.original));
                let entries = crate::utils::hashmap_to_entries(&glossary);

                let cfg_snapshot = config.lock().unwrap().clone();
                match api::translate_one(
                    &item.preprocessed,
                    &cfg_snapshot,
                    &ctx.file_name,
                    Some(&entries),
                )
                .await
                {
                    Ok(translated) => {
                        let restored = postprocess_text(&translated, &item.markers);
                        let cleaned = validate_and_cleanup(&restored);
                        item.translated = Some(hanconv::s2tw(&cleaned));
                        success_count += 1;
                        let already_done = total_items - pending_indices.len();
                        progress.store(
                            ((ctx.global_items_offset + already_done + success_count) as f32)
                                .to_bits(),
                            Ordering::SeqCst,
                        );
                    }
                    Err(e) => {
                        crate::utils::add_log(
                            &log,
                            &i18n.log_single_final_failed.replace("{}", &e.to_string()),
                            &cfg.source_lang,
                            &cfg.target_lang,
                            &ctx.file_name,
                        );
                    }
                }
            }
        }
    }

    // 移除全域統計日誌，由 pipeline 統一按檔案路徑輸出
    Ok(())
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
    glossary_automaton: &'a crate::translation::glossary::GlossaryAutomaton,
    is_retry: bool,
    i18n: &'a crate::i18n::CommonLabels,
    file_name: &'a str,
}

/// 執行單一批次的 LLM 翻譯請求
async fn process_one_global_batch(
    ctx: BatchContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mode_str = if ctx.is_retry {
        &ctx.i18n.status_retry
    } else {
        &ctx.i18n.status_translating
    };

    *ctx.status_arc.lock().unwrap() = format!("{}...", mode_str);

    // 1. 準備批次文本 (優化：使用批次內相對索引)
    let (tagged_texts, texts_to_translate) =
        build_tagged_batch_texts(ctx.all_items, ctx.batch_indices);

    // 2. 提取術語
    let glossary = ctx
        .glossary_automaton
        .extract(texts_to_translate.as_slice());
    let entries = crate::utils::hashmap_to_entries(&glossary);

    // 3. 呼叫翻譯服務
    let cfg = ctx.config.lock().unwrap().clone();
    match api::translate_batch(&tagged_texts, &cfg, ctx.file_name, Some(&entries)).await {
        Ok(results_map) => {
            // 4. 解析結果 (優化：使用正則表達式更靈活地從結果地圖中提取)
            let resolved_any = apply_batch_results(ctx.all_items, ctx.batch_indices, &results_map);

            if resolved_any {
                Ok(())
            } else {
                Err(ctx.i18n.log_batch_invalid.clone().into())
            }
        }
        Err(e) => Err(e),
    }
}

/// 準備批次文本 (優化：使用批次內相對索引)
fn build_tagged_batch_texts(
    all_items: &[GlobalBatchItem],
    batch_indices: &[usize],
) -> (Vec<String>, Vec<String>) {
    let mut texts_to_translate = Vec::new();
    let mut tagged_texts = Vec::new();
    let mut seen_texts = std::collections::HashSet::new();

    for (p_idx, &idx) in batch_indices.iter().enumerate() {
        let item = &all_items[idx];
        if seen_texts.contains(&item.preprocessed) {
            continue;
        }
        seen_texts.insert(item.preprocessed.clone());

        tagged_texts.push(format!("[i{}]{}", p_idx, item.preprocessed));
        texts_to_translate.push(item.preprocessed.clone());
    }
    (tagged_texts, texts_to_translate)
}

/// 解析配對並套用批次結果
fn apply_batch_results(
    all_items: &mut [GlobalBatchItem],
    batch_indices: &[usize],
    results_map: &std::collections::HashMap<String, String>,
) -> bool {
    let mut resolved_any = false;
    let tag_re = regex::Regex::new(r"\[i(\d+)\]").unwrap();

    for (orig_tagged, trans_tagged) in results_map {
        if let Some(caps) = tag_re.captures(orig_tagged) {
            if let Ok(relative_idx) = caps[1].parse::<usize>() {
                if relative_idx < batch_indices.len() {
                    let abs_idx = batch_indices[relative_idx];
                    let orig_text = all_items[abs_idx].original.clone();

                    let clean_translated = tag_re.replace_all(trans_tagged, "").trim().to_string();
                    let restored = postprocess_text(&clean_translated, &all_items[abs_idx].markers);
                    let cleaned = validate_and_cleanup(&restored);
                    let final_trans = hanconv::s2tw(&cleaned);

                    for &other_abs_idx in batch_indices {
                        if all_items[other_abs_idx].original == orig_text {
                            all_items[other_abs_idx].translated = Some(final_trans.clone());
                        }
                    }
                    resolved_any = true;
                }
            }
        }
    }
    resolved_any
}
