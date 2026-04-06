# AI Context: mc_translator_rs 快速接手說明

這份文件以目前程式碼為準，提供維護者可快速對照的「事實版」行為摘要。

## 核心資料流

- 掃描 `.jar` / `.json` / `.js` 建立 `FileTask` 與 `GlobalBatchItem`
- 依群組鍵分組: JAR 以檔案本身為群組，非 JAR 以父資料夾為群組
- 全域批次翻譯採三段降級: 批次 -> 半批次 -> 單筆
- 全域進度包含 Offset，避免跨檔案視窗造成進度倒退

## 增量與跳過

- 若目標語言檔案已存在且該欄位與原文不同，視為已翻譯並直接預填
- `should_skip_key` 與 `should_skip_value` 會跳過 ID、布林值、純數字、snake_case、命名空間 ID 等
- 格式保護會將 `§`、`&`、`#HEX`、`%s`、`{0}`、`\n` 等轉為暫存標記，避免 LLM 破壞
- 會先套用 `excluded_paths` 全域排除清單
- JourneyMap `.theme2.json` 會被跳過

## 輸出規則

- 輸出根目錄固定為 `LLMTranslator/`
- 只有 JAR 來源或原本位於 `assets/`、`patchouli_books/` 的 JSON 會進入資源包暫存
- 非資源結構 JSON 與 JS 直接以原相對路徑輸出
- 資源包輸出完成後產生 `LLMTranslator.zip` 並清理暫存目錄

## 字典與術語

- 官方詞庫: 由 `dicts/en_us.json` 與 `dicts/zh_tw.json` 對照建立精確匹配
- 推論詞庫: 由官方詞庫推論，寫入 `dicts/official/{target_lang}.json`
- 使用者詞庫: `dicts/user/{target_lang}.json`，由 GUI 建議詞管理器維護
- Glossary 僅作提示，不直接替換原文
- Glossary 優先序由 `glossary_priority` 決定 (official 或 user)

## 翻譯記憶體

- `translation_memory` 為執行期記憶體結構，未做持久化或自動學習
- 目前持久化的只有使用者詞庫 (dicts/user)

## 進入點分流

- GUI 模式: `src-tauri` 提供 Tauri commands，前端在 `frontend/`
- CLI 模式: `src/cli/main.rs` 支援 headless 參數與互動式流程
- 共通流程: 兩者皆呼叫 `translation/pipeline.rs::start_translation_workflow`

## 設定與金鑰

- 設定檔: `settings/config.cfg` 與 `settings/style.cfg`
- API Key: 僅使用 OS Keyring，不寫入 `config.cfg`
- `api_base_url` 可在 GUI 設定面板中直接修改
- StyleConfig 支援 `instance_overrides`，可針對單一元件覆寫主題顏色或圓角

## 翻譯 API 行為

- 空 API Key 時會返回明確錯誤 (`API_KEY_REQUIRED`)，不會靜默 fallback 到 Google Free
- 前端會鎖定翻譯按鈕直到 API Key 填入
- CLI headless 模式下空 key 會導致翻譯失敗

## 日誌

- LLM 通訊日誌: `llm_communication.log`
- Debug 日誌: `logs/debug.log` (啟用 `enable_debug_log` 時)

## 重要約定

- 長耗時 I/O 需放入 `tokio::task::spawn_blocking`
- UI 狀態以 `Atomic*` + `Arc<Mutex<...>>` 控制
- 背景任務以 `tokio::spawn` 併發

---

此文件僅記錄「目前程式確定存在的行為」。若功能變更，請同步更新本文檔。
