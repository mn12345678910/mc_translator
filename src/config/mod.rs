//! # 配置模組
//! 集中管理應用程式設定、辭典操作與加密功能。

pub mod dictionary;
pub mod encryption;
pub mod settings;

// === 向後相容：重新匯出所有公開 API ===
pub use dictionary::{
    ensure_dicts_dir, get_official_dict_path, get_user_dict_path, load_dict,
    load_translation_memory, save_dict, save_translation_memory, DICT_DIR,
};
pub use settings::{AppConfig, StyleConfig, DEFAULT_PROMPT};
