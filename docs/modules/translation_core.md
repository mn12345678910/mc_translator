# 翻譯核心

本文件描述翻譯核心流程與批次邏輯。

## 核心流程

- 由 `translation/pipeline.rs` 組裝 `JobConfig` 與 `JobSharedState`
- 初始化字典與術語資料
- 交由 `file/pipeline.rs` 分組並進行全域批次翻譯

## 批次翻譯

- 批次條件: `batch_size` 與 `batch_max_chars`
- 預處理: `preprocess_text` 將格式標記轉為佔位符
- 結果清理: `validate_and_cleanup` 移除多餘回應內容
- 失敗降級: 批次 -> 半批次 -> 單筆

## 預填翻譯

- 若目標語言欄位已存在且與原文不同，會視為已翻譯並預填
- 預填項目仍會參與進度統計

## 格式保護

支援保護的格式標記:

- `§`、`&` 色碼
- `#RRGGBB`
- `%s`、`%1$s`
- `{0}`
- `\\n`

## 進度與狀態

- 全域進度: 檔案數量為基準
- 條目進度: 全域翻譯項目數量
- 批次進度: `current_batch` / `total_batches`

## 相關檔案

- [src/translation/pipeline.rs](/src/translation/pipeline.rs)
- [src/translation/batching.rs](/src/translation/batching.rs)
- [src/translation/engine.rs](/src/translation/engine.rs)
- [src/utils/text_processing.rs](/src/utils/text_processing.rs)

## 相關連結

- [檔案流水線](file_pipeline.md)
- [術語系統](glossary_system.md)
- [翻譯記憶體](translation_memory.md)
