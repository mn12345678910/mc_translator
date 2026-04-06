<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# 新增翻譯提供商

本指南說明如何在本專案中新增一個 API Provider。

## 修改點總覽

新增 Provider 需要修改 **7 個位置**：

| #   | 檔案                            | 行   | 用途                                              |
| --- | ------------------------------- | ---- | ------------------------------------------------- |
| 1   | `src/translation/api/client.rs` | ~137 | `translate_one` 單筆翻譯路由                      |
| 2   | `src/translation/api/client.rs` | ~214 | `translate_batch` 批次翻譯路由                    |
| 3   | `src/translation/api/client.rs` | ~566 | OpenAI 相容端點 URL 建構（如適用）                |
| 4   | `src-tauri/src/commands.rs`     | ~22  | `get_models_from_provider` 動態模型拉取（如適用） |
| 5   | `frontend/modules/config.js`    | ~270 | 前端 Provider 下拉選單                            |
| 6   | `src/cli/main.rs`               | ~240 | CLI 互動選單 Provider 選項                        |
| 7   | `i18n 資產`                     | -    | 新增 Provider 的 i18n 標籤                        |

## 步驟 1: 新增翻譯函數（如需要）

如果新 Provider 使用現有的 API 協定（如 OpenAI 相容），可以直接複用現有函數：

```rust
// 在 translate_one 的 match 中加入
"MyProvider" => {
    if config.api_key.expose_secret().is_empty() {
        return Err("API_KEY_REQUIRED:MyProvider 需要 API Key，請在設定中提供".into());
    }
    translate_with_openai_compatible(text, config, file_name, glossary).await
}
```

如果需要全新的 API 協定，需要建立新的翻譯函數：

```rust
async fn translate_with_myprovider(
    text: &str,
    config: &JobConfig,
    file_name: &str,
    glossary: Option<&[GlossaryEntry]>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 實作 API 呼叫邏輯
}
```

## 步驟 2: 更新 translate_one（行 ~137）

```rust
match config.api_provider.as_str() {
    // ... 現有分支 ...
    "MyProvider" => {
        if config.api_key.expose_secret().is_empty() {
            return Err("API_KEY_REQUIRED:MyProvider 需要 API Key".into());
        }
        translate_with_myprovider(text, config, file_name, glossary).await
    }
    // ...
}
```

## 步驟 3: 更新 translate_batch（行 ~214）

```rust
let provider_result = match config.api_provider.as_str() {
    // ... 現有分支 ...
    "MyProvider" => translate_with_myprovider(&batch_instruction, config, file_name, glossary).await?,
    // ...
};
```

## 步驟 4: 更新 URL 建構（行 ~566，如適用）

如果新 Provider 使用 OpenAI 相容 API，加入預設 URL：

```rust
match config.api_provider.as_str() {
    "DeepSeek" => "https://api.deepseek.com/v1/chat/completions".to_string(),
    "Mistral" => "https://api.mistral.ai/v1/chat/completions".to_string(),
    "MyProvider" => "https://api.myprovider.com/v1/chat/completions".to_string(),
    _ => "https://api.openai.com/v1/chat/completions".to_string(),
}
```

## 步驟 5: 更新 get_models_from_provider（如適用）

在 `src-tauri/src/commands.rs` 的 `get_models_from_provider` 函數中加入新 Provider 的模型拉取邏輯。

## 步驟 6: 更新前端下拉選單

在 `frontend/modules/config.js` 中找到 Provider 下拉選單的定義，加入新選項：

```javascript
const providers = ['Gemini', 'OpenAI', 'DeepSeek', 'Mistral', 'Ollama', 'DeepL', 'Google Free', 'MyProvider'];
```

同時更新 `noKeyProviders` 陣列（如果新 Provider 不需要 API Key）：

```javascript
const noKeyProviders = ['Ollama', 'Google Free', '無', 'MyProvider'];
```

## 步驟 7: 更新 CLI 互動選單

在 `src/cli/main.rs` 的互動流程中找到 Provider 選單，加入新選項：

```rust
let providers = [
    "Gemini", "OpenAI", "DeepSeek", "Mistral",
    "Ollama", "DeepL", "Google Free", "MyProvider",
];
```

## 步驟 8: 更新 i18n 標籤

在 `i18n_assets/gui/en_us.json` 和 `i18n_assets/gui/zh_tw.json` 中加入新 Provider 的顯示名稱：

```json
{
    "provider_myprovider": "MyProvider",
    "provider_myprovider_hint": "需要 API Key"
}
```

## 測試

1. 執行 `cargo test` 確認所有測試通過
2. 執行 `pnpm test` 確認前端測試通過
3. 手動測試 GUI 和 CLI 的 Provider 選擇與翻譯功能
