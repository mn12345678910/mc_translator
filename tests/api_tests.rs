use mc_translator::translation::api::translate_with_ollama;
use mc_translator::translation::job::JobConfig;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_ollama_api_success_json() {
    let mock_server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "response": "{\"translated\": \"主選單\"}"
    });

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&mock_server)
        .await;

    let config = create_ollama_config(mock_server.uri());

    let result = translate_with_ollama("Main Menu", &config, "test_file", None).await;
    assert!(result.is_ok(), "應翻譯成功: {:?}", result.err());
    assert_eq!(result.unwrap(), "主選單");
}

#[tokio::test]
async fn test_ollama_api_fallback_raw_string() {
    let mock_server = MockServer::start().await;

    // 模擬 Ollama 直接回傳純字串 (非 JSON translated key)
    let mock_response = serde_json::json!({
        "response": "主選單"
    });

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&mock_server)
        .await;

    let config = create_ollama_config(mock_server.uri());

    let result = translate_with_ollama("Main Menu", &config, "test_file", None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "主選單");
}

#[tokio::test]
async fn test_ollama_api_server_error_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let config = create_ollama_config(mock_server.uri());

    let result = translate_with_ollama("Main Menu", &config, "test_file", None).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Ollama API Error"));
}

fn create_ollama_config(url: String) -> JobConfig {
    JobConfig {
        api_key: "".to_string(),
        api_provider: "Ollama".to_string(),
        selected_model: "mock-model".to_string(),
        ollama_url: url,
        api_base_url: "".to_string(),
        user_prompt: "".to_string(),
        system_prompt: "".to_string(),
        timeout: 60,
        batch_size: 150,
        batch_max_chars: 3500,
        output_dir: "output_test".to_string(),
        pack_format: 15,
        glossary_priority: "official".to_string(),
        skip_json: false,
        skip_js: false,
        skip_jar: false,
        skip_book: false,
        enable_llm_log: false,
        source_lang: "en_us".to_string(),
        target_lang: "zh_tw".to_string(),
        cleanup_prefixes: vec![],
        cleanup_contains: vec![],
        enable_debug_log: false,
        excluded_paths: Vec::new(),
    }
}
