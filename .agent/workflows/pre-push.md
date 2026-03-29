---
description: 推送前自我審查流程，確保 0 警告、0 錯誤且無冗餘檔案
---

此工作流用於在 git push 之前進行全面的本地驗證。

# 1. 清除冗餘檔案
// turbo
1. 執行以下命令移除已知冗餘檔案：
```pwsh
Remove-Item -Path "repro_serde.rs", "ci_log.txt", "job_log.txt", "ci_failure.log" -ErrorAction SilentlyContinue
Remove-Item -Path "LLMTranslator" -Recurse -Force -ErrorAction SilentlyContinue
```

# 2. 品質與安全檢查
// turbo
2. 執行格式檢查、靜態分析與安全性審核：
```pwsh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check
```

# 3. 功能測試
// turbo
3. 執行各模組測試：
```pwsh
cargo test --all-features
pnpm lint
pnpm test
```

# 4. 提交規範確認 (git-cliff)
> [!TIP]
> **提交訊息規範建議**：
> - **語言**：請使用「繁體中文」撰寫標題與內容。
> - **格式**：`<type>(<scope>): <subject>`
> - **常用類型 (Type)**：
>   - `feat`: 新功能 | `fix`: 修復錯誤 | `doc`: 文檔更新
>   - `refactor`: 重構 | `perf`: 效能優化 | `style`: 樣式調整
>   - `test`: 測試 | `ci`: CI/CD | `chore`: 雜務 (如版本發布)

4. 執行變更日誌預覽，確保符合預期：
```pwsh
git-cliff --latest
```

# 5. 完成確認
5. 如果以上所有步驟皆通過，即可執行 `git push`。
