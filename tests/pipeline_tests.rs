use mc_translator::config::AppConfig;
use mc_translator::translation::pipeline::start_translation_workflow;
use mc_translator::translation::ACTIVE_JOB;
use std::fs;
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
