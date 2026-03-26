# 測試策略

本專案以單元測試為主，重點放在純函式與邏輯模組。

## 三向測試原則

1. 正常路徑 (Happy Path)
2. 邊界與 UTF-8
3. 異常與防呆 (Robustness)

## 建議覆蓋範圍

- `utils/text_processing.rs`：前後處理、循環偵測
- `utils/skip_rules.rs`：跳過規則
- `translation/glossary/automaton.rs`：術語匹配
- `translation/api/client.rs`：JSON 抽取與容錯

## 測試工具鏈

- **後端 (Rust)**: 使用標準 `cargo test`。E2E 整合測試會自動啟動本地 `TcpListener` 模擬 API 連線與回應。
- **前端 (JavaScript)**: 使用 `Vitest` + `Happy DOM` 環境。設定位於 `vitest.config.js`，主要藉由 Mocking 隔離 Tauri `invoke` 等 API 進行 DOM 交互渲染驗證。

## 測試位置

- 單元測試放在各檔案 `mod tests` 內
- 整合測試放在 `tests/`

## 注意事項

- 網路與 API 相關測試建議以 `#[ignore]` 控制
- 需要檔案 I/O 的測試請使用臨時目錄
