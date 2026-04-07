# 安全政策

## 支援版本

| 版本         | 支援狀態    |
| ------------ | ----------- |
| 最新發布版本 | ✅ 安全更新 |
| 前一版本     | ✅ 安全更新 |
| 更早版本     | ❌ 不再支援 |

## 漏洞報告

如果您發現安全漏洞，請**不要**在 GitHub Issues 中公開報告。

**報告方式：**

1. 透過 GitHub 的 [Private Vulnerability Reporting](https://github.com/mn12345678910/mc_translator/security/advisories/new) 提交
2. 或發送電子郵件至專案維護者
3. 請包含以下資訊：
    - 漏洞描述
    - 重現步驟
    - 影響範圍
    - 建議修復方式（如有）

**回應時間：** 我們會在 48 小時內確認報告，並在 7 天內提供修復計畫。

## 安全掃描工具

本專案在 CI 中整合以下自動化安全掃描：

| 工具                                                                 | 用途                                            | 觸發時機            |
| -------------------------------------------------------------------- | ----------------------------------------------- | ------------------- |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)            | Rust 依賴許可證檢查、重複版本偵測、已知漏洞掃描 | 每次 push/PR        |
| [rustsec/audit-check](https://github.com/RustSec/rustsec)            | Rust 依賴安全公告檢查（RUSTSEC）                | 每次 push/PR        |
| [detect-private-key](https://github.com/pre-commit/pre-commit-hooks) | 偵測意外提交的私鑰檔案                          | 每次 commit         |
| [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)          | Rust 程式碼覆蓋率報告                           | 每次 push（Ubuntu） |
| [Vitest v8](https://vitest.dev/guide/coverage.html)                  | JavaScript 程式碼覆蓋率報告                     | 每次 push           |
| [Codecov](https://codecov.io/)                                       | 覆蓋率追蹤與 PR 評論                            | 每次 push/PR        |
| [Dependabot](https://github.com/dependabot)                          | 自動化依賴版本更新（cargo/npm/github-actions）  | 每週                |

## 敏感資料處理政策

### 絕不提交的檔案

以下檔案類型的內容**絕對不可**提交到版本控制：

- API Key、密碼、Token
- `config.cfg`、`.env` 等配置檔案
- `dicts/`、`settings/`、`langs/` 等生成目錄
- 任何包含個人資訊的檔案

這些檔案已在 `.gitignore` 中排除。

### API Key 處理

- 使用者的 API Key 透過 **OS Keyring**（作業系統金鑰庫）儲存
- Keyring 內容不會被序列化或上傳
- 開發環境中可使用 mock 值替代

### 第三方依賴政策

- 所有 Rust 依賴必須使用以下許可證之一：MIT、Apache-2.0、BSD-3-Clause、ISC、CC0-1.0、Zlib、bzip2-1.0.6、Unicode-3.0、CDLA-Permissive-2.0
- 所有 npm 依賴由 Dependabot 每週自動檢查更新
- GitHub Actions 版本由 Dependabot 每週自動更新
- 已知漏洞由 `cargo-deny` 和 `rustsec` 自動偵測

## 分支保護

`main` 分支受以下規則保護：

- 必須通過所有 CI 檢查才能合併
- 禁止直接推送（必須透過 PR）
- 禁止 force-push
- 建議要求 PR 審核

## 發布安全

- 發布版本使用 `v*` tag 觸發自動化 Build & Release
- Release assets 由 CI 自動編譯並上傳，確保二進位檔來源可追溯
- 安全相關的修復應優先合併並盡快發布
