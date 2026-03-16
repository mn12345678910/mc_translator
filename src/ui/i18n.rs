use serde::{Deserialize, Serialize};
use std::fs;

 
pub const DEFAULT_LANG: &str = "zh_tw";

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub label_fps: String,
    pub label_fps_preset_vsync: String,
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
    pub cat_all_buttons: String,
    pub cat_all_labels: String,
    pub cat_all_inputs: String,
    pub cat_all_logs: String,
    pub cat_all_tabs: String,
    pub cat_all_progress: String,
    pub cat_all_bg: String,
    pub cat_nav_bar: String,

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
        if !zh_tw_path.exists() {
            let labels = Self::default_zh_tw();
            let json = serde_json::to_string_pretty(&labels).map_err(std::io::Error::other)?;
            fs::write(zh_tw_path, json).map_err(std::io::Error::other)?;
        }

        let default_files = [
            ("zh_cn.json", include_str!("../i18n_assets/zh_cn.json")),
            ("en_us.json", include_str!("../i18n_assets/en_us.json")),
            ("ja_jp.json", include_str!("../i18n_assets/ja_jp.json")),
        ];

        for (name, content) in default_files {
            let p = langs_dir.join(name);
            if !p.exists() {
                fs::write(p, content).map_err(std::io::Error::other)?;
            }
        }

        Ok(())
    }

    /// 從檔案載入 i18n
    pub fn load_from_file(lang: &str) -> Option<Self> {
        let path = Self::get_langs_dir().join(format!("{}.json", lang));
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(labels) = serde_json::from_str::<Self>(&content) {
                return Some(labels);
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

            label_fps: "FPS:".to_string(),
            label_fps_preset_vsync: "(預設:vsync)".to_string(),
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
            cat_all_buttons: "全部按鈕".to_string(),
            cat_all_labels: "全部標籤".to_string(),
            cat_all_inputs: "全部輸入框".to_string(),
            cat_all_logs: "全部日誌區域".to_string(),
            cat_all_tabs: "全部建議詞分頁".to_string(),
            cat_all_progress: "全部進度條".to_string(),
            cat_all_bg: "全部面板背景".to_string(),
            cat_nav_bar: "頂部導覽列".to_string(),

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
        }
    }
}
