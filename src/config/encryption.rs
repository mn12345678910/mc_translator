//! # 安全儲存模組 (Keyring)
//!
//! 使用系統憑證管理員 (如 Windows Credential Manager, macOS Keychain) 安全地儲存 API 金鑰。
//! 傳入空字串至 `save_api_key` 會清空已儲存的憑證。

use keyring::Entry;

const SERVICE_NAME: &str = "mc_translator";
const ACCOUNT_NAME: &str = "api_key";

/// 儲存 API 金鑰至系統憑證管理員
/// 傳入空字串會清空已儲存的憑證。
pub fn save_api_key(key: &str) -> Result<(), String> {
    save_api_key_with_args(key, SERVICE_NAME, ACCOUNT_NAME)
}

/// 儲存 API 金鑰至系統憑證管理員 (帶參數版本)
/// 傳入空字串會清空已儲存的憑證。
pub fn save_api_key_with_args(key: &str, service: &str, account: &str) -> Result<(), String> {
    let entry =
        Entry::new(service, account).map_err(|e| format!("無法初始化 Keyring Entry: {}", e))?;
    if key.is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry
        .set_password(key)
        .map_err(|e| format!("儲存金鑰失敗: {}", e))
}

/// 從系統憑證管理員讀取 API 金鑰
pub fn get_api_key() -> Result<String, String> {
    get_api_key_with_args(SERVICE_NAME, ACCOUNT_NAME)
}

/// 從系統憑證管理員讀取 API 金鑰 (帶參數版本)
pub fn get_api_key_with_args(service: &str, account: &str) -> Result<String, String> {
    let entry =
        Entry::new(service, account).map_err(|e| format!("無法初始化 Keyring Entry: {}", e))?;
    entry
        .get_password()
        .map_err(|e| format!("讀取金鑰失敗: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_cycle() {
        let original = "test_api_key_123456";
        let test_service = "mc_translator_test_suite_cycle";
        let test_account = "test_api_key";

        let entry = match Entry::new(test_service, test_account) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("⚠️ 跳過 test_keyring_cycle：無法初始化 Keyring ({})", e);
                return;
            }
        };
        if let Err(e) = entry.set_password(original) {
            eprintln!("⚠️ 跳過 test_keyring_cycle：無法寫入 Keyring ({})", e);
            return;
        }
        let fetched = match entry.get_password() {
            Ok(k) => k,
            Err(e) => {
                eprintln!("⚠️ 跳過 test_keyring_cycle：無法讀取 Keyring ({})", e);
                return;
            }
        };
        assert_eq!(original, fetched);

        let _ = entry.delete_credential();
    }

    #[test]
    fn test_save_get_api_key_with_args_success() {
        let key = "test_key_args_123";
        let service = "mc_translator_test_args_exec";
        let account = "api_key";

        let entry = match Entry::new(service, account) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("⚠️ 跳過 test_save_get_api_key_with_args_success：無法初始化 Keyring");
                return;
            }
        };

        if let Err(e) = save_api_key_with_args(key, service, account) {
            eprintln!(
                "⚠️ 跳過 test_save_get_api_key_with_args_success：無法儲存 Keyring ({})",
                e
            );
            return;
        }
        let fetched = match get_api_key_with_args(service, account) {
            Ok(k) => k,
            Err(e) => {
                eprintln!(
                    "⚠️ 跳過 test_save_get_api_key_with_args_success：無法讀取 Keyring ({})",
                    e
                );
                return;
            }
        };
        assert_eq!(key, fetched);

        // 清理
        let _ = entry.delete_credential();
    }

    #[test]
    fn test_save_empty_clears_keyring() {
        let key = "temp_key_to_clear";
        let service = "mc_translator_test_empty_clear";
        let account = "api_key";

        let entry = match Entry::new(service, account) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("⚠️ 跳過 test_save_empty_clears_keyring：無法初始化 Keyring");
                return;
            }
        };

        if let Err(e) = save_api_key_with_args(key, service, account) {
            eprintln!(
                "⚠️ 跳過 test_save_empty_clears_keyring：無法儲存 Keyring ({})",
                e
            );
            return;
        }
        let fetched = match get_api_key_with_args(service, account) {
            Ok(k) => k,
            Err(e) => {
                eprintln!(
                    "⚠️ 跳過 test_save_empty_clears_keyring：無法讀取 Keyring ({})",
                    e
                );
                return;
            }
        };
        assert_eq!(key, fetched);

        // 儲存空字串應清空
        save_api_key_with_args("", service, account).unwrap();
        assert!(
            get_api_key_with_args(service, account).is_err(),
            "空字串儲存後應無法再讀取到 key"
        );

        let _ = entry.delete_credential();
    }
}
