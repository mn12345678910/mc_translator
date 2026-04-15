//! # 翻譯核心模組
//! 集中管理翻譯引擎、API 呼叫與批次處理邏輯。

pub mod api;
pub mod batching;
pub mod context;
pub mod engine;
pub mod glossary;
pub mod job;
pub mod pipeline;

// 重新匯出 engine 與 api 中的公開 API（明確列出）
pub use api::models::{
    fetch_dynamic_models, fetch_mc_versions, fetch_mc_versions_with_url, fetch_ollama_models,
    version_to_pack_format,
};
pub use api::{
    build_batch_instruction, build_system_prompt, finalize_translation, log_llm_communication,
    map_lang_deepl, map_lang_google, parse_json_from_text, translate_batch, translate_one,
    translate_with_ollama, with_timeout,
};
pub use engine::{collect_translatable_strings, count_strings, translate_json_recursive};

use crate::translation::job::JobSharedState;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LogLevel {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: i64,
    #[serde(default)]
    pub segments: Vec<LogSegment>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LogSegment {
    pub kind: String,
    pub text: String,
}

lazy_static! {
    /// 當前正在執行的翻譯任務狀態，用於跨指令（如暫停、終止）進行連鎖控制
    pub static ref ACTIVE_JOB: Mutex<Option<JobSharedState>> = Mutex::new(None);
}
