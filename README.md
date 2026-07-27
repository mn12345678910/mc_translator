<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# Minecraft Mod Auto-Translator (mc_translator)

[![codecov](https://codecov.io/gh/mn12345678910/mc_translator/graph/badge.svg)](https://codecov.io/gh/mn12345678910/mc_translator)

> [!WARNING]
由於大多數硬體限制，評估目前環境還無法達到理想效果，暫時停止維護

> [!WARNING]
> 安全須知:
> 本工具使用系統憑證管理鏈 (`keyring` crate) 對 API KEY 進行安全儲存。
> 但本工具 100% 由 AI 撰寫，請自行評估風險，如有疑慮請使用 Ollama 本地模型等不需要 API Key 的翻譯服務。

> [!IMPORTANT]
> 翻譯品質須知:
> 本工具適合快速翻譯與初稿整理，品質無法取代人工翻譯。請不要將輸出內容直接提交給模組作者。

> [!NOTE]
> Ollama 和 Google Free 不需要 API Key。

`mc_translator` 是一款為 Minecraft 模組與整合包本地化設計的工具，支援 GUI (Tauri) 與 CLI 兩種操作模式。
核心流程為: 掃描檔案 -> 擷取可翻譯字串 -> 呼叫翻譯服務 -> 輸出資源包或鏡像檔案。

## 主要功能

- 支援 `.jar`、`.json`、`.js` 檔案的可翻譯內容掃描
- 全域批次翻譯與降級重試 (批次 -> 半批次 -> 單筆)
- 可在翻譯過程中暫停、繼續、停止
- 術語系統與建議詞管理 (官方詞庫 + 推論詞庫 + 使用者詞庫)
- GUI 提供即時樣式與調色盤自訂
- 可輸出資源包 `LLMTranslator.zip` 與鏡像檔案

## 支援的翻譯服務

GUI 與 CLI 都提供相同的服務商選項，但「模型列表的動態拉取能力」不同。

**GUI 服務商選項 (下拉選單)**

- Gemini
- OpenAI
- DeepSeek
- Mistral
- DeepL
- Ollama (Local)
- Google Free
- 無 (None)

**GUI 模型列表動態拉取**

- 支援: Ollama, Gemini, OpenAI, DeepSeek
- 不支援時會顯示「無可用模型」

**CLI 可用提供商**

- Gemini
- OpenAI
- DeepSeek
- Mistral
- Ollama
- DeepL
- Google Free

## 支援的檔案類型

- `.jar` (支援內部語言檔與 Patchouli 手冊路徑)
- `.json`
- `.js`

## 快速開始

### GUI

1. 啟動程式後點選 `⚙` 開啟設定
2. 選擇服務商與模型並填入 API Key (Ollama 免填)
3. 選擇檔案或資料夾
4. 設定輸出資料夾與參數
5. 點擊開始翻譯

### CLI

互動導覽:

```powershell
./mc_translator_cli.exe
```

參數模式:

```powershell
./mc_translator_cli.exe -i <輸入路徑> -p <提供商> -m <模型> -o <輸出目錄>
```

## CLI 參數

| 參數                        | 說明                                  | 預設                          |
| --------------------------- | ------------------------------------- | ----------------------------- |
| `-i, --input <INPUT>`       | 輸入檔案或資料夾路徑                  | 無 (必填)                     |
| `-o, --output <OUTPUT>`     | 輸出資料夾路徑                        | 空值 (代表 `./LLMTranslator`) |
| `-p, --provider <PROVIDER>` | API 提供商                            | 設定檔的值                    |
| `-m, --model <MODEL>`       | 模型名稱                              | 設定檔的值                    |
| `--api-key <API_KEY>`       | 覆蓋 API Key (會取代 Keyring 讀取值)  | 空                            |
| `--log-llm`                 | 啟用 LLM 通訊日誌                     | 關閉                          |
| `--batch-size <BATCH_SIZE>` | 批次量                                | 150                           |
| `--batch-max-chars <CHARS>` | 批次字數上限                          | 3500                          |
| `--timeout <TIMEOUT>`       | API 逾時秒數                          | 60                            |
| `--glossary-priority <PRI>` | 術語優先級 (`official` 或 `user`)     | official                      |
| `--source-lang <SOURCE>`    | 來源語言                              | en_us                         |
| `--target-lang <TARGET>`    | 目標語言                              | zh_tw                         |
| `--skip-json`               | 跳過 `.json`                          | 關閉                          |
| `--skip-js`                 | 跳過 `.js`                            | 關閉                          |
| `--skip-jar`                | 跳過 `.jar`                           | 關閉                          |
| `--skip-book`               | 跳過 Patchouli 手冊                   | 關閉                          |
| `--log-debug`               | 啟用 Debug 日誌 (debug.log)           | 關閉                          |
| `--fast-convert`            | 啟用快速簡繁轉換 (僅限 zh_cn ↔ zh_tw) | 關閉                          |
| `-e, --exclude <EXCLUDE>`   | 追加排除路徑 (可重複)                 | 空                            |

## 輸出結構

輸出根目錄固定為 `LLMTranslator/`。

- 未指定輸出路徑時: `./LLMTranslator/`
- 指定輸出路徑時: `<輸出路徑>/LLMTranslator/`

**資源包輸出**

- 只有 JAR 來源或原本在 `assets/` 或 `patchouli_books/` 的 JSON 會進入資源包暫存
- 產出 `LLMTranslator.zip`
- 內含 `pack.mcmeta` 與標準 `assets/<modid>/lang/<目標語言>.json`
- Patchouli 手冊會將 `/<來源語言>/` 轉成 `/<目標語言>/`

**鏡像輸出**

- 非資源結構的 JSON 與 JS 會以原相對路徑輸出成實體檔案
- 來源語言 JSON 會依目標語言轉為對應檔名 (如 `zh_tw.json`)

**暫存區**

- 資源包暫存檔案會寫入 `LLMTranslator/temp_translator/`
- 資源包輸出完成後會自動清理此暫存目錄

## 設定與金鑰儲存

**設定檔位置**

- `settings/`: 設定資料夾 (首次執行會自動建立)
- `config.cfg`: App 設定
- `style.cfg`: GUI 樣式設定

**API Key 儲存方式**

- 透過作業系統 Keyring 儲存，不會寫入 `config.cfg`

## 設定值說明 (限制與預設值)

本工具對設定值設有基本的驗證邏輯，避免非法數值導致程式崩潰。

### App 設定 (`config.cfg` / CLI 參數)

| 設定項            | 說明         | 預設值 | 限制 (最小值 ~ 最大值)                |
| ----------------- | ------------ | ------ | ------------------------------------- |
| `batch_size`      | 翻譯批次量   | 150    | 1 ~ 500 (若設定為 0 或超出範圍會校正) |
| `batch_max_chars` | 批次字數上限 | 3500   | 1 ~ 20,000                            |
| `timeout`         | API 逾時秒數 | 60     | 1 ~ 300                               |
| `pack_format`     | 資源包版本   | 15     | 1 ~ 128 (依 Minecraft 版本)           |

### GUI 樣式設定 (`style.cfg`)

| 設定項               | 說明                | 預設值   | 限制 / 範圍 |
| -------------------- | ------------------- | -------- | ----------- |
| `font_size`          | 介面字體大小        | 15.0     | 12.0 ~ 30.0 |
| `btn_rounding`       | 按鈕圓角值          | 4.0      | 0.0 ~ 100.0 |
| `pulse_speed`        | 進度條動畫速度      | 1.0      | 0.1 ~ 10.0  |
| `border_alpha`       | 邊框透明度          | 0.15     | 0.01 ~ 1.0  |
| `panel_alpha`        | 面板背景透明度      | 0.03     | 0.01 ~ 1.0  |
| `backdrop_alpha`     | 彈窗遮罩透明度      | 0.6      | 0.01 ~ 1.0  |
| `space_sm / md / lg` | 元件間距 (小/中/大) | 10/15/20 | 0.0 ~ 100.0 |

## 日誌

- LLM 通訊日誌: `llm_communication.log` (啟用 `--log-llm` 或 GUI 開關)
- Debug 日誌: `logs/debug.log` (啟用 `--log-debug` 或 GUI 開關)

## 快速簡繁轉換 (Fast Chinese Conversion)

當  **目標語言** 設定為 `zh_cn` (簡體中文) 與 `zh_tw` (繁體中文) 時，您可以開啟此功能。

- **優點**：完全在本地執行，速度極快（毫秒級），且不消耗任何 LLM API 額度。
- **在地化保證**：系統會優先使用 **術語表 (Glossary)** 進行精確替換，以確保 Minecraft 官方譯名（如：下界、紅石）正確轉換，剩餘部分才使用通用簡繁轉換。
- **啟用方式**：介面選對語言後，在 API 設定面板勾選「快速簡繁轉換」即可。

## CI 發行產物

CI 僅在 tag 版號 (`v*`) 時會產出以下檔名:

- `mc_translator_cli_win_x64.exe`
- `mc_translator_cli_linux_x64`
- `mc_translator_gui_win_x64.exe`
- `mc_translator_gui_linux_x64`

詳細流程請參考: [release_artifacts](/docs/guides/release_artifacts.md) 與 [.github/workflows/ci.yml](/.github/workflows/ci.yml)

## 文檔

完整技術文檔請參閱 [docs/README.md](docs/README.md)。
