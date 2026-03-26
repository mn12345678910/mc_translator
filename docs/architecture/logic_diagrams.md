# 系統邏輯判斷 Mermaid 流程圖

以下根據系統文件（`translation_core.md`、`file_pipeline.md`、`error_handling.md`）整理出的核心邏輯判斷圖。

---

## 1. 翻譯主流程 (Main Translation Flow)

此流程涵蓋了從檔案掃描到最終打包的完整生命週期。

```mermaid
graph TD
    Start([開始]) --> Scan[掃描與收集檔案]
    Scan --> Group[檔案分類與分組排序]
    Group --> CreateBatch[建立跨檔案窗口批次]
    CreateBatch --> Preprocess[文本預處理與格式保護]
    Preprocess --> Deduplicate[批次內去重 & 相對標籤優化]
    Deduplicate --> CallAPI[呼叫翻譯服務]
    CallAPI --> Clean[清理、格式同步與還原]
    Clean --> DetectLoop{偵測到翻譯循環?}
    DetectLoop -- 是 --> SkipItem[跳過 / 記錄警告]
    DetectLoop -- 否 --> ApplyDeduplication[同步套用去重譯文]
    SkipItem --> ApplyDeduplication
    ApplyDeduplication --> WriteTemp[寫入暫存資源包]
    WriteTemp --> CheckFinished{所有檔案處理完成?}
    CheckFinished -- 否 --> CreateBatch
    CheckFinished -- 是 --> Package[打包產出 LLMTranslator.zip]
    Package --> End([結束])
```

---

## 2. 批次降級與重試邏輯 (Batch Fallback & Retry)

當 API 呼叫失敗時，系統會自動縮減批次大小以提高成功率。

```mermaid
graph TD
    Start([呼叫翻譯批次]) --> TryFull[嘗試原始批次翻譯]
    TryFull --> CheckFull{成功?}

    CheckFull -- 成功 --> End([完成])

    CheckFull -- 失敗 --> FallbackHalf["1. 降級為「半批次」<br/>降低字元上限"]
    FallbackHalf --> TryHalf[嘗試半批次翻譯]
    TryHalf --> CheckHalf{成功?}

    CheckHalf -- 成功 --> End

    CheckHalf -- 失敗 --> FallbackSingle["2. 降級為「單筆翻譯」"]
    FallbackSingle --> TrySingle[嘗試單筆翻譯]
    TrySingle --> CheckSingle{成功?}

    CheckSingle -- 成功 --> End

    CheckSingle -- 失敗 --> LogError["3. 記錄日誌 / 標記失敗"] --> End
```

---

## 3. 批次內去重邏輯 (Intra-batch Deduplication)

在單一批次中，減少對 API 的重複請求並同步結果。

```mermaid
graph TD
    Start([準備批次數據]) --> Loop1[發送端：遍歷批次內條目]
    Loop1 --> ExistCheck{原文在此批次中已出現?}

    ExistCheck -- 是 --> Merged["僅標記為重複<br/>不另配相對標籤"]
    ExistCheck -- 否 --> NewTag["分配批次內相對標籤<br/>e.g. i0, i1"]

    Merged --> Loop1
    NewTag --> Loop1

    Loop1 -- 遍歷結束 --> Send[發送 API 請求]
    Send --> Receive[接收翻譯結果]

    Receive --> Loop2[接收端：遍歷批次內條目]
    Loop2 --> FindMatch[根據原文匹配譯文]
    FindMatch --> Apply[一併同步套用譯文]

    Apply --> Loop2
    Loop2 -- 遍歷結束 --> End([去重套用完成])
```

---

## 4. 檔案分組決策 (File Grouping Decision)

決定翻譯項目如何分組以建立批次窗口。

```mermaid
graph TD
    Start([獲取掃描到的檔案]) --> CheckJar{檔案類型是否為 .jar?}

    CheckJar -- "是 (JAR 檔案)" --> GroupJar["以「檔案本身」為分組鍵"]
    CheckJar -- "否 (JSON / JS)" --> GroupFolder["以「父資料夾」為分組鍵"]

    GroupJar --> Reorder[依 File Task 順序重新排序 global_items]
    GroupFolder --> Reorder

    Reorder --> End([分組排序完成])
```

---

## 5. 啟動前模型檢查 (Startup Validation)

驗證配置是否允許啟動系統。

```mermaid
graph TD
    Start([檢查啟動配置]) --> CheckGoogle{所選服務商為 Google Free?}

    CheckGoogle -- 是 --> Allow[允許啟動]

    CheckGoogle -- 否 --> CheckModel{已選擇/配置特定模型?}
    CheckModel -- 是 --> Allow
    CheckModel -- 否 --> Block[阻止啟動並寫入錯誤日誌]

    Allow --> End([進入運行狀態])
    Block --> End
```

---

## 6. 路徑安全處理 (Path Security)

避免路徑跳脫或系統差異導致的錯誤。

```mermaid
graph TD
    Start([處理檔案路徑]) --> Phase1[1. 掃描階段]
    Phase1 --> NormPath[路徑規範化 / 統一斜線]
    NormPath --> FixDrive[處理解決 Windows 磁碟機大小寫差異]

    FixDrive --> Phase2[2. 輸出階段]
    Phase2 --> StripSlash[移除前導斜線]
    StripSlash --> StripDrive[移除磁碟機前綴]

    StripDrive --> End([路徑處理完成])
```
