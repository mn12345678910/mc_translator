# AI Context: Quick Lookup System (mc_translator_rs)

> [!IMPORTANT]
> **AI 接手與未來開發指引 (Handover & Future Roadmap)**  
> 致下一位開發 AI：以下是目前核心邏輯與架構規範：
> 1. **核心譯文匹配與術語同步**：
>    - **翻譯匹配順序**：官方建議詞 (Official) > 翻譯記憶體 (User) > LLM 翻譯。
>    - **規範化辭典**：系統統一使用 `dicts/user.json` 與 `dicts/official.json`。
>    - **術語時機**：實作了 `refresh_all_dictionaries` 四大同步時機，確保建議詞始終最新。
>    - **遷移邏輯 (Migration Mode)**：對「官方建議詞」進行任何編輯或取代操作後，該條目必須「移入」使用者字典 (`user.json`) 並從官方字典 (`official.json`) 中移除。
- **UI 佈局與穩定性規範**:
  - **版面還原**: 主畫面佈局已依據備份版本還原，確保精細的 Grid 對齊與視覺美感。
  - **樣式隔離 (Style Isolation)**: 副視窗必須使用 `ui.set_style` 而非 `ctx.set_style` (Revision 10 Fix)。
  - **細粒度 UI 鎖定與進度邏輯 (Revision 12.3 & 13)**: 
    - **精確鎖定**: 翻譯期間僅鎖定具備交互功能的控制項 (ComboBox, TextEdit, Slider)，**所有 Label (標籤) 必須保持解鎖狀態**，以維持主題色彩（不變灰）。
    - **導航解鎖**: ⚙ (設定)、🌓 (主題)、🔧 (開發者) 等功能在任務執行期間必須保持可點擊。
    - **進度同步**: 
      - **選取即更新**: 檔案/資料夾選取後必須立即更新 `global_total` 為選取數量。
      - **掃描即時反饋**: 處理掃描過程時，分母應隨文件發現即時增加，而非等待結束。
    - **視覺精簡**: 狀態文字禁止包含冗餘的 `...`（由 UI 動態補足）；進度條不標註「條目」或「檔案」字樣，僅顯示 `(current/total)`。
  - **交互規範 (DragValue)**: 數值組件配備 `speed` 參數，支援滾輪與鍵盤調整。
- **系統精簡 (CKIP Removal)**:
  - **已徹底移除** CKIP 相關依賴。
- **UI 交互與鍵盤賦能 (Revision 14.4 - 14.6)**:
  - **鍵盤支援**: 所有彈出對話框 (新增/取代) 與表格編輯模式均支援 Enter 鍵提交。
  - **佈局靈活性**: 建議詞管理器內的對話框設為可移動 (非 fixed anchor)，提升多視窗作業便利性。
  - **術語取代標準**: 取代功能需支援「全字匹配」開關，且欄位命名遵循「原Value:」與「新Value:」規範。
  - **按鈕順序**: 遵循主要執行按鈕 (如 💾, 🗑) 置於最右側的順序規範。
> 4. **執行緒安全**：
>    - 使用 `tokio::sync::mpsc` 處理主子視窗通訊，並透過 `Arc<Mutex<T>>` / `Atomic` 確保多執行緒下的資料一致性。

## 系統快速索引 (Quick Lookup)
- **[核心架構與併發模型](docs/architecture.md)**
  - 描述多執行緒事件驅動設計 (Notify/MPSC) 與 UI 渲染循環。
- **[翻譯邏輯與數據緩存](docs/translation.md)**
  - 術語推論 (Inference)、單次遍歷替換 (Single-pass)、字典優先級規範。
- **[UI 互動標準與建議詞管理器](docs/ui.md)**
  - 詳述 Viewport 閃爍修復、視窗規格、細粒度鎖定邏輯及表格渲染基準。
- **[測試框架與自動化驗證](docs/testing.md)**
  - 包含 Aho-Corasick 匹配驗證與 JSON 輸出格式校驗。
- **歷史維護紀錄 (Maintenance Log)**
  - 追蹤從基礎架構到 Revision 15.1 的所有重大修補、功能演進與版本控制歷程。
- **版本控制**: 系統使用 Git 進行管理。核心配置文件如 `.env` 與 `config.cfg` 已被忽略以確保安全。

## 系統快速索引 (Quick Lookup)
- **[核心架構與併發模型](docs/architecture.md)**
  - 描述多執行緒事件驅動設計 (Notify/MPSC)，並詳細說明 UI 渲染循環。
- **[翻譯邏輯與數據緩存](docs/translation.md)**
  - 術語推論 (Inference)、單次遍歷替換 (Single-pass)、字典優先級規範與匹配邏輯。
- **[UI 互動標準與建議詞管理器](docs/ui.md)**
  - 詳述 Viewport 閃爍修復、視窗規格、細粒度鎖定邏輯及表格渲染基準。
- **[測試框架與自動化驗證](docs/testing.md)**
  - 包含 Aho-Corasick 匹配驗證與 JSON 輸出格式校驗。
- **[歷史維護紀錄 (Maintenance Log)](docs/MAINTENANCE_LOG.md)**
  - 完整記錄專案從初期至今的所有版本變遷 (Revision 1-15.1)。

---
*備註：此檔案為「快查系統」的入口。若需開發，請務必先 read 對應的細節文件。*
