# 翻譯記憶體與使用者詞庫

## 檔案位置

- 使用者詞庫：`dicts/user/{ui_lang}.json`
- 推論詞庫：`dicts/official/{ui_lang}.json`

## 使用方式

- 使用者詞庫由字典管理器維護
- 支援新增、編輯、刪除、匯入、匯出、批次取代
- 變更會立即寫回 `dicts/user/{ui_lang}.json`

## 在翻譯流程中的角色

- 使用者詞庫會參與 Glossary 建議
- 只作提示，不會直接替換原文

