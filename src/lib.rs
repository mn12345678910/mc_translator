pub mod config;
pub mod file;
pub mod i18n;
pub mod translation;
pub mod utils;

// 向後相容：保留舊路徑可用
pub use config::dictionary::{
    get_official_dict_path, get_user_dict_path, load_dict, save_dict, DICT_DIR,
};
pub use config::encryption::{get_api_key, save_api_key};
pub use config::settings::AppConfig;
pub use translation::glossary::automaton::{GlossaryAutomaton, GlossaryEntry, TermType};
pub use translation::job::{JobConfig, JobSharedState};
pub use utils::helpers::{add_log, format_log_message};
