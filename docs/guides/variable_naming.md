# 變數命名規範

## 目標與範圍
本文件定義本專案（Rust + JavaScript）變數命名規範，目標是提升可讀性、降低誤解與維護成本，並避免 shadowing 與未使用變數。

適用範圍：
- Rust：`src/`, `src-tauri/`, `tests/`
- JS：`frontend/`, `tests/`

## Rust 命名規則
1. 一律使用 `snake_case`。
2. 名稱應具體描述用途，避免過短縮寫（如 `p`, `c`, `v`, `s`, `l`）。
3. 同一作用域或鄰近區塊避免重複語意的名稱（例如多個 `items`, `entries`）。
4. 容器或集合建議使用複數名詞（`tasks`, `items`, `entries`）。
5. 路徑/字串/物件/計數器等常見語意，使用明確字首：
   - `*_path`, `*_dir`（路徑）
   - `*_content`, `*_text`, `*_json`（內容/字串/JSON）
   - `*_count`, `*_idx`（計數或索引）

## JS 命名規則
1. 一律使用 `camelCase`。
2. DOM 元素變數加上語意尾綴：`inputEl`, `selectEl`, `buttonEl`, `labelEl`。
3. 避免單字母或縮寫（`el`, `p`, `k`, `v`），除非在極短、單一用途區塊且語意清晰。
4. 顏色、樣式、狀態建議加上語意字首：
   - `backgroundColor`, `textColor`, `buttonBgColor` 等
5. 事件處理中避免使用模糊名稱（如 `data`, `info`, `result`）。

## 避免 shadowing 與未使用變數
1. 禁止同一作用域或巢狀區塊使用相同變數名稱（shadowing）。
2. 若需要轉換/清理後的新值，使用新名稱表達語意（例如 `raw_value` → `cleaned_value`）。
3. 未使用變數需移除；若刻意保留，請以註解說明原因。

## Bad / Better 範例
### Rust
**Bad**
```rust
let p = dir.join(format!("{}.json", lang));
let c = fs::read_to_string(p)?;
let l_val = serde_json::from_str(&c)?;
```

**Better**
```rust
let lang_path = dir.join(format!("{}.json", lang));
let file_content = fs::read_to_string(lang_path)?;
let lang_json = serde_json::from_str(&file_content)?;
```

### JS
**Bad**
```js
const el = document.getElementById(id);
const k = dictInputKey.value.trim();
const v = dictInputValue.value.trim();
```

**Better**
```js
const inputEl = document.getElementById(id);
const dictKey = dictInputKey.value.trim();
const dictValue = dictInputValue.value.trim();
```

## 建議檢查指令
Rust：
```bash
cargo check --all-targets
cargo clippy --all-targets -- -W clippy::shadow_reuse -W clippy::shadow_same -W clippy::shadow_unrelated
```

JS：
```bash
npm run lint
npx eslint "tests/**/*.js" "frontend/tests/**/*.js" --no-config-lookup --rule "no-unused-vars:warn" --parser-options=ecmaVersion:latest,sourceType:module
```
