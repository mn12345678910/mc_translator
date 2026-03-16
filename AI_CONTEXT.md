# AI Context: mc_translator_rs 快速接手說明

這份文件是給後續維護者的「事實對照版」。內容以目前程式碼為準，避免與現況脫節。

## 核心行為摘要

**翻譯資料流**
- 掃描 `.jar` / `.json` / `.js`，建立 `FileTask` 與全域 `GlobalBatchItem`。
- 依群組鍵分批：JAR 依檔案本身，非 JAR 依「父資料夾」。
- 批次翻譯採三段降級：原批次 -> 半批次 -> 單筆。
- 進度包含全域 offset，避免跨檔案窗口時進度倒退。

**增量與跳過**
- `zh_tw` 既有翻譯且與原文不同時視為已翻譯，直接預填。
- `should_skip_key` / `should_skip_value` 會跳過 ID、布林值、純數字、snake_case、命名空間 ID 等。

**輸出規則**
- 輸出根目錄固定為 `LLMTranslator/`。
- 只要是 JAR 來源或原本在 `assets/`/`patchouli_books/` 內的 JSON 會進入資源包暫存，再由 `LLMTranslator.zip` 輸出。
- 非資源結構 JSON 與 JS 會以原相對路徑輸出為實體檔案。
- `en_us.json` 會轉成 `zh_tw.json`；Patchouli 會將 `/en_us/` 轉為 `/zh_tw/`。

## 術語與字典

**資料來源**
- 官方詞庫：由 `dicts/en_us.json` 與 `dicts/zh_tw.json` 的對照生成精確匹配。
- 推論詞庫：由 `analyze_dictionary` 從官方詞庫推導，存到 `dicts/official.json`。
- 使用者詞庫：`dicts/user.json`，由字典管理器維護。

**使用方式**
- Glossary 只作提示，不做替換。
- Priority 由 `glossary_priority` 控制，同 key 時「先載入的優先」。
- Prompt 注入僅挑選最多 30 筆術語。

## 併發與 I/O 規範

- 長耗時 I/O 必須放入 `tokio::task::spawn_blocking`。
- UI 狀態以 `Atomic*` + `Arc<Mutex<...>>` 控制。
- 背景任務統一用 `runtime.spawn`。

## i18n

- UI 文字來自 `langs/<lang>.json`，預設 `zh_tw`。
- `I18nLabels::ensure_langs_exists()` 會確保預設語系檔案存在。

## 字典監控

- 監控 `dicts/` 目錄中的 `en_us.json`/`zh_cn.json`/`zh_tw.json`。
- 變動會觸發 `refresh_dictionaries_core` 重新推論並更新 `official.json`。

## LLM 記錄

- `enable_llm_log` 開啟時會將完整對話寫入 `llm_communication.log`。

## Git 規範 (維護習慣)

- 重要修改後請先 `git add .`。
- Commit 請使用繁體中文。
- 新分支命名請遵循 `feature/` 前綴。

---
此檔案只記錄「目前程式確定存在的行為」。若功能變更，請同步更新本文檔。

