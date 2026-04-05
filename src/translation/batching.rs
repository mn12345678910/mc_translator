use crate::translation::api;
use crate::translation::job::JobConfig;
use crate::translation::{LogEntry, LogLevel};
use crate::utils::helpers::add_log_event;
use crate::utils::text_processing::{postprocess_text, preprocess_text, validate_and_cleanup};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

static BATCH_TAG_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[i(\d+)\]").unwrap());

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
    /// 來源檔案相對路徑 (用於日誌追蹤)
    pub rel_path: String,
    /// 在檔案中的鍵或路徑標記
    pub key: String,
    /// 最終翻譯結果
    pub translated: Option<String>,
    /// 兄弟中文語言檔案的對應值（用於跨語言快速簡繁轉換）
    /// 例如：來源為 en_us，目標為 zh_tw，若 zh_cn 存在同鍵，則此欄存放 zh_cn 的中文文本
    pub alt_source: Option<String>,
}

impl GlobalBatchItem {
    pub fn new(original: &str, file_id: usize, rel_path: &str, key: &str) -> Self {
        let (preprocessed, markers) = preprocess_text(original);
        Self {
            original: original.to_string(),
            preprocessed,
            markers,
            file_id,
            rel_path: rel_path.to_string(),
            key: key.to_string(),
            translated: None,
            alt_source: None,
        }
    }
}

