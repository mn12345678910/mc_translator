use serde::{Deserialize, Serialize};
use std::fs;

 
pub const DEFAULT_LANG: &str = "zh_tw";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default = "I18nLabels::default_zh_tw")]
pub struct I18nLabels {
    // --- 頂部導覽與標題 ---
    pub app_title: String,
    pub btn_nav_settings: String,
    pub btn_nav_dict: String,
    pub btn_nav_palette: String,
    pub btn_nav_theme: String,
    pub btn_nav_dev: String,

    // --- 檔案選擇與路徑 ---
    pub btn_select_file: String,
    pub btn_select_folder: String,
    pub btn_output_dir: String,
    pub btn_open_output: String,
    pub label_output_path: String,
    pub label_default_path: String,
    pub dialog_filter_jar_json_js: String,
    pub label_input_path: String,
    pub label_ui_lang: String,

    // --- 翻譯操作 ---
    pub btn_run_trans: String,
    pub btn_pause: String,
    pub btn_stop: String,
    pub btn_clear_log: String,
    pub status_ready: String,
    pub status_processing: String,
    pub status_paused: String,
    pub label_current_status: String,
    pub label_current_file: String,
    pub label_global_progress: String,
    pub label_log_area: String,

    // --- 設定面板 ---
    pub header_api_settings: String,
    pub label_provider: String,
    pub label_model: String,
    pub label_api_key: String,
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

    // --- 狀態與提示 ---
    pub label_user_prompt: String,
    pub label_api_status: String,
    pub status_connected: String,
    pub status_not_ready: String,
    pub prompt_enter_key: String,
    pub prompt_select_model: String,
    pub prompt_update_list: String,

    // --- 調色盤與主題 ---
    pub header_palette: String,
    pub label_edit_mode: String,
    pub mode_light: String,
    pub mode_dark: String,
    pub label_edit_target: String,
    pub btn_add_target: String,
    pub btn_reset_all: String,
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
    pub log_pause_requested: String,
    pub log_stopped: String,
    pub status_stopped: String,

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
    pub label_disable_log: String,
    pub label_system_prompt: String,
    pub title_confirm_clear_log: String,
    pub text_confirm_clear_log: String,
    pub btn_confirm_clear_log: String,
    pub label_provider_none: String,
    pub label_ollama_url: String,
    pub label_presets: String,
    pub log_log_cleared: String,

    // --- 類別名稱 ---
    pub group_batch: String,
    pub group_specific: String,

    // --- 特定元件名稱 ---
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

    // --- 建議詞管理器 ---
    pub glossary_title: String,
    pub glossary_desc: String,
    pub glossary_tab_user: String,
    pub glossary_tab_official: String,
    pub btn_add: String,
    pub btn_replace: String,
    pub btn_import: String,
    pub btn_export: String,
    pub btn_clear_all: String,
    pub glossary_add_title: String,
    pub glossary_key: String,
    pub glossary_value: String,
    pub btn_confirm_add: String,
    pub glossary_replace_title: String,
    pub glossary_replace_desc: String,
    pub glossary_old_value: String,
    pub glossary_new_value: String,
    pub glossary_replace_exact: String,
    pub btn_confirm_replace: String,
    pub glossary_clear_title: String,
    pub glossary_clear_desc: String,
    pub btn_confirm_clear: String,
    pub label_search: String,
    pub glossary_page_info: String,
    pub glossary_priority_hover: String,
    pub glossary_priority_user: String,
    pub glossary_priority_official: String,
    pub glossary_col_actions: String,
    pub glossary_empty: String,
    pub btn_edit: String,
    pub btn_delete: String,
    pub btn_save: String,

    // --- 更多日誌與狀態 ---
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
    
    // --- CLI 獨佔提示 ---
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

