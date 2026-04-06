<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# UI 規格 (Tauri GUI)

## 主要區塊

- 頂部控制列: 選檔、選資料夾、輸出路徑、介面語言、開啟輸出資料夾
- API 設定面板: 服務商、模型、API Key、Ollama URL
- 翻譯參數面板: 批次量、字數上限、逾時、語言、pack_format
- 進度區: 全域進度、批次進度、狀態文字
- 動作區: 開始、暫停、繼續、停止
- 日誌區: 事件與錯誤訊息
- 建議詞管理器: 獨立視窗 `dict.html`
- 調色盤/主題: 深淺主題切換與樣式參數

## 啟動預設狀態

- 狀態: 待機
- 已選檔案: 空
- 輸出路徑: `./LLMTranslator` (顯示為空值但視為預設)
- API 設定面板: 收合
- 開發者模式: 收合
- 調色盤面板: 收合

## 設定面板預設值

- 服務商: `無`
- 模型: 空值
- Ollama URL: `http://localhost:11434`
- 批次量: 150
- 批次字數上限: 3500
- 逾時秒數: 60
- 來源語言: en_us
- 目標語言: zh_tw
- 介面語言: zh_tw
- 資源包版本: 15

## pack_format 選單

- 1.21.4 -> 46
- 1.21.2 -> 42
- 1.21 -> 34
- 1.20.4 -> 22
- 1.20.2 -> 18
- 1.20.1 -> 15
- 1.19.4 -> 13
- 1.19.2 -> 10
- 1.18.2 -> 9
- 1.16.5 -> 6

## 開發者模式

- 跳過 JSON / JS / JAR / 手冊
- 記錄 LLM 通訊日誌
- 記錄 Debug 日誌

## 視覺與樣式預設值

- 主題: dark
- 字體大小: 15
- 按鈕圓角: 啟用，預設 4.0
- 進度條樣式: default (可切換 aurora / neon)

## 主題顏色對照表

以下表格對照 `StyleConfig` 預設值與 `applyColors` 寫入的 CSS 變數，並標示影響範圍。完整欄位來源請參考 [docs/modules/config_system.md](/docs/modules/config_system.md)。

| StyleConfig 欄位 | 深色預設值 (RGB) | 淺色預設值 (RGB) | CSS 變數 | 影響元件/區域 |
| --- | --- | --- | --- | --- |
| `dark_bg` / `light_bg` | `[30, 30, 35]` | `[252, 252, 253]` | `--bg-color` | 全域背景、輸入區與卡片基底 |
| `dark_text` / `light_text` | `[200, 160, 100]` | `[30, 30, 35]` | `--text-color` | 全域文字、主要標題 |
| `dark_accent` / `light_accent` | `[212, 175, 55]` | `[0, 120, 212]` | `--accent-color` | 強調色、主標題、按鈕 hover |
| `dark_danger` / `light_danger` | `[170, 17, 17]` | `[170, 17, 17]` | `--danger-color` | 危險按鈕、錯誤提示 |
| `dark_label` / `light_label` | `[200, 160, 100]` | `[30, 30, 35]` | `--label-color` | label 文案 |
| `dark_text_muted` / `light_text_muted` | `[170, 170, 170]` | `[102, 102, 102]` | `--text-muted` | 次要文字、狀態提示 |
| `dark_btn_bg` / `light_btn_bg` | `[45, 45, 50]` | `[240, 240, 245]` | `--btn-bg` | 按鈕背景 |
| `dark_btn_text` / `light_btn_text` | `[220, 220, 220]` | `[45, 45, 50]` | `--btn-text` | 按鈕文字 |
| `dark_input_bg` / `light_input_bg` | `[20, 20, 25]` | `[255, 255, 255]` | `--input-bg` | 輸入框背景 |
| `dark_list_bg` / `light_list_bg` | `[25, 25, 30]` | `[250, 250, 252]` | `--list-bg` | 日誌/列表背景 |
| `dark_tab_active` / `light_tab_active` | `[60, 60, 70]` | `[230, 235, 245]` | `--tab-active-bg` | Tab 啟用背景 |
| `dark_tab_inactive` / `light_tab_inactive` | `[35, 35, 40]` | `[245, 245, 250]` | `--tab-inactive-bg` | Tab 未啟用背景 |
| `dark_border_color` / `light_border_color` | `[60, 60, 66]` | `[210, 210, 220]` | `--border-color` | 邊框、分隔線 |
| `dark_hover_bg` / `light_hover_bg` | `[56, 56, 64]` | `[225, 235, 250]` | `--hover-bg` | hover 背景 |
| `dark_slider_bg` / `light_slider_bg` | `[42, 42, 48]` | `[220, 220, 210]` | `--slider-bg` | Slider 軌道 |
| `dark_slider_thumb` / `light_slider_thumb` | `[224, 224, 224]` | `[80, 80, 80]` | `--slider-thumb` | Slider 拖曳點 |
| `dark_switch_bg` / `light_switch_bg` | `[26, 26, 31]` | `[230, 230, 220]` | `--switch-bg` | Switch 背景 |
| `dark_progress_bg` / `light_progress_bg` | `[51, 51, 51]` | `[235, 235, 240]` | `--progress-bg` | 進度條背景 |
| `dark_header_bg` / `light_header_bg` | `[37, 37, 43]` | `[235, 235, 240]` | `--header-bg` | 頂部工具列背景 |
| `dark_log_info` / `light_log_info` | `[200, 200, 200]` | `[30, 30, 35]` | `--log-info-color` | 日誌 Info |
| `dark_log_warn` / `light_log_warn` | `[217, 119, 6]` | `[180, 100, 0]` | `--log-warn-color` | 日誌 Warning |
| `dark_log_error` / `light_log_error` | `[255, 85, 85]` | `[170, 17, 17]` | `--log-error-color` | 日誌 Error |
| `dark_log_success` / `light_log_success` | `[60, 180, 120]` | `[5, 150, 105]` | `--log-success-color` | 日誌 Success |
| `dark_log_dir` / `light_log_dir` | `[212, 175, 55]` | `[150, 110, 0]` | `--log-dir-color` | 日誌路徑顯示 |
| `dark_log_file` / `light_log_file` | `[85, 255, 255]` | `[0, 120, 212]` | `--log-file-color` | 日誌檔名顯示 |
| `aurora_1` | `[255, 0, 127]` | `[255, 0, 127]` | `--aurora-1` | 進度條 aurora 起始色 |
| `aurora_2` | `[127, 0, 255]` | `[127, 0, 255]` | `--aurora-2` | 進度條 aurora 中間色 |
| `aurora_3` | `[0, 255, 255]` | `[0, 255, 255]` | `--aurora-3` | 進度條 aurora 結束色 |
| `neon_color` | `[0, 255, 204]` | `[0, 255, 204]` | `--neon-color` | 進度條 neon 色 |

