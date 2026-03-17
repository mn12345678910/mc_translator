#[cfg(test)]
mod tests {
    use mc_translator_rs::ui::i18n::I18nLabels;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_i18n_deserialization() {
        let files = ["en_us.json", "ja_jp.json", "zh_cn.json"];
        let dir = Path::new("src/i18n_assets");

        for file_name in &files {
            let path = dir.join(file_name);
            let content = fs::read_to_string(&path).unwrap();
            let res = serde_json::from_str::<I18nLabels>(&content);
            assert!(res.is_ok(), "Failed on {}: {:?}", file_name, res.err());
        }
    }
}
