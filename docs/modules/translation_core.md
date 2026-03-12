# 翻譯核心與辭典系統 (Translation Core & Glossary)

## 1. 翻譯與儲存機制 (Translation & Storage)

### 翻譯流程
- **即時磁碟寫入 (Temp Partitioning)**: 翻譯產出後立即寫入 `temp_translator` 目錄，降低 RAM 占用。
- **JAR `zh_tw` 提取**: 優先檢查並提取已存在的 `zh_tw.json`。
- **資源包匯出**: 產出 `LLMTranslator.zip`。

### 辭典系統
- **雙分頁設計**: 分為「📝 使用者建議詞」與「📚 官方建議詞」。
- **術語自動匹配 (AC Automaton)**: 採用 Aho-Corasick 演算法快速找出術語。

## 2. 穩定性與錯誤處理 (Stability & Error Handling)

### 失敗降級策略 (Adaptive Batching)
1. **正常重試**: 網路或超時失敗時自動重試。
2. **二分降級 (Halving)**: 批次失敗後自動將 `batch_size` 減半。
3. **單筆回退 (Single-item fallback)**: 最終退回至單筆強制翻譯，確保進度。

### 執行緒與崩潰防護
- **Panic Prevention**: 嚴禁在未經校驗情況下直接對字串進行 Raw Slicing，必須使用 UTF-8 安全 API（如 `ends_with`, `starts_with`）。
- **非同步互斥**: 透過 `Arc<Mutex<T>>` 或 `tokio::sync::Notify` 進行狀態同步。

## 3. API 模型整合規範

- **Ollama 推理模型**: 針對具備 CoT 能力的模型，將 `num_predict` 提升至 `8192`。
- **輕量化模型**: 處理 `translategemma` 等模型時，建議將 `batch_size` 限制在 20 筆以內，防止上下文混亂。
- **Gemini API**: 注意免費額度的 429 請求限制。
