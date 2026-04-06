//! # 配置模組
//! 集中管理應用程式設定、辭典操作與加密功能。

pub mod dictionary;
pub mod encryption;
pub mod settings;

// === 重新匯出所有公開 API ===
pub use dictionary::{
    ensure_dicts_dir, get_available_dict_langs, get_official_dict_path, get_user_dict_path,
    load_dict, load_translation_memory, save_dict, save_translation_memory, DICT_DIR,
};
pub use encryption::{get_api_key, get_api_key_with_args, save_api_key, save_api_key_with_args};
pub use settings::{AppConfig, StyleConfig, DEFAULT_PROMPT, DEFAULT_SYSTEM_PROMPT};
