<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# 檔案流水線

本文件描述檔案掃描、分組與輸出行為。

## 掃描與收集

- 支援 `.jar` / `.json` / `.js`
- 由 `file/scanner.rs` 遞迴掃描資料夾
- 由 `file/*_handler.rs` 建立 `FileTask` 與 `GlobalBatchItem`
- 會先套用 `excluded_paths` 全域排除清單

## 分組策略

- JAR: 以檔案本身為群組
- JSON/JS: 以父資料夾為群組

## JSON 處理

- 讀取來源檔案與目標語言檔案
- 依 `should_skip_key` 與 `should_skip_value` 過濾
- 若目標欄位已翻譯，會預填並跳過翻譯
- Patchouli 書籍會跳過目標語言目錄

## JS 處理

- 以正則規則擷取常見的文字片段
- 避免翻譯技術值或純數字內容

## JAR 處理

- 掃描 JAR 內 JSON
- 支援 Patchouli 手冊語言路徑轉換
- 若來源語言檔不存在，會以 `en_us` 作為 fallback
- JourneyMap `.theme2.json` 會強制跳過

## 輸出分流

- 只有 JAR 來源或原資源結構 JSON 進入 `temp_translator`
- 其他 JSON/JS 直接鏡像輸出
- `output_resource_pack` 會產出 `LLMTranslator.zip`

## 相關檔案

- [src/file/pipeline.rs](/src/file/pipeline.rs)
- [src/file/json_handler.rs](/src/file/json_handler.rs)
- [src/file/js_handler.rs](/src/file/js_handler.rs)
- [src/file/jar_handler.rs](/src/file/jar_handler.rs)
- [src/file/pack_gen.rs](/src/file/pack_gen.rs)

## 相關連結

- [翻譯核心](translation_core.md)
- [增量比對策略](translation_comparison.md)
