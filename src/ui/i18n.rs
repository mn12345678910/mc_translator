//! # i18n 模組
//! 定義 UI 介面所使用的所有字串，便於多語系擴充與集中管理。

pub struct I18nLabels {
    // --- 頂部導覽與標題 ---
    pub app_title: &'static str,
    pub btn_nav_settings: &'static str,
    pub btn_nav_dict: &'static str,
    pub btn_nav_palette: &'static str,
    pub btn_nav_theme: &'static str,
    pub btn_nav_dev: &'static str,

    // --- 檔案選擇與路徑 ---
    pub btn_select_file: &'static str,
    pub btn_select_folder: &'static str,
    pub btn_output_dir: &'static str,
    pub btn_open_output: &'static str,
    pub label_output_path: &'static str,
    pub label_default_path: &'static str,
    pub dialog_filter_jar_json_js: &'static str,

    // --- 翻譯操作 ---
    pub btn_run_trans: &'static str,
    pub btn_pause: &'static str,
    pub btn_stop: &'static str,
    pub btn_clear_log: &'static str,
    pub status_ready: &'static str,
    pub status_processing: &'static str,
    pub status_paused: &'static str,

    // --- 設定面板 ---
    pub header_api_settings: &'static str,
    pub label_provider: &'static str,
    pub label_model: &'static str,
    pub label_api_key: &'static str,
    pub label_batch_size: &'static str,
    pub label_max_chars: &'static str,
    pub label_timeout: &'static str,
    pub label_font_size: &'static str,
    pub label_pack_format: &'static str,
    pub label_fps: &'static str,
    pub label_fps_preset_vsync: &'static str,
    pub btn_restore_defaults: &'static str,
    pub btn_confirm_restore: &'static str,
    pub btn_cancel: &'static str,
    pub confirm_restore_title: &'static str,
    pub confirm_restore_text: &'static str,

    // --- 狀態與提示 ---
    pub label_user_prompt: &'static str,
    pub label_api_status: &'static str,
    pub status_connected: &'static str,
    pub status_not_ready: &'static str,
    pub prompt_enter_key: &'static str,
    pub prompt_select_model: &'static str,
    pub prompt_update_list: &'static str,

    // --- 調色盤與主題 ---
    pub header_palette: &'static str,
    pub label_edit_mode: &'static str,
    pub mode_light: &'static str,
    pub mode_dark: &'static str,
    pub label_edit_target: &'static str,
    pub btn_add_target: &'static str,
    pub label_style_attr: &'static str,
}

pub const ZH_TW: I18nLabels = I18nLabels {
    app_title: "Minecraft 模組翻譯器",
    btn_nav_settings: "⚙ API 翻譯設定",
    btn_nav_dict: "📖 建議詞管理器",
    btn_nav_palette: "🎨 自定義調色盤",
    btn_nav_theme: "🌓 切換主題",
    btn_nav_dev: "🔧 開發人員模式",

    btn_select_file: "📁 選擇檔案",
    btn_select_folder: "📂 選擇資料夾",
    btn_output_dir: "📤 輸出資料夾",
    btn_open_output: "📂 打開輸出",
    label_output_path: "輸出路徑: ",
    label_default_path: "預設: ./LLMTranslator",
    dialog_filter_jar_json_js: "JAR, JS & JSON 檔案",

    btn_run_trans: "🚀 開始翻譯",
    btn_pause: "⏸ 暫停",
    btn_stop: "⏹ 停止",
    btn_clear_log: "🧹 清除執行日誌",
    status_ready: "就緒",
    status_processing: "正在翻譯...",
    status_paused: "已暫停",

    header_api_settings: "⚙ API 服務設定",
    label_provider: "服務商:",
    label_model: "選擇模型:",
    label_api_key: "API Key:",
    label_batch_size: "批次量:",
    label_max_chars: "上限:",
    label_timeout: "逾時:",
    label_font_size: "字體:",
    label_pack_format: "資源包版本:",
    label_fps: "FPS:",
    label_fps_preset_vsync: "(預設:vsync)",
    btn_restore_defaults: "⟲ 恢復預設",
    btn_confirm_restore: "確定恢復",
    btn_cancel: "取消",
    confirm_restore_title: "確認恢復預設",
    confirm_restore_text: "您確定要將所有設定恢復為系統預設值嗎？\n這將覆蓋您目前的所有設定。",

    label_user_prompt: "📝 使用者翻譯提示:",
    label_api_status: "🔍 API 連線狀態:",
    status_connected: "[已連線]",
    status_not_ready: "[未就緒]",
    prompt_enter_key: "請輸入 API 金鑰",
    prompt_select_model: "請選取模型",
    prompt_update_list: "請先更新列表...",

    header_palette: "🎨 調色盤管理",
    label_edit_mode: "當前編輯模式:",
    mode_light: "☀️ 淺色設定",
    mode_dark: "🌙 深色設定",
    label_edit_target: "選擇編輯目標:",
    btn_add_target: "+ 新增目標",
    label_style_attr: "調整屬性樣式:",
};

// 預設語言
pub const DEFAULT_LANG: &I18nLabels = &ZH_TW;
