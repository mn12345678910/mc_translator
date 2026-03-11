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
    "item",
    "block",
    "tag",
    "registry_name",
    "entry",
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

    // 命名空間 ID，例如 "tconstruct:broad_axe"
    let contains_space = s.contains(' ');
    if !contains_space && s.contains(':') {
        return true;
    }

    // 以 # 或 @ 開頭的標記
    if !bytes.is_empty() && (bytes[0] == b'#' || bytes[0] == b'@') {
        return true;
    }

    // snake_case ID
    if !contains_space
        && s.contains('_')
        && bytes.iter().all(|&c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'/' || c == b'.'
        })
    {
        return true;
    }

    false
}
