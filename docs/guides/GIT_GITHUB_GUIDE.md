<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

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

- Pre-commit / pre-push 會自動執行對應 hooks
- Rust 格式化: `cargo fmt --all`
- Rust Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Rust 測試: `cargo test --all-features` (pre-push)
- Frontend 格式化: `pnpm format`
- Frontend Lint: `pnpm lint`
- Frontend 測試: `pnpm test`
- Tauri 同步檢查: `node tool/check_tauri_sync.js`
- 版本同步檢查: `node tool/check_all_versions.js`
- Tag 同步檢查: `node tool/check_tag_sync.js`
- 設定同步檢查: `node tool/check_config_sync.js`
- Markdown 連結檢查: `node tool/check_md_links.js`
- Commit 訊息檢查: `node tool/check_commit_msg.js`
