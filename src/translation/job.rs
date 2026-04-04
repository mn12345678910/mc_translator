//! # 翻譯任務定義與共用狀態
//! 封裝翻譯任務所需的設定參數與跨執行緒共享的狀態物件。

use crate::config::settings::AppConfig;
use crate::i18n::CommonLabels;
use crate::translation::LogEntry;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Arc, Mutex};

/// 翻譯任務的狀態枚舉
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Idle,
    Running,
    Paused,
}

/// 翻譯任務的靜態設定參數
#[derive(Clone, Default)]
pub struct JobConfig {
    pub api_key: SecretString,
    pub api_provider: String,
    pub selected_model: String,
    pub ollama_url: String,
    pub api_base_url: String,
    pub cleanup_prefixes: Vec<String>,
    pub cleanup_contains: Vec<String>,
    pub user_prompt: String,
    pub system_prompt: String,

    pub timeout: u64,
    pub batch_size: u32,
    pub batch_max_chars: u32,
    pub output_dir: String,
    pub pack_format: u32,
    pub glossary_priority: String,
    pub skip_json: bool,
    pub skip_js: bool,
    pub skip_jar: bool,
    pub skip_book: bool,
    pub enable_llm_log: bool,
    pub source_lang: String,
    pub target_lang: String,
    pub enable_debug_log: bool,
    pub excluded_paths: Vec<String>,
    pub fast_convert: bool,
}

/// 翻譯任務在執行過程中的共享狀態物件 (Arc<Mutex<...>>)
#[derive(Clone)]
pub struct JobSharedState {
    pub log: Arc<Mutex<Vec<LogEntry>>>,
    pub status: Arc<Mutex<String>>,
    pub current_state: Arc<Mutex<JobStatus>>,
    pub progress: Arc<AtomicU32>,
    pub progress_total: Arc<AtomicU32>,
    pub cancelled: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub translation_memory: Arc<Mutex<HashMap<String, String>>>,
    pub global_progress: Arc<AtomicU32>,
    pub global_total: Arc<AtomicU32>,
    pub current_processing_path: Arc<Mutex<String>>,
    pub current_batch: Arc<AtomicU32>,
    pub total_batches: Arc<AtomicU32>,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub config: Arc<Mutex<JobConfig>>,
    pub i18n: CommonLabels,
}

impl JobConfig {
    /// 根據傳入的個別參數建立 JobConfig
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        api_key: String,
        api_provider: String,
        selected_model: String,
        ollama_url: String,
        api_base_url: String,
        cleanup_prefixes: Vec<String>,
        cleanup_contains: Vec<String>,
        user_prompt: String,
        system_prompt: String,

        timeout: u32,
        batch_size: u32,
        batch_max_chars: u32,
        output_dir: String,
        pack_format: u32,
        glossary_priority: String,
        skip_json: bool,
        skip_js: bool,
        skip_jar: bool,
        skip_book: bool,
        enable_llm_log: bool,
        source_lang: String,
        target_lang: String,
        enable_debug_log: bool,
        excluded_paths: Vec<String>,
        fast_convert: bool,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            api_provider,
            selected_model,
            ollama_url,
            api_base_url,
            cleanup_prefixes,
            cleanup_contains,
            user_prompt,
            system_prompt,

            timeout: timeout as u64,
            batch_size,
            batch_max_chars,
            output_dir,
            pack_format,
            glossary_priority,
            skip_json,
            skip_js,
            skip_jar,
            skip_book,
            enable_llm_log,
            source_lang,
            target_lang,
            enable_debug_log,
            excluded_paths,
            fast_convert,
        }
    }

    /// 從 AppConfig 和 CommonLabels 建立 JobConfig
    pub fn from_app_config_and_i18n(config: &AppConfig, i18n: &CommonLabels) -> Self {
        Self {
            api_key: config.api_key.clone(),
            api_provider: config.api_provider.clone(),
            selected_model: config.model.clone(),
            ollama_url: config.ollama_url.clone(),
            api_base_url: config.api_base_url.clone(),
            cleanup_prefixes: i18n.cleanup_prefixes.clone(),
            cleanup_contains: i18n.cleanup_contains.clone(),
            user_prompt: config.user_prompt.clone(),
            system_prompt: config.system_prompt.clone(),
            timeout: config.timeout as u64,
            batch_size: config.batch_size,
            batch_max_chars: config.batch_max_chars,
            output_dir: config.output_dir.clone(),
            pack_format: config.pack_format,
            glossary_priority: config.glossary_priority.clone(),
            skip_json: config.skip_json,
            skip_js: config.skip_js,
            skip_jar: config.skip_jar,
            skip_book: config.skip_book,
            enable_llm_log: config.enable_llm_log,
            source_lang: config.source_lang.clone(),
            target_lang: config.target_lang.clone(),
            enable_debug_log: config.enable_debug_log,
            excluded_paths: config.excluded_paths.clone(),
            fast_convert: config.fast_convert,
        }
    }
}
