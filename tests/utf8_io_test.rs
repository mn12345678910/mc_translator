use std::fs;
use std::path::Path;

#[test]
fn test_utf8_write_read_normal() {
    let temp_path = Path::new("test_utf8_normal.txt");
    let content = "這是一個標準的 UTF-8 測試。";

    fs::write(temp_path, content).expect("Failed to write file");
    let read_content = fs::read_to_string(temp_path).expect("Failed to read file");

    assert_eq!(content, read_content);
    let _ = fs::remove_file(temp_path);
}

#[test]
fn test_utf8_complex_characters() {
    let temp_path = Path::new("test_utf8_complex.txt");
    // 包含中文字、表情符號、以及 Minecraft 特有的符號 §
    let content = "§6等級: §a100 ❄️ 火球術 🔥 [繁體中文測試]";

    fs::write(temp_path, content).expect("Failed to write complex file");
    let read_content = fs::read_to_string(temp_path).expect("Failed to read complex file");

    assert_eq!(content, read_content);
    let _ = fs::remove_file(temp_path);
}

#[test]
fn test_utf8_invalid_fallback_safety() {
    let temp_path = Path::new("test_invalid_utf8.bin");
    // 寫入無效的 UTF-8 位元組序列
    let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
    fs::write(temp_path, invalid_bytes).expect("Failed to write binary file");

    // 驗證 Rust 的 read_to_string 會安全地返回錯誤，而不是引發無限迴圈或崩潰
    let result = fs::read_to_string(temp_path);
    assert!(result.is_err());

    let _ = fs::remove_file(temp_path);
}
