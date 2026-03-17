//! # 翻譯核心模組
//! 集中管理翻譯引擎、API 呼叫與批次處理邏輯。

pub mod job;
pub mod glossary;
pub mod api;
pub mod context;
pub mod batching;
pub mod engine;
pub mod pipeline;

// 向後相容：重新匯出 engine 與 api 中的所有公開 API
pub use engine::*;
pub use api::*;
