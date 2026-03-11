use mc_translator_rs;

#[test]
fn test_config_initialization() {
    // 這裡示範如何測試公開的 AppConfig
    let config = mc_translator_rs::config::AppConfig::default();
    assert_eq!(config.main_width, 750.0);
    assert_eq!(config.main_height, 550.0);
}

#[test]
fn test_translation_memory_path() {
    // 測試工具模組的公開函式或路徑
    let dict_dir = mc_translator_rs::config::DICT_DIR;
    assert!(dict_dir.ends_with("dicts"));
}
