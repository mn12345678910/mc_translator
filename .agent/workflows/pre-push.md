---
description: 推送前自我審查流程，確保 0 警告、0 錯誤且無冗餘檔案
---

此工作流用於在 git push 之前進行全面的本地驗證。

# 1. 清除冗餘檔案
// turbo
1. 執行以下命令移除已知冗餘檔案：
```pwsh
Remove-Item -Path "repro_serde.rs", "ci_log.txt", "job_log.txt" -ErrorAction SilentlyContinue
Remove-Item -Path "LLMTranslator" -Recurse -Force -ErrorAction SilentlyContinue
```

# 2. Rust 代碼品質檢查
// turbo
2. 執行格式檢查與靜態分析：
```pwsh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

# 3. 安全性與授權審核
// turbo
3. 檢查依賴項安全性與授權合規性：
```pwsh
cargo audit
cargo deny check
```

# 4. 全面功能測試
// turbo
4. 執行各模組測試，確保無功能迴歸：
```pwsh
cargo test --all-features
pnpm lint
pnpm test
```

# 5. 完成確認
5. 如果以上所有步驟皆通過，即可執行 `git push`。
