use mc_translator::i18n::CommonLabels;
use mc_translator::translation::batching::{
    run_translation_batch, GlobalBatchItem, RunBatchContext,
};
use mc_translator::translation::glossary::GlossaryAutomaton;
use mc_translator::translation::job::JobConfig;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Arc, Mutex};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_fast_convert_bypass_llm() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": "LLM_OUTPUT"
        })))
        .mount(&server)
        .await;

    let config = JobConfig {
        api_provider: "Ollama".to_string(),
        ollama_url: server.uri(),
        selected_model: "llama3".to_string(),
        source_lang: "zh_cn".to_string(),
        target_lang: "zh_tw".to_string(),
        fast_convert: true,
        timeout: 60,
        ..JobConfig::default()
    };

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut items = vec![GlobalBatchItem::new("下界", 0, "test.json", "key1")];

    let glossary = GlossaryAutomaton::new(
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        "official",
    );
    let i18n = CommonLabels::default();

    let ctx = RunBatchContext {
        items: &mut items,
        config: Arc::new(Mutex::new(config)),
        status: Arc::new(Mutex::new(String::new())),
        progress: Arc::new(AtomicU32::new(0)),
        current_batch: Arc::new(AtomicU32::new(0)),
        total_batches: Arc::new(AtomicU32::new(0)),
        log: log.clone(),
        cancelled: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        pause_notifier: Arc::new(tokio::sync::Notify::new()),
        glossary_automaton: &glossary,
        i18n: &i18n,
        file_name: "test.json".to_string(),
        group_dir: "".to_string(),
        group_file_count: 1,
        global_items_offset: 0,
    };

    run_translation_batch(ctx).await.unwrap();

    assert!(items[0].translated.is_some());
    {
        let log_locked = log.lock().unwrap();
        assert!(
            log_locked
                .iter()
                .any(|e| e.message.contains("[Fast Convert]")),
            "Should use fast path"
        );
    } // Drop log_locked before await

    let calls = server.received_requests().await;
    assert!(
        calls.is_none() || calls.unwrap().is_empty(),
        "LLM should NOT be called"
    );
}

#[tokio::test]
async fn test_fast_convert_with_glossary_priority() {
    let config = JobConfig {
        source_lang: "zh_cn".to_string(),
        target_lang: "zh_tw".to_string(),
        fast_convert: true,
        timeout: 60,
        ..JobConfig::default()
    };

    let mut official = HashMap::new();
    official.insert("下界".to_string(), "地獄".to_string());
    let glossary = GlossaryAutomaton::new(&official, &HashMap::new(), &HashMap::new(), "official");

    let mut items = vec![GlobalBatchItem::new("下界", 0, "test.json", "key1")];

    let log = Arc::new(Mutex::new(Vec::new()));
    let i18n = CommonLabels::default();

    let ctx = RunBatchContext {
        items: &mut items,
        config: Arc::new(Mutex::new(config)),
        status: Arc::new(Mutex::new(String::new())),
        progress: Arc::new(AtomicU32::new(0)),
        current_batch: Arc::new(AtomicU32::new(0)),
        total_batches: Arc::new(AtomicU32::new(0)),
        log: log.clone(),
        cancelled: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        pause_notifier: Arc::new(tokio::sync::Notify::new()),
        glossary_automaton: &glossary,
        i18n: &i18n,
        file_name: "test.json".to_string(),
        group_dir: "".to_string(),
        group_file_count: 1,
        global_items_offset: 0,
    };

    run_translation_batch(ctx).await.unwrap();
    assert_eq!(items[0].translated.as_ref().unwrap(), "地獄");
}

