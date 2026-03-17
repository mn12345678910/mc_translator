# 檔案處理流水線

## 掃描與收集

- `scanner::scan_files_recursive` 遞迴掃描 `.jar`、`.json`、`.js`。
- `collect_json_task` 解析 JSON 並建立翻譯項目（詳見 [語言檔案比對機制](translation_comparison.md)）。
- `collect_js_task` 依規則擷取可翻譯字串。
- `collect_jar_tasks` 讀取 JAR 內 `en_us.json` 與 Patchouli 手冊。

## 分組與排序

- JAR 以檔案本身為分組鍵
- 非 JAR 以父資料夾為分組鍵
- 重新排序 `global_items`，確保順序與 `file_tasks` 一致

## 跨檔案窗口翻譯

- 以分組為單位建立「跨檔案窗口」批次
- `translate_global_batches` 會在窗口內進行批次、降級、重試

## 輸出與封裝

- 先寫入 `LLMTranslator/temp_translator` 作為資源包暫存
- 窗口完成後立即輸出鏡像檔案
- 全部完成後產出 `LLMTranslator.zip`

