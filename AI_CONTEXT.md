# AI Context: Quick Lookup System (mc_translator_rs)

> [!IMPORTANT]
> **AI 接手與未來開發指引 (Handover & Future Roadmap)**  
> 致下一位開發 AI：以下是目前核心邏輯與架構規範：
> 1. **核心譯文匹配與術語同步**：
>    - **翻譯匹配順序**：官方建議詞 (Official) > 翻譯記憶體 (User) > LLM 翻譯。
>    - **規範化辭典**：系統統一使用 `dicts/user.json` 與 `dicts/official.json`。
>    - **術語時機**：實作了 `refresh_all_dictionaries` 四大同步時機，確保建議詞始終最新。
>    - **遷移邏輯 (Migration Mode)**：對「官方建議詞」進行任何編輯或取代操作後，該條目必須「移入」使用者字典 (`user.json`) 並從官方字典 (`official.json`) 中移除。
     - **設定檔隔離**: 系統拆分為 `config.cfg` (核心功能) 與 `style.cfg` (視覺樣式)。
     - **金鑰隔離**: `api_key` 僅保留於 `.env` 中，且必須在 `AppConfig` 結構中使用 `#[serde(skip)]` 標記，禁止寫入 `config.cfg`。
     - **狀態持久化**: 所有關鍵 UI 狀態均需同步至對應設定檔。`style.cfg` 包含主題、字體、配色及自定義調色盤覆寫。
     - **向後相容性 (`Alias`)**: 為確保升級後不丟失設定，設定結構所有中文化欄位均須保留對應的 English `alias`。
     - **即時存檔**: 所有設定項與視窗幾何資訊在變更後必須立即發動 `trigger_save()` 非同步存檔。
    - **視窗啟動延遲 (Latency)**: 為確保幾何穩定與視覺消色，子視窗啟動設定為兩層延遲（主執行緒 10 幀，子視窗內部亮顯 10 幀）。位置同步門檻設定為 20 幀。
  - **UI 視覺規格與連動**:
    - **原生視窗裝飾**: 視窗必須使用 OS 原生標題列與邊框，嚴格禁止隱藏裝飾或實作自定義標題列。
    - **自定義調色盤 (Palette)**: 支援進度條、按鈕、輸入框等全組件自定義色彩與圓角，具備屬性動態隱藏與實例覆寫機制。
  - **對話框與 Viewport 同步 (Visual Sync)**：
    - **深色模式標籤統一**：開發者模式與全視窗標籤在深色模式下預設連動琥珀色 (`dark_text`)，達成視覺絕對統一。 (Revision 15.35)
    - **子視窗樣式注入**：建議詞管理器等 Viewport 採用全域樣式注入機制，確保標題列圓角、按鈕圓角及內部彈出對話框 (Window) 與主視窗調色盤同步，徹底解決硬編碼 (NavajoWhite) 殘留問題。
  - **進度條自定義**：支援分別調整「目前檔案」與「總進度條」的色調與文字顏色同步。
  - **國際化架構 (i18n)**：全系統 UI 標籤、狀態與執行日誌全面國際化。標籤定義於 `src/ui/i18n.rs`，透過 `AppState.i18n` (及其衍生的 `JobSharedState.i18n` 與 `TranslationContext.i18n`) 全域傳遞。
  - **警告清理規範**: 專案維持 `cargo check --all-targets` 零警告標準，對於未使用的舊邏輯 (如舊版 Google Free 接口) 應果斷移除而非註解。
    - **圓角規範**: 全域按鈕統一套用 **可調圓角** (預設 4.0/8.0)。唯一例外為「檔案/資料夾」選擇按鈕，維持 0.0 圓角以提升辨識度。
    - **進度條動畫**: 運行中具備脈衝 (Pulse) 動畫，頻率可自定義。
    - **壓縮與鏡像規範**: 
      - **JAR 資源**: 僅整合於 `LLMTranslator.zip`，不產出資料夾鏡像，確保結構精簡。
      - **獨立檔案**: 僅產出資料夾鏡像，不進入 Zip，確保過濾邏輯精確。
    - **配色同步化**: 淺色模式背景使用暖橘色 (#FFFDF0)，文字與標籤統一使用 **軟灰色 (#222)**。API 指示燈需依主題切換深/淺語義色。
    - **日誌增強**: 所有日誌輸出必須帶有 `[HH:MM:SS]` 時間戳記。
    - **語義化日誌顏色**: 
      - **錯誤 (Err/失敗)**: 強制顯示為 **紅色 (Red)**。
      - **成功 (Done/完成)**: 淺色模式下使用 **深綠色 (#006400)** 以提升對比度，深色模式下維持 **綠色 (Green)**。
      - **資訊 (Info)**: 深色模式下連動 `LABEL_COLOR_DARK` (淺沙色)，淺色模式下維持深灰色。
    - **輸入框背景同步**: 淺色模式下多行輸入框 (TextEdit) 的背景色需與 `DragValue` 等組件同步為橘粉色 (#E3C395)。
  - **代碼風格與清理**:
    - **縮排規範**: 專案嚴格禁止使用 TAB 字元 (`\t`)，所有檔案必須使用純空白縮排，以確保開發工具一致性。
    - **屬性強制**: 所有 RichText 標籤統一套用 `.strong()` (粗體) 並移除 `.italics()` 與 `.small()`。
    - **交互規範 (DragValue)**: 數值組件配備 `speed` 參數，支援滾輪與鍵盤調整。懸停於數值上時應支援滑鼠滾輪增減。
  - **佈局防遮擋規範**:
    - **導覽按鈕位移**: ⚙, 📖, 🔧 等導覽按鈕必須於 `render_header_controls` 中固定置右。
    - **路徑標籤獨立**: 輸出資料夾路徑標籤應獨立成行並使用 `truncate(true)`，確保在小視窗時不覆蓋按鈕。
- **系統精簡 (CKIP Removal)**:
  - **已徹底移除** CKIP 相關依賴。
- **UI 交互與鍵盤賦能**:
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
