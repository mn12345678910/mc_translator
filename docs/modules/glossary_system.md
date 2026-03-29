# 術語系統

本系統用於提供翻譯提示，不直接替換原文。

## 資料來源

- 官方詞庫: `dicts/en_us.json` + `dicts/zh_tw.json`
- 推論詞庫: 由官方詞庫推論後寫入 `dicts/official/{ui_lang}.json`
- 使用者詞庫: `dicts/user/{ui_lang}.json`

## 流程圖

```mermaid
flowchart TD
    A["官方詞庫 en_us/zh_tw"] --> D["精確匹配表"]
    B["推論詞庫 official/{ui_lang}"] --> E["推論表"]
    C["使用者詞庫 user/{ui_lang}"] --> F["使用者表"]
    D --> G["合併與優先序"]
    E --> G
    F --> G
    G --> H["Glossary Automaton"]
    H --> I["批次翻譯提示"]
```

## 推論規則

- 只在 `en_us -> zh_tw` 時啟用
- 以詞頻與 CJK 字元比例推論常見中文片語
- 黑名單詞彙會被剔除

## 優先序

- `glossary_priority = official`: 官方優先
- `glossary_priority = user`: 使用者優先

## 匹配行為

- 使用 Aho-Corasick 進行多模式匹配
- 僅在字詞邊界匹配成功時生效
- 每批次最多使用 30 筆術語提示

## 相關檔案

- [src/translation/glossary/mc_lang.rs](/src/translation/glossary/mc_lang.rs)
- [src/translation/glossary/analyzer.rs](/src/translation/glossary/analyzer.rs)
- [src/translation/glossary/automaton.rs](/src/translation/glossary/automaton.rs)
- [src/config/dictionary.rs](/src/config/dictionary.rs)

## 相關連結

- [翻譯核心](translation_core.md)
- [翻譯記憶體](translation_memory.md)
- [UI 交互](../ui/interactions.md)
