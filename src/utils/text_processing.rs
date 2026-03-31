//! # 文本處理模組
//! 負責翻譯前後的文本預處理、後處理、格式還原與結果驗證。
//! 此模組**不依賴** `data_processing` 或 `translation_service`，
//! 因此可同時被兩者引用，解除循環依賴。

use std::collections::HashMap;
use std::sync::LazyLock;

/// 解析翻譯結果，移除 Markdown 代碼塊或修復損壞的標籤
pub fn validate_and_cleanup(text: &str, prefixes: &[String], contains: &[String]) -> String {
    let mut cleaned = text.trim().to_string();

    if cleaned.contains("```") {
        if let Some(start) = cleaned.find("```") {
            let sub = &cleaned[start + 3..];
            if let Some(end) = sub.find("```") {
                let content = sub[..end].trim();
                let lines: Vec<&str> = content.lines().collect();
                if !lines.is_empty()
                    && (lines[0] == "json"
                        || lines[0] == "text"
                        || lines[0] == "javascript"
                        || lines[0] == "js")
                {
                    cleaned = lines[1..].join("\n").trim().to_string();
                } else {
                    cleaned = content.to_string();
                }
            }
        }
    }

    if cleaned.contains('\n') {
        let lines: Vec<&str> = cleaned
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        for l in &lines {
            if prefixes.iter().any(|p| l.contains(p)) {
                if let Some(pos) = l.find('：') {
                    let candidate = l[pos + '：'.len_utf8()..]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('「')
                        .trim_matches('」')
                        .trim_matches('『')
                        .trim_matches('』')
                        .to_string();
                    if !candidate.is_empty() && candidate != "{}" && candidate != "{ }" {
                        return candidate;
                    }
                }
                if let Some(pos) = l.find(':') {
                    let candidate = l[pos + 1..]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('「')
                        .trim_matches('」')
                        .trim_matches('『')
                        .trim_matches('』')
                        .to_string();
                    if !candidate.is_empty() && candidate != "{}" && candidate != "{ }" {
                        return candidate;
                    }
                }
            }
        }
    }

    if (cleaned.starts_with('{') && cleaned.ends_with('}'))
        || (cleaned.starts_with('[') && cleaned.ends_with(']'))
    {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&cleaned) {
            if let Some(obj) = v.as_object() {
                if obj.len() == 1 {
                    if let Some(val) = obj.values().next().and_then(|node| node.as_str()) {
                        cleaned = val.to_string();
                    }
                } else if obj.is_empty() {
                    cleaned = String::new();
                }
            } else if let Some(arr) = v.as_array() {
                if arr.len() == 1 {
                    if let Some(val) = arr.first().and_then(|node| node.as_str()) {
                        cleaned = val.to_string();
                    }
                } else if arr.is_empty() {
                    cleaned = String::new();
                }
            }
        }
    }

    for p in prefixes {
        if cleaned.to_lowercase().starts_with(&p.to_lowercase()) {
            cleaned = cleaned[p.len()..].trim().to_string();
        }
    }

    if contains.iter().any(|c| cleaned.contains(c)) {
        let lines: Vec<&str> = cleaned
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .filter(|l| !contains.iter().any(|c| l.contains(c)))
            .collect();
        cleaned = lines.join("\n");
    }

    cleaned = cleaned.trim().to_string();

    let mut chars: Vec<char> = cleaned.chars().collect();
    if chars.len() >= 2 {
        let first = chars[0];
        let last = chars[chars.len() - 1];
        if (first == '"' && last == '"')
            || (first == '\'' && last == '\'')
            || (first == '「' && last == '」')
            || (first == '『' && last == '』')
        {
            chars.pop();
            chars.remove(0);
            cleaned = chars.into_iter().collect::<String>().trim().to_string();
        }
    }

    if cleaned == "{}" || cleaned == "{ }" || cleaned == "[]" || cleaned == "[ ]" {
        return String::new();
    }

    cleaned
}

/// 偵測翻譯文字是否陷入無限循環
pub fn detect_loop(text: &str) -> bool {
    if text.len() > 2000 {
        return true;
    } // 基本長度防護 (Minecraft 條目通常不會這麼長)

    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 10 {
        return false;
    }

    // 1. 簡單重複偵測 (例如 "文字文字文字...")
    for chunk_size in 2..=10 {
        if chars.len() < chunk_size * 4 {
            continue;
        }
        for i in 0..(chars.len() - chunk_size * 3) {
            let chunk = &chars[i..i + chunk_size];
            let next1 = &chars[i + chunk_size..i + chunk_size * 2];
            let next2 = &chars[i + chunk_size * 2..i + chunk_size * 3];
            if chunk == next1 && chunk == next2 {
                return true;
            }
        }
    }
    false
}

