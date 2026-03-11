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
