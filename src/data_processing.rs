pub use crate::translation::context::*;
pub use crate::translation::engine::*;
pub use crate::translation::batching::*;

pub use crate::utils::text_processing::{
    detect_loop, postprocess_text, preprocess_text, sync_formatting, validate_and_cleanup,
};

pub fn hashmap_to_entries(map: &std::collections::HashMap<String, String>) -> Vec<crate::translation::glossary::GlossaryEntry> {
    map.iter()
        .map(|(k, v)| crate::translation::glossary::GlossaryEntry {
            original: k.clone(),
            translated: v.clone(),
            source: crate::translation::glossary::TermType::Official,
        })
        .collect()
}
