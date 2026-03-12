//! # 翻譯任務定義與共用狀態
//! 封裝翻譯任務所需的設定參數與跨執行緒共享的狀態物件。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 翻譯任務的靜態設定參數
#[derive(Clone)]
pub struct JobConfig {
    pub api_key: String,
    pub api_provider: String,
    pub selected_model: String,
    pub ollama_url: String,
    pub prompt: String,
    pub ollama_timeout: u64,
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
    pub technical_constraints: String,
}

/// 翻譯任務在執行過程中的共享狀態物件 (Arc<Mutex<...>>)
#[derive(Clone)]
pub struct JobSharedState {
    pub log: Arc<Mutex<Vec<String>>>,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<Mutex<f32>>,
    pub progress_total: Arc<Mutex<f32>>,
    pub cancelled: Arc<Mutex<bool>>,
    pub paused: Arc<Mutex<bool>>,
    pub translation_memory: Arc<Mutex<HashMap<String, String>>>,
    pub global_progress: Arc<Mutex<f32>>,
    pub global_total: Arc<Mutex<f32>>,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub config: Arc<Mutex<JobConfig>>,
}

impl JobConfig {
    /// 根據傳入的個別參數建立 JobConfig
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        api_key: String,
        api_provider: String,
        selected_model: String,
        ollama_url: String,
        prompt: String,
        ollama_timeout: u32,
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
        technical_constraints: String,
    ) -> Self {
        Self {
            api_key,
            api_provider,
            selected_model,
            ollama_url,
            prompt,
            ollama_timeout: ollama_timeout as u64,
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
            technical_constraints,
        }
    }
}