/// 匹配需要保留的格式代碼或預留位置 (如 §, &, #Hex, %s, %1$s, {0}, \n)
static PLACEHOLDER_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?x)
    (§[0-9a-fk-orA-FK-OR]) |
    (&[0-9a-fk-orA-FK-OR]) |
    (\#[0-9a-fA-F]{6}) |
    (%\d+\$[sd]|%[sd]) |
    (\{\d+\}) |
    (\\n)
"#,
    )
    .unwrap()
});

/// 強韌的佔位符還原正則：匹配形如 %%MC_0%%, %%mc_0%%, MC_0%%, %%0%%, %MC_0% 等變體
/// 核心是捕捉中間的數字索引 (\d+)。安全性：使用二分支邏輯確保精確匹配並排除 100% 等常見文字
static ROBUST_PLACEHOLDER_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)%%(\d+)%{0,2}|(?i)%{0,2}(?:MC|HEX|VAR)_?(\d+)%{0,2}").unwrap()
});

/// 將文本中的格式化標記替換為臨時預留位置，以防止 LLM 破壞格式
pub fn preprocess_text(text: &str) -> (String, Vec<String>) {
    let mut markers = Vec::new();
    let processed = PLACEHOLDER_RE
        .replace_all(text, |caps: &regex::Captures| {
            let matched = caps.get(0).unwrap().as_str();
            let idx = markers.len();
            markers.push(matched.to_string());

            if matched.starts_with('§') || matched.starts_with('&') {
                format!("%%MC_{}%%", idx)
            } else if matched.starts_with('#') {
                format!("%%HEX_{}%%", idx)
            } else {
                format!("%%VAR_{}%%", idx)
            }
        })
        .to_string();

    (processed, markers)
}

/// 將預留位置還原為原始格式標記
pub fn postprocess_text(text: &str, markers: &[String]) -> String {
    ROBUST_PLACEHOLDER_RE
        .replace_all(text, |caps: &regex::Captures| {
            // 從二進制分路徑中獲取索引 (擷取自 group 1 或 group 2)
            let idx_str = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str());
            if let Some(s) = idx_str {
                if let Ok(idx) = s.parse::<usize>() {
                    if idx < markers.len() {
                        return markers[idx].clone();
                    }
                }
            }
            // 若索引越界或標籤純屬幻覺，則將其移除 (Stripping Hallucinations)
            String::new()
        })
        .to_string()
}