#[tokio::test]
async fn test_fast_convert_disabled_uses_llm() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": "LLM_OUTPUT"
        })))
        .mount(&server)
        .await;

    let config = JobConfig {
        api_provider: "Ollama".to_string(),
        ollama_url: server.uri(),
        selected_model: "llama3".to_string(),
        source_lang: "zh_cn".to_string(),
        target_lang: "zh_tw".to_string(),
        fast_convert: false,
        timeout: 60,
        ..JobConfig::default()
    };

    let mut items = vec![GlobalBatchItem::new("测试", 0, "test.json", "key1")];

    let log = Arc::new(Mutex::new(Vec::new()));
    let i18n = CommonLabels {
        log_single_final_failed: "FAILED_WITH: {}".to_string(),
        ..CommonLabels::default()
    };

    let ctx = RunBatchContext {
        items: &mut items,
        config: Arc::new(Mutex::new(config)),
        status: Arc::new(Mutex::new(String::new())),
        progress: Arc::new(AtomicU32::new(0)),
        current_batch: Arc::new(AtomicU32::new(0)),
        total_batches: Arc::new(AtomicU32::new(0)),
        log: log.clone(),
        cancelled: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        pause_notifier: Arc::new(tokio::sync::Notify::new()),
        glossary_automaton: &GlossaryAutomaton::new(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "official",
        ),
        i18n: &i18n,
        file_name: "test.json".to_string(),
        group_dir: "".to_string(),
        group_file_count: 1,
        global_items_offset: 0,
    };

    run_translation_batch(ctx).await.unwrap();

    if items[0].translated.is_none() {
        let logs = log.lock().unwrap();
        for e in logs.iter() {
            println!("LOG: {}", e.message);
        }
        panic!("Translation failed");
    }

    assert_eq!(items[0].translated.as_ref().unwrap(), "LLM_OUTPUT");
    let calls = server.received_requests().await;
    assert!(
        calls.is_some() && !calls.unwrap().is_empty(),
        "LLM should be called"
    );
}

/// 測試：快速轉換路徑中的短語替換功能
/// 當術語表有「下界 → 地獄」時，「下界巖」應被正確轉換為「地獄巖」
#[tokio::test]
async fn test_fast_convert_glossary_phrase_replacement() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": "UNEXPECTED_LLM_CALL"
        })))
        .mount(&server)
        .await;

    let config = JobConfig {
        api_provider: "Ollama".to_string(),
        ollama_url: server.uri(),
        selected_model: "llama3".to_string(),
        source_lang: "zh_cn".to_string(),
        target_lang: "zh_tw".to_string(),
        fast_convert: true,
        timeout: 60,
        ..JobConfig::default()
    };

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut items = vec![GlobalBatchItem::new(
        "下界巖",
        0,
        "test.json",
        "tag.block.netherrack",
    )];

    // 術語表：下界 → 地獄
    let mut user_map = HashMap::new();
    user_map.insert("下界".to_string(), "地獄".to_string());

    let glossary = GlossaryAutomaton::new(&HashMap::new(), &user_map, &HashMap::new(), "official");
    let i18n = CommonLabels::default();

    let ctx = RunBatchContext {
        items: &mut items,
        config: Arc::new(Mutex::new(config)),
        status: Arc::new(Mutex::new(String::new())),
        progress: Arc::new(AtomicU32::new(0)),
        current_batch: Arc::new(AtomicU32::new(0)),
        total_batches: Arc::new(AtomicU32::new(0)),
        log: log.clone(),
        cancelled: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        pause_notifier: Arc::new(tokio::sync::Notify::new()),
        glossary_automaton: &glossary,
        i18n: &i18n,
        file_name: "test.json".to_string(),
        group_dir: "".to_string(),
        group_file_count: 1,
        global_items_offset: 0,
    };

    run_translation_batch(ctx).await.unwrap();

    let result = items[0].translated.as_deref().unwrap_or("");
    assert_eq!(
        result, "地獄巖",
        "「下界巖」應透過術語替換+hanconv 得到「地獄巖」，實際得到：「{}」",
        result
    );

    // LLM 不應被呼叫
    let calls = server.received_requests().await;
    assert!(
        calls.is_none_or(|v| v.is_empty()),
        "快速轉換路徑不應呼叫 LLM"
    );
}
