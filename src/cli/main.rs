use clap::Parser;
use dialoguer::{Select, Input, Password};
use std::path::{Path, PathBuf};
use std::io::Write;

use mc_translator_rs::config::AppConfig;
use mc_translator_rs::translation::pipeline::start_translation_workflow;

#[derive(Parser, Debug)]
#[command(name = "mc_translator_cli", about = "Minecraft 模組翻譯工具 - CLI 模式")]
struct Args {
    /// 輸入檔案或資料夾路徑 (例如: ./mods/test.jar)
    #[arg(short, long)]
    input: Option<String>,

    /// 輸出資料夾路徑 [預設: ./LLMTranslator]
    #[arg(short, long)]
    output: Option<String>,

    /// API 提供商 (Gemini, OpenAI, DeepSeek, Mistral, Ollama, DeepL)
    #[arg(short = 'p', long)]
    provider: Option<String>,

    /// 模型名稱
    #[arg(short = 'm', long)]
    model: Option<String>,

    /// 覆蓋 API Key (.env 已存檔金鑰將優先使用，傳入將覆蓋)
    #[arg(long)]
    api_key: Option<String>,

    /// 啟用 LLM 通訊日誌
    #[arg(long)]
    log_llm: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = AppConfig::load();

    println!("=========================================");
    println!("=== Minecraft 模組翻譯工具 - CLI 模式 ===");
    println!("=========================================\n");

    let is_headless = args.input.is_some();

    if is_headless {
        // --- 1. 靜態 Headless 參數模式 ---
        let input_path_str = args.input.unwrap();
        let input_path = PathBuf::from(&input_path_str);

        if !input_path.exists() {
            println!("❌ 錯誤: 輸入路徑不存在 ({})", input_path_str);
            return Ok(());
        }

        if let Some(o) = args.output {
            config.output_dir = o;
        }
        if let Some(p) = args.provider {
            config.api_provider = p;
        }
        if let Some(m) = args.model {
            config.model = m;
        }
        if let Some(k) = args.api_key {
            config.api_key = k;
        }
        config.enable_llm_log = args.log_llm;

        println!("-> 偵測到指令參數，進入靜態 Headless 模式...");
        println!("   提供商: {}", config.api_provider);
        println!("   模型: {}", config.model);
        println!("   輸入: {}", input_path_str);
        println!("   輸出: {}", if config.output_dir.is_empty() { "LLMTranslator/ (預設)" } else { &config.output_dir });
        println!();

        run_translation(config, input_path).await?;
    } else {
        // --- 2. dialoguer 互動式模式 ---
        println!("-> 未偵測到輸入檔案參數，進入互動選單模式...\n");

        // 2-1. API 提供商
        let providers = vec!["Gemini", "OpenAI", "DeepSeek", "Mistral", "Ollama", "DeepL", "Google Free"];
        let default_provider_idx = providers.iter().position(|&p| p == config.api_provider).unwrap_or(0);

        let provider_idx = Select::new()
            .with_prompt("請選擇 API 提供商")
            .items(&providers)
            .default(default_provider_idx)
            .interact()?;
        
        config.api_provider = providers[provider_idx].to_string();

        // 2-2. 模型名稱
        if config.api_provider != "Google Free" {
            let model_prompt = format!("請輸入模型名稱 (目前預設: [{}])", if config.model.is_empty() { "無" } else { &config.model });
            let model: String = Input::new()
                .with_prompt(&model_prompt)
                .allow_empty(true)
                .interact()?;
            
            if !model.trim().is_empty() {
                config.model = model.trim().to_string();
            }
        }

        // 2-3. API Key
        if config.api_provider != "Ollama" && config.api_provider != "Google Free" {
            let has_saved_key = !config.api_key.is_empty();
            let key_prompt = if has_saved_key {
                "請輸入 API Key (預設使用已存檔金鑰，按 Enter 跳過)"
            } else {
                "請輸入 API Key"
            };

            let key: String = Password::new()
                .with_prompt(key_prompt)
                .allow_empty_password(has_saved_key)
                .interact()?;
            
            if !key.trim().is_empty() {
                config.api_key = key.trim().to_string();
            }
        }

        // 2-4. 輸入路徑
        let input_prompt = "請選取或輸入要翻譯的檔案/資料夾路徑 (例如 ./mods/test.jar)";
        let input_path_str: String = Input::new()
            .with_prompt(input_prompt)
            .interact()?;
        
        let input_path = PathBuf::from(input_path_str.trim());
        if !input_path.exists() {
            println!("❌ 錯誤: 輸入路徑不存在！");
            return Ok(());
        }

        // 2-5. 輸出資料夾
        let default_output = if config.output_dir.is_empty() { "LLMTranslator" } else { &config.output_dir };
        let output_prompt = format!("請輸入輸出資料夾 [預設: {}]", default_output);
        let output_dir: String = Input::new()
            .with_prompt(&output_prompt)
            .allow_empty(true)
            .interact()?;
        
        if !output_dir.trim().is_empty() {
            config.output_dir = output_dir.trim().to_string();
        }

        // 2-6. 確定執行
        let start = Select::new()
            .with_prompt("確定要開始執行翻譯嗎？")
            .items(&["[是] 我確定", "[否] 取消離開"])
            .default(0)
            .interact()?;
        
        if start == 1 {
            println!("🔒 操作已取消。");
            return Ok(());
        }

        println!("\n-> 正在啟動翻譯管線...\n");
        config.save(); // 下載互動異動存檔
        run_translation(config, input_path).await?;
    }

    Ok(())
}

async fn run_translation(config: AppConfig, input_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 包裹成管線需要的 Tuple 格式 (Path Buf, Rel Path)
    let rel_path = input_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let paths = vec![(input_path, rel_path)];

    let logger = |msg: &str| {
        // 去除換行避免與進度條衝突
        let clean_msg = msg.replace("\n", " ").trim().to_string();
        if !clean_msg.is_empty() {
             // 覆蓋當前進度條行並打上日誌
             print!("\r\x1B[K-> {}\n", clean_msg);
             let _ = std::io::stdout().flush();
        }
    };

    let progress_updater = |pct: f32, status: &str| {
        let bar_len = 25;
        let filled = (pct * bar_len as f32).max(0.0) as usize;
        let filled = filled.min(bar_len);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_len - filled));
        print!("\r\x1B[K[{}] {:.0}% | {}", bar, (pct * 100.0).max(0.0).min(100.0), status);
        let _ = std::io::stdout().flush();
    };

    let res = start_translation_workflow(
        config,
        paths,
        logger,
        progress_updater,
    ).await;

    println!("\n\n-> 管線運作結束。");

    match res {
        Ok(_) => println!("✅ 恭喜！所有翻譯任務已成功完成。"),
        Err(e) => println!("❌ 失敗退出: {}", e),
    }

    Ok(())
}