## 衍生透明度/顏色

| 欄位 | 預設值 | 影響 CSS 變數 | 說明 |
| --- | --- | --- | --- |
| `border_alpha` | `0.15` | `--border-light` | 由 `border_color` 結合透明度產生淡邊框 |
| `panel_alpha` | `0.03` | `--panel-bg` | 由 `bg-color` 結合透明度產生面板底色 |
| `backdrop_alpha` | `0.6` | `--backdrop-bg` | 由 `bg-color` 結合透明度產生背板遮罩 |

## 特定元件覆寫 (instance_overrides)

`instance_overrides` 允許針對單一元件覆寫顏色與圓角，並依當前主題寫入 `dark_*` 或 `light_*` 欄位。

| 元件 ID | 可覆寫欄位 | 對應 UI 元件名稱 | 套用條件 |
| --- | --- | --- | --- |
| `btn-translate` | `dark_bg` / `light_bg` / `dark_text` / `light_text` / `rounding` | 開始翻譯按鈕 | 依主題寫入對應欄位 |
| `btn-pause` | 同上 | 暫停按鈕 | 依主題寫入對應欄位 |
| `btn-stop` | 同上 | 停止按鈕 | 依主題寫入對應欄位 |
| `btn-browse-file` | 同上 | 選檔按鈕 | 依主題寫入對應欄位 |
| `btn-browse-dir` | 同上 | 選資料夾按鈕 | 依主題寫入對應欄位 |
| `btn-browse-output` | 同上 | 輸出資料夾按鈕 | 依主題寫入對應欄位 |
| `btn-browse-output-open` | 同上 | 開啟輸出資料夾 | 依主題寫入對應欄位 |
| `user-prompt` | 同上 | User Prompt 輸入框 | 依主題寫入對應欄位 |
| `system-prompt` | 同上 | System Prompt 輸入框 | 依主題寫入對應欄位 |
| `input-path` | 同上 | 輸入路徑輸入框 | 依主題寫入對應欄位 |
| `output-dir` | 同上 | 輸出路徑輸入框 | 依主題寫入對應欄位 |
| `dict-dialog` | 同上 | 建議詞管理器區塊 | 依主題寫入對應欄位 |
| `log-output` | 同上 | 日誌輸出區 | 依主題寫入對應欄位 |
| `progress-bar` | 同上 | 主進度條 | 依主題寫入對應欄位 |
| `batch-progress-bar` | 同上 | 批次進度條 | 依主題寫入對應欄位 |

## 視窗

- 主視窗會在關閉時寫入尺寸與座標
- 建議詞管理器為獨立 Webview 視窗，不是 dialog
