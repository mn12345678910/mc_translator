<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# Git / GitHub 推送規範

本文件定義專案的推送規範、分支保護策略、發布流程與安全檢查清單。

## 推送規範（三層閘門）

所有推送到 `main` 分支的變更必須依序通過以下三層檢查：

```
本地修改
  │
  ├─ 閘門 1: Pre-commit Hooks（commit 時自動觸發，不可繞過）
  │   ├── cargo fmt          → Rust 格式化
  │   ├── cargo clippy       → Rust Linter（-D warnings）
  │   ├── pnpm format        → 前端格式化
  │   ├── 語法檢查           → YAML / TOML / JSON
  │   ├── 安全檢查           → 私鑰偵測、合併衝突、大檔案
  │   └── commit 訊息檢查    → 繁體中文格式驗證
  │
  ├─ 閘門 2: Pre-push Hooks（push 時自動觸發，不可繞過）
  │   ├── cargo test         → Rust 完整測試
  │   ├── pnpm test          → 前端完整測試（含覆蓋率）
  │   ├── pnpm lint          → 前端 Linter
  │   ├── 同步檢查           → Tauri API / UI 元素 / 設定檔
  │   ├── 版本檢查           → 版本一致性 / Tag 同步
  │   ├── 工作樹乾淨度       → 拒絕髒工作樹推送
  │   └── i18n 檢查          → 鍵值一致性 / 腳手架
  │
  └─ 閘門 3: GitHub Actions（push 到 remote 後自動觸發）
      ├── Lint & Security Audit  → fmt + clippy + audit + deny
      ├── Test (ubuntu)          → Rust nextest + Vitest + 覆蓋率
      ├── Test (windows)         → Rust nextest（分割為 2 分區）
      ├── E2E Tests              → Playwright 瀏覽器測試
      └── Build & Release        → 僅 tag 觸發
```

**任何一層失敗都必須修正後才能繼續。**

## 分支策略

| 分支        | 用途                          | 保護規則           |
| ----------- | ----------------------------- | ------------------ |
| `main`      | 穩定版本，每次推送自動觸發 CI | 需通過所有 CI 檢查 |
| `feature/*` | 新功能開發                    | 無特殊限制         |
| `fix/*`     | 錯誤修復                      | 無特殊限制         |
| `chore/*`   | 維護性變更（依賴更新、文檔）  | 無特殊限制         |

### main 分支保護規則

建議在 GitHub 設定以下分支保護（Repository → Settings → Branches）：

- [ ] **Require status checks to pass before merging**
    - `Lint & Security Audit`
    - `Test on ubuntu-latest`
    - `Test on windows-latest`
    - `E2E Tests (Playwright)`
- [ ] **Require branches to be up to date before merging**
- [ ] **Dismiss stale PR reviews**
- [ ] **Include administrators**
- [ ] **Restrict who can push to matching branches**

## Commit 規範

- **語言**：使用繁體中文描述
- **範圍**：一次 commit 只處理一組相關變更
- **格式**：由 `node tool/check_commit_msg.js` 自動檢查

### 常見 Commit 訊息範例

```
feat: 新增字典管理 UI
fix: 修正配置載入時的空值處理
ci: 新增 Playwright E2E 測試流程
chore: 更新依賴版本
test: 補齊前端測試覆蓋率至 84%
docs: 更新推送規範文檔
```

## 發布流程

1. 確認 `main` 分支通過所有 CI 檢查
2. 打 tag 並推送：

```bash
git tag -a v1.1.0 -m "發布 v1.1.0"
git push origin v1.1.0
```

3. CI 自動觸發 `Build & Release` job：
    - 在 Ubuntu 和 Windows 上編譯 Tauri 應用程式
    - 編譯 CLI 工具
    - 上傳 release assets 到 GitHub Releases

**注意**：tag 必須以 `v` 開頭（如 `v1.0.0`），否則不會觸發發布流程。

## 依賴更新合併流程

Dependabot 每週自動建立 PR 更新依賴。合併流程：

1. 等待 CI 全部通過（綠色）
2. 檢查變更是否為 patch/minor 版本（通常安全）
3. 使用 Squash Merge 合併
4. 如果 CI 失敗，先檢查是否為專案自身的 flaky test，再決定是否合併

### 快速合併所有安全的 Dependabot PR

```bash
# 檢查所有 open PR 的 CI 狀態
gh pr list --state open --json number,title,mergeStateStatus

# 合併狀態為 CLEAN 的 PR
gh pr merge <number> --squash --delete-branch
```

## 安全檢查清單

推送前確認以下事項：

- [ ] 沒有在程式碼中硬編碼 API Key 或密碼（使用 OS Keyring）
- [ ] `config.cfg`、`.env` 等敏感檔案已加入 `.gitignore`
- [ ] 沒有意外提交 `dicts/`、`settings/`、`langs/` 等生成目錄
- [ ] 依賴版本沒有已知漏洞（CI 的 `cargo audit` 會自動檢查）
- [ ] 新增的第三方依賴符合許可證要求（CI 的 `cargo-deny` 會自動檢查）

## 工具速查表

| 用途                | 命令                                                    |
| ------------------- | ------------------------------------------------------- |
| Rust 格式化         | `cargo fmt --all`                                       |
| Rust Lint           | `cargo clippy --workspace --all-targets -- -D warnings` |
| Rust 測試           | `cargo test --workspace`                                |
| Rust 覆蓋率（本地） | `cargo llvm-cov --workspace`                            |
| 前端格式化          | `pnpm format`                                           |
| 前端 Lint           | `pnpm lint`                                             |
| 前端測試            | `pnpm test`                                             |
| 前端覆蓋率          | `pnpm vitest run --coverage`                            |
| E2E 測試            | `pnpm test:e2e`                                         |
| 全部測試            | `pnpm test:all`                                         |
| Tauri 同步檢查      | `node tool/check_tauri_sync.js`                         |
| 版本同步檢查        | `node tool/check_all_versions.js`                       |
| Tag 同步檢查        | `node tool/check_tag_sync.js`                           |
| 設定同步檢查        | `node tool/check_config_sync.js`                        |
| Markdown 連結檢查   | `node tool/check_md_links.js`                           |
| Commit 訊息檢查     | `node tool/check_commit_msg.js`                         |