/// 針對原始 JSON 檔案內容進行增量更新，儘量保留原始縮排與格式
pub fn sync_formatting(original: &str, translations: &HashMap<String, Vec<String>>) -> String {
    let mut result = String::with_capacity(original.len() + 2048);
    let mut counters = HashMap::<String, usize>::new();

    static KV_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#"^(\s*)"([^"]+)"\s*:\s*"[^"]*"(,?\s*)$"#).unwrap());
    static VAL_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#"^(\s*)"([^"]+)"(,?\s*)$"#).unwrap());

    for line in original.lines() {
        if let Some(caps) = KV_RE.captures(line) {
            let indent = &caps[1];
            let key = &caps[2];
            let suffix = &caps[3];

            let idx = counters.entry(key.to_string()).or_insert(0);
            if let Some(list) = translations.get(key) {
                if let Some(translated) = list.get(*idx) {
                    *idx += 1;
                    let json_str = serde_json::to_string(translated)
                        .unwrap_or_else(|_| format!(r#""{}""#, translated));
                    let escaped = &json_str[1..json_str.len() - 1];
                    result.push_str(&format!(r#"{}"{}" : "{}"{}"#, indent, key, escaped, suffix));
                    result.push('\n');
                    continue;
                }
            }
        } else if let Some(caps) = VAL_RE.captures(line) {
            let indent = &caps[1];
            let val = &caps[2];
            let suffix = &caps[3];

            if val != "[" && val != "{" {
                let key = "__ARRAY_ELEMENT__";
                let idx = counters.entry(key.to_string()).or_insert(0);
                if let Some(list) = translations.get(key) {
                    if let Some(translated) = list.get(*idx) {
                        *idx += 1;
                        let json_str = serde_json::to_string(translated)
                            .unwrap_or_else(|_| format!(r#""{}""#, translated));
                        let escaped = &json_str[1..json_str.len() - 1];
                        result.push_str(&format!(r#"{}"{}"{}"#, indent, escaped, suffix));
                        result.push('\n');
                        continue;
                    }
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

#[cfg(test)]
fn validate_and_cleanup_test(text: &str) -> String {
    let prefixes = vec![
        "Translation:".to_string(),
        "Translated:".to_string(),
        "翻譯：".to_string(),
        "譯文：".to_string(),
        "Note:".to_string(),
        "註：".to_string(),
        "結果：".to_string(),
        "Result:".to_string(),
    ];

    let contains = vec![
        "我們已將".to_string(),
        "以下是翻譯".to_string(),
        "JSON 格式".to_string(),
        "請確認".to_string(),
    ];
    validate_and_cleanup(text, &prefixes, &contains)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1. 正常路徑測試（正常流程）
    #[test]
    fn test_cleanup_standard() {
        let input = "Translation: \"Hello World\"";
        assert_eq!(validate_and_cleanup_test(input), "Hello World");

        let md_input = "```json\n{ \"text\": \"測試文字\" }\n```";
        assert_eq!(validate_and_cleanup_test(md_input), "測試文字");
    }

    /// 2. 邊界值與 UTF-8 測試（邊界案例 / UTF-8）
    #[test]
    fn test_cleanup_utf8_edge_cases() {
        // 測試中文引號與繁簡轉換支援 (UTF-8)
        let input = "翻譯：「這是特殊的 ❄️ 表情與繁體中文」";
        assert_eq!(
            validate_and_cleanup_test(input),
            "這是特殊的 ❄️ 表情與繁體中文"
        );

        // 測試空字串與特殊空白
        assert_eq!(validate_and_cleanup_test("   "), "");
        assert_eq!(validate_and_cleanup_test("{}"), "");
    }

    /// 3. 強韌性與無限迴圈偵測（健壯性 / 迴圈防護）
    #[test]
    fn test_detect_loop_infinite_prevention() {
        // 測試重複文字偵測
        let looping_text = "跳轉跳轉跳轉跳轉跳轉跳轉";
        assert!(detect_loop(looping_text));

        // 測試超長文字防護
        let long_text = "A".repeat(2001);
        assert!(detect_loop(&long_text));

        // 正常長文字不應判定為迴圈
        let normal_long = "這是一段很長但沒有重複的正常文字，用於測試偵測器不會誤判。".repeat(5);
        assert!(!detect_loop(&normal_long));
    }

    /// 4. 預處理與後處理測試 (格式保留)
    #[test]
    fn test_preprocess_postprocess_formatting() {
        let text = "Hello §aGreen &bBlue #ff00ff %s {0} \\nWorld";
        let (processed, markers) = preprocess_text(text);

        assert!(processed.contains("%%MC_0%%")); // §a
        assert!(processed.contains("%%MC_1%%")); // &b
        assert!(processed.contains("%%HEX_2%%")); // #ff00ff
        assert!(processed.contains("%%VAR_3%%")); // %s

        let restored = postprocess_text(&processed, &markers);
        assert_eq!(restored, text);
    }

    /// 5. 格式同步測試 (JSON 結構)
    #[test]
    fn test_sync_formatting_json_structure() {
        let original = r#"{
    "menu.title": "Main Menu",
    "btn_start": "Start",
    "list": [
        "item1",
        "item2"
    ]
}"#;
        let mut translations = HashMap::new();
        translations.insert("menu.title".to_string(), vec!["主選單".to_string()]);
        translations.insert("btn_start".to_string(), vec!["開始".to_string()]);
        translations.insert(
            "__ARRAY_ELEMENT__".to_string(),
            vec!["項目一".to_string(), "項目二".to_string()],
        );

        let synced = sync_formatting(original, &translations);
        assert!(synced.contains(r#""menu.title" : "主選單""#));
        assert!(synced.contains(r#""btn_start" : "開始""#));
        assert!(synced.contains(r#""項目一""#));
        assert!(synced.contains(r#""項目二""#));
    }

    /// 6. 進階解析與降級測試
    #[test]
    fn test_cleanup_advanced_vectors() {
        // Markdown 標題覆蓋 (text, javascript, js)
        assert_eq!(validate_and_cleanup_test("```text\nTarget\n```"), "Target");
        assert_eq!(
            validate_and_cleanup_test("```javascript\nTarget\n```"),
            "Target"
        );
        assert_eq!(validate_and_cleanup_test("```js\nTarget\n```"), "Target");

        // Markdown 未知標題降級 (rust) (Line 27)
        assert_eq!(
            validate_and_cleanup_test("```rust\nlet x = 1;\n```"),
            "rust\nlet x = 1;"
        );

        // 英文前綴及冒號 (包含中繼換行以防 trim)
        assert_eq!(
            validate_and_cleanup_test("Line1\nResult: \"Success\""),
            "Success"
        );

        // 中文前綴及全形冒號
        assert_eq!(validate_and_cleanup_test("Line1\n翻譯：『成功』 "), "成功");
    }

    #[test]
    fn test_cleanup_json_array_fallback() {
        // Array 單一元素降級
        assert_eq!(
            validate_and_cleanup_test("[ \"Array Element\" ]"),
            "Array Element"
        );

        // 空 Array 等等 (觸發 96 和 159)
        assert_eq!(validate_and_cleanup_test("[]"), "");
        assert_eq!(validate_and_cleanup_test("{ }"), "");
        assert_eq!(validate_and_cleanup_test("[ ]"), "");

        // 雜訊過濾
        assert_eq!(
            validate_and_cleanup_test("我們已將該段文字翻譯如下：\n翻譯完成"),
            "翻譯完成"
        );
    }

    #[test]
    fn test_sync_formatting_exhaustion() {
        let original = r#"{
    "key1": "Value1",
    "key2": "Value2"
}"#;
        let mut translations = HashMap::new();
        translations.insert("key1".to_string(), vec!["新值1".to_string()]); // key2 為 None 觸發 271

        let synced = sync_formatting(original, &translations);
        assert!(synced.contains(r#""key1" : "新值1""#));
        assert!(synced.contains(r#""key2": "Value2""#));
    }

    #[test]
    fn test_cleanup_dynamic_parameters() {
        let prefixes = vec!["CustomPrefix:".to_string(), "自訂：".to_string()];
        let contains = vec!["剔除我".to_string()];

        let input1 = "CustomPrefix: \"Hello\"";
        assert_eq!(validate_and_cleanup(input1, &prefixes, &contains), "Hello");

        let input2 = "自訂：「世界」";
        assert_eq!(validate_and_cleanup(input2, &prefixes, &contains), "世界");

        let input3 = "正常文字\n剔除我\n保留我";
        assert_eq!(
            validate_and_cleanup(input3, &prefixes, &contains),
            "正常文字\n保留我"
        );
    }

    #[test]
    fn test_postprocess_robustness() {
        let markers = vec!["§a".to_string(), "§r".to_string()];

        // 1. 正常還原
        assert_eq!(
            postprocess_text("%%MC_0%%Text%%MC_1%%", &markers),
            "§aText§r"
        );

        // 2. 大小寫不一致 (Fuzzy Case)
        assert_eq!(
            postprocess_text("%%mc_0%%Text%%Mc_1%%", &markers),
            "§aText§r"
        );

        // 3. 部分格式損壞 (Missing percents)
        assert_eq!(postprocess_text("MC_0%%Text%%MC_1", &markers), "§aText§r");

        // 4. 只有索引 (Minimal)
        assert_eq!(postprocess_text("%%0%%Text%%1%%", &markers), "§aText§r");

        // 5. 幻覺標籤清理 (Hallucination Stripping)
        // 只有 2 個 marker (0, 1)，%%MC_99%% 應該被剔除
        assert_eq!(
            postprocess_text("%%MC_0%%Hello%%MC_99%%", &markers),
            "§aHello"
        );

        // 6. 批次污染處理 (空 markers 但 LLM 加上標籤)
        let empty_markers: Vec<String> = vec![];
        assert_eq!(
            postprocess_text("Apotheosis %%MC_0%%Enchanting%%MC_1%%", &empty_markers),
            "Apotheosis Enchanting"
        );

        // 7. 安全性測試 (Normal percentages shouldn't be stripped)
        assert_eq!(
            postprocess_text("Value is 100% finished", &markers),
            "Value is 100% finished"
        );
        assert_eq!(
            postprocess_text("Accuracy: 95.5%", &markers),
            "Accuracy: 95.5%"
        );
    }
}
