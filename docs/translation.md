# Translation Logic & AI Integration

## 1. 翻譯與儲存機制 (Translation & Storage)
1. **建議詞管理器 (Glossary Manager)**: 存在裡面的文字將作為術語表建議 LLM 如何翻譯該文字。UI 中對應 📖 按鈕。
   - **雙分頁設計**: 分為「📝 使用者建議詞」與「📚 官方建議詞」。
   - **數據獨立性**: 清空操作僅作用於當前分頁。在官方分頁編輯時，系統會自動轉存至使用者建議詞。
   - **時機統一 (Sync Triggers)**: 辭典會在以下時機自動同步：
     1. 程式啟動。
     2. 開啟建議詞管理器視窗。
     3. 按下「🔄 整理」按鈕。
     4. 翻譯任務啟動前。
2. **專門過濾器**: 防止修改 Minecraft 模組的內部代碼，如 ID `tconstruct:broad_axe`、全數字、`@` 或 `#` 開頭標籤。
3. **JAR `zh_tw` 提取**: 掃描 JAR 時會同步檢查是否存在 `zh_tw.json`。若有，則提取之並作為翻譯基準，實現「續傳/跳過」邏輯。
4. **即時磁碟寫入 (Temp Partitioning)**: 翻譯產出後立即寫入 `temp_translator` 目錄，大幅降低大型模組包處理時的記憶體 (RAM) 占用。
5. **資源包匯出**: 壓縮 `temp_translator` 內容產出 `LLMTranslator.zip`。若輸出目錄已存在同名檔案，會發出警告日誌並覆蓋。

## 2. 辭典系統與術語規格 (Dictionaries & Terms)
1. **雙分頁辭典架構**:
    - 「📝 使用者建議詞」：使用者手動新增或編輯後的存儲區（存於 `dicts/user.json`）。
    - 「📚 官方建議詞」：系統自動加載的官方術語（存於 `dicts/official.json`）。
2. **術語自動匹配 (AC Automaton)**：採用 Aho-Corasick 演算法快速找出原文中的術語並傳遞給 LLM 作為參考。
3. **繁體化處理**：譯後統一使用 `hanconv` 進行繁體化，確保一致性。


## 3. 例外與錯誤處理 & 穩定性 (Error Handling & Stability)
- 網路或 LLM 失敗 (如 `OLLAMA_TIMEOUT`, `EMPTY_RESPONSE`): 
  - **Adaptive Batching (失敗降級策略)**: 當批次翻譯失敗且重試無效時，系統會自動啟動 **二分降級 (Halving)**。若將批次切半仍失敗，則退回至 **單筆強制翻譯 (Single-item fallback)**，確保即使只有部分字串導致異常，也不會影響整批進度。
  - **Exponential Backoff**: 自動啟用指數退避重試，若最終仍無法處理則標記為 `FileStatus::Skipped`。
- **記憶體鎖定機制**: 大量非同步執行緒共享日誌與進度條時，必須透過細粒度 `Arc<Mutex<T>>` 上鎖（如引入 `tokio::sync::Notify` 進行事件等待）。
- **執行緒崩潰防護 (Panic Prevention)**: 在處理如 `json` 的 `key` 名稱時（特別是 `ldlib` 這類可能包含未規範中文的模組），嚴禁直接使用 `key[key.len()-3..]` 進行原始字串切片。應統一轉換為 `.as_bytes()` 或使用 UTF-8 安全的 API（如 `ends_with`, `starts_with`）進行比對，以防止發生 "not a char boundary" 的 Panic 導致 Tokio 背景任務靜默死亡（讓 UI 卡在 `正在分析` 狀態無法復原）。
- **統一檔案跳過規範 (Unified File Skipping)**:
  - **預掃描機制 (Pre-scan Phase)**: 所有檔案（JAR, JS, JSON）在正式處理前，必須先經過 `check_*_has_target` 函式檢查是否含有可翻譯內容。未通過檢查的檔案不會進入翻譯迴圈，且不計入進度條總數，確保進度顯示精確。
  - **日誌簡化**: 對於跳過的檔案，系統不再逐一列出檔名，而是在預掃描結束後統一輸出單一行日誌：`跳過處理 (N).`。
  - **跳過判定標準**:
    - JAR: 位於 `mods` 資料夾內且包含 `en_us.json` 或 Patchouli 補丁書。
    - JS: 內容符合 KubeJS 翻譯正則且包含非技術性字串。
    - JSON: 包含至少一個非技術性（非 ID、非純數字等）且未翻譯的字串。
- **循環生成與重複內容風險 (Repetition Loop Prevention)**:
    - **識別風險**: 極高頻率或無意義的重複字元會導致 LLM 注意力崩潰。在傳遞數據前，系統應盡可能過濾冗餘片段。
    - **應對機制**: 若偵測到 API 回傳內容呈現高度循環或異常重複，程式應啟動防護機制（如略過該批次或提示使用者降低 `BATCH_SIZE`）。亦應在開發階段避免將可能引發循環的無意義字串作爲上下文傳遞。

## 4. API 模型整合與限制 (2026-03-04 更新)
- **Ollama 推理模型 (Reasoning Models)**: 諸如 `qwen3.5:4b` 等具備思考脈絡（CoT）能力的模型，在生成最終譯文前會輸出大量 `<thought>` 標籤。為避免回應因 Token 限制而遭截斷 (`EMPTY_RESPONSE`)，程式內部的 `num_predict` (最大生成 token 數) 設定已從預設의 `2048` 提升至 `8192`。
- **Ollama 專項模型 (Standard Models)**: 針對 `translategemma:4b` 等 non-reasoning 模型，其單筆回應極快。但在處理大型 JSON 模組語言檔（如 `ldlib`）時，若 `batch_size` 過高（如 150 筆），極易導致語言模型的上下文混亂，造成原文字串原樣回傳或遺失 JSON/Regex 格式。建議針對此類輕量模型將單次翻譯數量 `batch_size` 縮減至 20 筆以確保穩定對應。
- **Gemini API**: 驗證 `gemini-flash-latest` 可提供穩定的 JSON 格式翻譯，且延遲極低。但在免費額度下快速請求部分新模型（如 2.0-flash）極易觸發 HTTPS 429 限制，需妥善控制 `BATCH_SIZE`。
