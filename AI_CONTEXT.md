# AI Context: Quick Lookup System (mc_translator_rs)

> [!IMPORTANT]
> **AI 接手與未來開發指引 (Handover & Future Roadmap)**  
> 致下一位開發 AI：以下是目前核心邏輯與架構規範：
> 1. **核心譯文匹配與術語同步**：
>    - **翻譯匹配順序**：官方建議詞 (Official) > 翻譯記憶體 (User) > LLM 翻譯。
>    - **規範化辭典**：系統統一使用 `dicts/user.json` 與 `dicts/official.json`。
>    - **術語時機**：實作了 `refresh_all_dictionaries` 四大同步時機，確保建議詞始終最新。
>    - **遷移邏輯 (Migration Mode)**：對「官方建議詞」進行任何編輯或取代操作後，該條目必須「移入」使用者字典 (`user.json`) 並從官方字典 (`official.json`) 中移除。
  - **UI 標籤樣式與顏色統一 (Revision 15.18 - 15.20)**:
    - **屬性強制**: 所有 RichText 標籤統一套用 `.strong()` (粗體) 並移除 `.italics()` 與 `.small()`。
    - **常量化管理**: 顏色統一透過 `ui.rs` 頂部的 `LABEL_COLOR_LIGHT` (#1E3C78 深靛藍) 與 `LABEL_COLOR_DARK` (#C8A064 淺沙) 管理。
    - **高對比度**: 淺色模式必須使用深藍色系以確保在淺橘色背景下的清晰度。
  - **佈局防遮擋規範 (Revision 15.19)**:
    - **導航按鈕位移**: ⚙, 🌓, 📖, 🔧 等導航按鈕必須於 `render_header_controls` 中固定置右。
    - **路徑標籤獨立**: 輸出資料夾路徑標籤應獨立成行並使用 `truncate(true)`，確保在小視窗時不覆蓋按鈕。
  - **交互規範 (DragValue)**: 數值組件配備 `speed` 參數，支援滾輪與鍵盤調整。
- **系統精簡 (CKIP Removal)**:
  - **已徹底移除** CKIP 相關依賴。
- **UI 交互與鍵盤賦能 (Revision 14.4 - 14.6)**:
  - **鍵盤支援**: 所有彈出對話框 (新增/取代) 與表格編輯模式均支援 Enter 鍵提交。
  - **佈局靈活性**: 建議詞管理器內的對話框設為可移動 (非 fixed anchor)，提升多視窗作業便利性。
  - **術語取代標準**: 取代功能需支援「全字匹配」開關，且欄位命名遵循「原Value:」與「新Value:」規範。
  - **按鈕順序**: 遵循主要執行按鈕 (如 💾, 🗑) 置於最右側的順序規範。
> 4. **Git 與文件紀錄規範 (Records & Documentation)**：
>    - **同步提交**：每次完成階段性修改後，開發 AI 必須執行 `git add .`。
>    - **繁體中文 Commit**：所有 Commit 紀錄必須使用**繁體中文**撰寫，以維護開發歷程對使用者的直覺性與可讀性。
>    - **日誌倒序**：`MAINTENANCE_LOG.md` 與其他歷史紀錄文件必須遵循「**最新紀錄置於最上方**」的規則，確保開發者能第一時間查閱最新變動。
> 5. **執行緒安全**：
>    - 使用 `tokio::sync::mpsc` 處理主子視窗通訊，並透過 `Arc<Mutex<T>>` / `Atomic` 確保多執行緒下的資料一致性。

## 系統快速索引 (Quick Lookup)
- **[核心架構與流程](docs/architecture/overview.md)**
  - 包含 [狀態管理](docs/architecture/state_management.md) 與 [錯誤處理](docs/architecture/error_handling.md)。
- **[翻譯核心與模組](docs/modules/translation_core.md)**
  - 包含 [檔案流水線](docs/modules/file_pipeline.md)、[術語系統](docs/modules/glossary_system.md) 與 [翻譯記憶體](docs/modules/translation_memory.md)。
- **[UI 規格與交互](docs/ui/specs.md)**
  - 包含 [詳細狀態交互地圖](docs/ui/interactions.md)。
- **[測試策略與規範](docs/guides/testing_strategy.md)**
  - 包含 3-Test Rule、UTF-8 安全與 Windows 開發環境設定。
- **[Git 指南](docs/guides/GIT_GITHUB_GUIDE.md)**
  - 本地與雲端同步工作流、Commit 規範與指令參考。
- **[歷史維護紀錄](docs/guides/MAINTENANCE_LOG.md)**
  - 完整記錄專案從初期至今的所有版本變遷。

---
*備註：此檔案為「快查系統」的入口。若需開發，請務必先 read 對應的細節文件。*
