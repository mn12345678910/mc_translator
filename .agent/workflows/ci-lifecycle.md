---
description: CI 生命週期管理（監控與修復循環）
---

此工作流用於在 `git push` 後追蹤 CI 狀態，並在失敗時引導進入診斷與修復流程。

# 1. 監控 CI 狀態 (Watch)

當您執行完 `git push` 後，應立即開始追蹤最新的 Action 執行進度。

// turbo

1. 執行以下命令觀察最新的 Action Run：

```pwsh
gh run watch
```

2. 觀察終端輸出直到流程結束。
3. **成功**：如果所有工作 (Jobs) 顯示綠色勾號，則發布流程成功完成。
4. **失敗**：如果出現紅色交叉，請進入下方的「修復流程」。

## CI Jobs 一覽

| Job                      | 說明                           | 依賴                      |
| ------------------------ | ------------------------------ | ------------------------- |
| `Lint & Security Audit`  | 格式、Linter、安全掃描         | 無                        |
| `Test on ubuntu-latest`  | Rust nextest + Vitest + 覆蓋率 | Lint                      |
| `Test on windows-latest` | Rust nextest（2 分區）+ Vitest | Lint                      |
| `E2E Tests (Playwright)` | 瀏覽器端對端測試               | 無（獨立執行）            |
| `Miri Analysis`          | unsafe 記憶體分析              | 無（每週/手動）           |
| `Build & Release`        | Tauri 編譯與發布               | Test + E2E（僅 tag 觸發） |

# 2. CI 失敗修復流程 (Fix)

當 CI 出現失敗時，執行以下步驟進行診斷與本地重驗。

## 2.1 日誌診斷

// turbo

1. 下載並分析最近一次失敗的執行日誌：

```pwsh
$RUN_ID = (gh run list --limit 1 --status failure --json databaseId -q ".[0].databaseId")
gh run view $RUN_ID --log > ci_failure.log
```

2. 搜尋日誌中的 `error:` 或 `warning:` 關鍵字，找出具體失敗處。

## 2.2 常見失敗類型與處理

| 失敗 Job                 | 常見原因                               | 處理方式                                              |
| ------------------------ | -------------------------------------- | ----------------------------------------------------- |
| `Lint & Security Audit`  | clippy 警告、格式錯誤、依賴漏洞        | 本地執行 `cargo clippy -- -D warnings` 或 `cargo fmt` |
| `Test on ubuntu/windows` | Rust 測試失敗、Vitest 失敗、覆蓋率不足 | 本地執行 `cargo test` 或 `pnpm test`                  |
| `E2E Tests (Playwright)` | DOM 結構變更、Mock 不匹配              | 本地執行 `pnpm test:e2e` 檢查實際行為                 |
| `Build & Release`        | 編譯錯誤、Tauri 相依性問題             | 本地執行 `cargo tauri build`                          |

## 2.3 覆蓋率相關失敗

| 錯誤                           | 原因                 | 處理方式                                       |
| ------------------------------ | -------------------- | ---------------------------------------------- |
| Vitest coverage threshold 失敗 | 覆蓋率低於門檻       | 執行 `pnpm vitest run --coverage` 查看未覆蓋行 |
| cargo-llvm-cov 失敗            | Rust 測試覆蓋率問題  | 執行 `cargo llvm-cov --workspace` 查看報告     |
| Codecov 上傳失敗               | CODECOV_TOKEN 未設定 | 在 GitHub Settings → Secrets 中設定            |

## 2.4 本地重驗

修正程式碼後，必須確保本地通過所有自動化檢查：

- 執行 `pre-commit run --all-files --hook-stage pre-push`。
- 確保格式與測試皆為綠燈。

# 3. 循環與再次推送

1. 提交變更並再次進行 `git push`（確保 Commit Message 為繁體中文）。
2. 推送完成後，**回到「步驟 1. 監控 CI 狀態」**，循環執行直至成功。