    // --- 提示與回饋日誌 [NEW] ---
    pub header_dict_mgr: String,
    pub placeholder_input_path: String,
    pub placeholder_output_dir: String,
    pub placeholder_api_key: String,
    pub placeholder_search_terms: String,
    pub placeholder_dict_key: String,
    pub placeholder_dict_value: String,
    pub label_dict_official: String,
    pub label_dict_user: String,
    pub label_page_info: String,
    pub btn_page_prev: String,
    pub btn_page_next: String,
    pub label_loading_models: String,
    pub label_no_models: String,
    pub label_none_provider: String,

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
    pub label_glossary_priority: String,
    pub label_palette_target_type: String,
    pub label_palette_target_item: String,
    pub label_palette_property: String,
    pub label_palette_color: String,
    pub label_palette_rounding: String,
    pub label_global_rounding: String,
    pub label_pulse_speed: String,
    pub btn_save_config: String,
    pub btn_save_style: String,
    pub label_items: String,
    pub label_files: String,
}

impl I18nLabels {
    fn get_langs_dir() -> std::path::PathBuf {
        let cwd_langs = std::path::PathBuf::from("langs");
        if cwd_langs.exists() {
            return cwd_langs;
        }
        if let Ok(mut exe_path) = std::env::current_exe() {
            exe_path.pop();
            let exe_langs = exe_path.join("langs");
            if exe_langs.exists() {
                return exe_langs;
            }
        }
        cwd_langs
    }

    /// 確保 langs/ 目錄與預設 JSON 檔案存在
    pub fn ensure_langs_exists() -> Result<(), Box<dyn std::error::Error>> {
        let langs_dir = Self::get_langs_dir();
        if !langs_dir.exists() {
            fs::create_dir_all(&langs_dir).map_err(std::io::Error::other)?;
        }

        let zh_tw_path = langs_dir.join("zh_tw.json");
        let labels = Self::default_zh_tw();
        let json = serde_json::to_string_pretty(&labels).map_err(std::io::Error::other)?;
        fs::write(zh_tw_path, json).map_err(std::io::Error::other)?;

        let default_files = [
            ("zh_cn.json", include_str!("../i18n_assets/zh_cn.json")),
            ("en_us.json", include_str!("../i18n_assets/en_us.json")),
            ("ja_jp.json", include_str!("../i18n_assets/ja_jp.json")),
        ];

        for (name, content) in default_files {
            let p = langs_dir.join(name);
            fs::write(p, content).map_err(std::io::Error::other)?;
        }

        Ok(())
    }

