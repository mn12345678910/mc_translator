#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_i18n_deserialization() {
        use mc_translator::i18n::{CliLabels, GuiLabels};
        let files = ["en_us.json", "ja_jp.json", "zh_cn.json", "zh_tw.json"];
        let dir = Path::new("src/i18n_assets");

        for file_name in &files {
            // 1. Test Gui
            let p_gui = dir.join("gui").join(file_name);
            let c_gui = fs::read_to_string(&p_gui).unwrap();
            let res_gui = serde_json::from_str::<GuiLabels>(&c_gui);
            assert!(
                res_gui.is_ok(),
                "Gui fail on {}: {:?}",
                file_name,
                res_gui.err()
            );

            // 2. Test Cli
            let p_cli = dir.join("cli").join(file_name);
            let c_cli = fs::read_to_string(&p_cli).unwrap();
            let res_cli = serde_json::from_str::<CliLabels>(&c_cli);
            assert!(
                res_cli.is_ok(),
                "Cli fail on {}: {:?}",
                file_name,
                res_cli.err()
            );
        }
    }
}
