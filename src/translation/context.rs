use crate::translation::job::JobConfig;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 翻譯上下文，攜帶字典與當前任務狀態等資訊
pub struct TranslationContext<'a> {
    pub config: Arc<Mutex<JobConfig>>,
    pub inferred: &'a HashMap<String, String>,
    pub terms: &'a Vec<(String, String)>,
    pub glossary_automaton: &'a crate::translation::glossary::GlossaryAutomaton,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<Mutex<f32>>,
    pub total_progress: Arc<Mutex<f32>>,
    pub cancelled: Arc<Mutex<bool>>,
    pub paused: Arc<Mutex<bool>>,
    pub current_log: Arc<Mutex<Vec<String>>>,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub filename: String,
    pub counter: Arc<Mutex<usize>>,
    pub translations: Arc<Mutex<HashMap<String, Vec<String>>>>,
    pub translation_memory: Arc<Mutex<HashMap<String, String>>>,
    pub skip_memory: bool,
    /// 預先填滿的項項目 (original, key, translated)
    pub prefilled: Arc<Mutex<Vec<(String, String, String)>>>,
    pub i18n: &'a crate::ui::i18n::I18nLabels,
}

pub struct ContextOptions<'a> {
    pub config: Arc<Mutex<JobConfig>>,
    pub inferred: &'a HashMap<String, String>,
    pub terms: &'a Vec<(String, String)>,
    pub glossary_automaton: &'a crate::translation::glossary::GlossaryAutomaton,
    pub status: Arc<Mutex<String>>,
    pub progress: Arc<Mutex<f32>>,
    pub total_progress: Arc<Mutex<f32>>,
    pub cancelled: Arc<Mutex<bool>>,
    pub paused: Arc<Mutex<bool>>,
    pub current_log: Arc<Mutex<Vec<String>>>,
    pub filename: String,
    pub translation_memory: Arc<Mutex<HashMap<String, String>>>,
    pub skip_memory: bool,
    pub pause_notifier: Arc<tokio::sync::Notify>,
    pub i18n: &'a crate::ui::i18n::I18nLabels,
}

impl<'a> TranslationContext<'a> {
    pub fn new(opts: ContextOptions<'a>) -> Self {
        Self {
            config: opts.config,
            inferred: opts.inferred,
            terms: opts.terms,
            glossary_automaton: opts.glossary_automaton,
            status: opts.status,
            progress: opts.progress,
            total_progress: opts.total_progress,
            cancelled: opts.cancelled,
            paused: opts.paused,
            current_log: opts.current_log,
            filename: opts.filename,
            counter: Arc::new(Mutex::new(0)),
            translations: Arc::new(Mutex::new(HashMap::new())),
            translation_memory: opts.translation_memory,
            skip_memory: opts.skip_memory,
            prefilled: Arc::new(Mutex::new(Vec::new())),
            pause_notifier: opts.pause_notifier,
            i18n: opts.i18n,
        }
    }
}
