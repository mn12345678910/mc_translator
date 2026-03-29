# 測試策略

本專案以單元測試與輕量整合測試為主，重點在純函式、流程邏輯與前端互動。

## 三向測試原則

1. 正常路徑 (Happy Path)
2. 邊界與 UTF-8
3. 異常與防呆 (Robustness)

## 測試覆蓋範圍

- [src/utils/text_processing.rs](/src/utils/text_processing.rs): 前後處理、格式保護、無限循環偵測
- [src/utils/skip_rules.rs](/src/utils/skip_rules.rs): 跳過規則判斷
- [src/translation/glossary/automaton.rs](/src/translation/glossary/automaton.rs): 術語匹配與優先序
- [src/translation/api/client.rs](/src/translation/api/client.rs): 回應解析與容錯
- [src/file/*](/src/file/*): JSON/JS/JAR 解析與輸出
- [src/translation/*](/src/translation/*): 批次翻譯與降級重試
- [tests/](/tests/): 整合測試與流程測試
- [tests/frontend/](/tests/frontend/): 前端 DOM 與 Tauri invoke 行為

## 測試工具鏈

- Rust: `cargo test`
- Frontend: `vitest run` (設定在 `vitest.config.js`)

## 測試位置

- 單元測試在各模組 `mod tests` 內
- 整合測試在 `tests/` 目錄
- 前端測試在 [tests/frontend/](/tests/frontend/)

## 注意事項

- 需網路的測試應避免依賴真實服務，使用 mock server
- 需檔案 I/O 的測試請使用臨時目錄
- i18n 一致性由 [tests/i18n_consistency.rs](/tests/i18n_consistency.rs) 驗證
