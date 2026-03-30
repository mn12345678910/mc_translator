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

## 2.2 本地重驗
修正程式碼後，必須確保本地通過所有自動化檢查：
- 執行 `pre-commit run --all-files --hook-stage pre-push`。
- 確保格式與測試皆為綠燈。

# 3. 循環與再次推送
6. 提交變更並再次進行 `git push`（確保 Commit Message 為繁體中文）。
7. 推送完成後，**回到「步驟 1. 監控 CI 狀態」**，循環執行直至成功。
