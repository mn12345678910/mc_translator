//! # 加密模組
//! 使用 Windows DPAPI 加密/解密 API 金鑰；非 Windows 環境下使用 Base64。

use base64::{engine::general_purpose, Engine as _};
#[cfg(target_os = "windows")]
use std::ptr;
#[cfg(target_os = "windows")]
use winapi::um::dpapi::{CryptProtectData, CryptUnprotectData};
#[cfg(target_os = "windows")]
use winapi::um::wincrypt::DATA_BLOB;

/// 使用 Windows DPAPI 加密字串
#[cfg(target_os = "windows")]
pub fn encrypt_string(data: &str) -> Result<String, String> {
    let bytes = data.as_bytes();
    if bytes.is_empty() {
        return Ok(String::new());
    }
    assert!(bytes.len() <= u32::MAX as usize, "Data too large for DPAPI");

    let mut input = DATA_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = DATA_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    unsafe {
        let result = CryptProtectData(
            &mut input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut output,
        );

        if result != 0 {
            let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
            let encoded = general_purpose::STANDARD.encode(slice);
            winapi::um::winbase::LocalFree(output.pbData as *mut _);
            Ok(encoded)
        } else {
            Err("CryptProtectData failed".to_string())
        }
    }
}

/// 非 Windows 环境下，不進行 DPAPI 加密，直接回傳 Base64 對原始資料加碼
#[cfg(not(target_os = "windows"))]
pub fn encrypt_string(data: &str) -> Result<String, String> {
    Ok(general_purpose::STANDARD.encode(data.as_bytes()))
}

/// 使用 Windows DPAPI 解密字串
#[cfg(target_os = "windows")]
pub fn decrypt_string(encoded_data: &str) -> Result<String, String> {
    let decoded = general_purpose::STANDARD
        .decode(encoded_data)
        .map_err(|_| "Base64 decode failed".to_string())?;

    if decoded.is_empty() {
        return Ok(String::new());
    }
    assert!(
        decoded.len() <= u32::MAX as usize,
        "Data too large for DPAPI unprotect"
    );

    let mut input = DATA_BLOB {
        cbData: decoded.len() as u32,
        pbData: decoded.as_ptr() as *mut u8,
    };
    let mut output = DATA_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    unsafe {
        let result = CryptUnprotectData(
            &mut input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut output,
        );

        if result != 0 {
            let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
            let decoded_str = String::from_utf8_lossy(slice).into_owned();
            winapi::um::winbase::LocalFree(output.pbData as *mut _);
            Ok(decoded_str)
        } else {
            Err("CryptUnprotectData failed".to_string())
        }
    }
}

/// 非 Windows 环境下，將 Base64 解碼回原始字串
#[cfg(not(target_os = "windows"))]
pub fn decrypt_string(encoded_data: &str) -> Result<String, String> {
    let decoded = general_purpose::STANDARD
        .decode(encoded_data)
        .map_err(|_| "Base64 decode failed".to_string())?;
    String::from_utf8(decoded).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1. 正常路徑測試（正常流程）
    #[test]
    fn test_encryption_cycle_standard() {
        let original = "Hello World 123";
        let encrypted = encrypt_string(original).expect("Encryption failed");
        let decrypted = decrypt_string(&encrypted).expect("Decryption failed");
        assert_eq!(original, decrypted);
    }

    /// 2. 邊界值與 UTF-8 測試（邊界案例 / UTF-8）
    #[test]
    fn test_encryption_cycle_utf8() {
        let original = "測試文字 ❄️ 表情符號 繁體中文";
        let encrypted = encrypt_string(original).expect("Encryption failed");
        let decrypted = decrypt_string(&encrypted).expect("Decryption failed");
        assert_eq!(original, decrypted);
    }

    /// 3. 強韌性與異常處理（健壯性 / 負向案例）
    #[test]
    fn test_decryption_invalid_base64() {
        // 提供無效的 Base64 字串
        let invalid = "Invalid!Base64!String";
        let result = decrypt_string(invalid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Base64 decode failed");
    }

    #[test]
    fn test_encryption_empty_string() {
        // 空字串應安全返回空字串
        let original = "";
        let encrypted = encrypt_string(original).expect("Encryption failed");
        let decrypted = decrypt_string(&encrypted).expect("Decryption failed");
        assert_eq!(original, decrypted);
    }
}
