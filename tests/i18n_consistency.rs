use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[test]
fn test_i18n_cli_consistency() {
    check_directory_consistency("src/i18n_assets/cli");
}

#[test]
fn test_i18n_gui_assets_consistency() {
    check_directory_consistency("src/i18n_assets/gui");
}

#[test]
fn test_i18n_langs_gui_consistency() {
    check_directory_consistency("langs/gui");
}

fn check_directory_consistency<P: AsRef<Path>>(dir: P) {
    let dir = dir.as_ref();
    if !dir.exists() {
        return; // Skip if directory doesn't exist (e.g. in some environments)
    }

    let mut baseline_keys: Option<(String, HashSet<String>)> = None;

    for entry in fs::read_dir(dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read file: {:?}", path));
            let json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse JSON: {:?}", path));

            let obj = json.as_object().expect("JSON root must be an object");
            let keys: HashSet<String> = obj.keys().cloned().collect();
            let filename = path.file_name().unwrap().to_string_lossy().into_owned();

            if let Some((ref b_name, ref b_keys)) = baseline_keys {
                // Check for missing keys in current file relative to baseline
                let missing_in_current: Vec<_> = b_keys.difference(&keys).collect();
                assert!(
                    missing_in_current.is_empty(),
                    "File '{}' is missing keys found in '{}': {:?}",
                    filename,
                    b_name,
                    missing_in_current
                );

                // Check for extra keys in current file relative to baseline
                let extra_in_current: Vec<_> = keys.difference(b_keys).collect();
                assert!(
                    extra_in_current.is_empty(),
                    "File '{}' has extra keys not found in '{}': {:?}",
                    filename,
                    b_name,
                    extra_in_current
                );
            } else {
                baseline_keys = Some((filename, keys));
            }
        }
    }
}
