// tests/verify_ancient.rs
// 驗證 ancient.json 的批次內重複條目去重與自動套用邏輯

use mc_translator::translation::batching::GlobalBatchItem;
use std::collections::HashSet;

#[test]
fn test_intra_batch_deduplication_ancient_json() {
    println!("=== 開始驗證 ancient.json 去重邏輯 ===");

    let target_text = "This profile is read only and cannot be modified! If you want to make a new profile based on this then you can make a copy to a new name";

    // 1. 模擬 5 筆重複條目
    let mut items = [
        GlobalBatchItem::new(target_text, 1, "cities.__readonly__"),
        GlobalBatchItem::new(target_text, 1, "client.__readonly__"),
        GlobalBatchItem::new(target_text, 1, "explosions.__readonly__"),
        GlobalBatchItem::new(target_text, 1, "lostcity.__readonly__"),
        GlobalBatchItem::new(target_text, 1, "cityspheres.__readonly__"),
    ];

    let batch_indices = vec![0, 1, 2, 3, 4]; // 假設這5個都在同一個批次中

    // --------------------------------------------------
    // 2. 模擬 [標籤生成] 段落 (與 process_one_global_batch 代碼完全相同)
    // --------------------------------------------------
    let mut tagged_texts = Vec::new();
    let mut texts_to_translate = Vec::new();
    let mut seen_texts = HashSet::new();
    let mut current_file_id = usize::MAX;

    // 暫時模擬一組 file_map
    let mut file_map = std::collections::HashMap::new();
    let mut file_relative_id = 0;

    for (p_idx, &idx) in batch_indices.iter().enumerate() {
        let item = &items[idx];

        if seen_texts.contains(&item.preprocessed) {
            println!("   -> 項目索引 {} ({}) 被去重跳過！", idx, item.key);
            continue;
        }
        seen_texts.insert(item.preprocessed.clone());

        if item.file_id != current_file_id {
            current_file_id = item.file_id;
            let rel_f_id = *file_map.entry(current_file_id).or_insert_with(|| {
                let id = file_relative_id;
                file_relative_id += 1;
                id
            });
            tagged_texts.push(format!("[f{}]", rel_f_id));
        }
        tagged_texts.push(format!("[i{}]{}", p_idx, item.preprocessed));
        texts_to_translate.push(item.preprocessed.clone());
    }

    // 驗證標籤生成結果
    println!("\n[發送給 LLM 的標籤計數]:");
    println!("- tagged_texts 數量: {} (含檔案標籤)", tagged_texts.len());
    println!("- texts_to_translate 數量: {}", texts_to_translate.len());

    assert_eq!(texts_to_translate.len(), 1, "應該只發送 1 條翻譯項給 LLM！");

    // --------------------------------------------------
    // 3. 模擬 [結果解析] 段落 (與 process_one_global_batch 代碼完全相同)
    // --------------------------------------------------
    println!("\n--- 模擬 LLM 回傳成功 ---");
    let mut results_map = std::collections::HashMap::new();
    let orig_tag = format!("[i0]{}", target_text);
    let trans_tag =
        "[i0]此設定檔是唯讀的，無法修改！如果您想基於此建立一個新設定檔，可以複製為新名稱"
            .to_string();
    results_map.insert(orig_tag, trans_tag);

    let tag_re = regex::Regex::new(r"\[i(\d+)\]").unwrap();

    for (orig_tagged, trans_tagged) in &results_map {
        if let Some(caps) = tag_re.captures(orig_tagged) {
            if let Ok(relative_idx) = caps[1].parse::<usize>() {
                if relative_idx < batch_indices.len() {
                    let abs_idx = batch_indices[relative_idx];
                    let orig_text = items[abs_idx].original.clone();

                    let clean_translated = tag_re.replace_all(trans_tagged, "").trim().to_string();
                    let final_trans = clean_translated; // 略過 postprocess

                    for &other_abs_idx in &batch_indices {
                        if items[other_abs_idx].original == orig_text {
                            items[other_abs_idx].translated = Some(final_trans.clone());
                        }
                    }
                }
            }
        }
    }

    println!("\n[結果驗證]:");
    let mut filled_count = 0;
    for (idx, item) in items.iter().enumerate() {
        if let Some(ref res) = item.translated {
            filled_count += 1;
            println!("  項目 {}: {} -> 成功填入! ({})", idx, item.key, res);
        }
    }

    assert_eq!(filled_count, 5, "批次內 5 個條目應該都要被填滿翻譯！");
    println!("\n✅ 驗證 100% 通過：ancient.json 重複條目只發送 1 次，並自動同步套用。");
}
