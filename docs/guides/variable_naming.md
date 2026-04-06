<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# 變數命名規範

本規範維持與 Rust 與前端 JavaScript 的實作風格一致。

## Rust

- 模組與檔名使用 `snake_case`
- 型別與結構使用 `PascalCase`
- 常數使用 `SCREAMING_SNAKE_CASE`
- 布林值使用 `is_` 或 `has_` 前綴

## JavaScript (frontend)

- 函式與變數使用 `camelCase`
- 常數使用 `UPPER_SNAKE_CASE`
- DOM id 使用 `kebab-case`

## i18n Keys

- 統一以 `snake_case` 風格
- 新增 key 時需同步到 [src/i18n.rs](/src/i18n.rs) 與 [src/i18n_assets/](/src/i18n_assets/)

## 建議

- 優先使用語義明確的名稱
- 避免縮寫，除非是專案通用縮寫 (例如 LLM)
