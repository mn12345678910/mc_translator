use serde::{Deserialize, Serialize};
use std::fs;

pub const DEFAULT_LANG: &str = "zh_tw";

// --- 重構後之共通結構體 ---
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CommonLabels {
    pub app_title: String,
    pub label_output_path: String,
    pub dialog_filter_jar_json_js: String,
    pub btn_pause: String,
    pub btn_stop: String,
    pub btn_clear_log: String,
    pub status_ready: String,
    pub status_processing: String,
    pub status_paused: String,
    pub label_model: String,
    pub label_api_key: String,
    pub status_connected: String,
    pub status_not_ready: String,
    pub prompt_enter_key: String,
    pub prompt_select_model: String,
    pub prompt_update_list: String,
    pub btn_add_target: String,
    pub btn_reset_all: String,
    pub log_pause_requested: String,
    pub log_stopped: String,
    pub status_stopped: String,
    pub log_log_cleared: String,
    pub btn_add: String,
    pub btn_replace: String,
    pub btn_import: String,
    pub btn_export: String,
    pub btn_clear_all: String,
    pub btn_confirm_add: String,
    pub btn_confirm_replace: String,
    pub btn_confirm_clear: String,
    pub btn_edit: String,
    pub btn_delete: String,
    pub btn_save: String,
    pub status_analyzing_dict: String,
    pub status_analyzing_files: String,
    pub status_cancelled: String,
    pub status_error: String,
    pub status_finished: String,
    pub log_finished: String,
    pub log_cancelled: String,
    pub log_start_job: String,
    pub log_start_failed: String,
    pub log_resuming: String,
    pub log_generic_error: String,
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
    pub status_retry: String,
    pub status_translating: String,
    pub status_translating_batch_simple: String,
    pub status_translating_batch: String,
    pub log_batch_invalid: String,
    pub status_processing_label: String,
    pub log_selected_files: String,
    pub log_output_dir_set: String,
    pub status_idle: String,
    pub status_scanning_files: String,
    pub log_processing_finished: String,
    pub log_generating_pack: String,
    pub log_pack_item_exists_warn: String,
    pub log_pack_gen_finished: String,
    pub default_user_prompt: String,
    pub default_system_prompt: String,
    pub placeholder_input_path: String,
    pub placeholder_output_dir: String,
    pub placeholder_api_key: String,
    pub placeholder_search_terms: String,
    pub placeholder_dict_key: String,
    pub placeholder_dict_value: String,
    pub btn_page_prev: String,
    pub btn_page_next: String,
    pub status_load_config_failed: String,
    pub status_save_config_success: String,
    pub status_save_config_failed: String,
    pub status_save_style_success: String,
    pub status_save_style_failed: String,
    pub status_browse_path_failed: String,
    pub status_open_dir_failed: String,
    pub status_ui_lang_changed: String,
    pub status_theme_changed: String,
    pub status_restore_config_confirm: String,
    pub status_restore_config_success: String,
    pub status_restore_config_failed: String,
    pub status_restore_style_confirm: String,
    pub status_restore_style_success: String,
    pub status_restore_style_failed: String,
    pub status_input_path_empty: String,
    pub status_trans_starting: String,
    pub status_trans_command_sent: String,
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
    pub btn_save_config: String,
    pub btn_save_style: String,
    pub cat_all_bg: String,
    pub cat_all_buttons: String,
    pub cat_all_inputs: String,
    pub cat_all_labels: String,
    pub cat_all_logs: String,
    pub cat_all_progress: String,
    pub cat_all_tabs: String,
    pub cat_nav_bar: String,
    pub cat_all_btn_text: String,
    pub cat_all_tab_active: String,
    pub cat_all_tab_inactive: String,
    pub status_failed_or_cancelled: String,
    pub status_output_dir_empty: String,
    pub status_trans_failed_mask: String,
    pub status_progress_mask: String,
    pub status_batch_mask: String,
    #[serde(default)]
    pub cleanup_prefixes: Vec<String>,
    #[serde(default)]
    pub cleanup_contains: Vec<String>,
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
        if let Some(l) = Self::load_from_file(lang) {
            return l;
        }
        if lang != "en_us" {
            if let Some(e) = Self::load_from_file("en_us") {
                return e;
            }
        }
        Self::default_zh_tw()
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
    pub label_default_path: String,
    pub label_input_path: String,
    pub label_ui_lang: String,
    pub btn_run_trans: String,
    pub label_current_status: String,
    pub label_current_file: String,
    pub label_global_progress: String,
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
    pub btn_confirm_restore: String,
    pub btn_cancel: String,
    pub confirm_restore_title: String,
    pub confirm_restore_text: String,
    pub label_user_prompt: String,
    pub label_api_status: String,
    pub header_palette: String,
    pub label_edit_mode: String,
    pub mode_light: String,
    pub mode_dark: String,
    pub label_edit_target: String,
    pub hover_reset_all: String,
    pub label_style_attr: String,
    pub label_palette_step_1: String,
    pub label_palette_step_2: String,
    pub label_slot_count: String,
    pub label_bg_color: String,
    pub label_text_color: String,
    pub label_custom_rounding: String,
    pub label_force_global_rounding: String,
    pub label_rounding_value: String,
    pub label_enable_pulse: String,
    pub label_anim_speed: String,
    pub label_palette_hint: String,
    pub hover_remove_slot: String,
    pub btn_resume: String,
    pub hover_select_file_first: String,
    pub hover_select_model_first: String,
    pub title_confirm_stop: String,
    pub text_confirm_stop: String,
    pub btn_confirm_stop: String,
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
    pub label_llm_log: String,
    pub label_disable_log: String,
    pub label_system_prompt: String,
    pub title_confirm_clear_log: String,
    pub text_confirm_clear_log: String,
    pub btn_confirm_clear_log: String,
    pub label_provider_none: String,
    pub label_ollama_url: String,
    pub label_presets: String,
    pub group_batch: String,
    pub group_specific: String,
    pub spec_btn_select_file: String,
    pub spec_btn_select_folder: String,
    pub spec_btn_output_dir: String,
    pub spec_btn_open_output: String,
    pub spec_btn_run_trans: String,
    pub spec_btn_pause: String,
    pub spec_btn_stop: String,
    pub spec_btn_clear_log: String,
    pub spec_btn_nav_settings: String,
    pub spec_btn_nav_dict: String,
    pub spec_btn_nav_palette: String,
    pub spec_btn_nav_theme: String,
    pub spec_btn_nav_dev: String,
    pub spec_input_search: String,
    pub spec_area_dict: String,
    pub spec_label_output: String,
    pub spec_progress_current: String,
    pub spec_progress_total: String,
    pub glossary_title: String,
    pub glossary_desc: String,
    pub glossary_tab_user: String,
    pub glossary_tab_official: String,
    pub glossary_add_title: String,
    pub glossary_key: String,
    pub glossary_value: String,
    pub glossary_replace_title: String,
    pub glossary_replace_desc: String,
    pub glossary_old_value: String,
    pub glossary_new_value: String,
    pub glossary_replace_exact: String,
    pub glossary_clear_title: String,
    pub glossary_clear_desc: String,
    pub label_search: String,
    pub glossary_page_info: String,
    pub glossary_priority_hover: String,
    pub glossary_priority_user: String,
    pub glossary_priority_official: String,
    pub glossary_col_actions: String,
    pub glossary_empty: String,
    pub header_dict_mgr: String,
    pub label_dict_official: String,
    pub label_dict_user: String,
    pub label_page_info: String,
    pub label_none_provider: String,
    pub label_glossary_priority: String,
    pub label_palette_target_type: String,
    pub label_palette_target_item: String,
    pub label_palette_property: String,
    pub label_palette_color: String,
    pub label_palette_rounding: String,
    pub label_global_rounding: String,
    pub label_pulse_speed: String,
    pub label_items: String,
    pub label_files: String,
    pub label_fps: String,
    pub label_fps_preset_vsync: String,
    pub btn_dict_open_json: String,
    pub btn_palette_clear_item: String,
    pub label_progress_style: String,
    pub style_default: String,
    pub style_aurora: String,
    pub style_neon: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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
    pub cli_input_path_prompt: String,
    pub cli_error_path_not_exist: String,
    pub cli_output_path_prompt: String,
    pub cli_op_cancelled: String,
    pub cli_adv_settings_synced: String,
    pub cli_starting_pipeline: String,
    pub cli_pipeline_ended: String,
    pub cli_pipeline_success: String,
    pub cli_pipeline_failed: String,
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

impl GuiLabels {
    pub fn ensure_langs_exists() -> Result<(), Box<dyn std::error::Error>> {
        let dir = get_langs_dir("gui");
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(std::io::Error::other)?;
        }
        let zh_tw_path = dir.join("zh_tw.json");
        if cfg!(debug_assertions) || !zh_tw_path.exists() {
            let default_labels = Self::default_zh_tw();
            fs::write(
                &zh_tw_path,
                serde_json::to_string_pretty(&default_labels).unwrap(),
            )
            .map_err(std::io::Error::other)?;
        }
        let files = [
            ("zh_cn.json", include_str!("i18n_assets/gui/zh_cn.json")),
            ("en_us.json", include_str!("i18n_assets/gui/en_us.json")),
            ("ja_jp.json", include_str!("i18n_assets/gui/ja_jp.json")),
        ];
        for (file_name, file_content) in files {
            let file_path = dir.join(file_name);
            if cfg!(debug_assertions) || !file_path.exists() {
                fs::write(file_path, file_content).map_err(std::io::Error::other)?;
            }
        }
        Ok(())
    }
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let lang_path = get_langs_dir("gui").join(format!("{}.json", lang));
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
        if let Some(l) = Self::load_from_file(lang) {
            return l;
        }
        if lang != "en_us" {
            if let Some(e) = Self::load_from_file("en_us") {
                return e;
            }
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

impl CliLabels {
    pub fn ensure_langs_exists() -> Result<(), Box<dyn std::error::Error>> {
        let dir = get_langs_dir("cli");
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(std::io::Error::other)?;
        }
        let zh_tw_path = dir.join("zh_tw.json");
        if cfg!(debug_assertions) || !zh_tw_path.exists() {
            let default_labels = Self::default_zh_tw();
            fs::write(
                &zh_tw_path,
                serde_json::to_string_pretty(&default_labels).unwrap(),
            )
            .map_err(std::io::Error::other)?;
        }
        let files = [
            ("zh_cn.json", include_str!("i18n_assets/cli/zh_cn.json")),
            ("en_us.json", include_str!("i18n_assets/cli/en_us.json")),
            ("ja_jp.json", include_str!("i18n_assets/cli/ja_jp.json")),
        ];
        for (file_name, file_content) in files {
            let file_path = dir.join(file_name);
            if cfg!(debug_assertions) || !file_path.exists() {
                fs::write(file_path, file_content).map_err(std::io::Error::other)?;
            }
        }
        Ok(())
    }
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let lang_path = get_langs_dir("cli").join(format!("{}.json", lang));
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
        if let Some(l) = Self::load_from_file(lang) {
            return l;
        }
        if lang != "en_us" {
            if let Some(e) = Self::load_from_file("en_us") {
                return e;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_labels_methods() {
        let _ = GuiLabels::ensure_langs_exists();

        let def = GuiLabels::load_or_default("zh_tw");
        assert!(!def.common.app_title.is_empty());

        let en_labels = GuiLabels::load_or_default("en_us");
        assert!(!en_labels.common.app_title.is_empty());

        let bad = GuiLabels::load_or_default("invalid_lang");
        assert!(!bad.common.app_title.is_empty());

        let langs = GuiLabels::get_available_ui_langs();
        assert!(!langs.is_empty());
    }

    #[test]
    fn test_cli_labels_methods() {
        let _ = CliLabels::ensure_langs_exists();

        let def = CliLabels::load_or_default("zh_tw");
        assert!(!def.common.app_title.is_empty());

        let list = CliLabels::get_available_ui_langs();
        assert!(!list.is_empty());
    }

    #[test]
    fn test_common_labels_methods() {
        // 先確保檔案存在以免並行測試讀取失敗
        let _ = GuiLabels::ensure_langs_exists();

        let l = CommonLabels::load_or_default("zh_tw");
        assert!(!l.app_title.is_empty());

        let bad = CommonLabels::load_or_default("invalid");
        assert!(!bad.app_title.is_empty());
    }

    #[test]
    fn debug_common_labels_err() {
        let _ = GuiLabels::ensure_langs_exists();
        for lang in ["en_us", "zh_cn", "ja_jp"] {
            let file_content = fs::read_to_string(format!("langs/gui/{}.json", lang)).unwrap();
            let lang_json: serde_json::Value = serde_json::from_str(&file_content).unwrap();
            let mut default_json: serde_json::Value =
                serde_json::from_str(include_str!("i18n_assets/gui/zh_tw.json")).unwrap();
            if let (Some(l_obj), Some(d_obj)) =
                (lang_json.as_object(), default_json.as_object_mut())
            {
                for (k, v) in l_obj {
                    d_obj.insert(k.clone(), v.clone());
                }
            }
            let res = serde_json::from_value::<CommonLabels>(default_json);
            if let Err(e) = res {
                panic!("Deserialization Error for {}: {:?}", lang, e);
            }
        }
    }

    #[test]
    fn test_get_langs_dir_fallback() {
        let real_dir = std::path::PathBuf::from("langs");
        let backup = std::path::PathBuf::from("langs_backup");
        let exists = real_dir.exists();

        if exists {
            let _ = fs::rename(&real_dir, &backup);
        }

        // --- 新增：模擬 exe_langs 存在 ---
        if let Ok(mut exe_path) = std::env::current_exe() {
            exe_path.pop();
            let exe_langs = exe_path.join("langs").join("gui");
            let _ = fs::create_dir_all(&exe_langs);
        }

        let dir = get_langs_dir("gui");
        assert!(dir.to_string_lossy().contains("langs"));

        // --- 新增：測試 get_available_ui_langs() 當目錄為空且 fallback 觸發 ---
        let gui_langs = GuiLabels::get_available_ui_langs();
        let cli_langs = CliLabels::get_available_ui_langs();
        assert!(!gui_langs.is_empty());
        assert!(!cli_langs.is_empty());

        // --- 恢復與清理 ---
        if let Ok(mut exe_path) = std::env::current_exe() {
            exe_path.pop();
            let exe_langs = exe_path.join("langs");
            let _ = fs::remove_dir_all(&exe_langs);
        }

        if exists {
            let _ = fs::rename(&backup, &real_dir);
        }
    }

    #[test]
    fn test_common_labels_fallbacks() {
        let _ = GuiLabels::ensure_langs_exists();
        let path = get_langs_dir("gui").join("en_us.json");
        let backup = get_langs_dir("gui").join("en_us.json.bak");
        let exists = path.exists();
        if exists {
            let _ = fs::rename(&path, &backup);
        }

        let l = CommonLabels::load_or_default("en_us");
        assert!(!l.app_title.is_empty());

        if exists {
            let _ = fs::rename(&backup, &path);
        }

        // --- 新增：測最底階 fallback to default_zh_tw ---
        let zh_path = get_langs_dir("gui").join("zh_tw.json");
        let zh_bak = get_langs_dir("gui").join("zh_tw.json.bak");
        let zh_exists = zh_path.exists();
        if zh_exists {
            let _ = fs::rename(&zh_path, &zh_bak);
        }

        let l_def = CommonLabels::load_or_default("zh_tw");
        assert!(!l_def.app_title.is_empty());

        if zh_exists {
            let _ = fs::rename(&zh_bak, &zh_path);
        }
    }

    #[test]
    fn test_gui_labels_fallbacks() {
        let _ = GuiLabels::ensure_langs_exists();
        let path = get_langs_dir("gui").join("en_us.json");
        let backup = get_langs_dir("gui").join("en_us.json.bak");
        let exists = path.exists();
        if exists {
            let _ = fs::rename(&path, &backup);
        }

        let l = GuiLabels::load_or_default("en_us");
        assert!(!l.common.app_title.is_empty());

        if exists {
            let _ = fs::rename(&backup, &path);
        }

        // Gui extreme fallback
        let zh_path = get_langs_dir("gui").join("zh_tw.json");
        let zh_bak = get_langs_dir("gui").join("zh_tw.json.bak");
        let zh_exists = zh_path.exists();
        if zh_exists {
            let _ = fs::rename(&zh_path, &zh_bak);
        }

        let l_def = GuiLabels::load_or_default("zh_tw");
        assert!(!l_def.common.app_title.is_empty());

        if zh_exists {
            let _ = fs::rename(&zh_bak, &zh_path);
        }
    }

    #[test]
    fn test_cli_labels_fallbacks() {
        let _ = CliLabels::ensure_langs_exists();
        let path = get_langs_dir("cli").join("en_us.json");
        let backup = get_langs_dir("cli").join("en_us.json.bak");
        let exists = path.exists();
        if exists {
            let _ = fs::rename(&path, &backup);
        }

        let l = CliLabels::load_or_default("en_us");
        assert!(!l.common.app_title.is_empty());

        if exists {
            let _ = fs::rename(&backup, &path);
        }

        // Cli extreme fallback
        let zh_path = get_langs_dir("cli").join("zh_tw.json");
        let zh_bak = get_langs_dir("cli").join("zh_tw.json.bak");
        let zh_exists = zh_path.exists();
        if zh_exists {
            let _ = fs::rename(&zh_path, &zh_bak);
        }

        let l_def = CliLabels::load_or_default("zh_tw");
        assert!(!l_def.common.app_title.is_empty());

        if zh_exists {
            let _ = fs::rename(&zh_bak, &zh_path);
        }
    }

    #[test]
    fn test_i18n_corrupt_json_fallback() {
        // 建立毀損的 JSON 測試反序列化失敗 (ok? fallback)
        let _ = GuiLabels::ensure_langs_exists();
        let en_path = get_langs_dir("gui").join("en_us.json");
        let backup = get_langs_dir("gui").join("en_us.json.bak");

        if en_path.exists() {
            let _ = fs::rename(&en_path, &backup);
        }

        // 寫入類型錯誤的 JSON: app_title 預期型別 String，寫入 Number
        fs::write(&en_path, r#"{"app_title": 12345}"#).unwrap();

        // 測試載入 169 行: if let Ok(labels) = serde_json::from_value::<Self>(default_json) { -> None
        let res = CommonLabels::load_from_file("en_us");
        assert!(res.is_none());

        // 清理
        let _ = fs::remove_file(&en_path);
        if backup.exists() {
            let _ = fs::rename(&backup, &en_path);
        }
    }
}
