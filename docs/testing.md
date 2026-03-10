# Testing & Quality Assurance (mc_translator_rs)

> [!IMPORTANT]
> **生產環境說明 (2026-03-07)**: 為確保本版本之執行效率與源碼純淨，所有開發階段之內置測試碼 (`#[cfg(test)]`)、測試模組 (`new_tests.rs`) 及測試腳本 (`src/bin/test_download.rs`) 已全面移除。以下文件僅保留作為開發規範參考。

## 1. 測試策略 (Testing Strategy)
- **測試架構 (Modular Testing)**:
  - 專案採用模組化測試結構。核心源碼（如 `ui.rs`, `utils.rs`）內部僅保留 `#[cfg(test)] mod new_tests;` 宣告。
  - 實際測試代碼存放於對應的 `src/[module]/new_tests.rs` 檔案中，實現開發邏測與測試邏輯的實體隔離。

## 2. 開發環境設定 (Windows Native)
- **開發環境設定 (Windows Native)**:
  - 專案已解決 OS 32 檔案佔用問題。測試請**直接在 Windows Native 底下使用 `cargo test` 運行**，不再強制依賴 WSL。
  - 原理：藉由 `.cargo/config.toml` 設定 `[build] target-dir = "../cargo-target"`，大幅減少目標資料夾衝突與防毒軟體佔用鎖定。
  - 防火牆已開啟允許 `11434` 埠，本機 Ollama 的整合測試已完全無阻。

## 3. 測試覆蓋標準
- **測試覆蓋標準**:
  - 每一個核心邏輯函式必須有**至少 3 個差異化測試**。
  - **Unicode 安全規範**: 所有測試中的中文硬編碼字串應統一使用 Unicode 轉義序列 (如 `\u{4f60}`)，以防止跨平台開發時源碼編碼損發。
  - 測試總數維繫在 **125+** 項，確保重大更新（如路徑標準化、2-phase 重構、雙分頁辭典分離）均有驗證。
- **整合測試 (Integration Tests)**:
  - 對於呼叫 Ollama API 或實機掃描的測試，應使用 `#[ignore]` 標記。
