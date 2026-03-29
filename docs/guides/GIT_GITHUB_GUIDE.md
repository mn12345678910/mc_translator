# Git / GitHub 指南

本文件提供專案內部的協作約定，請依團隊流程調整。

## 分支命名

- 建議使用 `feature/`、`fix/`、`chore/` 前綴
- 例: `feature/dictionary-ui`, `fix/zip-output`

## Commit 規範

- 建議使用繁體中文描述
- 一次 commit 只處理一組相關變更

## PR 建議

- 標題清楚描述變更目的
- 提供重現步驟或驗證方式
- 明確標註破壞性變更

## 工具與檢查

- 前端格式化: `pnpm format`
- 前端 lint: `pnpm lint`
- 前端測試: `pnpm test`
- Tauri 同步檢查: `pnpm check-tauri`
