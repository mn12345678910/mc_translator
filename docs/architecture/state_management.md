# 狀態管理

本專案以 `JobSharedState` 作為翻譯流程的共享狀態容器，並由 GUI/CLI 共同使用。

## JobStatus

- `Idle`: 待機
- `Running`: 翻譯進行中
- `Paused`: 已暫停

## 共享狀態組成

- 進度: `progress`, `progress_total`, `global_progress`, `global_total`
- 控制: `cancelled`, `paused`, `pause_notifier`
- 當前狀態: `current_state`, `current_processing_path`
- 日誌: `log`
- 翻譯記憶體: `translation_memory` (執行期用)
- 設定快照: `config`
- i18n: `CommonLabels`

## GUI 狀態同步

GUI 透過 Tauri 事件與 command 呼叫同步狀態。

完整事件列表與說明請參考: [docs/ui/interactions.md#事件與狀態同步](/docs/ui/interactions.md#事件與狀態同步)

- `job-state-changed`: `Idle` / `Running` / `Paused`
- `translation-progress`: 進度條與狀態訊息
- `translation-batch-update`: 批次進度
- `translation-log`: 日誌事件
- `translation-finished`: 完成/失敗通知

## 暫停與繼續

- `pause_translation`: 將 `paused` 設為 true 並送出警告日誌
- `resume_translation`: 更新 config 快照後解除暫停
- 暫停中允許修改設定，下一批次會生效

## 停止

- `stop_translation`: 將 `cancelled` 設為 true
- pipeline 會在安全點停止並回到 `Idle`

## CLI 狀態呈現

CLI 以標準輸出顯示:

- 進度條與百分比
- 批次條目數
- 狀態文字

CLI 不使用 GUI 事件機制，而是直接在 `start_translation_workflow` 注入回呼。
