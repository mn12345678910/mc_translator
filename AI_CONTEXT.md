# AI Context: Quick Lookup System (mc_translator_rs)

> [!IMPORTANT]
> **AI 接手與未來開發指引 (Handover & Future Roadmap)**  
> 致下一位開發 AI：以下是目前核心邏輯與架構規範：
> 1. **核心譯文匹配與術語同步**：
>    - **翻譯匹配順序**：官方建議詞 (Official) > 翻譯記憶體 (User) > LLM 翻譯。
>    - **規範化辭典**：系統統一使用 `dicts/user.json` 與 `dicts/official.json`。
>    - **遷移邏輯 (Migration Mode)**：對「官方建議詞」進行任何編輯或取代操作後，該條目必須「移入」使用者字典 (`user.json`) 並從官方字典 (`official.json`) 中移除。
> 2. **狀態管理與併發規範 (Atomic Refactor)**：
>    - **原子化狀態**: 為消除鎖競爭，高頻旗標 (`is_processing`, `is_paused`, `is_cancelled`) 與進度值 均使用原子類型 (`AtomicBool`, `AtomicU32`)。
>    - **存取規範**: 使用 `.load(Ordering::SeqCst)` 與 `.store(...)`。對於 `f32` 進度，透過位元轉換 (`to_bits`/`from_bits`) 儲存於 `AtomicU32`。
>    - **Runtime 安全**: 從 UI 觸發的非同步任務必須使用 `self.runtime.spawn`。
>    - **鎖定範圍**: 僅對複雜容器（如 `Vec`, `HashMap`）使用 `Arc<Mutex<T>>`。
    - **色彩狀態 (HSVA)**: 調色盤編輯應使用 `palette_hsva_bg`/`text` 與 `palette_hsva_target` 暫存狀態，以避免直接操作 SRGB 時因色相丟失（HUE Reset）導致的選色圓圈跳回問題。
> 3. **設定與持久化**：
>    - **設定檔隔離**: 分為 `config.cfg` (核心功能) 與 `style.cfg` (視覺樣式)。
>    - **即時存檔**: 變更後立即調用 `trigger_save()` 進行非同步寫入。
>    - **金鑰安全**: `api_key` 僅存於 `.env`，禁止寫入設定檔。
> 4. **國際化與 UI 規範 (i18n & UI)**：
>    - **JSON 驅動**: UI 標籤支援從 `langs/{lang}.json` 動態載入，不再硬編碼。
    - **視覺連動**: 全域套用自定義調色盤 (Palette)，包含進度條脈衝動畫與各組件色彩同步。
    - **色彩解耦**: 導覽列與一般組件的樣式應通過 `instance_overrides` 進行解耦，避免類別批量更動影響特定區域。
    - **深色模式**: 標籤預設連動 `LABEL_COLOR_DARK` (琥珀色)。
    - **視窗穩定**: 具備 Drift Fix (5.0 閾值) 與幾何穩定延遲機制。
    - **進度計算法**: 全域進度條 (`global_progress`) 統一按「檔案數量」計增，而狀態列則顯示精確的「條目處理索引範圍」。
> 5. **Git 與紀錄規範**：
>    - **同步提交**：階段性修改後執行 `git add .`。
>    - **繁體中文 Commit**：所有 Commit 紀錄必須使用**繁體中文**。
>    - **日誌倒序**：`MAINTENANCE_LOG.md` 等文件遵循「最新紀錄置於最上方」。

## 系統快速索引 (Quick Lookup)
- **[核心架構與流程](docs/architecture/overview.md)**
  - 包含 [狀態管理](docs/architecture/state_management.md) 與 [錯誤處理](docs/architecture/error_handling.md)。
- **[翻譯核心與模組](docs/modules/translation_core.md)**
  - 包含 [檔案流水線](docs/modules/file_pipeline.md)、[術語系統](docs/modules/glossary_system.md) 與 [翻譯記憶體](docs/modules/translation_memory.md)。
- **[UI 規格與交互](docs/ui/specs.md)**
- **[測試策略與規範](docs/guides/testing_strategy.md)**
- **[Git 指南](docs/guides/GIT_GITHUB_GUIDE.md)**
- **[歷史維護紀錄](docs/guides/MAINTENANCE_LOG.md)**

---
*備註：此檔案為「快查系統」的入口。若需開發，請務必先閱讀對應的細節文件。*
