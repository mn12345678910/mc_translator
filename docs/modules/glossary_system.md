# 術語系統

## 資料來源

- 官方詞庫：由 Minecraft 官方語言檔 (`dicts/en_us.json` 與 `dicts/zh_tw.json`) 對照產生
- 推論詞庫：由 `analyze_dictionary` 從官方詞庫推導並存到 `dicts/official.json`
- 使用者詞庫：`dicts/user.json`

## 優先級

- `glossary_priority` 決定同 key 的覆蓋順序
- `official` 優先時：官方詞庫先載入，再載入使用者詞庫
- `user` 優先時：使用者詞庫先載入，再載入官方詞庫

## Aho-Corasick 自動機

- 以 `LeftmostLongest` 匹配策略
- 大小寫不敏感
- 僅在英文字邊界符合時視為有效術語

## UI 行為摘要

- 官方分頁的編輯內容會轉存到使用者詞庫
- 字典管理器支援匯入、匯出、搜尋、批次取代

## 監控與刷新

- `dicts/` 目錄中的 `en_us.json` / `zh_cn.json` / `zh_tw.json` 變動會觸發重新推論
- 推論結果會覆寫 `dicts/official.json`

