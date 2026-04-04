use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

use mc_translator::file::json_handler::{apply_json_task, collect_json_task};
use mc_translator::i18n::CommonLabels;
use mc_translator::translation::job::{JobConfig, JobSharedState};
use secrecy::SecretString;

fn create_mock_state(output_dir: String) -> JobSharedState {
    let config = JobConfig {
        api_key: SecretString::from("".to_string()),
        api_provider: "Gemini".to_string(),
        selected_model: "gemini-1.5-pro".to_string(),
        ollama_url: "".to_string(),
        api_base_url: "".to_string(),
        user_prompt: "".to_string(),
        system_prompt: "".to_string(),
        timeout: 60,
        batch_size: 150,
        batch_max_chars: 3500,
        output_dir,
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
        fast_convert: false,
    };

    JobSharedState {
        log: Arc::new(Mutex::new(Vec::new())),
        status: Arc::new(Mutex::new(String::new())),
        current_state: Arc::new(Mutex::new(mc_translator::translation::job::JobStatus::Idle)),
        progress: Arc::new(AtomicU32::new(0)),
        progress_total: Arc::new(AtomicU32::new(0)),
        cancelled: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        translation_memory: Arc::new(Mutex::new(HashMap::new())),
        global_progress: Arc::new(AtomicU32::new(0)),
        global_total: Arc::new(AtomicU32::new(0)),
        current_processing_path: Arc::new(Mutex::new(String::new())),
        current_batch: Arc::new(AtomicU32::new(0)),
        total_batches: Arc::new(AtomicU32::new(0)),
        pause_notifier: Arc::new(tokio::sync::Notify::new()),
        config: Arc::new(Mutex::new(config)),
        i18n: CommonLabels::default(),
    }
}

#[tokio::test]
async fn test_json_task_collection_and_application() {
    let t_dir = tempdir().unwrap();
    let input_dir = t_dir.path().join("input");
    let output_dir = t_dir.path().join("output");
    fs::create_dir_all(&input_dir).unwrap();

    let mock_state = create_mock_state(output_dir.to_string_lossy().to_string());

    let json_file_path = input_dir.join("test_file.json");
    let json_content = r#"{
        "menu.title": "Main Menu",
        "btn.start": "Start Game"
    }"#;
    fs::write(&json_file_path, json_content).unwrap();

    // 1. 測試收集 (Collection)
    let collect_res = collect_json_task(
        101, // file_id
        &json_file_path,
        "test_file.json".to_string(),
        &mock_state,
    )
    .await;

    assert!(collect_res.is_ok(), "收集 JSON 任務失敗");
    let opt = collect_res.unwrap();
    assert!(opt.is_some(), "不應該傳回 None");
    let (task, items) = opt.unwrap();

    assert_eq!(task.file_id, 101);
    assert_eq!(items.len(), 2, "應該擷取到 2 筆翻譯項目");

    // 模擬翻譯好後的狀態
    let mut translated_items = items.clone();
    translated_items[0].translated = Some("主選單".to_string());
    translated_items[1].translated = Some("開始遊戲".to_string());

    // 2. 測試套用 (Apply + 寫入)
    let apply_res = apply_json_task(&task, &translated_items, &mock_state.config).await;

    assert!(apply_res.is_ok(), "套用 JSON 任務失敗");

    // 檢查輸出的檔案是否存在且內容正確
    let output_file_path = output_dir.join("LLMTranslator").join("test_file.json");

    assert!(output_file_path.exists(), "輸出檔案不存在");
    let written_content = fs::read_to_string(&output_file_path).unwrap();
    assert!(written_content.contains("主選單"), "未包含翻譯：主選單");
    assert!(written_content.contains("開始遊戲"), "未包含翻譯：開始遊戲");
}

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_e2e_translation_workflow() {
    // 1. 建立 Mock HTTP 伺服器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mock_url = format!("http://{}", addr);

    // 背景執行 Mock 伺服器監聽
    let server_handle = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = [0; 1024];
            let _ = stream.read(&mut buf).await; // 讀取請求 (忽略細節)

            // 回應模擬 Ollama 的 response 欄位 (包含批次翻譯 JSON 映射)
            let response_body = serde_json::json!({
                "response": "{\"1\": \"主選單\", \"2\": \"開始遊戲\"}"
            });
            let body_str = response_body.to_string();

            let http_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body_str.len(),
                body_str
            );

            let _ = stream.write_all(http_response.as_bytes()).await;
            let _ = stream.flush().await;
        }
    });

    // 2. 準備實體檔案環境
    let t_dir = tempdir().unwrap();
    let input_dir = t_dir.path().join("e2e_input");
    let output_dir = t_dir.path().join("e2e_output");
    fs::create_dir_all(&input_dir).unwrap();

    let lang_file_path = input_dir.join("en_us.json");
    fs::write(
        &lang_file_path,
        r#"{
    "menu.title": "Main Menu",
    "btn.start": "Start Game"
}"#,
    )
    .unwrap();

    // 3. 設定 AppConfig 指向對應位址
    let cfg = mc_translator::config::settings::AppConfig {
        api_provider: "Ollama".to_string(),
        ollama_url: mock_url,
        api_base_url: "".to_string(),
        model: "mock-model".to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        ..Default::default()
    };

    // 4. 觸發 start_translation_workflow
    let paths = vec![(lang_file_path.clone(), "en_us.json".to_string())];
    let logger = |_: mc_translator::translation::LogEntry| {};
    let progress_updater = |_: f32, _: f32, _: f32, _: f32, _: &str| {};

    let res = mc_translator::translation::pipeline::start_translation_workflow(
        cfg,
        paths,
        logger,
        progress_updater,
    )
    .await;

    assert!(res.is_ok(), "E2E 管線工作流執行失敗: {:?}", res.err());

    // 5. 驗證輸出
    let output_path = output_dir.join("LLMTranslator").join("zh_tw.json");

    assert!(output_path.exists(), "E2E 輸出檔案不存在");
    let written = fs::read_to_string(&output_path).unwrap();
    assert!(written.contains("主選單"), "未包含翻譯結果: 主選單");
    assert!(written.contains("開始遊戲"), "未包含翻譯結果: 開始遊戲");

    // 清理
    server_handle.abort();
}
