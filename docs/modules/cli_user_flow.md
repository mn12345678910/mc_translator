# CLI 使用者操作流程 (User Flow)

本文件展現 `mc_translator_cli` 的互動式操作生命週期與導覽支援。

## 流程圖

```mermaid
graph TD
    Start([1. 啟動 mc_translator_cli]) --> Step1[2. 選擇介面語言 Select UI Language]
    Step1 --> Step2[3. 選擇 API 提供商 Provider]

    Step2 -- 按上一步 --> Step1
    Step2 --> CondKey{4. 是否需 API Key?}

    CondKey -- 是 (Gemini/OpenAI...) --> Step3[5. 輸入 API Key]
    CondKey -- 否 (Ollama/Google Free) --> Step4

    Step3 -- 按上一步 --> Step2
    Step3 --> Step4[6. 選擇模型 Model]

    Step4 -- 按上一步 --> Step3
    Step4 --> Step5[7. 輸入待翻譯檔案/資料夾路徑]

    Step5 -- 按上一步 --> Step4
    Step5 --> Step6[8. 確認輸出資料夾]

    Step6 -- 按上一步 --> Step5
    Step6 --> Step7[9. 確認開始 / 進階設定]

    Step7 -- 按上一步 --> Step6
    Step7 --> Run([10. 呼叫底層 Pipeline 執行翻譯])

    Run --> Finish{11. 翻譯完成，是否開啟新任務?}

    Finish -- 否 (離開) --> End([程式結束])
    Finish -- 是 (開啟新任務) --> LoopBack[12. 循環回歸]

    LoopBack --> Step5
```

> [!TIP]
> **導覽連動規則**：
> - 在各步驟按下 **上一步** 會穩定依照歷史歷程退至前一個實質節點（絕不重複來回）。
> - 開啟新任務時，會保留前置 Provider/Model 狀態，直接從 **Step 5 (輸入檔案)** 恢復詢問。若在此時退回，將順暢連接至 **Step 4 (模型)**。
