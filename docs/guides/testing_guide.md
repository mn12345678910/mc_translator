<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# 測試指南

本指南說明如何編寫、執行和除錯本專案的測試。

## 測試架構

### Rust 測試

| 類型        | 命令                                 | 說明              |
| ----------- | ------------------------------------ | ----------------- |
| 單元測試    | `cargo test --lib`                   | 測試單一函數/模組 |
| 整合測試    | `cargo test`                         | 測試模組間互動    |
| Binary 測試 | `cargo test --bin mc_translator_cli` | 測試 CLI 互動流程 |

### 前端測試

| 類型      | 命令              | 說明                   |
| --------- | ----------------- | ---------------------- |
| Vitest    | `pnpm vitest run` | 前端單元測試           |
| Mock 工具 | 自動載入          | 開發環境模擬 Tauri API |

## 如何編寫 Rust 測試

### 基本單元測試

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function("input");
        assert_eq!(result, "expected");
    }
}
```

### 非同步測試

```rust
#[tokio::test]
async fn test_async_function() {
    let result = my_async_function().await;
    assert!(result.is_ok());
}
```

### 測試錯誤處理

```rust
#[tokio::test]
async fn test_error_case() {
    let config = JobConfig {
        api_key: SecretString::from("".to_string()),
        api_provider: "Gemini".to_string(),
        ..JobConfig::default()
    };
    let res = translate_one("hello", &config, "test.json", None).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("API_KEY_REQUIRED"));
}
```

## 如何編寫前端測試

```javascript
import { describe, it, expect } from 'vitest';

describe('my module', () => {
    it('should do something', () => {
        const result = myFunction('input');
        expect(result).toBe('expected');
    });
});
```

## 測試隔離注意事項

### `TEST_LOCK` 序列化

某些測試會操作共享資源（如 `dicts/` 目錄），需要使用 `TEST_LOCK` 避免並行衝突：

```rust
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_dict_operations() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_test_dir();
    // ... 測試邏輯 ...
    teardown_test_dir();
}
```

### `dicts/` 目錄清理

每個測試應在開始和結束時清理 `dicts/` 目錄：

```rust
fn setup_test_dir() {
    let _ = fs::remove_dir_all(DICT_DIR);
    ensure_dicts_dir();
}

fn teardown_test_dir() {
    let _ = fs::remove_dir_all(DICT_DIR);
}
```

### Windows Race Condition

Windows 上目錄刪除可能需要等待檔案控制代碼釋放：

```rust
#[cfg(windows)]
std::thread::sleep(std::time::Duration::from_millis(50));
```

## Keyring 測試

Keyring 測試在 CI 環境（無 `org.freedesktop.secrets`）中會自動跳過：

```rust
#[test]
fn test_keyring_cycle() {
    let entry = match Entry::new(test_service, test_account) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("⚠️ 跳過 test_keyring_cycle：無法初始化 Keyring ({})", e);
            return;
        }
    };
    // ... 測試邏輯 ...
}
```

**重要：** 不要在 Keyring 測試中使用 `unwrap()`，因為 CI 環境沒有 Keyring 服務。

## CI 測試除錯指南

### 常見失敗原因

| 錯誤                                  | 原因               | 解決方法                 |
| ------------------------------------- | ------------------ | ------------------------ |
| `DBus error: org.freedesktop.secrets` | CI 無 Keyring 服務 | 使用 graceful skip 模式  |
| `dicts/` 目錄衝突                     | 測試並行衝突       | 使用 `TEST_LOCK`         |
| `unwrap()` on `Err`                   | 未處理錯誤情況     | 改為 `match` 或 `if let` |

### 查看 CI 日誌

```bash
gh run view <run-id> --log-failed | grep -E "error\[|FAILED|panic"
```

## Mock 工具使用

前端 Mock 工具在開發環境自動載入，模擬 Tauri API 呼叫：

```javascript
// frontend/modules/mock.js
const mocks = {
    get_config: () => ({ api_provider: 'Gemini', ... }),
    get_models_from_provider: () => ['gpt-4', 'gpt-3.5-turbo'],
    start_translation: async () => { /* 模擬翻譯 */ },
};
```

新增 Mock 指令時，同步更新 `allMockCommands` 陣列以確保覆蓋率追蹤正確。
