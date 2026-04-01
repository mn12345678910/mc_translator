# 新對話上下文：Minecraft 翻譯器三大問題調查摘要

**專案路徑**：`x:\D\MCTEST\mc_translator_rs`
**目前版本**：v0.9.11（已推送至 GitHub）
**穩定參考版本**：v0.9.7 / v0.8.9

---

## 問題一：UI 介面標籤遺失（`介面語言` + `快速簡繁轉換`）

### 症狀
- `介面語言:` 標籤（`label_ui_lang`）在 GUI 上無文字顯示
- `快速簡繁轉換` 標籤（`label_fast_convert`）在 GUI 上無文字顯示

### 根本原因（已確認）
**`i18n.rs` 的 `GuiLabels` 結構體缺少 `label_fast_convert`、`label_fast_convert_on`、`label_fast_convert_off` 三個欄位。**

- `label_ui_lang` 在 `i18n.rs` 的 L192 存在 ✅
- `label_fast_convert` 系列在 `i18n.rs` 中完全**不存在** ❌

JSON 資源檔中這些 KEY 是存在的（L220-L222 of all 4 JSON files）：
```json
"label_fast_convert": "快速簡繁轉換",
"label_fast_convert_on": "開啟簡繁轉換",
"label_fast_convert_off": "關閉簡繁轉換"
```

HTML 綁定存在（`frontend/index.html` L211、L217）：
```html
<label data-i18n="label_fast_convert"></label>
<small id="label-fast-convert-state">...</small>
```

JS 引用存在（`frontend/modules/i18n.js` L186-L213）。

### 修復方向
在 `src/i18n.rs` 的 `GuiLabels` 結構體中，在適當位置加入三個欄位：
```rust
pub label_fast_convert: String,
pub label_fast_convert_on: String,
pub label_fast_convert_off: String,
```
位置建議：緊接在 `label_ui_lang`（L192）之後。

---

## 問題二：建議詞管理器又回到主視窗（迴歸）

### 症狀
點擊「建議詞管理器」按鈕後，dict.html 的內容在主視窗中載入，而非獨立視窗。

### 根本原因（已確認）
此問題在 v0.8.9（commit `ce532bf`）時已修復，修復方法是在 `vite.config.js` 中明確將 `dict.html` 作為獨立的 Rollup 輸入入口：
```js
// vite.config.js ce532bf 的修復內容
rollupOptions: {
  input: {
    main: resolve(__dirname, 'frontend/index.html'),
    dict: resolve(__dirname, 'frontend/dict.html'),
  },
},
```

**目前版本的 `vite.config.js` 已有此設定**（L11-L16），但由於 root 設定為 `./frontend`，路徑寫法有差異：
```js
// 目前版本 (root: './frontend')
rollupOptions: {
  input: {
    main: 'index.html',  // 相對於 root
    dict: 'dict.html',
  },
},
```

### 還需確認的事項
- **Tauri 的 capabilities / 權限設定** 是否允許 `dict.html` 視窗的 API 呼叫（`src-tauri/capabilities/` 目錄）
- `src-tauri/src/commands.rs` L676-L679 的 `open_dict_window` 使用 `WebviewUrl::App("dict.html".into())` ，在開發模式下（Vite server）這對應 `http://localhost:5173/dict.html`，需確認路由正確
- 建議用 Vite dev server 實際驗證

---

## 問題三：快速簡繁轉換（fast_convert）部分條目未正確替換

### 症狀
來源：`X:/D/rstest/ctw/occultism-1.20.1-1.152.1.jar`
輸出：`X:/D/rstest/trs/LLMTranslator/LLMTranslator.zip`

以下條目依舊沒有正確替換（保留了不自然的簡繁混用或原文）：
- `#1647 "tag.block.occultism.netherrack" : "下界巖"` — 應轉換為繁體「下界巖 → 下界岩」或類似
- `#1654 "tag.item.forge.dusts.end_stone" : "粉碎末地石"` — 應轉換為繁體

### fast_convert 核心邏輯位置（已確認）
**情況 A（zh_cn ↔ zh_tw 直接轉換）**：`src/translation/batching.rs` L194-L235
**情況 B（來源非中文 + 目標為中文，使用兄弟檔案）**：`batching.rs` L237-L290
**兄弟檔案讀取（alt_map）**：`src/file/json_handler.rs` L109-L126

### 可能的根本原因
1. **TAG 類條目（`tag.block.*`, `tag.item.*`）的 KEY 在 JAR 的兄弟 zh_cn.json 中不存在**，導致 `alt_source` 為 `None`，跳過了快速轉換，回退至 LLM 翻譯，但 LLM 輸出的結果沒有再次經過 hanconv 轉換。
2. **hanconv 字典缺乏 `巖→岩` 這類地名用字的對應規則**，或詞組未收錄。
3. `apply_glossary_then_hanconv` 函式的行為需要確認（位置：`src/translation/batching.rs` 或 `src/utils/text_processing.rs`）。

### 還需調查的事項
- 確認 `apply_glossary_then_hanconv` 函式定義位置與邏輯
- 確認 LLM 翻譯後的結果是否也會通過 hanconv（目前看起來**不會**，只有 fast_convert 直接路徑才走 hanconv）
- 查看 `src/translation/glossary/mc_lang.rs` 是否有針對這些 TAG 條目的特殊處理

---

## 待修復清單（按優先順序）

| 優先 | 問題 | 修復位置 | 預估難度 |
|------|------|----------|----------|
| 高 | i18n.rs 缺少 label_fast_convert 欄位 | `src/i18n.rs` GuiLabels 結構體 | 低 |
| 中 | 建議詞視窗迴歸（需實際測試確認） | `src-tauri/capabilities/` 可能也需更新 | 中 |
| 高 | TAG 類條目 fast_convert 未轉換 | `src/translation/batching.rs` 或後處理邏輯 | 高 |

---

## 關鍵檔案速查

| 檔案 | 問題 |
|------|------|
| `src/i18n.rs` | GuiLabels 結構體，需加 label_fast_convert 系列欄位 |
| `src/i18n_assets/gui/*.json` | JSON 資源，label_fast_convert 已存在（L220-L222） |
| `frontend/index.html` | HTML 綁定，L114（label_ui_lang）、L211（label_fast_convert） |
| `frontend/modules/i18n.js` | updateToggleStateLabel()，L209-L214（chk-fast-convert 特殊處理） |
| `src-tauri/src/commands.rs` | open_dict_window()，L672-L701 |
| `src-tauri/tauri.conf.json` | 視窗配置，目前只有 main 視窗靜態宣告 |
| `vite.config.js` | dict.html 入口設定（L11-L16），已存在但需驗證 |
| `src/translation/batching.rs` | fast_convert 核心邏輯（L193-L290） |
| `src/file/json_handler.rs` | alt_map（兄弟語言檔讀取），L109-L126 |
| `src/file/jar_handler.rs` | JAR 的 fast_convert 處理（L44-L57） |
