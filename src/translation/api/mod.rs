//! # 翻譯 API 模組
//! 封裝所有翻譯 API 的呼叫邏輯。

pub mod client;
pub mod models;

// 向後相容：重新匯出 client 與 models 中的所有公開 API
pub use client::*;
pub use models::*;
