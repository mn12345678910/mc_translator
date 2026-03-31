# 單元測試對照表

本文件整理目前程式碼中可對應的測試範圍與重點。

## utils/text_processing.rs

- `preprocess_text` / `postprocess_text` 格式標記保護
- `validate_and_cleanup` 清理與輸出規範
- `detect_loop` 無限迴圈偵測
- `sync_formatting` JSON 內容更新

## utils/skip_rules.rs

- `should_skip_key` 跳過鍵名
- `should_skip_value` 跳過值規則

## translation/glossary/automaton.rs

- 術語匹配與邊界判斷
- 優先序合併 (official/user)

## translation/api/models.rs

- 動態模型列表拉取與 fallback
- Minecraft 版本對應 pack_format

## file/json_handler.rs

- JSON 解析與目標語言檔讀取
- 預填翻譯項目流程

## file/js_handler.rs

- JS 文本規則匹配
- 字串替換與輸出

## file/jar_handler.rs

- JAR 內 JSON 解析
- Patchouli 路徑處理
- repack_jar 流程

## translation/batching.rs

- 批次分割
- 降級重試邏輯

## tests/ 目錄

- [tests/pipeline_tests.rs](/tests/pipeline_tests.rs): pipeline 集成流程
- [tests/file_tests.rs](/tests/file_tests.rs): 檔案處理與輸出行為
- [tests/i18n_consistency.rs](/tests/i18n_consistency.rs): i18n keys 一致性
- [tests/frontend/](/tests/frontend/): GUI DOM 與 invoke 行為
