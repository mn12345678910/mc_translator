pub mod config;
pub mod state;
pub mod translation;
pub mod file;
pub mod ui;
pub mod utils;

// 向後相容：保留舊路徑可用
pub use config::settings::AppConfig;
pub use config::dictionary::{get_official_dict_path, get_user_dict_path, load_dict, save_dict, DICT_DIR};
pub use config::encryption::{save_api_key, get_api_key};
pub use state::app_state::AppState;
pub use state::viewer_state::{ViewerSharedState, ViewerUpdate};
pub use translation::job::{JobConfig, JobSharedState};
pub use translation::glossary::automaton::{GlossaryAutomaton, GlossaryEntry, TermType};
pub use utils::helpers::{add_log, format_log_message};
