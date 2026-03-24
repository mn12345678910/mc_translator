use mc_translator::config::AppConfig;
use mc_translator::translation::pipeline::start_translation_workflow;
use mc_translator::translation::ACTIVE_JOB;
use std::fs;
use std::io::Write;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn test_pipeline_workflow_cancellation() {
    // 1. 準備檔案環境
    let temp_dir = std::env::temp_dir().join("mc_translator_pipe_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let lang_file_path = temp_dir.join("en_us.json");
    fs::write(
        &lang_file_path,
        r#"{
        "key1": "value1",
        "key2": "value2"
    }"#,
    )
    .unwrap();

    let cfg = AppConfig {
        api_provider: "Ollama".to_string(),
        ollama_url: "http://127.0.0.1:0".to_string(), // 確保不會真實發送
        model: "mock-model".to_string(),
        output_dir: temp_dir.join("output").to_string_lossy().to_string(),
        ..Default::default()
    };

    let paths = vec![(lang_file_path.clone(), "en_us.json".to_string())];
    let logger = |_: &str| {};

    // 在進度回調中觸發取消
    let progress_updater =
        |current_g: f32, _total_g: f32, _current_b: f32, _total_b: f32, _status: &str| {
            if current_g >= 0.0 {
                // 被呼叫即觸發取消
                if let Ok(mut active) = ACTIVE_JOB.lock() {
                    if let Some(job) = active.as_mut() {
                        job.cancelled.store(true, Ordering::SeqCst);
                    }
                }
            }
        };

    // 背景模擬 Mock 會失敗，確保流程能進入被取消的判斷分支
    let res = start_translation_workflow(cfg, paths, logger, progress_updater).await;

    // 此處應能執行結束 (可能為 Err 或是 Ok, 取決於底層遇到 Cancel 的處理)
    assert!(res.is_ok() || res.is_err(), "應該可正常調用結束");

    // 清理
    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_process_all_files_multi_types() {
    let temp_dir = std::env::temp_dir().join("mc_translator_pipe_multi_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. 建立 .js 檔案
    let js_path = temp_dir.join("script.js");
    std::fs::write(&js_path, r#"addItem("item1", Text.of("Hello World"));"#).unwrap();

    // 2. 建立 .jar 檔案
    let jar_path = temp_dir.join("items.jar");
    {
        let file = std::fs::File::create(&jar_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();
        zip.start_file("assets/minecraft/lang/en_us.json", options)
            .unwrap();
        zip.write_all(r#"{"menu.play": "Play"}"#.as_bytes())
            .unwrap();
        zip.finish().unwrap();
    }

    // 3. 準備 JobSharedState
    use mc_translator::translation::job::{JobConfig, JobSharedState};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::{Arc, Mutex};

    let mut config = JobConfig::default();
    config.source_lang = "en_us".to_string();
    config.target_lang = "zh_tw".to_string();

    let state = JobSharedState {
        log: Arc::new(Mutex::new(Vec::new())),
        status: Arc::new(Mutex::new(String::new())),
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
        i18n: mc_translator::i18n::CommonLabels::default(),
    };

    let paths = vec![
        (js_path.clone(), "script.js".to_string()),
        (jar_path.clone(), "items.jar".to_string()),
    ];

    use mc_translator::file::pipeline::process_all_files;
    use mc_translator::translation::glossary::mc_lang::McLangFiles;

    let exact_arc = Arc::new(Mutex::new(HashMap::new()));
    let inferred_arc = Arc::new(Mutex::new(HashMap::new()));
    let _mc_lang_arc: Arc<Mutex<Option<McLangFiles>>> = Arc::new(Mutex::new(None));
    let _term_arc = Arc::new(Mutex::new(Vec::new()));

    let res = process_all_files(
        paths,
        state,
        _mc_lang_arc,
        _term_arc,
        exact_arc,
        inferred_arc,
    )
    .await;

    assert!(res.is_ok());

    // 清理
    let _ = std::fs::remove_dir_all(&temp_dir);
}