    /// 從檔案載入 i18n
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let path = Self::get_langs_dir().join(format!("{}.json", lang));
        if let Ok(content) = fs::read_to_string(path) {
            match serde_json::from_str::<Self>(&content) {
                Ok(labels) => return Some(labels),
                Err(e) => {
                    println!("-> [Detail Error] {} parse failed: {}", lang, e);
                }
            }
        }
        None
    }

    /// 根據目標語言載入，若失敗則回退至預設 zh_tw
    pub fn load_or_default(lang: &str) -> Self {
        if let Some(labels) = Self::load_from_file(lang) {
            return labels;
        }
    // 若指定語言無效，嘗試載入 zh_tw
        if lang != "zh_tw" {
            if let Some(zh_tw) = Self::load_from_file("zh_tw") {
                return zh_tw;
            }
        }
        // 最後回退至內置的 default_zh_tw
        Self::default_zh_tw()
    }

    pub fn get_available_ui_langs() -> Vec<String> {
        let mut langs = Vec::new();
        let langs_dir = Self::get_langs_dir();
        if let Ok(entries) = std::fs::read_dir(&langs_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Some(stem) = entry.path().file_stem() {
                            langs.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        if langs.is_empty() {
            langs.push("zh_tw".to_string());
        }
        langs.sort();
        langs
    }

    pub fn default_zh_tw() -> Self {
        Self {
            app_title: "Minecraft 模組翻譯器".to_string(),
            btn_nav_settings: "⚙ API 翻譯設定".to_string(),
            btn_nav_dict: "📖 建議詞管理器".to_string(),
            btn_nav_palette: "🎨 自定義調色盤".to_string(),
            btn_nav_theme: "🌓 切換主題".to_string(),
            btn_nav_dev: "🔧 開發人員模式".to_string(),

            btn_select_file: "📁 選擇檔案".to_string(),
            btn_select_folder: "📂 選擇資料夾".to_string(),
            btn_output_dir: "📤 輸出資料夾".to_string(),
            btn_open_output: "📂 打開輸出".to_string(),
            label_output_path: "輸出路徑: ".to_string(),
            label_default_path: "預設: ./LLMTranslator".to_string(),
            dialog_filter_jar_json_js: "JAR, JS & JSON 檔案".to_string(),
            label_input_path: "待翻譯路徑:".to_string(),
            label_ui_lang: "介面語言:".to_string(),

            btn_run_trans: "🚀 開始翻譯".to_string(),
            btn_pause: "⏸ 暫停".to_string(),
            btn_stop: "⏹ 停止".to_string(),
            btn_clear_log: "🧹 清除執行日誌".to_string(),
            status_ready: "就緒".to_string(),
            status_processing: "正在翻譯...".to_string(),
            status_paused: "已暫停".to_string(),
            label_current_status: "目前狀態: ".to_string(),
            label_current_file: "條目進度: ".to_string(),
            label_global_progress: "檔案完成度: ".to_string(),
            label_log_area: "執行日誌:".to_string(),

            header_api_settings: "⚙ API 服務設定".to_string(),
            label_provider: "服務商:".to_string(),
            label_model: "選擇模型:".to_string(),
            label_api_key: "API Key:".to_string(),
            label_batch_size: "批次量:".to_string(),
            label_max_chars: "上限:".to_string(),
            label_timeout: "逾時:".to_string(),
            label_font_size: "字體:".to_string(),
            label_pack_format: "資源包版本:".to_string(),
            label_source_lang: "來源語言:".to_string(),
            label_target_lang: "目標語言:".to_string(),
            btn_resume: "▶ 繼續".to_string(),
            hover_select_file_first: "請先選取檔案或資料夾".to_string(),
            hover_select_model_first: "請先於設定中選取翻譯模型".to_string(),
            title_confirm_stop: "⚠ 確認停止翻譯".to_string(),
            text_confirm_stop: "確定要停止翻譯嗎？此操作無法復原。".to_string(),
            btn_confirm_stop: "確定停止".to_string(),
            log_pause_requested: ">>> 使用者請求暫停...".to_string(),
            log_stopped: ">>> 翻譯已中斷。".to_string(),
            status_stopped: "已中止".to_string(),

            header_dev_mode: "🔧 開發人員模式".to_string(),
            label_skip_json: "跳過 .json".to_string(),
            label_no_skip_json: "不跳過 .json".to_string(),
            label_skip_jar: "跳過 .jar".to_string(),
            label_no_skip_jar: "不跳過 .jar".to_string(),
            label_skip_js: "跳過 .js".to_string(),
            label_no_skip_js: "不跳過 .js".to_string(),
            label_skip_book: "跳過手冊".to_string(),
            label_no_skip_book: "不跳過手冊".to_string(),
            label_enable_log: "開啟記錄日誌".to_string(),
            label_disable_log: "關閉記錄日誌".to_string(),
            label_system_prompt: "📜 系統技術指令".to_string(),
            title_confirm_clear_log: "⚠ 確認清除日誌".to_string(),
            text_confirm_clear_log: "確定要清除目前所有的執行日誌嗎？\n此操作無法復原。".to_string(),
            btn_confirm_clear_log: "確定清除".to_string(),
            label_provider_none: "無".to_string(),
            label_ollama_url: "Ollama URL".to_string(),
            label_presets: "常用版本".to_string(),
            log_log_cleared: ">>> 使用者已清除執行日誌。".to_string(),

            btn_restore_defaults: "⟲ 恢復預設".to_string(),
            btn_confirm_restore: "確定恢復".to_string(),
            btn_cancel: "取消".to_string(),
            confirm_restore_title: "確認恢復預設".to_string(),
            confirm_restore_text: "您確定要將所有設定恢復為系統預設值嗎？\n這將覆蓋您目前的所有設定。".to_string(),

            label_user_prompt: "📝 使用者翻譯提示:".to_string(),
            label_api_status: "🔍 API 連線狀態:".to_string(),
            status_connected: "[已連線]".to_string(),
            status_not_ready: "[未就緒]".to_string(),
            prompt_enter_key: "請輸入 API 金鑰".to_string(),
            prompt_select_model: "請選取模型".to_string(),
            prompt_update_list: "請先更新列表...".to_string(),

            header_palette: "🎨 調色盤管理".to_string(),
            label_edit_mode: "當前編輯模式:".to_string(),
            mode_light: "☀️ 淺色設定".to_string(),
            mode_dark: "🌙 深色設定".to_string(),
            label_edit_target: "選擇編輯目標:".to_string(),
            btn_add_target: "+ 新增目標".to_string(),
            btn_reset_all: "⟲ 全部重置".to_string(),
            hover_reset_all: "將全程式所有設定、顏色、圓角恢復至預設值".to_string(),
            label_style_attr: "調整屬性樣式:".to_string(),
            label_palette_step_1: "【 1. 選擇編輯目標 】".to_string(),
            label_palette_step_2: "【 2. 調整屬性樣式 】".to_string(),
            label_slot_count: "共 {} 個槽位".to_string(),
            label_bg_color: "背景顏色".to_string(),
            label_text_color: "文字顏色".to_string(),
            label_custom_rounding: "自定義圓角".to_string(),
            label_force_global_rounding: "強制啟用全域按鈕圓角".to_string(),
            label_rounding_value: "圓角數值".to_string(),
            label_enable_pulse: "啟用進度條呼吸脈衝動畫".to_string(),
            label_anim_speed: "動畫速度:".to_string(),
            label_palette_hint: "ℹ 提示：特定元件覆寫的色彩優先級高於類別批量設定。".to_string(),
            hover_remove_slot: "移除此編輯槽位".to_string(),

            group_batch: "類別批量設定".to_string(),
            group_specific: "特定元件 (精確覆寫)".to_string(),

            spec_btn_select_file: "[特定] 選擇檔案按鈕".to_string(),
            spec_btn_select_folder: "[特定] 選擇資料夾按鈕".to_string(),
            spec_btn_output_dir: "[特定] 輸出資料夾按鈕".to_string(),
            spec_btn_open_output: "[特定] 打開輸出按鈕".to_string(),
            spec_btn_run_trans: "[特定] 開始翻譯按鈕".to_string(),
            spec_btn_pause: "[特定] 暫停按鈕".to_string(),
            spec_btn_stop: "[特定] 停止按鈕".to_string(),
            spec_btn_clear_log: "[特定] 清除執行日誌按鈕".to_string(),
            spec_btn_nav_settings: "[特定] ⚙ 設定按鈕".to_string(),
            spec_btn_nav_dict: "[特定] 📖 字典按鈕".to_string(),
            spec_btn_nav_palette: "[特定] 🎨 調色盤按鈕".to_string(),
            spec_btn_nav_theme: "[特定] 🌓 主題按鈕".to_string(),
            spec_btn_nav_dev: "[特定] 🔧 開發按鈕".to_string(),
            spec_input_search: "[特定] 建議詞搜尋框".to_string(),
            spec_area_dict: "[特定] 字典列表區域".to_string(),
            spec_label_output: "[特定] 輸出路徑標籤".to_string(),
            spec_progress_current: "[特定] 目前檔案進度條".to_string(),
            spec_progress_total: "[特定] 總進度條".to_string(),

            glossary_title: "📖 建議詞管理器".to_string(),
            glossary_desc: "存在裡面的文字將作為術語表建議 LLM 如何翻譯該文字（僅建議，不一定會使用）".to_string(),
            glossary_tab_user: "📝 使用者建議詞".to_string(),
            glossary_tab_official: "📚 官方建議詞".to_string(),
            btn_add: "➕ 新增".to_string(),
            btn_replace: "🔄 取代".to_string(),
            btn_import: "📥 匯入".to_string(),
            btn_export: "📤 匯出".to_string(),
            btn_clear_all: "🗑 清空全部".to_string(),
            glossary_add_title: "➕ 新增建議詞".to_string(),
            glossary_key: "原文 (Key):".to_string(),
            glossary_value: "翻譯 (Value):".to_string(),
            btn_confirm_add: "確定新增".to_string(),
            glossary_replace_title: "🔄 批量取代翻譯".to_string(),
            glossary_replace_desc: "將目前分頁中所有符合的翻譯內容進行取代。".to_string(),
            glossary_old_value: "原Value:".to_string(),
            glossary_new_value: "新Value:".to_string(),
            glossary_replace_exact: "全部符合才取代".to_string(),
            btn_confirm_replace: "執行取代".to_string(),
            glossary_clear_title: "⚠ 確認清空字典".to_string(),
            glossary_clear_desc: "確定刪除全部內容？此操作無法復原。".to_string(),
            btn_confirm_clear: "確定清空".to_string(),
            label_search: "🔍 搜尋:".to_string(),
            glossary_page_info: "第 {}/{} 頁 (顯示 {}-{}/{})".to_string(),
            glossary_priority_hover: "切換 官方優先 (關) 或 使用者優先 (開)".to_string(),
            glossary_priority_user: "使用者優先".to_string(),
            glossary_priority_official: "官方優先".to_string(),
            glossary_col_actions: "操作".to_string(),
            glossary_empty: "(目前的字典分頁是空的)".to_string(),
            btn_edit: "✏".to_string(),
            btn_delete: "🗑".to_string(),
            btn_save: "💾".to_string(),

            status_analyzing_dict: "正在分析辭典...".to_string(),
            status_analyzing_files: "正在分析檔案".to_string(),
            status_cancelled: "任務已取消".to_string(),
            status_error: "發生錯誤: {}".to_string(),
            status_finished: "任務完成".to_string(),
            log_finished: ">>> 所有翻譯任務已完成！".to_string(),
            log_cancelled: ">>> 任務已由使用者手動取消。".to_string(),
            log_start_job: ">>> 啟動翻譯任務 | 服務商: {} | 模型: {}".to_string(),
            log_start_failed: "⚠ 啟動失敗：目前服務商 [{}] 需要選取模型。".to_string(),
            log_resuming: ">>> 使用者恢復翻譯任務中...".to_string(),
            log_generic_error: ">>> 錯誤: {}".to_string(),
            log_batch_error: "批次翻譯出錯: {}, 將改用單筆重試".to_string(),
            log_loop_detected: "⚠ 偵測到條目 ({}) 陷入翻譯循環，已跳過".to_string(),
            status_processing_item: "正在處理 {} ({}/{})".to_string(),
            status_processing_batch: "正在翻譯批次 ({}/{})，檔案 ({})".to_string(),
            status_translating_item: "正在翻譯條目 ({})".to_string(),
            log_batch_failed_retry: "⚠ 批次 {}/{} 翻譯失敗: {}, 已加入重試佇列".to_string(),
            log_retry_start: ">>> 開始失敗批次重試 ({} 條)...".to_string(),
            log_single_failed: "單筆翻譯失敗: {}".to_string(),
            log_single_retry_start: ">>> 開始最終單筆重試 ({} 條)...".to_string(),
            log_single_final_failed: "❌ 條目翻譯最終失敗: {}".to_string(),
            status_retry: "重試".to_string(),
            status_translating: "翻譯".to_string(),
            status_translating_batch_simple: "{}中 (批次 {}/{})".to_string(),
            status_translating_batch: "{}中 ({}/{})".to_string(),
            log_batch_invalid: "批次翻譯結果無效，無法解析任何條目".to_string(),
            status_processing_label: "正在處理: ".to_string(),
            log_selected_files: "已選擇 {} 個檔案".to_string(),
            log_output_dir_set: "輸出資料夾已設定: {}".to_string(),
            status_idle: "待機中".to_string(),
            status_scanning_files: "正在掃描檔案...".to_string(),
            log_processing_finished: "處理完成並寫入目標。".to_string(),
            log_generating_pack: "正在生成資源包 (LLMTranslator.zip)...".to_string(),
            log_pack_item_exists_warn: "警告：已存在相同的資源包檔案 {}，將會被直接覆蓋。".to_string(),
            log_pack_gen_finished: "資源包 (LLMTranslator.zip) 生成完成。".to_string(),
            default_user_prompt: "你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。".to_string(),
            default_system_prompt: "\n\n[內部技術指令 - 請務必遵守]\n1. 僅針對 %%VAR_n%%, %%MC_n%%, %%HEX_n%% 等技術佔位符執行「保持原樣」操作（不可修改、翻譯或增刪標籤）。\n2. 除上述佔位符外的其餘文本內容均「必須」按要求翻譯，絕對不可將全文原樣輸出。".to_string(),
            
            prompt_select_provider_cli: "請選擇 API 提供商".to_string(),
            prompt_advanced_settings_cli: "是否需要調整進階設定？".to_string(),
            prompt_confirm_start_cli: "確定要開始執行翻譯嗎？".to_string(),
            prompt_task_finished_cli: "任務已結束，請問接下來？".to_string(),
            prompt_new_task_cli: "開啟新翻譯任務".to_string(),
            label_back_to_prev_cli: "<- 回到上一步".to_string(),
            label_custom_input_cli: "自訂輸入...".to_string(),
            label_yes_confirm_cli: "[是] 我確定".to_string(),
            label_no_cancel_cli: "[否] 取消離開".to_string(),

            cli_banner_title: "=== Minecraft 模組翻譯工具 - CLI 模式 ===".to_string(),
            cli_mode_headless: "-> 偵測到指令參數，進入靜態 Headless 模式...".to_string(),
            cli_mode_interactive: "-> 未偵測到輸入檔案參數，進入互動選項模式...\n".to_string(),
            cli_select_ui_lang: "請選擇介面語言 / Select UI Language".to_string(),
            cli_fetching_models: "-> 正在獲取 {} 模型列表...".to_string(),
            cli_model_fetch_failed: " (⚠️ 無法動態獲取清單，請確認連線/APIKey)".to_string(),
            cli_custom_model_prompt: "請輸入自訂模型名稱 (鍵入 '<' 為返回選單)".to_string(),
            cli_input_path_prompt: "請選取要翻譯的檔案/資料夾路徑 (鍵入 '<' 回上一步)".to_string(),
            cli_error_path_not_exist: "❌ 錯誤: 輸入路徑不存在！".to_string(),
            cli_output_path_prompt: "{} [預設: {}] (鍵入 '<' 回上一步)".to_string(),
            cli_op_cancelled: "🔒 操作已取消。".to_string(),
            cli_adv_settings_synced: "💡 參數對齊：目前各項進階數值已由啟動參數/存檔加載！".to_string(),
            cli_starting_pipeline: "\n-> 正在啟動翻譯管線...\n".to_string(),
            cli_pipeline_ended: "\n\n-> 管線運作結束。".to_string(),
            cli_pipeline_success: "✅ 恭喜！所有翻譯任務已成功完成。".to_string(),
            cli_pipeline_failed: "❌ 失敗退出: {}".to_string(),

            // --- 提示與回饋日誌 [NEW] ---
            header_dict_mgr: "📖 字典管理器".to_string(),
            placeholder_input_path: "輸入或拖放路徑至此...".to_string(),
            placeholder_output_dir: "./LLMTranslator (預設)".to_string(),
            placeholder_api_key: "輸入 API Key...".to_string(),
            placeholder_search_terms: "🔍 搜尋術語...".to_string(),
            placeholder_dict_key: "原文 (Key)".to_string(),
            placeholder_dict_value: "翻譯 (Value)".to_string(),
            label_dict_official: "官方推論".to_string(),
            label_dict_user: "使用者詞庫".to_string(),
            label_page_info: "第 {} / {} 頁".to_string(),
            btn_page_prev: "上一頁".to_string(),
            btn_page_next: "下一頁".to_string(),
            label_loading_models: "載入中...".to_string(),
            label_no_models: "(無可用模型)".to_string(),
            label_none_provider: "無".to_string(),

            status_load_config_failed: "❌ 載入配置失敗: {}".to_string(),
            status_save_config_success: "✅ 核心參數儲存成功！".to_string(),
            status_save_config_failed: "❌ 儲存配置失敗: {}".to_string(),
            status_save_style_success: "🎨 調色盤與佈局保存成功！".to_string(),
            status_save_style_failed: "❌ 保存樣式失敗: {}".to_string(),
            status_browse_path_failed: "❌ 瀏覽路徑失敗: {}".to_string(),
            status_open_dir_failed: "❌ 無法打開資料夾: {}".to_string(),
            status_ui_lang_changed: "🌍 介面語言已變更為：{}".to_string(),
            status_theme_changed: "🌓 主題已切換為：{}".to_string(),
            status_restore_config_confirm: "確定要將參數恢復為預設值嗎？".to_string(),
            status_restore_config_success: "✅ 參數已恢復預設！".to_string(),
            status_restore_config_failed: "❌ 恢復參數失敗: {}".to_string(),
            status_restore_style_confirm: "確定要將外觀佈景恢復為預設嗎？".to_string(),
            status_restore_style_success: "🎨 外觀已恢復預設！".to_string(),
            status_restore_style_failed: "❌ 恢復樣式失敗: {}".to_string(),
            status_input_path_empty: "⚠️ 請輸入或選取待翻譯路徑！".to_string(),
            status_trans_starting: "🚀 翻譯任務開始發射...".to_string(),
            status_trans_command_sent: "✅ 任務執行指令送達後端。".to_string(),
            status_trans_error: "❌ 執行出錯: {}".to_string(),
            status_trans_paused: "任務已暫停。面板解鎖，可改動設定。".to_string(),
            status_trans_resumed: "▶️ 任務已繼續。".to_string(),
            status_trans_stopping: "⏹️ 正在送出終止信號...".to_string(),
            status_dict_item_updated: "📖 字典更新：{}".to_string(),
            status_dict_item_delete_confirm: "確定刪除條目 {} 嗎？".to_string(),
            status_dict_load_failed: "❌ 載入字典失敗: {}".to_string(),
            status_dict_key_empty: "原文 (Key) 不可為空".to_string(),
            status_dict_add_success: "➕ 已新增使用者建議詞: {} -> {}".to_string(),
            status_dict_add_failed: "❌ 新增失敗: {}".to_string(),
            status_dict_replace_empty: "請在 Key 填舊詞，Value 填新詞以進行批量取代。".to_string(),
            status_dict_replace_confirm: "確定把所有翻譯內容為 \"{}\" 取代為 \"{}\" 嗎？".to_string(),
            status_dict_replace_sent: "🔄 已送出批量取代或更新請求：{} -> {}".to_string(),
            status_dict_replace_failed: "❌ 取代失敗: {}".to_string(),
            status_dict_clear_success: "✅ 字典已清空！".to_string(),
            status_dict_import_success: "📥 字典匯入成功！".to_string(),
            status_dict_export_success: "📤 字典已匯出至: {}".to_string(),
            status_palette_clear_item: "🎨 已清除元件覆寫: {}".to_string(),
            err_ollama_connect: "無法連線至 Ollama，請檢查服務是否啟動。".to_string(),
            err_api_key_empty: "API Key 為空，請先填入 API Key。".to_string(),
            err_gemini_models: "無法取得 Gemini 模型列表，請檢查 API Key 或網路連線。".to_string(),
            err_openai_models: "無法取得 OpenAI 模型列表，請檢查 API Key 或網路連線。".to_string(),
            err_deepseek_models: "無法取得 DeepSeek 模型列表，請檢查 API Key 或網路連線。".to_string(),
            err_unsupported_provider: "未支援的提供商".to_string(),
            err_no_active_job: "無正在執行的翻譯任務".to_string(),
            lang_zh_tw: "繁體中文 (zh_tw)".to_string(),
            lang_zh_cn: "简体中文 (zh_cn)".to_string(),
            lang_ja_jp: "日本語 (ja_jp)".to_string(),
            lang_en_us: "English (en_us)".to_string(),
            label_glossary_priority: "術語優先級".to_string(),
            label_palette_target_type: "編輯對象".to_string(),
            label_palette_target_item: "選擇對象".to_string(),
            label_palette_property: "設定屬性".to_string(),
            label_palette_color: "調整顏色".to_string(),
            label_palette_rounding: "圓角數值".to_string(),
            label_global_rounding: "全局圓角".to_string(),
            label_pulse_speed: "進度條光暈速度".to_string(),
            btn_save_config: "💾 儲存核心參數".to_string(),
            btn_save_style: "💾 儲存佈景配置".to_string(),
            label_items: "條目".to_string(),
            label_files: "檔案".to_string(),
        }
    }
}
