# AI Context: Quick Lookup System (mc_translator_rs)

> [!IMPORTANT]
> **AI 接手與未來開發指引 (Handover & Future Roadmap)**  
> 致下一位開發 AI：以下是目前核心邏輯與架構規範：
> 1. **核心譯文匹配與術語同步**：
>    - **翻譯匹配順序**：官方建議詞 (Official) > 翻譯記憶體 (User) > LLM 翻譯。
>    - **規範化辭典**：系統統一使用 `dicts/user.json` 與 `dicts/official.json`。
>    - **遷移邏輯 (Migration Mode)**：對「官方建議詞」進行任何編輯或取代操作後，該條目必須「移入」使用者字典 (`user.json`) 並從官方字典 (`official.json`) 中移除。
> 2. **狀態管理與併發規範 (Pipeline & Concurrency)**：
>    - **原子化狀態**: 高頻旗標與進度值均使用原子類型 (`AtomicBool`, `AtomicU32`)。進度值透過位元轉換儲存。
>    - **I/O 隔離**: 所有涉及實體磁碟、JAR 重構等同步 I/O 操作**必須**包裹於 `tokio::task::spawn_blocking` 中，嚴禁直接在 async 循環內呼叫，以防止 UI 凍結。
>    - **Runtime 安全**: 非同步任務必須使用引用的 `self.runtime.spawn` 執行。
    - **漏答救援防護 (Retry Coalescing)**: `run_translation_batch` 在批次獲得 Partial Success (部分答覆) 時，會將仍為 `None` 的殘留條目推入 `failed_indices`。這確保了 Adaptive Batching 等 3 級降級重試 100% 覆蓋所有漏答項。
> 3. **檔案處理與流水線 (File Pipeline)**：
>    - **有序分組 (Task Grouping / Folder Merging)**: 使用 `get_group_key`（JAR 按檔案路徑，獨立文件按 `.parent()` 父資料夾）進行 **Hybrid 混合排序分組**。此舉能在維護來源隔離的前提下，將同目錄獨立檔案「打碎合併」進單一批次，根除 fragment skipping 缺陷並最大化 Batch 吞吐力。
>    - **資源包聚合 (ZIP Output)**: 來自 JAR 的成員以 `[BUNDLE]` 標記進入暫存流，管線結束時觸發 `output_resource_pack` 生成單一 `LLMTranslator.zip`。
>    - **獨立檔案鏡像 (Mirroring)**: JS 檔案與非模組 JSON 執行「完整相對路徑鏡像」，輸出至 `LLMTranslator/` 下並保留原始層次結構。
>    - **Patchouli 修復**: 強制執行目錄級取代 (`/en_us/` -> `/zh_tw/`)。
> 4. **國際化與精準進度 (i18n & Progress UX)**：
>    - **三層進度架構**: UI 層次：`Status` -> `Current Processing Path` -> `Global Progress` (模組完成度) & `Batch Progress` (條目進度)。
>    - **JSON 驅動**: UI 標籤支援從 `langs/{lang}.json` 動態載入，確保界面顯示無硬編碼。
    - **日誌情境標記 (Context Logs)**: 通訊紀錄現行貫穿 `file_name` 與語言配置。日誌首尾預置 `<來源->目標>` 語言配對及 `[檔案]` 標頭，大幅減降 LLM API 軌跡之排查成本。
> 5. **Git 與紀錄規範**：
>    - **同步提交**：階段性修改後執行 `git add .`。
>    - **繁體中文 Commit**：所有 Commit 紀錄必須使用**繁體中文**。
>    - **日誌倒序**：`MAINTENANCE_LOG.md` 等文件遵循「最新紀錄置於最上方」。
6. **UI 樣式管理與隔離規範 (Style Isolation & Theme)**：
   - **避免全域洩漏**：調整類別顏色（例如：全部輸入框背景）時，**嚴禁**直接修改與 egui 共享的全域 `Visuals` 變量（如 `v.extreme_bg_color` 或 `v.selection.bg_fill`），以防波及 `Slider` 或 `ComboBox` 等原生元件。
   - **純覆寫機制 (Instance Overrides)**：批次更新類別顏色應轉向對清單內的所有 IDs 疊加 `instance_overrides`。
   - **局部畫布壓印**：渲染端查詢特定 ID 樣式後，使用 `ui.visuals_mut().extreme_bg_color = override_bg` 等區域內壓印或 `Frame/scope` 加掛方式渲染，達致 100% 視覺乾淨隔離。

## 系統快速索引 (Quick Lookup)
- **[核心架構與流程](docs/architecture/overview.md)**
  - 包含 [狀態管理](docs/architecture/state_management.md) 與 [錯誤處理](docs/architecture/error_handling.md)
- **[翻譯核心與模組](docs/modules/translation_core.md)**
  - 包含 [檔案流水線](docs/modules/file_pipeline.md)、[術語系統](docs/modules/glossary_system.md) 與 [翻譯記憶體](docs/modules/translation_memory.md)
- **[UI 規格與交互](docs/ui/specs.md)**
- **[測試策略與規範](docs/guides/testing_strategy.md)**
- **[Git 指南](docs/guides/GIT_GITHUB_GUIDE.md)**
- **[歷史維護紀錄](docs/guides/MAINTENANCE_LOG.md)**

---
*備註：此檔案為「快查系統」的入口。若需開發，請務必先閱讀對應的細節文件。*
