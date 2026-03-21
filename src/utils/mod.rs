//! # 通用工具模組
//! 包含 mc_lang 字典載入與共用資料結構。

pub mod helpers;
pub mod skip_rules;
pub mod text_processing;

// === 向後相容：重新匯出所有公開 API ===
pub use helpers::{add_log, extract_display_path, format_log_message, hashmap_to_entries};
pub use skip_rules::{should_skip_key, should_skip_value};

// === 向後相容：將原本的 Glossary 相關結構重新匯出 ===
pub use crate::translation::glossary::analyzer::{
    analyze_dictionary, clean_inferred_zh, find_common_hanzi, is_cjk, INFERENCE_BLACKLIST,
};
pub use crate::translation::glossary::automaton::{GlossaryAutomaton, GlossaryEntry, TermType};
pub use crate::translation::glossary::mc_lang::{load_mc_dicts, McLangFiles};
