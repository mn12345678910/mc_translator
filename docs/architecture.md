# System Architecture: mc_translator_rs

## 1. 專案概覽 (Project Overview)
`mc_translator_rs` 是一個為 Minecraft 繁體中文在地化設計的自動翻譯工具，基於 Rust 開發並使用 `egui` 建立圖形介面 (GUI)。它能讀取 `mods` 資料夾中的 JAR 模組檔、資源包，或獨立的 JS/JSON，自動提取待翻譯字串，並利用多種線上/本機 LLM (Ollama, Gemini 等) 進行精準的在地化翻譯。

## 2. 系統架構 (Architecture)
- `ui.rs`: 基於 `egui` 的 GUI 邏輯。主要調整包含主子視窗通訊從 `std::sync::mpsc` 遷移至 **`tokio::sync::mpsc` (Unbounded)** 與原子信號 (`AtomicBool`)，以解決 `Sync` 執行緒安全衝突。
- `data_processing.rs`: JSON 結構的遞迴遍歷、跳過技術性金鑰、過濾特殊語法。實作了 **2-phase 全域批次翻譯** (Collection -> Global Translation -> Application)。整合了 **雙分頁建議詞存儲系統**（使用者建議、官方建議分開處理）。
- `translation_service.rs`: 與 LLM 互動，組裝 Prompt、處理 **Adaptive Batching (自適應分批)**、以及 Ollama 穩定性控制。
- `config.rs`: 應用程式設定管理。採用 **「金鑰分離」** 模式：加密的 `API_KEY` 儲存在 `.env`，其餘設定儲存在明文格式的 `config.cfg`。整合 Windows DPAPI 與新版 `dicts/` 管理邏輯。
- `state_and_log.rs`: 共享的應用程式狀態（進度、暫停、取消）。定義了 `AppState` 與 `ViewerSharedState`。實作了 **靜態方法化** 的 `refresh_dictionaries_core`，確保辭典刷新邏輯能跨執行緒與視窗閉包安全呼叫，解決生命週期借用問題。
- `translation_job.rs`: 定義 `JobConfig` 與 `JobSharedState`。
- `utils.rs`: 術語自動機 (Glossary Automaton)、日誌格式化單元。包含高效的 **Aho-Corasick 匹配**，確保英文字串與術語精確匹配。
- `file_handler.rs`: 處理檔案解壓縮、尋找目標語言檔 (`en_us.json`, Patchouli books)、JS (KubeJS) 腳本讀寫。實作了 **非同步並行收集架構 (JoinSet Parallel Scanning)**，在預掃描階段顯著降低 I/O 阻塞。
- `python_env.rs`: (已於 2026-03-09 移除 CKIP 支援) 目前僅保留基礎結構，不再管理 Python 環境或模型。
