# 批次內去重與標籤優化 維護紀錄 (Maintenance Record)

- **實作日期**：2026-03-16
- **分支名稱**：`feature/cache-tag-optim`

---

## 📌 優化背景
當執行大型 Minecraft 整合包翻譯時，有兩大痛點需要被解決：
1. **標籤過長**：全域絕對索引 `[i10000]` 標籤在極大專案時佔用過多 Token。
2. **條目重複消耗**：在一個批次（例如 50 筆）中，可能存在許多**完全相同**的警示說明或預設文字，預設會重複送給 LLM 造成浪費。

---

## 🛠️ 實作項目與設定

### 1. 相對標籤優化 (Relative Tagging)
檔案：`src/translation/batching.rs`
- 在 `process_one_global_batch` 中，改用當前批次內的 **相對索引** 作為 LLM 節點標籤（如 `[i0]`, `[i1]`）。
- 徹底降壓 Token 的數字長度壓力。

### 2. 批次內去重機制 (Intra-batch Deduplication)
檔案：`src/translation/batching.rs`
- **發送端過濾**：
  在生成 `tagged_texts` 時，使用 `HashSet` 判斷當前批次內是否已生成過該文本的 `[i{}]` 標籤。
  * 若重複，**直接跳過不加入送出清單**。
- **回傳端套用**：
  當 LLM 順利讀取並回傳某個相對索引的譯文後，不僅僅更新對應的 `ctx.all_items[abs_idx]`，更會 **遍歷該批次的所有成員 (`ctx.batch_indices`)**。
  * 只要 `original` 相同，即**一併補滿翻譯結果**。

---

## ✅ 驗證紀錄與測試

- **測試案建 A**：`tests/verify_ancient.rs`
  模擬 `ancient.json` 中的 5 次重複 `__readonly__` 字串條目。
- **結果**：
  - 發送次數：確實由 5 條降為 **1 條**。
  - 回傳填充：5個條目皆 **100% 成功一併填寫譯文**。

---

## 💡 維護注意
若未來要新增「跨批次、跨檔案全局快取」：
- 可以使用 `translation_memory`，並在批次運行前、運行後進行簡單 lookup/insert 即可，與此「批次內去重」邏輯可相輔相成。
- 目前的去重邏輯已完全滿足 100% 相同字串在批次內一律只佔用 1 份 Token 的最省錢狀態。
