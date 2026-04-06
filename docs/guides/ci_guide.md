<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# CI 指南

本指南說明 CI 流程、Pre-commit/Pre-push Hooks 以及除錯方法。

## CI 流程概述

CI 由 GitHub Actions 觸發，包含以下階段：

```
Push → Pre-commit Hooks → Pre-push Hooks → GitHub Actions
                                              ├── Lint & Security Audit
                                              ├── Test (ubuntu-latest + windows-latest)
                                              ├── Miri Analysis (每週)
                                              └── Build & Release (tag 觸發)
```

## Pre-commit Hooks

每次 commit 時執行：

| Hook                      | 說明                         |
| ------------------------- | ---------------------------- |
| `cargo fmt`               | Rust 程式碼格式化            |
| `cargo clippy`            | Rust Linter（`-D warnings`） |
| `pnpm format`             | 前端程式碼格式化             |
| `trailing-whitespace`     | 移除行尾空白                 |
| `end-of-file-fixer`       | 確保檔案以換行結尾           |
| `check-yaml`              | YAML 語法檢查                |
| `check-toml`              | TOML 語法檢查                |
| `check-json`              | JSON 語法檢查                |
| `check-merge-conflict`    | 檢查合併衝突標記             |
| `detect-private-key`      | 偵測私鑰                     |
| `check-added-large-files` | 檢查大檔案                   |
| `check commit message`    | Commit 訊息格式檢查          |

## Pre-push Hooks

每次 push 時執行：

| Hook                   | 說明               |
| ---------------------- | ------------------ |
| `cargo test`           | Rust 所有測試      |
| `pnpm test`            | 前端所有測試       |
| `check tauri api`      | Tauri API 橋接檢查 |
| `check ui elements`    | UI 元素同步檢查    |
| `pnpm lint`            | 前端 Linter        |
| `check config sync`    | 設定檔同步檢查     |
| `check md links`       | Markdown 連結檢查  |
| `cleanup assets`       | 清理冗餘檔案       |
| `check dirty worktree` | 檢查工作樹乾淨度   |
| `check tag sync`       | Tag 版本同步檢查   |
| `check all versions`   | 版本一致性檢查     |

## 如何除錯 CI 失敗

### 查看 CI 日誌

```bash
# 列出最近的 CI 執行
gh run list --limit 3

# 查看特定執行的失敗日誌
gh run view <run-id> --log-failed

# 篩選錯誤訊息
gh run view <run-id> --log-failed | grep -E "error\[|FAILED|panic"
```

### 常見失敗原因

| 錯誤                     | 原因                               | 解決方法                                    |
| ------------------------ | ---------------------------------- | ------------------------------------------- |
| `cargo clippy` 失敗      | 有 lint 警告/錯誤                  | 執行 `cargo clippy -- -D warnings` 本地檢查 |
| `cargo test` 失敗        | 測試失敗                           | 執行 `cargo test` 本地檢查                  |
| `pnpm test` 失敗         | 前端測試失敗                       | 執行 `pnpm test` 本地檢查                   |
| `check yaml` 失敗        | YAML 語法錯誤                      | 檢查 `.github/workflows/ci.yml` 縮排        |
| `sync version` 失敗      | 版本不一致                         | 執行 `node tool/sync_version.js`            |
| `check config sync` 失敗 | config.rs 與 config.test.js 不同步 | 執行 `node tool/check_config_sync.js`       |

## Node.js 版本注意事項

CI 已設定 `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"` 環境變數，強制使用 Node.js 24 執行 JavaScript Actions。

**受影響的 Actions：**

- `actions/setup-node@v4`
- `mozilla-actions/sccache-action@v0.0.9`
- `pnpm/action-setup@v4`

## Windows 測試平行化

Windows 測試已分割為 2 個平行分區以減少執行時間：

```yaml
- name: Run Rust Tests (Nextest, Windows partition 1/2)
  if: matrix.os == 'windows-latest'
  run: cargo nextest run --workspace --all-targets --partition count:1/2

- name: Run Rust Tests (Nextest, Windows partition 2/2)
  if: matrix.os == 'windows-latest'
  run: cargo nextest run --workspace --all-targets --partition count:2/2
```

## Sccache 命中率

CI 使用 Sccache 加速編譯。命中率顯示在 CI 日誌中：

```
100% - 810 hits, 1 misses, 0 errors
```

高命中率表示快取運作正常。

## 本地模擬 CI

在 push 前執行以下命令模擬 CI 檢查：

```bash
# Pre-commit 檢查
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
pnpm format

# Pre-push 檢查
cargo test --workspace --all-targets
pnpm test
pnpm lint
node tool/check_config_sync.js
node tool/check_all_versions.js
```
