---
description: 推送後持續觀察 GitHub Actions 執行狀態
---

此工作流用於在推送代碼後自動追蹤 CI 進度。

# 1. 開始追蹤
// turbo
1. 執行以下命令觀察最新的 Action Run：
```pwsh
gh run watch
```

# 2. 結果判定
2. 觀察終端輸出直到流程結束。
3. 如果所有工作 (Jobs) 顯示綠色勾號，則發布流程成功完成。
4. 如果出現紅色交叉，請立即執行 `fix-ci` 工作流。
