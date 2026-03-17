//! # 跳過規則
//! 定義不需要翻譯的鍵名與值的判斷邏輯。

/// 不需要翻譯的鍵名列表
const SKIP_KEYS: &[&str] = &[
    "icon",
    "id",
    "type",
    "category",
    "entity",
    "recipe",
    "recipe2",
    "advancement",
    "predicate",
    "parent",
    "flag",
    "ingredient",
    "registry_name",
    "model",
    "texture",
    "reset to default",
    "restore defaults",
    "default settings",
    "back",
    "next",
];

pub fn should_skip_key(key: &str) -> bool {
    if SKIP_KEYS.iter().any(|&s| key.eq_ignore_ascii_case(s)) {
        return true;
    }

    let bytes = key.as_bytes();
    if bytes.len() >= 3 {
        let end = &bytes[bytes.len() - 3..];
        if end.eq_ignore_ascii_case(b"_id") {
            return true;
        }
        let start = &bytes[..3];
        if start.eq_ignore_ascii_case(b"id_") {
            return true;
        }
    }
    false
}

pub fn should_skip_value(val: &str) -> bool {
    if val.is_empty() {
        return true;
    }
    let s = val.trim();
    if s.is_empty() {
        return false;
    } // 只有空格的字串不應該被跳過

    let bytes = s.as_bytes();

    // 布林值
    if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
        return true;
    }

    // 正則表達式模式 (常見於 KubeJS)
    if bytes.len() > 1 && bytes[0] == b'/' && bytes[bytes.len() - 1] == b'/' {
        return true;
    }

    // 純數字、點、負號
    if bytes
        .iter()
        .all(|&c| c.is_ascii_digit() || c == b'.' || c == b'-')
    {
        return true;
    }

    // 檔名、副檔名或路徑 (無空格，以特定後綴結尾)
    let contains_space = s.contains(' ');
    // 無空格之特殊格式過濾 (聚合在一組以簡化邏輯)
    if !contains_space {
        // 1. 檔名、副檔名或路徑
        if s.ends_with(".jar") || s.ends_with(".zip") || s.ends_with(".json") ||
           s.ends_with(".js") || s.ends_with(".png") || s.ends_with(".jpg") {
            return true;
        }

        // 2. 命名空間 ID，例如 "tconstruct:broad_axe"
        if s.contains(':') {
            return true;
        }

        // 3. 16 進位字串 / 雜湊碼 / 顏色碼排除 (長度 6、8 或 >=16)
        if bytes.iter().all(|&c| c.is_ascii_hexdigit()) && (s.len() == 6 || s.len() == 8 || s.len() >= 16) {
            return true;
        }

        // 4. UUID 排除規則 (長度 36 且包含 4 個連字號)
        if s.len() == 36 && s.chars().filter(|&c| c == '-').count() == 4 {
            return true;
        }

        // 5. 變數與常數排除 (包含底線 _)
        if s.contains('_') {
            // 全大寫常數 (ALL_CAPS)
            if s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
                return true;
            }
            // snake_case ID
            if bytes.iter().all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'/' || c == b'.' || c == b'-') {
                return true;
            }
        }

        // 6. Base64 結尾排除規則
        if s.len() >= 8 && s.ends_with('=') {
            return true;
        }
    }

    // 7. 日期格式排除規則 (包含 '.' 與 ':' 且除去後全為數字)
    let no_symbols = s.replace(['.', ':', ' '], "");
    if s.contains('.') && s.contains(':') && !no_symbols.is_empty() && no_symbols.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    false
}
