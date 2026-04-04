//! # 翻譯 API 模組
//! 封裝所有翻譯 API 的呼叫邏輯。

pub mod client;
pub mod models;

// 重新匯出 client 與 models 中的公開 API（明確列出）
pub use client::{
    build_batch_instruction, build_system_prompt, finalize_translation, log_llm_communication,
    map_lang_deepl, map_lang_google, parse_json_from_text, translate_batch, translate_one,
    translate_with_ollama, with_timeout,
};
pub use models::{
    fetch_dynamic_models, fetch_mc_versions, fetch_mc_versions_with_url, fetch_ollama_models,
    version_to_pack_format,
};
