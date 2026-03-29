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
3. 針對 Rust 錯誤，檢查 `cargo clippy` 的具體建議或 `cargo test` 的斷言失敗訊息。
4. 針對前端錯誤，檢查 `pnpm lint` 或 `vitest` 的輸出結果。

# 3. 本地重驗
5. 修正完畢後，必須依序執行：
   - `pre-commit run --all-files` (確保滿足基礎格式與安全掛鉤)
   - `pre-push` 工作流 (執行完整的全域驗證)

# 4. 再次推送
6. 提交變更並再次進行 `git push`（請確保 Commit Message 為繁體中文）。
