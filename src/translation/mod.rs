//! # 翻譯核心模組
//! 集中管理翻譯引擎、API 呼叫與批次處理邏輯。

pub mod api;
pub mod engine;

// 向後相容：重新匯出 engine 中的所有公開 API
pub use engine::*;
