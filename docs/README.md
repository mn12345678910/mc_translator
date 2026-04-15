<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# 文檔索引

本專案的完整技術文檔。

## 📐 架構文檔

| 文檔                                         | 說明                                |
| -------------------------------------------- | ----------------------------------- |
| [系統架構概覽](architecture/overview.md)     | 核心模組、執行流程、主要資料結構    |
| [狀態管理](architecture/state_management.md) | JobConfig、JobSharedState、進度控制 |
| [錯誤處理](architecture/error_handling.md)   | 錯誤分級、批次降級、日誌紀錄        |
| [邏輯流程圖](architecture/logic_diagrams.md) | 視覺化流程圖                        |

## 📦 模組文檔

| 文檔                                              | 說明                                         |
| ------------------------------------------------- | -------------------------------------------- |
| [翻譯核心](modules/translation_core.md)           | 翻譯管線、API 呼叫、引擎                     |
| [檔案流水線](modules/file_pipeline.md)            | 檔案掃描、JAR/JSON/JS 處理、資源包輸出       |
| [術語系統](modules/glossary_system.md)            | 官方詞庫、推論詞庫、使用者詞庫、Aho-Corasick |
| [翻譯記憶體](modules/translation_memory.md)       | 執行期記憶體結構                             |
| [增量比對策略](modules/translation_comparison.md) | 翻譯結果比對與跳過邏輯                       |
| [CLI 運作流程](modules/cli_user_flow.md)          | Headless 模式、互動式導覽                    |
| [設定系統](modules/config_system.md)              | AppConfig、StyleConfig、讀寫流程             |
| [工具模組](modules/utils.md)                      | 跳過規則、文字前後處理、日誌工具             |

## 🎨 UI 文檔

| 文檔                          | 說明               |
| ----------------------------- | ------------------ |
| [UI 規格](ui/specs.md)        | 介面元件、佈局     |
| [UI 交互](ui/interactions.md) | 事件處理、狀態同步 |

## 📖 指南

| 文檔                                             | 說明                          |
| ------------------------------------------------ | ----------------------------- |
| [測試策略](guides/testing_strategy.md)           | 測試架構、覆蓋率目標          |
| [變數命名](guides/variable_naming.md)            | 命名規範                      |
| [CI 產物與 Release](guides/release_artifacts.md) | 發行流程、產物列表            |
| [Git 指南](guides/GIT_GITHUB_GUIDE.md)           | 分支策略、Commit 規範         |
| [新增翻譯提供商](guides/adding_new_provider.md)  | 如何新增 API Provider         |
| [測試指南](guides/testing_guide.md)              | 如何編寫測試、除錯 CI         |
| [前端開發指南](guides/frontend_development.md)   | 前端架構、Mock 工具、樣式系統 |
| [CI 指南](guides/ci_guide.md)                    | CI 流程、Hooks、除錯指南      |
| [依賴告警處理](guides/dependency_advisories.md)  | RustSec 告警分類與修復策略    |

## 📜 歷史文檔

歷史文檔已歸檔，如需查閱舊版（egui 時期）設計，請查看 git 歷史記錄。
