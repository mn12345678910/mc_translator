//! # 通用工具模組
//! 包含 mc_lang 字典載入與共用資料結構。

pub mod helpers;
pub mod skip_rules;
pub mod text_processing;

// === 向後相容：重新匯出所有公開 API ===
pub use helpers::{add_log, extract_display_path, format_log_message, hashmap_to_entries};
pub use skip_rules::{should_skip_key, should_skip_value};
