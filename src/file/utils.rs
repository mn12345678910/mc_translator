use std::collections::HashMap;

/// 遞迴遍歷 JSON，產生 (leaf_key, string_value) 序列
pub fn flatten_json_values(
    value: &serde_json::Value,
    key: Option<&str>,
    out: &mut Vec<(String, String)>,
) {
    match value {
        serde_json::Value::String(s) => {
            let k = key.unwrap_or("__ARRAY_ELEMENT__").to_string();
            out.push((k, s.clone()));
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                flatten_json_values(v, Some(k), out);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                flatten_json_values(v, None, out);
            }
        }
        _ => {}
    }
}
