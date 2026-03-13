use crate::translation::api;
use crate::translation::job::JobConfig;
use crate::utils::text_processing::{
    postprocess_text, preprocess_text, validate_and_cleanup,
};
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
    progress: Arc<Mutex<f32>>,
    progress_total: Arc<Mutex<f32>>, // 目前檔案總條目數
    cancelled: Arc<Mutex<bool>>,
    paused: Arc<Mutex<bool>>,
    log: Arc<Mutex<Vec<String>>>,
    pause_notifier: Arc<tokio::sync::Notify>,
    glossary_automaton: &crate::translation::glossary::GlossaryAutomaton,
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
    pub glossary_automaton: &'a crate::translation::glossary::GlossaryAutomaton,
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

    // 1. 初次嘗試：僅處理尚未翻譯的項目
    let pending_indices: Vec<usize> = (0..items.len())
        .filter(|&i| items[i].translated.is_none())
        .collect();

    if pending_indices.is_empty() {
        *progress.lock().unwrap() = total_items as f32;
        return Ok(());
    }

    let initial_batches = create_adaptive_batches_from_indices(items, &pending_indices, cfg.batch_size, cfg.batch_max_chars);

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
                // 進度應包含原本就已翻譯的項目
                let already_done = total_items - pending_indices.len();
                *progress.lock().unwrap() = (already_done + success_count) as f32;
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
                    let already_done = total_items - pending_indices.len();
                    *progress.lock().unwrap() = (already_done + success_count) as f32;
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
                match api::translate_one(
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
                        let already_done = total_items - pending_indices.len();
                        *progress.lock().unwrap() = (already_done + success_count) as f32;
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
    match api::translate_batch(&tagged_texts, &cfg, Some(&entries)).await {
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
