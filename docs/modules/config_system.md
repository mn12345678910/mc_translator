# 設定系統

本文件說明 AppConfig 與 StyleConfig 的載入、預設值與儲存位置。

## AppConfig

- 檔案: `settings/config.cfg`
- 主要欄位分類: 提供商與模型、Prompt 與語言、翻譯參數、輸出與資源、面板狀態、檔案過濾、日誌開關、全域排除、視窗幾何
- 提供商與模型: `api_provider`, `model`, `ollama_url`, `api_base_url`
- Prompt 與語言: `user_prompt`, `system_prompt`, `source_lang`, `target_lang`, `ui_lang`
- 翻譯參數: `batch_size`, `batch_max_chars`, `timeout`, `glossary_priority`
- 輸出與資源: `output_dir`, `pack_format`
- 面板狀態: `show_api_settings`, `show_developer_mode`
- 檔案過濾: `skip_json`, `skip_js`, `skip_jar`, `skip_book`
- 日誌開關: `enable_llm_log`, `enable_debug_log`
- 全域排除: `excluded_paths`
- 視窗幾何: `main_x`, `main_y`, `main_width`, `main_height`, `viewer_x`, `viewer_y`, `viewer_width`, `viewer_height`
- `api_key` 不會寫入檔案，改由 OS Keyring 儲存
- `api_base_url` 可用於 OpenAI 相容端點 (僅能手動修改 config.cfg)

## StyleConfig

- 檔案: `settings/style.cfg`
- 內容包含主題、字體大小、調色盤、間距與進度條樣式
- 顏色欄位: `dark_*` / `light_*` (背景、文字、按鈕、標籤、hover、tab、進度條、日誌色彩)
- 透明度: `border_alpha`, `panel_alpha`, `backdrop_alpha`
- 間距: `space_sm`, `space_md`, `space_lg`
- 圓角與動畫: `btn_rounding_enabled`, `btn_rounding_value`, `progress_pulse_enabled`, `progress_pulse_speed`, `progress_style`
- 特效色: `aurora_1`, `aurora_2`, `aurora_3`, `neon_color`
- 支援特定元件覆寫 `instance_overrides`

## 讀寫流程

- `AppConfig::load` 與 `StyleConfig::load` 會建立 `settings/` 目錄
- 讀取失敗會回退到預設值
- **反序列化防護**: 所有欄位都設有 `#[serde(default)]` 或 `#[serde(default = "...")]`，部分缺失的設定檔不會導致整個檔案失效
- 儲存時會先進行 `validate` 驗證
- **載入後驗證**: `load` 完成後會自動呼叫 `validate()` 校正數值範圍與空字串
- API Key 透過 Keyring 儲存與讀取

## 流程圖

```mermaid
flowchart TD
    A[Load Config] --> B{檔案存在?}
    B -- 否 --> C[使用預設值]
    B -- 是 --> D[反序列化 (含 serde default 防護)]
    D --> E{成功?}
    E -- 否 --> C
    E -- 是 --> F[進行 validate (載入後校正)]
    F --> G[回傳 Config]
    G --> H[Save Config]
    H --> I[validate + 寫入檔案]
    H --> J[API Key 寫入 Keyring]
```

## 相關檔案

- [src/config/settings.rs](/src/config/settings.rs)
- [src/config/dictionary.rs](/src/config/dictionary.rs)
