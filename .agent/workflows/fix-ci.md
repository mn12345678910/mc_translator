---
description: CI 失敗時的自動化檢查與修正流程
---

當 GitHub Actions 失敗時，執行此流程進行診斷與修復。

# 1. 日誌診斷
// turbo
1. 下載並分析最近一次失敗的執行日誌：
```pwsh
$RUN_ID = (gh run list --limit 1 --status failure --json databaseId -q ".[0].databaseId")
gh run view $RUN_ID --log > ci_failure.log
```

# 2. 錯誤定位
2. 搜尋日誌中的 `error:` 或 `warning:` 關鍵字，找出具體失敗的行數與檔案。
3. 針對問題點進行修正（如：修復代碼、更新依賴或調整 CI 配置）。

# 3. 本地重驗
4. 修正完畢後，必須重新執行 `pre-push` 工作流確保本地 0 問題。

# 4. 再次推送
5. 提交變更並再次嘗試 `git push`。
