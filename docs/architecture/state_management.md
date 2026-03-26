# 狀態管理

## AppState 結構

`AppState` 是 UI 與背景任務的核心狀態容器，保存翻譯流程、UI 控制、設定與字典管理器狀態。

**主要狀態來源**
- 原子狀態：`is_processing`、`is_paused`、`is_cancelled`、進度數值
- 互斥狀態：日誌、狀態文字、目前處理路徑
- 設定鏡像：API 參數、批次參數、主題、視窗位置、以及 `ui_lang` (介面語系)

## 任務狀態流程

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Scanning: 開始翻譯
    Scanning --> Running: 建立批次完成
    Running --> Paused: 使用者暫停
    Paused --> Running: 使用者繼續
    Running --> Finished: 全部完成
    Running --> Cancelled: 使用者停止
    Paused --> Cancelled: 使用者停止
    Finished --> Idle
    Cancelled --> Idle
```

## 執行緒同步與資料流模型

以下說明「Tauri 前端 (HTML/JS)」與「後端背景任務執行緒」如何透過 **IPC (Invoke)** 與 **事件 (Events)** 安全地同步狀態。

```mermaid
graph TD
    subgraph UI_Frontend ["Tauri 前端 (HTML/JS)"]
        A[JS Render Loop] --> B{觸發 Invoke}
        B --> C["發送 start_translation, save_config"]
        D[監聽 Emitter] <-- "接收 translation-log / progress" --- E[Event Listener]
    end

    subgraph Backend_Thread ["背景任務執行緒 (Tauri/Tokio)"]
        F[Tauri Command] --> G[Pipeline 執行範疇]
        G --> H["使用 Arc<Mutex> / Atomic 控制共用狀態 (JobSharedState)"]
        H --> I["發送 app.emit() 即時更新給前端"]
        G --> J["監聽 取消/暫停 信號"]
    end

    C -- "tauri::invoke" --> F
    I -.-> D

    style UI_Frontend fill:#e6f3ff,stroke:#333,stroke-width:1px
    style Backend_Thread fill:#fff2e6,stroke:#333,stroke-width:1px
```


## 同步與併發

- `AtomicBool/AtomicU32` 供 UI 即時讀取。
- `Arc<Mutex<...>>` 用於跨執行緒共享字串與日誌。
- 任務透過 `runtime.spawn` 執行，避免阻塞 UI。
- I/O 與重度工作使用 `spawn_blocking` 包裹。

## 持久化策略

- `settings/config.cfg` 儲存主要設定。
- 通過系統憑證鏈 (Keyring) 儲存 `API_KEY`。
- `settings/style.cfg` 儲存外觀樣式設定。
- 主視窗與字典管理器視窗會在幾何變動時即時同步並儲存。
- `on_exit` 會觸發保存，避免異常關閉時遺失設定。

## 模型與啟動檢查

- 除 `Google Free` 外，未選模型會阻止啟動並寫入日誌。
- 服務商切換時會清空目前模型並重新載入清單。