/// 對文字執行 Aho-Corasick 全文術語替換，再交 hanconv 轉換
/// - `to_tw = true`：簡 → 繁（s2tw）
/// - `to_tw = false`：繁 → 簡（tw2s）
fn apply_glossary_then_hanconv(
    text: &str,
    automaton: &crate::translation::glossary::GlossaryAutomaton,
    to_tw: bool,
) -> String {
    // 1. 收集所有匹配位置（含重疊，使用 LeftmostLongest 策略）
    let mut matches: Vec<(usize, usize, String)> = automaton
        .ac
        .find_iter(text)
        .map(|mat| {
            let entry = &automaton.entries[mat.pattern().as_usize()];
            (mat.start(), mat.end(), entry.translated.clone())
        })
        .collect();

    // 2. 反向替換（避免 byte offset 位移）
    matches.sort_by(|a, b| b.0.cmp(&a.0));
    let mut result = text.to_string();
    for (start, end, replacement) in matches {
        // 確認 byte 邊界有效（防止 UTF-8 切割錯誤）
        if text.is_char_boundary(start) && text.is_char_boundary(end) {
            result.replace_range(start..end, &replacement);
        }
    }

    // 3. 再執行 hanconv 處理剩餘字元
    if to_tw {
        hanconv::s2tw(&result)
    } else {
        hanconv::tw2s(&result)
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
    log: Arc<Mutex<Vec<LogEntry>>>,
    pause_notifier: Arc<tokio::sync::Notify>,
    glossary_automaton: &crate::translation::glossary::GlossaryAutomaton,
    i18n: &crate::i18n::CommonLabels,
    file_name: &str,
    group_dir: &str,
    group_file_count: usize,
    global_items_offset: usize, // 全域 offset
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_translation_batch(RunBatchContext {
        items,
        config,
        status,
        progress,
        current_batch,
        total_batches,
        log,
        cancelled,
        paused,
        pause_notifier,
        glossary_automaton,
        i18n,
        file_name: file_name.to_string(),
        group_dir: group_dir.to_string(),
        group_file_count,
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
    pub log: Arc<Mutex<Vec<LogEntry>>>,
    pub cancelled: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub glossary_automaton: &'a crate::translation::glossary::GlossaryAutomaton,
    pub i18n: &'a crate::i18n::CommonLabels,
    pub file_name: String,
    pub group_dir: String,
    pub group_file_count: usize,
    pub global_items_offset: usize, // 全域 offset
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
        total_batches.store(1, Ordering::SeqCst); // 確保顯示 1/1 而非 0/0
        current_batch.store(1, Ordering::SeqCst);
        progress.store(
            ((ctx.global_items_offset + total_items) as f32).to_bits(),
            Ordering::SeqCst,
        );
        return Ok(());
    }

    // ── 快速簡繁轉換路徑 ──
    // 情況 A：來源與目標皆為簡繁中文互換（zh_cn↔zh_tw），直接全量轉換後返回
    let is_fast_pair = (cfg.source_lang == "zh_cn" && cfg.target_lang == "zh_tw")
        || (cfg.source_lang == "zh_tw" && cfg.target_lang == "zh_cn");

    if cfg.fast_convert && is_fast_pair {
        total_batches.store(1, Ordering::SeqCst);
        current_batch.store(1, Ordering::SeqCst);

        // 使用 apply_glossary_then_hanconv 進行逐條處理
        for &idx in &pending_indices {
            let original = &items[idx].original;

            // 術語優先全文替換 + hanconv
            let converted = apply_glossary_then_hanconv(
                original,
                glossary_automaton,
                cfg.source_lang == "zh_cn",
            );
            items[idx].translated = Some(converted);
            success_count += 1;
        }

        progress.store(
            ((ctx.global_items_offset + already_done + success_count) as f32).to_bits(),
            Ordering::SeqCst,
        );

        add_log_event(
            &log,
            LogLevel::Success,
            &format!(
                "[Fast Convert] {} items ({} → {})",
                success_count, cfg.source_lang, cfg.target_lang
            ),
            &cfg.source_lang,
            &cfg.target_lang,
            &ctx.group_dir,
            cfg.enable_debug_log,
        );

        return Ok(());
    }

    // ── 情況 B：混合快速轉換路徑 ──
    // 當 fast_convert 啟用且目標為中文，但來源為非中文語言時
    // 對有 alt_source（兄弟中文檔案的對應值）的條目直接進行 hanconv 轉換
    // 其餘條目繼續進入 LLM 批次
    let is_target_chinese = cfg.target_lang == "zh_cn" || cfg.target_lang == "zh_tw";

    if cfg.fast_convert && is_target_chinese && !is_fast_pair {
        let alt_indices: Vec<usize> = pending_indices
            .iter()
            .copied()
            .filter(|&i| items[i].alt_source.is_some())
            .collect();

        if !alt_indices.is_empty() {
            let mut fast_count = 0usize;
            for &idx in &alt_indices {
                let alt_text = items[idx].alt_source.clone().unwrap_or_default();

                // 術語優先全文替換 + hanconv
                let converted = apply_glossary_then_hanconv(
                    &alt_text,
                    glossary_automaton,
                    cfg.target_lang == "zh_tw",
                );
                items[idx].translated = Some(converted);
                fast_count += 1;
            }

            add_log_event(
                &log,
                LogLevel::Info,
                &format!(
                    "[Fast Convert] {} items via alt-source ({} → {}), {} items remaining for LLM",
                    fast_count,
                    cfg.source_lang,
                    cfg.target_lang,
                    pending_indices.len() - fast_count,
                ),
                &cfg.source_lang,
                &cfg.target_lang,
                &ctx.group_dir,
                cfg.enable_debug_log,
            );

            // 更新進度條
            progress.store(
                ((ctx.global_items_offset + already_done + fast_count) as f32).to_bits(),
                Ordering::SeqCst,
            );
        }
    }

    // [重要] 重新整理待翻譯清單：排除已透過混合快速轉換、術語表或 TM 預填完成的項目
    let pending_indices: Vec<usize> = pending_indices
        .into_iter()
        .filter(|&i| items[i].translated.is_none())
        .collect();

    // 重新計算 already_done，涵蓋 Case B 快速轉換的成果
    let already_done = total_items - pending_indices.len();

    // 如果所有項目都已透過快速轉換處理完畢，則提前返回
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
    current_batch.store(0, Ordering::SeqCst); // 確保重新開始時標籤為 0/N

    // [修復] 在進入批次迴圈前，僅顯示一次彙總資訊，依據分組格式顯示
    let log_msg = format!(
        "{}{}",
        i18n.log_processing_file_mask
            .replacen("{}", &ctx.group_file_count.to_string(), 1)
            .replacen("{}", "", 1),
        ctx.file_name
    );
    add_log_event(
        &log,
        LogLevel::Info,
        &log_msg,
        &cfg.source_lang,
        &cfg.target_lang,
        &ctx.group_dir,
        cfg.enable_debug_log,
    );

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
            glossary_automaton,
            i18n,
            file_name: &ctx.file_name,
            batch_idx,
            total_batches: initial_batches.len(),
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
                let current_progress = ctx.global_items_offset + already_done + success_count;
                progress.store((current_progress as f32).to_bits(), Ordering::SeqCst);
            }
            Err(e) => {
                let err_msg = e.to_string();
                add_log_event(
                    &log,
                    LogLevel::Error,
                    &i18n
                        .log_batch_failed_retry
                        .replacen("{}", &(batch_idx + 1).to_string(), 1)
                        .replacen("{}", &initial_batches.len().to_string(), 1)
                        .replacen("{}", &err_msg, 1),
                    &cfg.source_lang,
                    &cfg.target_lang,
                    &ctx.group_dir,
                    cfg.enable_debug_log,
                );
                failed_indices.extend(batch_item_indices.clone());
            }
        }
    }

    // 2. 失敗重試階段：收集失敗項目，切割更細批次 (降級：批次與字元上限減半)
    if !failed_indices.is_empty() && !cancelled.load(Ordering::SeqCst) {
        add_log_event(
            &log,
            LogLevel::Warn,
            &i18n
                .log_retry_start
                .replace("{}", &failed_indices.len().to_string()),
            &cfg.source_lang,
            &cfg.target_lang,
            &ctx.group_dir,
            cfg.enable_debug_log,
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

        for (retry_idx, batch) in retry_batches.iter().enumerate() {
            if cancelled.load(Ordering::SeqCst) {
                break;
            }
            total_batches.store(retry_batches.len() as u32, Ordering::SeqCst);
            current_batch.store((retry_idx + 1) as u32, Ordering::SeqCst);
            let res = process_one_global_batch(BatchContext {
                all_items: items,
                batch_indices: batch,
                config: &config,
                status_arc: &status,
                glossary_automaton,
                i18n,
                file_name: &ctx.file_name,
                batch_idx: retry_idx,
                total_batches: retry_batches.len(),
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
            add_log_event(
                &log,
                LogLevel::Warn,
                &i18n
                    .log_single_retry_start
                    .replace("{}", &second_failed_indices.len().to_string()),
                &cfg.source_lang,
                &cfg.target_lang,
                &ctx.group_dir,
                cfg.enable_debug_log,
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
                        let cleaned = validate_and_cleanup(
                            &restored,
                            &cfg_snapshot.cleanup_prefixes,
                            &cfg_snapshot.cleanup_contains,
                        );
                        let final_trans = if cfg_snapshot.target_lang == "zh_tw" {
                            hanconv::s2tw(&cleaned)
                        } else {
                            cleaned
                        };
                        item.translated = Some(final_trans);
                        success_count += 1;
                        progress.store(
                            ((ctx.global_items_offset + already_done + success_count) as f32)
                                .to_bits(),
                            Ordering::SeqCst,
                        );
                    }
                    Err(e) => {
                        let err_msg = format!(
                            "{}: {}",
                            item.rel_path,
                            i18n.log_single_final_failed.replace("{}", &e.to_string())
                        );
                        add_log_event(
                            &log,
                            LogLevel::Error,
                            &err_msg,
                            &cfg.source_lang,
                            &cfg.target_lang,
                            &ctx.group_dir,
                            cfg_snapshot.enable_debug_log,
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
    glossary_automaton: &'a crate::translation::glossary::GlossaryAutomaton,
    i18n: &'a crate::i18n::CommonLabels,
    file_name: &'a str,
    batch_idx: usize,
    total_batches: usize,
}

/// 執行單一批次的 LLM 翻譯請求
async fn process_one_global_batch(
    ctx: BatchContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 格式化狀態字串，僅顯示批次進度至狀態列
    *ctx.status_arc.lock().unwrap() = ctx
        .i18n
        .status_translating_batch
        .replacen("{}", &ctx.i18n.status_translating, 1)
        .replacen("{}", &(ctx.batch_idx + 1).to_string(), 1)
        .replacen("{}", &ctx.total_batches.to_string(), 1);

    // 3. 呼叫翻譯服務
    let cfg = ctx.config.lock().unwrap().clone();

    // 1. 準備批次文本 (優化：使用批次內相對索引)
    let (tagged_texts, texts_to_translate) =
        build_tagged_batch_texts(ctx.all_items, ctx.batch_indices);

    // 2. 提取術語
    let glossary = ctx
        .glossary_automaton
        .extract(texts_to_translate.as_slice());
    let entries = crate::utils::hashmap_to_entries(&glossary);

    match api::translate_batch(&tagged_texts, &cfg, ctx.file_name, Some(&entries)).await {
        Ok(results_map) => {
            // 4. 解析結果 (優化：使用正則表達式更靈活地從結果地圖中提取)
            let resolved_any = apply_batch_results(
                ctx.all_items,
                ctx.batch_indices,
                &results_map,
                &cfg.target_lang,
                &cfg.cleanup_prefixes,
                &cfg.cleanup_contains,
            );

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
    target_lang: &str,
    prefixes: &[String],
    contains: &[String],
) -> bool {
    let mut resolved_any = false;

    for (orig_tagged, trans_tagged) in results_map {
        if let Some(caps) = BATCH_TAG_RE.captures(orig_tagged) {
            if let Ok(relative_idx) = caps[1].parse::<usize>() {
                if relative_idx < batch_indices.len() {
                    let abs_idx = batch_indices[relative_idx];
                    let target_preprocessed = all_items[abs_idx].preprocessed.clone();

                    let clean_translated = BATCH_TAG_RE
                        .replace_all(trans_tagged, "")
                        .trim()
                        .to_string();

                    for &other_abs_idx in batch_indices {
                        if all_items[other_abs_idx].preprocessed == target_preprocessed {
                            let restored = postprocess_text(
                                &clean_translated,
                                &all_items[other_abs_idx].markers,
                            );
                            let cleaned = validate_and_cleanup(&restored, prefixes, contains);
                            let final_trans = if target_lang == "zh_tw" {
                                hanconv::s2tw(&cleaned)
                            } else {
                                cleaned
                            };
                            all_items[other_abs_idx].translated = Some(final_trans);
                        }
                    }
                    resolved_any = true;
                }
            }
        }
    }
    resolved_any
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_adaptive_batches_from_indices_limits() {
        let mut items = Vec::new();
        for i in 1..=5 {
            items.push(GlobalBatchItem::new(
                &format!("Item {}", i),
                i,
                "test.json",
                &format!("k{}", i),
            ));
        }

        let indices = vec![0, 1, 2, 3, 4];

        // 1. 測試數量上限：每批最多 2 件，不計字數
        let batches = create_adaptive_batches_from_indices(&items, &indices, 2, 1000);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec![0, 1]);
        assert_eq!(batches[1], vec![2, 3]);
        assert_eq!(batches[2], vec![4]);

        // 2. 測試字數上限：字數限制 13 (每個 Item 均為 6 字元，2件 12 字元，3件 18 字元)
        let batches_chars = create_adaptive_batches_from_indices(&items, &indices, 10, 13);
        assert_eq!(batches_chars.len(), 3); // 應切分為 [0, 1], [2, 3], [4]
        assert_eq!(batches_chars[0], vec![0, 1]);
        assert_eq!(batches_chars[1], vec![2, 3]);
        assert_eq!(batches_chars[2], vec![4]);
    }

    #[tokio::test]
    async fn test_run_translation_batch_empty() {
        let mut items: Vec<GlobalBatchItem> = Vec::new();
        let config = Arc::new(Mutex::new(JobConfig::default()));
        let status = Arc::new(Mutex::new(String::new()));
        let progress = Arc::new(AtomicU32::new(0));
        let current_batch = Arc::new(AtomicU32::new(0));
        let total_batches = Arc::new(AtomicU32::new(0));
        let log = Arc::new(Mutex::new(Vec::new()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(false));
        let pause_notifier = Arc::new(tokio::sync::Notify::new());
        let glossary_automaton = crate::translation::glossary::GlossaryAutomaton::new_simple(
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
        );
        let i18n = crate::i18n::CommonLabels::default();

        let ctx = RunBatchContext {
            items: &mut items,
            config,
            status,
            progress,
            current_batch,
            total_batches,
            log,
            cancelled,
            paused,
            pause_notifier,
            glossary_automaton: &glossary_automaton,
            i18n: &i18n,
            file_name: "test.json".to_string(),
            group_dir: "assets/lang/".to_string(),
            group_file_count: 1,
            global_items_offset: 0,
        };

        let result = run_translation_batch(ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_translation_batch_all_done() {
        let mut item = GlobalBatchItem::new("Apple", 1, "test.json", "k1");
        item.translated = Some("蘋果".to_string());
        let mut items = vec![item];

        let config = Arc::new(Mutex::new(JobConfig::default()));
        let status = Arc::new(Mutex::new(String::new()));
        let progress = Arc::new(AtomicU32::new(0));
        let current_batch = Arc::new(AtomicU32::new(0));
        let total_batches = Arc::new(AtomicU32::new(0));
        let log = Arc::new(Mutex::new(Vec::new()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(false));
        let pause_notifier = Arc::new(tokio::sync::Notify::new());
        let glossary_automaton = crate::translation::glossary::GlossaryAutomaton::new_simple(
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
        );
        let i18n = crate::i18n::CommonLabels::default();

        let ctx = RunBatchContext {
            items: &mut items,
            config,
            status,
            progress: Arc::clone(&progress),
            current_batch,
            total_batches,
            log,
            cancelled,
            paused,
            pause_notifier,
            glossary_automaton: &glossary_automaton,
            i18n: &i18n,
            file_name: "test.json".to_string(),
            group_dir: "assets/lang/".to_string(),
            group_file_count: 1,
            global_items_offset: 0,
        };

        let result = run_translation_batch(ctx).await;
        assert!(result.is_ok());

        // 進度應被記錄為 total_items = 1
        let val = f32::from_bits(progress.load(Ordering::SeqCst));
        assert_eq!(val, 1.0);
    }
}
