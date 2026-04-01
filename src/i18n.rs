// Recompile trigger 1
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const DEFAULT_LANG: &str = "zh_tw";

// --- 重構後之共通結構體 ---
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct CommonLabels {
    pub app_title: String,
    pub label_output_path: String,
    pub btn_pause: String,
    pub btn_stop: String,
    pub label_model: String,
    pub label_api_key: String,
    pub prompt_select_model: String,
    pub log_pause_requested: String,
    pub log_stopped: String,
    pub btn_save: String,
    pub status_analyzing_dict: String,
    pub status_analyzing_files: String,
    pub status_finished: String,
    pub log_resuming: String,
    pub log_batch_error: String,
    pub log_loop_detected: String,
    pub status_processing_item: String,
    pub status_processing_batch: String,
    pub status_translating_item: String,
    pub log_batch_failed_retry: String,
    pub log_retry_start: String,
    pub log_single_failed: String,
    pub log_single_retry_start: String,
    pub log_single_final_failed: String,
    pub status_translating: String,
    pub status_translating_batch: String,
    pub log_batch_invalid: String,
    pub status_idle: String,
    pub status_scanning_files: String,
    pub log_processing_finished: String,
    pub log_processing_file_mask: String,
    pub log_generating_pack: String,
    pub log_pack_item_exists_warn: String,
    pub log_pack_gen_finished: String,
    pub default_user_prompt: String,
    pub default_system_prompt: String,
    pub placeholder_input_path: String,
    pub placeholder_api_key: String,
    pub placeholder_search_terms: String,
    pub placeholder_dict_key: String,
    pub placeholder_dict_value: String,
    pub status_load_config_failed: String,
    pub status_save_config_failed: String,
    pub status_browse_path_failed: String,
    pub status_input_path_empty: String,
    pub status_trans_starting: String,
    pub status_trans_error: String,
    pub status_trans_paused: String,
    pub status_trans_resumed: String,
    pub status_trans_stopping: String,
    pub status_dict_item_updated: String,
    pub status_dict_item_delete_confirm: String,
    pub status_dict_load_failed: String,
    pub status_dict_key_empty: String,
    pub status_dict_add_success: String,
    pub status_dict_add_failed: String,
    pub status_dict_replace_empty: String,
    pub status_dict_replace_confirm: String,
    pub status_dict_replace_sent: String,
    pub status_dict_replace_failed: String,
    pub status_dict_clear_success: String,
    pub status_dict_import_success: String,
    pub status_dict_export_success: String,
    pub status_palette_clear_item: String,
    pub err_ollama_connect: String,
    pub err_api_key_empty: String,
    pub err_gemini_models: String,
    pub err_openai_models: String,
    pub err_deepseek_models: String,
    pub err_unsupported_provider: String,
    pub err_no_active_job: String,
    pub lang_zh_tw: String,
    pub lang_zh_cn: String,
    pub lang_ja_jp: String,
    pub lang_en_us: String,
    pub cat_all_bg: String,
    pub cat_all_buttons: String,
    pub cat_all_inputs: String,
    pub cat_all_labels: String,
    pub cat_all_logs: String,
    pub cat_all_btn_text: String,
    pub cat_all_tab_active: String,
    pub cat_all_tab_inactive: String,
    pub status_failed_or_cancelled: String,
    pub status_trans_failed_mask: String,
    pub status_batch_mask: String,
    #[serde(default)]
    pub status_progress_detailed_mask: String,
    #[serde(default)]
    pub cleanup_prefixes: Vec<String>,
    #[serde(default)]
    pub cleanup_contains: Vec<String>,
    #[serde(default)]
    pub error_read_jar_index: String,
    #[serde(default)]
    pub error_read_jar_file: String,
    #[serde(default)]
    pub error_pipeline_failed: String,
}

