use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[test]
fn test_i18n_cli_consistency() {
    let assets = HashMap::from([
        ("zh_tw", include_str!("../src/i18n_assets/cli/zh_tw.json")),
        ("zh_cn", include_str!("../src/i18n_assets/cli/zh_cn.json")),
        ("en_us", include_str!("../src/i18n_assets/cli/en_us.json")),
        ("ja_jp", include_str!("../src/i18n_assets/cli/ja_jp.json")),
    ]);
    check_memory_consistency(assets);
}

#[test]
fn test_i18n_gui_assets_consistency() {
    let assets = HashMap::from([
        ("zh_tw", include_str!("../src/i18n_assets/gui/zh_tw.json")),
        ("zh_cn", include_str!("../src/i18n_assets/gui/zh_cn.json")),
        ("en_us", include_str!("../src/i18n_assets/gui/en_us.json")),
        ("ja_jp", include_str!("../src/i18n_assets/gui/ja_jp.json")),
    ]);
    check_memory_consistency(assets);
}

fn check_memory_consistency(assets: HashMap<&str, &str>) {
    let mut baseline_keys: Option<(String, HashSet<String>)> = None;

    for (name, content) in assets {
        let json: Value = serde_json::from_str(content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON for {}", name));

        let obj = json.as_object().expect("JSON root must be an object");
        let keys: HashSet<String> = obj.keys().cloned().collect();

        if let Some((ref b_name, ref b_keys)) = baseline_keys {
            // Check for missing keys in current file relative to baseline
            let missing_in_current: Vec<_> = b_keys.difference(&keys).collect();
            assert!(
                missing_in_current.is_empty(),
                "File '{}' is missing keys found in '{}': {:?}",
                name,
                b_name,
                missing_in_current
            );

            // Check for extra keys in current file relative to baseline
            let extra_in_current: Vec<_> = keys.difference(b_keys).collect();
            assert!(
                extra_in_current.is_empty(),
                "File '{}' has extra keys not found in '{}': {:?}",
                name,
                b_name,
                extra_in_current
            );
        } else {
            baseline_keys = Some((name.to_string(), keys));
        }
    }
}
