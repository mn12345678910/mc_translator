# 翻譯核心

## 翻譯流程摘要

1. 收集可翻譯字串並去重
2. 依批次大小與字元上限切分
3. 呼叫翻譯服務
4. 清理與格式同步
5. 回寫至全域批次結果

## 服務商路由

- Gemini：`generateContent`
- OpenAI / DeepSeek / Mistral：OpenAI 相容 Chat Completions
- Ollama：本機 `/api/generate`
- DeepL：`api-free` 或 `api` 端點
- Google Free：`translate.googleapis.com`

## 批次與降級

- 批次限制為「條目數」與「字元上限」雙重限制
- 失敗時依序降級為半批次與單筆
- `GlobalBatchItem` 會保留預處理標記，以確保格式還原

## 批次優化 (Batch Optimizations)

- **相對標籤優化 (Relative Tagging)**：
  發送給 LLM 的標籤自代碼（如 `[i10000]`）改為僅在批次內自增的 **相對索引**（如 `[i0]`, `[i1]`），有效縮減大型模組大檔案的 Token 消耗、穩定上下文。
- **批次內去重 (Intra-batch Deduplication)**：
  在單一批次（如 50 筆）中：
  - **發送端**：使用快取判定，相同原文只附帶 1 個標籤送出。
  - **接收端**：LLM 回傳後，會遍歷當前批次內所有條目。只要原文相同，**一併同步套用譯文**，保證對 API 呼叫只送 1 次。

## 文本處理

- `preprocess_text`：保護格式字元與佔位符
- `postprocess_text`：還原格式
- `validate_and_cleanup`：清理空白與雜訊
- `detect_loop`：偵測翻譯循環輸出

## Glossary 注入

- Glossary 以提示詞方式注入，不做硬替換
- 由官方詞庫、推論詞庫、使用者詞庫組合
- 每次最多注入 30 條術語建議

