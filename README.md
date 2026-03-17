# Minecraft Mod Auto-Translator (mc_translator_rs)

> [!CAUTION]
> 警告:
> 本工具雖使用微軟DPAPI對API KEY進行加密，但由於本工具100%使用AI開發，
> 如有疑慮者請"不要"使用需要填入API KEY的服務商。

> [!IMPORTANT]
> 翻譯品質須知:
> 本工具適合快速翻譯與初稿整理，品質無法取代人工翻譯。請"不要"將輸出內容直接提交給模組作者。

`mc_translator_rs` 是一款為 Minecraft 整合包與模組本地化設計的 GUI 工具，以 Rust + egui 實作。它能掃描 JAR、JSON、JS，將可翻譯字串交由 LLM 或第三方翻譯服務處理，並輸出為資源包或原路徑鏡像檔案。

**支援的翻譯服務**
- Gemini
- OpenAI
- DeepSeek
- Mistral
- Ollama (本機)
- DeepL
- Google Free

**支援的檔案類型**
- `.jar` (僅處理內部 `en_us.json` 與 Patchouli 手冊路徑)
- `.json`
- `.js`

## 核心功能

**1) 動態模型列表**
- 依服務商自動拉取可用模型清單
- DeepL 提供 `deepl-free` 與 `deepl-standard`

**2) 智慧跳過與增量保留**
- 自動跳過純數字、布林值、命名空間 ID、snake_case 等不可翻譯內容
- 已存在且不同於原文的目標語言項目會被保留並視為已翻譯

**3) 穩定批次翻譯與降級重試**
- 批次翻譯支援「批次大小 + 字元上限」雙限制
- 失敗時依序降級為「半批次」再到「單筆」

**4) 術語表與翻譯記憶體**
- 內建官方詞庫推論與使用者詞庫管理
- 可匯入、匯出、搜尋、批次取代
- 以術語提示方式注入模型，不會直接替換原文

**5) 進度與控制**
- 條目進度、檔案進度、批次進度
- 支援暫停、繼續、停止

## 輸出結構

輸出根目錄固定為 `LLMTranslator/`：
- 未指定輸出路徑時：`./LLMTranslator/`
- 指定輸出路徑時：`<輸出路徑>/LLMTranslator/`

**資源包輸出**
- 只有 JAR 來源或原本已在資源結構內的 JSON 會進入資源包
- 產出 `LLMTranslator.zip`
- 內含 `pack.mcmeta` 與標準 `assets/<modid>/lang/<目標語言>.json`
- Patchouli 手冊會將 `/en_us/` 轉成 `/<目標語言>/`

**鏡像輸出**
- 非資源結構的 JSON 與 JS 會以原相對路徑輸出成實體檔案
- `en_us.json` 會依目標轉為對應語系檔名 (如 `zh_tw.json`)

## 需求環境

| 項目 | 需求 |
| --- | --- |
| 作業系統 | Windows 10/11 (DPAPI) |
| 網路 | 需可連線至 API (Ollama 為本機) |
| 硬體 | 依模型與服務商需求而定 |

## 快速操作

1. 啟動程式後點選 `⚙` 開啟設定
2. 選擇服務商與模型並填入 API Key (Ollama 免填)
3. 選擇檔案或資料夾
4. 設定輸出資料夾與參數
5. 點擊開始翻譯

## 文檔索引

- 架構總覽：`docs/architecture/overview.md`
- 狀態管理：`docs/architecture/state_management.md`
- 錯誤處理：`docs/architecture/error_handling.md`
- 翻譯核心：`docs/modules/translation_core.md`
- 檔案流水線：`docs/modules/file_pipeline.md`
- 術語系統：`docs/modules/glossary_system.md`
- 翻譯記憶體：`docs/modules/translation_memory.md`
- UI 規格：`docs/ui/specs.md`
- UI 交互：`docs/ui/interactions.md`
- 測試策略：`docs/guides/testing_strategy.md`
- 維護日誌：`docs/guides/MAINTENANCE_LOG.md`
- Git 指南：`docs/guides/GIT_GITHUB_GUIDE.md`

