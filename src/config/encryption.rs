//! # 安全儲存模組 (Keyring)
//!
//! 使用系統憑證管理員 (如 Windows Credential Manager, macOS Keychain) 安全地儲存 API 金鑰。

use keyring::Entry;

const SERVICE_NAME: &str = "mc_translator_rs";
const ACCOUNT_NAME: &str = "api_key";

/// 儲存 API 金鑰至系統憑證管理員
pub fn save_api_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Ok(());
    }
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| format!("無法初始化 Keyring Entry: {}", e))?;
    entry.set_password(key).map_err(|e| format!("儲存金鑰失敗: {}", e))
}

/// 從系統憑證管理員讀取 API 金鑰
pub fn get_api_key() -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| format!("無法初始化 Keyring Entry: {}", e))?;
    entry.get_password().map_err(|e| format!("讀取金鑰失敗: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_cycle() {
        let original = "test_api_key_123456";
        let test_service = "mc_translator_test_suite";
        let test_account = "test_api_key";

        let entry = Entry::new(test_service, test_account).expect("Entry failed");
        entry.set_password(original).expect("Save failed");
        let fetched = entry.get_password().expect("Get failed");
        assert_eq!(original, fetched);

        let _ = entry.delete_credential();
    }
}
