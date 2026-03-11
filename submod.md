## 完整模組重構規劃方案

---

### 最終目錄結構

```
mc_translator_rs/
├── src/
│   ├── lib.rs                    # 函式庫入口，統一匯出 API
│   ├── main.rs                   # 應用程式入口
│   ├── config/                   ← 配置模組
│   │   ├── mod.rs
│   │   ├── settings.rs           # 讀寫 config.cfg
│   │   └── encryption.rs         # Windows DPAPI 加密
│   ├── translation/              ← 翻譯核心模組
│   │   ├── mod.rs
│   │   ├── job.rs                # JobConfig, JobSharedState
│   │   ├── engine.rs             # 翻譯流程控制
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── client.rs          # LLM API 調用
│   │   │   └── models.rs          # 模型列表獲取
│   │   ├── batching.rs           # 批次處理邏輯
│   │   └── dictionary/
│   │       ├── mod.rs
│   │       ├── manager.rs         # 字典管理
│   │       └── cache.rs           # 推論快取
│   ├── file/                     ← 檔案處理模組
│   │   ├── mod.rs
│   │   ├── jar_handler.rs        # JAR 解壓/處理
│   │   ├── path_router.rs        # 路徑路由
│   │   └── pack_gen.rs           # 資源包 ZIP 生成
│   ├── ui/                       ← UI 模組
│   │   ├── mod.rs
│   │   ├── app.rs                # eframe::App impl
│   │   ├── theme.rs              # 主題與視覺風格
│   │   ├── components/
│   │   │   ├── mod.rs
│   │   │   ├── header.rs         # 頂部控制項
│   │   │   ├── settings.rs       # API 設定面板
│   │   │   ├── developer.rs      # 開發人員模式
│   │   │   ├── progress.rs       # 進度條
│   │   │   ├── actions.rs        # 操作按鈕
│   │   │   └── log.rs            # 日誌區域
│   │   ├── viewport/
│   │   │   ├── mod.rs
│   │   │   ├── memory_viewer.rs  # 建議詞管理器
│   │   │   ├── state.rs          # 同步狀態
│   │   │   └── dialogs.rs        # 對話框
│   │   └── widgets/
│   │       ├── mod.rs
│   │       └── toggle.rs         # 自訂 widget
│   └── utils/                    ← 工具模組
│       ├── mod.rs
│       ├── automaton.rs          # Aho-Corasick
│       └── helpers.rs             # 輔助函式
│
└── tests/                        ← 獨立測試目錄
    ├── fixtures/                  # 測試資料
    │   ├── sample_mod.jar
    │   ├── kubejs_script.js
    │   ├── en_us.json
    │   └── mock_api_response.json
    ├── translation/
    │   ├── batching_test.rs
    │   ├── dictionary_test.rs
    │   └── engine_test.rs
    ├── file/
    │   ├── path_router_test.rs
    │   └── pack_gen_test.rs
    ├── config/
    │   └── encryption_test.rs
    ├── utils/
    │   └── automaton_test.rs
    └── ui/
        ├── theme_test.rs
        └── components_test.rs
```

---

### 各模組職責對照表

| 模組 | 來源檔案 | 公開 API |
|------|----------|----------|
| **config** | | |
| `settings.rs` | config.rs | `Settings::load()`, `Settings::save()` |
| `encryption.rs` | config.rs | `encrypt()`, `decrypt()` |
| **translation** | | |
| `job.rs` | translation_job.rs | `JobConfig`, `JobSharedState` |
| `engine.rs` | data_processing.rs | `TranslationEngine::process()` |
| `api/client.rs` | translation_service.rs | `TranslationClient::translate()` |
| `api/models.rs` | translation_service.rs | `fetch_available_models()` |
| `batching.rs` | data_processing.rs | `BatchProcessor::split()` |
| `dictionary/manager.rs` | data_processing.rs | `DictionaryManager::query()` |
| `dictionary/cache.rs` | data_processing.rs | `InferredCache::get/set` |
| **file** | | |
| `jar_handler.rs` | file_handler.rs | `JarHandler::scan()`, `.extract()` |
| `path_router.rs` | file_handler.rs | `PathRouter::route()` |
| `pack_gen.rs` | file_handler.rs | `PackGenerator::generate()` |
| **ui** | | |
| `app.rs` | ui.rs | `AppState::update()` |
| `theme.rs` | ui.rs | `render_theme_application()` |
| `components/*` | ui.rs | 各 render_* 函式 |
| `viewport/*` | ui.rs | 建議詞管理器相關 |
| **utils** | | |
| `automaton.rs` | utils.rs | `Automaton::new()`, `.match_all()` |
| `helpers.rs` | utils.rs | 輔助函式集合 |

---

### 重構執行順序

1. **Phase 1**: 建立 `utils/` 和 `config/` 模組
2. **Phase 2**: 建立 `translation/` 模組（含 api/、dictionary/ 子模組）
3. **Phase 3**: 建立 `file/` 模組
4. **Phase 4**: 建立 `ui/` 模組（含 components/、viewport/、widgets/）
5. **Phase 5**: 更新 `lib.rs` API 匯出
6. **Phase 6**: 建立 `tests/` 目錄與測試

---

### lib.rs 向後相容設計

```rust
pub mod config;
pub mod translation;
pub mod file;
pub mod ui;
pub mod utils;

pub use config::{settings::Settings, encryption::{encrypt, decrypt}};
pub use translation::{
    engine::TranslationEngine,
    dictionary::{Dictionary, DictionaryManager},
    job::{JobConfig, JobSharedState},
    api::TranslationClient,
};
pub use file::{JarHandler, PackGenerator, PathRouter};
pub use utils::automaton::Automaton;
```

---

### 單元測試策略

- 使用 `#[cfg(test)]` 在各模組內部
- 獨立 `tests/` 目錄存放整合測試
- 使用 `mockall` 模擬 API 呼叫
- 測試資料存放於 `tests/fixtures/`

