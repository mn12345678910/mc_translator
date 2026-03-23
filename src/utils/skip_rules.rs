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
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    } // 只有空格的字串不應該被跳過

    let bytes = trimmed.as_bytes();

    // 布林值
    if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("false") {
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
    let contains_space = trimmed.contains(' ');
    // 無空格之特殊格式過濾 (聚合在一組以簡化邏輯)
    if !contains_space {
        // 1. 檔名、副檔名或路徑
        if trimmed.ends_with(".jar")
            || trimmed.ends_with(".zip")
            || trimmed.ends_with(".json")
            || trimmed.ends_with(".js")
            || trimmed.ends_with(".png")
            || trimmed.ends_with(".jpg")
        {
            return true;
        }

        // 2. 命名空間 ID，例如 "tconstruct:broad_axe"
        if trimmed.contains(':') {
            return true;
        }

        // 3. 16 進位字串 / 雜湊碼 / 顏色碼排除 (長度 6、8 或 >=16)
        if bytes.iter().all(|&c| c.is_ascii_hexdigit())
            && (trimmed.len() == 6 || trimmed.len() == 8 || trimmed.len() >= 16)
        {
            return true;
        }

        // 4. UUID 排除規則 (長度 36 且包含 4 個連字號)
        if trimmed.len() == 36 && trimmed.chars().filter(|&c| c == '-').count() == 4 {
            return true;
        }

        // 5. 變數與常數排除 (包含底線 _ 或在內部含有 .)
        if trimmed.contains('_')
            || (trimmed.contains('.') && !trimmed.ends_with('.'))
            || trimmed.contains('/')
        {
            // 全大寫常數 (ALL_CAPS)
            if trimmed
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                return true;
            }
            // 系統識別碼 (snake_case, 路徑, 原文 ID, 類別路徑等)
            if bytes.iter().all(|&c| {
                c.is_ascii_alphanumeric() || c == b'_' || c == b'/' || c == b'.' || c == b'-'
            }) {
                return true;
            }
        }

        // 6. Base64 結尾排除規則
        if trimmed.len() >= 8 && trimmed.ends_with('=') {
            return true;
        }

        // 7. 錨點 / 標籤錨點排除 (例如 #recipe, #tier#), 沒有空白且以 # 起手
        if trimmed.starts_with('#') {
            return true;
        }

        // 7. 短編碼排除 (例如 BB, BPPB, B0PB)：全大寫與數字，長度 1~6 之間，且無空白
        if trimmed.len() <= 6
            && trimmed
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return true;
        }
    }

    // 8. 日期格式排除規則 (包含 '.' 與 ':' 且除去後全為數字)
    let no_symbols = trimmed.replace(['.', ':', ' '], "");
    if trimmed.contains('.')
        && trimmed.contains(':')
        && !no_symbols.is_empty()
        && no_symbols.chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_key() {
        assert!(should_skip_key("icon"));
        assert!(should_skip_key("id"));
        assert!(should_skip_key("entity_id"));
        assert!(should_skip_key("id_name"));
        assert!(!should_skip_key("name"));
        assert!(!should_skip_key("description"));
    }

    #[test]
    fn test_should_skip_value_booleans_regex_numeric() {
        assert!(should_skip_value("true"));
        assert!(should_skip_value("false"));
        assert!(should_skip_value("/^regex$/"));
        assert!(should_skip_value("123"));
        assert!(should_skip_value("-123.45"));
        assert!(!should_skip_value("123 Apples")); // 有空格
    }

    #[test]
    fn test_should_skip_value_formats() {
        // 副檔名
        assert!(should_skip_value("icon.png"));
        assert!(should_skip_value("mod.jar"));
        // 命名空間
        assert!(should_skip_value("minecraft:apple"));
        // UUID / Hex
        assert!(should_skip_value("12345678-1234-1234-1234-1234567890ab"));
        assert!(should_skip_value("ff00ff")); // 長度 6 HEX
        assert!(should_skip_value("ffffffff")); // 長度 8 HEX
    }

    #[test]
    fn test_should_skip_value_technical_ids() {
        // snake_case 與底線
        assert!(should_skip_value("tconstruct_broad_axe"));
        // 含有點且大小寫混合 (CamelCase/Java Class)
        assert!(should_skip_value(
            "com.hollingsworth.arsnouveau.client.patchouli.component.RotatingItemListComponent"
        ));
        assert!(should_skip_value("item.minecraft.apple"));
        assert!(should_skip_value("Folder/SubFolder/Class"));
    }

    #[test]
    fn test_should_skip_value_tags_codes() {
        // 錨點
        assert!(should_skip_value("#recipe"));
        assert!(should_skip_value("#level"));
        assert!(should_skip_value("#tier#"));
        // 短編碼
        assert!(should_skip_value("BB"));
        assert!(should_skip_value("B0PB"));
        assert!(should_skip_value("A1"));
        assert!(!should_skip_value("Hello")); // 含有小寫
    }
}
