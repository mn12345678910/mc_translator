# 文檔系統重構計畫 (Documentation Refactor Plan)

本計畫旨在代碼重構（Phase 1-7）完成後，對 `docs/` 目錄下的文檔進行全面升級。目標是將現有的文字描述轉化為具備「高視覺化」、「精確規格」與「邏輯映射」的技術文檔。

3.  **相對路徑規範**：所有文檔內提及的檔案或目錄路徑必須使用**相對路徑**（以專案根目錄為基準），嚴禁紀錄絕對路徑（如 `C:\Users\...`）。

---

## 1. 文檔目錄預計結構

```
docs/
├── architecture/               # 架構維度
│   ├── overview.md             # 頂層依賴關係、跨模組通訊圖
│   ├── state_management.md     # AppState 與非同步任務狀態圖
│   └── error_handling.md       # 錯誤處理分類與重試邏輯
├── modules/                    # 邏輯維度
│   ├── translation_core.md     # 翻譯引擎、批次調度
│   ├── file_pipeline.md        # 檔案掃描、JAR 處理流程圖
│   ├── glossary_system.md      # 術語匹配優先級與自動機運作 [新增]
│   └── translation_memory.md   # .cache 檔案格式與持久化邏輯 [新增]
├── ui/                         # 視覺維度
│   ├── specs.md                # 元件位置/大小/顏色規格、初始化狀態
│   └── interactions.md         # UI 鎖定矩陣、邏輯觸發地圖
└── guides/                     # 維護維度
    ├── GIT_GITHUB_GUIDE.md     # 已存在
    └── MAINTENANCE_LOG.md      # 表格化細節規範：
        - **標頭**：`|日期時間|Commit編號|類型|位置(模組路徑)|說明 （使用-或<br>清單化）|`
        - **日期時間**：使用 `YYYY-MM-DD HH:MM` 格式。
        - **Commit 編號**：使用 7 碼短編號（例如 `c465ea8`）。
        - **類型**：使用如 `重構`, `修復`, `功能`, `文件` 等標籤。
        - **位置**：優先填寫模組目錄（如 `state/`, `ui/components/`），若很特定則寫單一檔案（如 `lib.rs`）。此處必須使用**相對路徑**。
        - **說明**：
            - 單項：直接撰寫。
            - 多項：使用 `- 項目 1<br>- 項目 2` 的格式，確保在表格儲存格內保持清單結構。
```

---

## 2. 各文檔核心內容規劃

### A. 架構與邏輯相關 (各系統說明)
- **邏輯/依賴圖 (Mermaid graph TD)**：
    - 精確展現 `ui -> state -> translation/file -> utils` 的單向流程。
    - 標註各模組公開的 API 入口點。
- **執行流程 (Mermaid sequenceDiagram)**：
    - **翻譯任務流**：從 UI 點擊 -> `actions::start` -> `file::pipeline` -> `translation::engine` -> `api::client` -> UI 回報。
- **狀態圖 (Mermaid stateDiagram-v2)**：
    - 定義 `AppState` 中的 `is_processing`, `is_paused`, `is_cancelled` 如何驅動 UI 與背景任務。
- **專屬邏輯說明 (New Modules)**：
    - **Glossary System**：定義「官方 > 使用者 > 推斷」的權重權重分配、Aho-Corasick 算法細節。
    - **Translation Memory**：描述 `.cache` 文件的 Key/Value 同步規則與併發寫入保護。

### B. UI 規格與互動 (UI 文檔核心)
- **視覺規格表 (Exhaustive Visual Table)**：
    - **所有 UI 元件清單**：需涵蓋 `Header`, `Settings`, `File List`, `Progress`, `Actions`, `Log`, `Memory Viewer` 所有按鈕與輸入框。
    - **屬性規範**：要求提供具體的 **十六進位色碼 (Hex Codes)**。
        | 具體元件 | 座標/對齊 | **淺色模式色碼** | **深色模式色碼** | 尺寸 | 預設鎖定狀態 |
        | :--- | :--- | :--- | :--- | :--- | :--- |
        | 🌐 提供者選單 | Header 置左 | `#1E3C78` | `#FFDEAD` | 120px | 解鎖 |
        | 🔑 API 金鑰框 | Settings | `#FFFFFF` | `#1B1B1B` | 彈性 | 解鎖 |
        | ... (所有元件) | ... | ... | ... | ... | ... |
- **全域狀態控制矩陣 (State-Driven Control Matrix)**：
    - 必須精確列出 **「暫停 (Paused)」** 狀態下的行為限制：
        | 元件名稱 | 待機 (Idle) | 掃描/翻譯 (Running) | **暫停 (Paused)** | 備註 |
        | :--- | :--- | :--- | :--- | :--- |
        | 📁 選擇檔案 | 解鎖 | **鎖定** | **鎖定** | 任務進行中禁止更改源 |
        | 🚀 開始翻譯 | 解鎖 (顯示) | 隱藏 | 隱藏 | |
        | ⏯️ 恢復翻譯 | 隱藏 | 隱藏 | **解鎖 (顯示)** | 暫停時出現 |
        | ⏸️ 暫停翻譯 | 隱藏 | **解鎖 (顯示)** | 隱藏 | 運行時出現 |
        | ⏹️ 停止翻譯 | 隱藏 | **解鎖 (顯示)** | **解鎖 (顯示)** | 任務時皆可終止 |
        | 🌐 API 提供者 | 解鎖 | **鎖定** | **鎖定** | 暫停時不允許切換供應商 |
        | 🤖 模型選取 | 解鎖 | **鎖定** | **解鎖** | **允許在暫停時切換模型** |
        | 💡 提示詞編輯 | 解鎖 | **鎖定** | **解鎖** | **允許在暫停時調校 Prompt** |
        | ⚙️ 其他核心設定 | 解鎖 | **鎖定** | **鎖定** | Batch Size 等需維持一致 |
- **元件動作地圖 (Interaction Map)**：
    - 當元件被觸發時，明確連結到底層 `AppState` 的變數變動與 `actions.rs` 的函數呼叫。

---

## 3. 實施步驟 (代碼重構完成後)

1.  **第一階段：基礎框架建立**
    - 建立新的目錄結構，搬移並分類舊有 `docs/` 內容。
2.  **第二階段：視覺化圖表注入**
    - 遍歷重構後的子模組，繪製 Mermaid 圖。每張圖必須對應到代碼中的具體 `struct` 或 `fn`。
3.  **第三階段：UI 鎖定與數據流映射**
    - 根據 `src/ui/` 下的子組件代碼，逐一核對按鈕位置、顏色常量與其觸發的 `AppState` 動作。
4.  **第四階段：一致性檢查**
    - 確保 `architecture.md` 的依賴圖與 `lib.rs` 的模組宣告完全一致。

---

---

**[ 註：此為計畫草案，將在 Phase 6-7 代碼完成後正式執行。]**
