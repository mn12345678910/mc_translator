<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# 翻譯記憶體與使用者詞庫

本文件說明目前的翻譯記憶體與使用者詞庫行為。

## 使用者詞庫

- 路徑: `dicts/user/{ui_lang}.json`
- 由 GUI 建議詞管理器維護
- 支援新增、編輯、刪除、匯入、匯出
- 變更會立即寫回檔案

## 推論詞庫

- 路徑: `dicts/official/{ui_lang}.json`
- 由官方詞庫推論後生成

## 翻譯記憶體 (執行期)

- `translation_memory` 目前為執行期記憶體結構
- 未做持久化或自動學習
- 主要用於 Glossary 提示流程中的參考資料

## 相關連結

- [術語系統](glossary_system.md)
- [UI 交互](../ui/interactions.md)