impl CommonLabels {
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let dir = get_langs_dir("gui");
        let lang_path = dir.join(format!("{}.json", lang));
        if let Ok(file_content) = fs::read_to_string(&lang_path) {
            let lang_json: serde_json::Value = serde_json::from_str(&file_content).ok()?;
            let mut default_json: serde_json::Value =
                serde_json::from_str(include_str!("i18n_assets/gui/en_us.json")).ok()?;
            if let (Some(lang_obj), Some(default_obj)) =
                (lang_json.as_object(), default_json.as_object_mut())
            {
                for (k, v) in lang_obj {
                    default_obj.insert(k.clone(), v.clone());
                }
            }
            if let Ok(labels) = serde_json::from_value::<Self>(default_json) {
                return Some(labels);
            }
        }
        None
    }

    pub fn load_or_default(lang: &str) -> Self {
        Self::load_or_default_with_dir(lang, &get_langs_dir("gui"))
    }

    fn load_or_default_with_dir(lang: &str, dir: &Path) -> Self {
        if let Some(l) = Self::load_from_file_with_dir(lang, dir) {
            return l;
        }
        if lang != "en_us" {
            if let Some(e) = Self::load_from_file_with_dir("en_us", dir) {
                return e;
            }
        }
        Self::default_zh_tw()
    }

    fn load_from_file_with_dir(lang: &str, dir: &Path) -> Option<Self> {
        let lang_path = dir.join(format!("{}.json", lang));
        if let Ok(file_content) = fs::read_to_string(&lang_path) {
            let lang_json: serde_json::Value = serde_json::from_str(&file_content).ok()?;
            let mut default_json: serde_json::Value =
                serde_json::from_str(include_str!("i18n_assets/gui/en_us.json")).ok()?;
            if let (Some(lang_obj), Some(default_obj)) =
                (lang_json.as_object(), default_json.as_object_mut())
            {
                for (k, v) in lang_obj {
                    default_obj.insert(k.clone(), v.clone());
                }
            }
            if let Ok(labels) = serde_json::from_value::<Self>(default_json) {
                return Some(labels);
            }
        }
        None
    }

    pub fn default_zh_tw() -> Self {
        serde_json::from_str(include_str!("i18n_assets/gui/zh_tw.json")).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GuiLabels {
    #[serde(flatten)]
    pub common: CommonLabels,

    pub btn_nav_settings: String,
    pub btn_nav_dict: String,
    pub btn_nav_palette: String,
    pub btn_nav_theme: String,
    pub btn_nav_dev: String,
    pub btn_clear: String,
    pub btn_select_file: String,
    pub btn_select_folder: String,
    pub btn_output_dir: String,
    pub btn_open_output: String,
    pub label_input_path: String,
    pub label_ui_lang: String,
    pub label_fast_convert: String,
    pub label_fast_convert_on: String,
    pub label_fast_convert_off: String,
    pub btn_run: String,
    pub label_current_status: String,
    pub label_log_area: String,
    pub header_api_settings: String,
    pub label_provider: String,
    pub label_loading_models: String,
    pub label_no_models: String,
    pub label_batch_size: String,
    pub label_max_chars: String,
    pub label_timeout: String,
    pub label_font_size: String,
    pub label_pack_format: String,
    pub label_source_lang: String,
    pub label_target_lang: String,
    pub btn_restore_defaults: String,
    pub label_user_prompt: String,
    pub header_palette: String,
    pub label_bg_color: String,
    pub label_text_color: String,
    pub label_custom_rounding: String,
    pub label_rounding_value: String,
    pub label_anim_speed: String,
    pub btn_resume: String,
    pub text_confirm_stop: String,
    pub header_dev_mode: String,
    pub label_skip_json: String,
    pub label_no_skip_json: String,
    pub label_skip_jar: String,
    pub label_no_skip_jar: String,
    pub label_skip_js: String,
    pub label_no_skip_js: String,
    pub label_skip_book: String,
    pub label_no_skip_book: String,
    pub label_enable_log: String,
    pub label_enable_debug_log: String,
    pub label_disable_debug_log: String,
    pub label_disable_log: String,
    pub label_system_prompt: String,
    pub label_ollama_url: String,
    pub group_batch: String,
    pub group_specific: String,
    pub spec_btn_select_file: String,
    pub spec_btn_select_folder: String,
    pub spec_btn_output_dir: String,
    pub spec_btn_open_output: String,
    pub spec_btn_run: String,
    pub spec_btn_pause: String,
    pub spec_btn_stop: String,
    pub spec_area_dict: String,
    pub spec_label_output: String,
    pub spec_progress_current: String,
    pub spec_progress_total: String,
    pub glossary_tab_user: String,
    pub glossary_tab_official: String,
    pub glossary_key: String,
    pub glossary_value: String,
    pub glossary_clear_title: String,
    pub glossary_priority_hover: String,
    pub glossary_priority_user: String,
    pub glossary_priority_official: String,
    pub glossary_col_actions: String,
    pub glossary_empty: String,
    pub header_dict_mgr: String,
    pub label_page_info: String,
    pub label_glossary_priority: String,
    pub label_palette_target_type: String,
    pub label_palette_target_item: String,
    pub label_palette_property: String,
    pub label_palette_color: String,
    pub label_palette_rounding: String,
    pub label_items: String,
    pub label_files: String,
    pub btn_dict_open_json: String,
    pub btn_palette_clear_item: String,
    pub label_progress_style: String,
    pub style_default: String,
    pub style_aurora: String,
    pub style_neon: String,
    pub cat_accent_color: String,
    pub cat_danger_color: String,
    pub cat_border_color: String,
    pub cat_hover_bg: String,
    pub cat_slider_bg: String,
    pub cat_slider_thumb: String,
    pub cat_switch_bg: String,
    pub cat_progress_bg: String,
    pub cat_log_info: String,
    pub cat_log_warn: String,
    pub cat_log_error: String,
    pub cat_log_success: String,
    pub cat_log_dir: String,
    pub cat_log_file: String,
    pub cat_aurora_start: String,
    pub cat_aurora_mid: String,
    pub cat_aurora_end: String,
    pub cat_neon_color: String,
    pub group_layout: String,
    pub label_space_sm: String,
    pub label_space_md: String,
    pub label_space_lg: String,
    pub label_alpha_border: String,
    pub label_alpha_panel: String,
    pub label_alpha_backdrop: String,
    pub palette_label_spacing: String,
    pub palette_label_alpha: String,
    pub palette_label_rounding: String,
    pub status_style_restored: String,
    #[serde(default)]
    pub label_excluded_paths: String,
    #[serde(default)]
    pub placeholder_excluded_paths: String,
    #[serde(default)]
    pub status_config_restored: String,
    #[serde(default)]
    pub status_dev_restored: String,
    #[serde(default)]
    pub label_show_debug_tools: String,
    #[serde(default)]
    pub label_hide_debug_tools: String,
    #[serde(default)]
    pub page_title: String,
    #[serde(default)]
    pub placeholder_output_path: String,
    #[serde(default)]
    pub label_llm_log: String,
}

impl GuiLabels {
    pub fn ensure_langs_exists() -> Result<(), std::io::Error> {
        Self::ensure_langs_exists_with_dir(&get_langs_dir("gui"))
    }

    fn ensure_langs_exists_with_dir(dir: &Path) -> Result<(), std::io::Error> {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        let zh_tw_path = dir.join("zh_tw.json");
        if !zh_tw_path.exists() {
            let default_labels = Self::default_zh_tw();
            fs::write(
                &zh_tw_path,
                serde_json::to_string_pretty(&default_labels).unwrap(),
            )?;
        }
        let files = [
            ("zh_cn.json", include_str!("i18n_assets/gui/zh_cn.json")),
            ("en_us.json", include_str!("i18n_assets/gui/en_us.json")),
            ("ja_jp.json", include_str!("i18n_assets/gui/ja_jp.json")),
        ];
        for (file_name, file_content) in files {
            let file_path = dir.join(file_name);
            if cfg!(debug_assertions) || !file_path.exists() {
                fs::write(file_path, file_content)?;
            }
        }
        Ok(())
    }

    pub fn load_from_file(lang: &str) -> Option<Self> {
        Self::load_from_file_with_dir(lang, &get_langs_dir("gui"))
    }

    fn load_from_file_with_dir(lang: &str, dir: &Path) -> Option<Self> {
        let lang_path = dir.join(format!("{}.json", lang));
        if let Ok(file_content) = fs::read_to_string(&lang_path) {
            let lang_json: serde_json::Value = serde_json::from_str(&file_content).ok()?;
            let mut default_json: serde_json::Value =
                serde_json::from_str(include_str!("i18n_assets/gui/en_us.json")).ok()?;
            if let (Some(lang_obj), Some(default_obj)) =
                (lang_json.as_object(), default_json.as_object_mut())
            {
                for (k, v) in lang_obj {
                    default_obj.insert(k.clone(), v.clone());
                }
            }
            if let Ok(labels) = serde_json::from_value::<Self>(default_json) {
                return Some(labels);
            }
        }
        None
    }

    pub fn load_or_default(lang: &str) -> Self {
        Self::load_or_default_with_dir(lang, &get_langs_dir("gui"))
    }

    fn load_or_default_with_dir(lang: &str, dir: &Path) -> Self {
        if let Some(loaded) = Self::load_from_file_with_dir(lang, dir) {
            return loaded;
        }
        Self::default_zh_tw()
    }

    pub fn default_zh_tw() -> Self {
        let json_str = include_str!("i18n_assets/gui/zh_tw.json");
        match serde_json::from_str::<Self>(json_str) {
            Ok(v) => v,
            Err(e) => {
                println!("\n[DEBUG] Serde error: {:?}", e);
                panic!("Serde error: {:?}", e);
            }
        }
    }

    pub fn get_available_ui_langs() -> Vec<String> {
        let mut langs_list = Vec::new();
        if let Ok(es) = std::fs::read_dir(get_langs_dir("gui")) {
            for e in es.flatten() {
                if e.path().extension().is_some_and(|x| x == "json") {
                    if let Some(stem) = e.path().file_stem() {
                        langs_list.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }
        if langs_list.is_empty() {
            langs_list.push("zh_tw".to_string());
        }
        langs_list.sort();
        langs_list
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct CliLabels {
    #[serde(flatten)]
    pub common: CommonLabels,

    pub prompt_select_provider_cli: String,
    pub prompt_advanced_settings_cli: String,
    pub prompt_confirm_start_cli: String,
    pub prompt_task_finished_cli: String,
    pub prompt_new_task_cli: String,
    pub label_back_to_prev_cli: String,
    pub label_custom_input_cli: String,
    pub label_yes_confirm_cli: String,
    pub label_no_cancel_cli: String,
    pub cli_banner_title: String,
    pub cli_mode_headless: String,
    pub cli_mode_interactive: String,
    pub cli_select_ui_lang: String,
    pub cli_fetching_models: String,
    pub cli_model_fetch_failed: String,
    pub cli_custom_model_prompt: String,
    #[serde(default)]
    pub cli_api_key_old_hint: String,
    pub cli_input_path_prompt: String,
    pub cli_error_path_not_exist: String,
    pub cli_output_path_prompt: String,
    pub cli_op_cancelled: String,
    pub cli_adv_settings_synced: String,
    pub cli_starting_pipeline: String,
    pub cli_pipeline_ended: String,
    pub cli_pipeline_success: String,
    pub cli_pipeline_failed: String,
    #[serde(default)]
    pub cli_error_input_not_exist: String,
    #[serde(default)]
    pub cli_detect_headless: String,
    #[serde(default)]
    pub cli_label_provider: String,
    #[serde(default)]
    pub cli_label_model: String,
    #[serde(default)]
    pub cli_label_input: String,
    #[serde(default)]
    pub cli_label_output: String,
    #[serde(default)]
    pub cli_label_default: String,
    #[serde(default)]
    pub cli_hint_config_exclude: String,
}

impl CliLabels {
    pub fn ensure_langs_exists() -> Result<(), std::io::Error> {
        Self::ensure_langs_exists_with_dir(&get_langs_dir("cli"))
    }

    fn ensure_langs_exists_with_dir(dir: &Path) -> Result<(), std::io::Error> {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        let zh_tw_path = dir.join("zh_tw.json");
        if !zh_tw_path.exists() {
            let default_labels = Self::default_zh_tw();
            fs::write(
                &zh_tw_path,
                serde_json::to_string_pretty(&default_labels).unwrap(),
            )?;
        }
        let files = [
            ("zh_cn.json", include_str!("i18n_assets/cli/zh_cn.json")),
            ("en_us.json", include_str!("i18n_assets/cli/en_us.json")),
            ("ja_jp.json", include_str!("i18n_assets/cli/ja_jp.json")),
        ];
        for (file_name, file_content) in files {
            let file_path = dir.join(file_name);
            if cfg!(debug_assertions) || !file_path.exists() {
                fs::write(file_path, file_content)?;
            }
        }
        Ok(())
    }

    pub fn load_from_file(lang: &str) -> Option<Self> {
        Self::load_from_file_with_dir(lang, &get_langs_dir("cli"))
    }

    fn load_from_file_with_dir(lang: &str, dir: &Path) -> Option<Self> {
        let lang_path = dir.join(format!("{}.json", lang));
        if let Ok(file_content) = fs::read_to_string(&lang_path) {
            let lang_json: serde_json::Value = serde_json::from_str(&file_content).ok()?;
            let mut default_json: serde_json::Value =
                serde_json::from_str(include_str!("i18n_assets/cli/en_us.json")).ok()?;
            if let (Some(lang_obj), Some(default_obj)) =
                (lang_json.as_object(), default_json.as_object_mut())
            {
                for (k, v) in lang_obj {
                    default_obj.insert(k.clone(), v.clone());
                }
            }
            if let Ok(labels) = serde_json::from_value::<Self>(default_json) {
                return Some(labels);
            }
        }
        None
    }

    pub fn load_or_default(lang: &str) -> Self {
        Self::load_or_default_with_dir(lang, &get_langs_dir("cli"))
    }

    fn load_or_default_with_dir(lang: &str, dir: &Path) -> Self {
        if let Some(loaded) = Self::load_from_file_with_dir(lang, dir) {
            return loaded;
        }
        Self::default_zh_tw()
    }

    pub fn default_zh_tw() -> Self {
        serde_json::from_str(include_str!("i18n_assets/cli/zh_tw.json")).unwrap()
    }

    pub fn get_available_ui_langs() -> Vec<String> {
        let mut langs_list = Vec::new();
        if let Ok(es) = std::fs::read_dir(get_langs_dir("cli")) {
            for e in es.flatten() {
                if e.path().extension().is_some_and(|x| x == "json") {
                    if let Some(stem) = e.path().file_stem() {
                        langs_list.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }
        if langs_list.is_empty() {
            langs_list.push("zh_tw".to_string());
        }
        langs_list.sort();
        langs_list
    }
}

// --- 輔助載入規劃 ---
fn get_langs_dir(sub: &str) -> std::path::PathBuf {
    let cwd_langs = std::path::PathBuf::from("langs").join(sub);
    if cwd_langs.exists() {
        return cwd_langs;
    }
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        let exe_langs = exe_path.join("langs").join(sub);
        if exe_langs.exists() {
            return exe_langs;
        }
    }
    std::path::PathBuf::from("langs").join(sub)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_gui_labels_methods() {
        let t_dir = tempdir().expect("Failed to create temp dir");
        let gui_dir = t_dir.path().join("gui");

        let _ = GuiLabels::ensure_langs_exists_with_dir(&gui_dir);

        let def = GuiLabels::load_or_default_with_dir("zh_tw", &gui_dir);
        assert!(!def.common.app_title.is_empty());

        let en_labels = GuiLabels::load_or_default_with_dir("en_us", &gui_dir);
        assert!(!en_labels.common.app_title.is_empty());

        let bad = GuiLabels::load_or_default_with_dir("invalid_lang", &gui_dir);
        assert!(!bad.common.app_title.is_empty());
    }

    #[test]
    fn test_cli_labels_methods() {
        let t_dir = tempdir().unwrap();
        let cli_dir = t_dir.path().join("cli");

        let _ = CliLabels::ensure_langs_exists_with_dir(&cli_dir);

        let def = CliLabels::load_or_default_with_dir("zh_tw", &cli_dir);
        assert!(!def.common.app_title.is_empty());
    }

    #[test]
    fn test_common_labels_methods() {
        let t_dir = tempdir().unwrap();
        let gui_dir = t_dir.path().join("gui");

        let _ = GuiLabels::ensure_langs_exists_with_dir(&gui_dir);
        let l = CommonLabels::load_or_default_with_dir("zh_tw", &gui_dir);
        assert!(!l.app_title.is_empty());
    }

    #[test]
    fn debug_gui_labels_err() {
        let assets = [
            ("en_us", include_str!("i18n_assets/gui/en_us.json")),
            ("zh_cn", include_str!("i18n_assets/gui/zh_cn.json")),
            ("ja_jp", include_str!("i18n_assets/gui/ja_jp.json")),
        ];

        for (lang, file_content) in assets {
            let lang_json: serde_json::Value = serde_json::from_str(file_content).unwrap();
            let mut default_json: serde_json::Value =
                serde_json::from_str(include_str!("i18n_assets/gui/zh_tw.json")).unwrap();

            if let (Some(l_obj), Some(d_obj)) =
                (lang_json.as_object(), default_json.as_object_mut())
            {
                for (k, v) in l_obj {
                    d_obj.insert(k.clone(), v.clone());
                }
            }
            let res = serde_json::from_value::<GuiLabels>(default_json);
            if let Err(e) = res {
                panic!("Deserialization Error for {}: {:?}", lang, e);
            }
        }
    }

    #[test]
    fn test_get_langs_dir_fallback() {
        let sub = "gui";
        let t_dir = tempdir().unwrap();
        let langs_dir = t_dir.path().join("langs");
        fs::create_dir_all(langs_dir.join(sub)).unwrap();
    }

    #[test]
    fn test_common_labels_fallbacks() {
        let t_dir = tempdir().unwrap();
        let gui_dir = t_dir.path().join("gui");
        let _ = GuiLabels::ensure_langs_exists_with_dir(&gui_dir);

        let path = gui_dir.join("en_us.json");
        fs::remove_file(path).unwrap();

        let l = CommonLabels::load_or_default_with_dir("en_us", &gui_dir);
        assert!(!l.app_title.is_empty());
    }

    #[test]
    fn test_gui_labels_fallbacks() {
        let t_dir = tempdir().unwrap();
        let gui_dir = t_dir.path().join("gui");
        let _ = GuiLabels::ensure_langs_exists_with_dir(&gui_dir);

        let path = gui_dir.join("en_us.json");
        fs::remove_file(path).unwrap();

        let l = GuiLabels::load_or_default_with_dir("en_us", &gui_dir);
        assert!(!l.common.app_title.is_empty());
    }

    #[test]
    fn test_cli_labels_fallbacks() {
        let t_dir = tempdir().unwrap();
        let cli_dir = t_dir.path().join("cli");
        let _ = CliLabels::ensure_langs_exists_with_dir(&cli_dir);

        let path = cli_dir.join("en_us.json");
        fs::remove_file(path).unwrap();

        let l = CliLabels::load_or_default_with_dir("en_us", &cli_dir);
        assert!(!l.common.app_title.is_empty());
    }

    #[test]
    fn test_i18n_corrupt_json_fallback_logic() {
        let corrupt_json: serde_json::Value = serde_json::json!({
            "app_title": 12345
        });

        let mut default_json: serde_json::Value =
            serde_json::from_str(include_str!("i18n_assets/gui/en_us.json")).unwrap();

        if let (Some(lang_obj), Some(default_obj)) =
            (corrupt_json.as_object(), default_json.as_object_mut())
        {
            for (k, v) in lang_obj {
                default_obj.insert(k.clone(), v.clone());
            }
        }

        let res = serde_json::from_value::<CommonLabels>(default_json);
        assert!(res.is_err(), "應該因為 app_title 是 Number 而解析失敗");
    }

    #[test]
    fn test_ensure_assets_alignment() {
        let gui_assets = [
            include_str!("i18n_assets/gui/zh_tw.json"),
            include_str!("i18n_assets/gui/zh_cn.json"),
            include_str!("i18n_assets/gui/en_us.json"),
            include_str!("i18n_assets/gui/ja_jp.json"),
        ];
        for content in gui_assets {
            let _: GuiLabels = serde_json::from_str(content).unwrap();
        }

        let cli_assets = [
            include_str!("i18n_assets/cli/zh_tw.json"),
            include_str!("i18n_assets/cli/zh_cn.json"),
            include_str!("i18n_assets/cli/en_us.json"),
            include_str!("i18n_assets/cli/ja_jp.json"),
        ];
        for content in cli_assets {
            let _: CliLabels = serde_json::from_str(content).unwrap();
        }
    }
}
