# 檔案處理流水線 (File Pipeline)

## 1. 檔案掃描與處理流程

本模組負責專案中所有檔案的識別、解壓與寫入。

### 掃描與收集階段 (Scanning & Collection)
- **循序收集**: 針對輸入路徑進行循序迭代，並依檔案類型 (`.jar`, `.json`, `.js`) 發配至專門處理器。
- **窗口批次化**: 將搜集到的項目統一分配至物理路徑窗口，提供 LLM 進行「跨檔案跨模組」滾動式批次翻譯。
- **路徑規範化**：引入 `canonicalize()` 處理，確保 Windows 平台下 `rel_path` 生成的穩定性。

## 2. JAR 處理流程圖

```mermaid
graph TD
    Entry[JAR 檔案進入] --> Extract[解壓 assets 資料夾]
    Extract --> ScanLang[搜尋 en_us.json]
    ScanLang --> Found{找到?}
    Found -- Yes --> CreateZH[建立 zh_tw.json 模板]
    CreateZH --> Translate[進入翻譯隊列]
    Found -- No --> ScanBook[搜尋 Patchouli Books]
    ScanBook --> BookFound{找到?}
    BookFound -- Yes --> ProcessBook[處理說明書翻譯]
    BookFound -- No --> Skip[跳過該 JAR]
```

## 3. 資源包封裝 (Packaging)
- **Temp Partitioning**: 翻譯結果暫存於系統臨時目錄。
- **路徑安全檢查**：寫入前強制執行相對路徑驗證，防止「路徑溢出」安全風險。
- **匯出 ZIP**: 將所有譯後檔案重新封裝為符合 Minecraft 規格的 `LLMTranslator.zip` 資源包。
