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
}

impl CommonLabels {
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let dir = get_langs_dir("gui"); 
        let p = dir.join(format!("{}.json", lang));
        if let Ok(c) = fs::read_to_string(p) {
            let l_val: serde_json::Value = serde_json::from_str(&c).ok()?;
            let mut d_val: serde_json::Value = serde_json::from_str(include_str!("i18n_assets/gui/zh_tw.json")).ok()?;
            if let (Some(l), Some(d)) = (l_val.as_object(), d_val.as_object_mut()) { for (k,v) in l { d.insert(k.clone(),v.clone()); } }
            if let Ok(lb) = serde_json::from_value::<Self>(d_val) { return Some(lb); }
        }
        None
    }
    pub fn load_or_default(lang: &str) -> Self {
        if let Some(l) = Self::load_from_file(lang) { return l; }
        if lang != "zh_tw" { if let Some(z) = Self::load_from_file("zh_tw") { return z; } }
        Self::default_zh_tw()
    }
    pub fn default_zh_tw() -> Self { serde_json::from_str(include_str!("i18n_assets/gui/zh_tw.json")).unwrap() }
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
    if cwd_langs.exists() { return cwd_langs; }
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        let exe_langs = exe_path.join("langs").join(sub);
        if exe_langs.exists() { return exe_langs; }
    }
    std::path::PathBuf::from("langs").join(sub)
}

impl GuiLabels {
    pub fn ensure_langs_exists() -> Result<(), Box<dyn std::error::Error>> {
        let dir = get_langs_dir("gui");
        if !dir.exists() { fs::create_dir_all(&dir).map_err(std::io::Error::other)?; }
        let l = Self::default_zh_tw();
        fs::write(dir.join("zh_tw.json"), serde_json::to_string_pretty(&l).unwrap()).map_err(std::io::Error::other)?;
        let files = [("zh_cn.json", include_str!("i18n_assets/gui/zh_cn.json")),("en_us.json", include_str!("i18n_assets/gui/en_us.json")),("ja_jp.json", include_str!("i18n_assets/gui/ja_jp.json"))];
        for (n, c) in files { fs::write(dir.join(n), c).map_err(std::io::Error::other)?; }
        Ok(())
    }
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let p = get_langs_dir("gui").join(format!("{}.json", lang));
        if let Ok(c) = fs::read_to_string(p) {
            let l_val: serde_json::Value = serde_json::from_str(&c).ok()?;
            let mut d_val: serde_json::Value = serde_json::from_str(include_str!("i18n_assets/gui/zh_tw.json")).ok()?;
            if let (Some(l), Some(d)) = (l_val.as_object(), d_val.as_object_mut()) { for (k,v) in l { d.insert(k.clone(),v.clone()); } }
            if let Ok(lb) = serde_json::from_value::<Self>(d_val) { return Some(lb); }
        }
        None
    }
    pub fn load_or_default(lang: &str) -> Self {
        if let Some(l) = Self::load_from_file(lang) { return l; }
        if lang != "zh_tw" { if let Some(z) = Self::load_from_file("zh_tw") { return z; } }
        Self::default_zh_tw()
    }
    pub fn default_zh_tw() -> Self { serde_json::from_str(include_str!("i18n_assets/gui/zh_tw.json")).unwrap() }
    pub fn get_available_ui_langs() -> Vec<String> {
        let mut l = Vec::new();
        if let Ok(es) = std::fs::read_dir(get_langs_dir("gui")) {
            for e in es.flatten() {
                 if e.path().extension().is_some_and(|x| x == "json") { if let Some(s) = e.path().file_stem() { l.push(s.to_string_lossy().to_string()); } }
            }
        }
        if l.is_empty() { l.push("zh_tw".to_string()); }
        l.sort(); l
    }
}

impl CliLabels {
    pub fn ensure_langs_exists() -> Result<(), Box<dyn std::error::Error>> {
        let dir = get_langs_dir("cli");
        if !dir.exists() { fs::create_dir_all(&dir).map_err(std::io::Error::other)?; }
        let l = Self::default_zh_tw();
        fs::write(dir.join("zh_tw.json"), serde_json::to_string_pretty(&l).unwrap()).map_err(std::io::Error::other)?;
        let files = [("zh_cn.json", include_str!("i18n_assets/cli/zh_cn.json")),("en_us.json", include_str!("i18n_assets/cli/en_us.json")),("ja_jp.json", include_str!("i18n_assets/cli/ja_jp.json"))];
        for (n, c) in files { fs::write(dir.join(n), c).map_err(std::io::Error::other)?; }
        Ok(())
    }
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let p = get_langs_dir("cli").join(format!("{}.json", lang));
        if let Ok(c) = fs::read_to_string(p) {
            let l_val: serde_json::Value = serde_json::from_str(&c).ok()?;
            let mut d_val: serde_json::Value = serde_json::from_str(include_str!("i18n_assets/cli/zh_tw.json")).ok()?;
            if let (Some(l), Some(d)) = (l_val.as_object(), d_val.as_object_mut()) { for (k,v) in l { d.insert(k.clone(),v.clone()); } }
            if let Ok(lb) = serde_json::from_value::<Self>(d_val) { return Some(lb); }
        }
        None
    }
    pub fn load_or_default(lang: &str) -> Self {
        if let Some(l) = Self::load_from_file(lang) { return l; }
        if lang != "zh_tw" { if let Some(z) = Self::load_from_file("zh_tw") { return z; } }
        Self::default_zh_tw()
    }
    pub fn default_zh_tw() -> Self { serde_json::from_str(include_str!("i18n_assets/cli/zh_tw.json")).unwrap() }
    pub fn get_available_ui_langs() -> Vec<String> {
        let mut l = Vec::new();
        if let Ok(es) = std::fs::read_dir(get_langs_dir("cli")) {
            for e in es.flatten() {
                 if e.path().extension().is_some_and(|x| x == "json") { if let Some(s) = e.path().file_stem() { l.push(s.to_string_lossy().to_string()); } }
            }
        }
        if l.is_empty() { l.push("zh_tw".to_string()); }
        l.sort(); l
    }
}
