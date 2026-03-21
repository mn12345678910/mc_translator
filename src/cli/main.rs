use clap::Parser;
use dialoguer::{Input, Password, Select};
use mc_translator::i18n::CliLabels;
use std::io::Write;
use std::path::PathBuf;

use mc_translator::config::AppConfig;
use mc_translator::translation::pipeline::start_translation_workflow;

#[derive(Parser, Debug)]
#[command(
    name = "mc_translator_cli",
    about = "Minecraft 模組翻譯工具 - CLI 模式"
)]
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

    /// 覆蓋 API Key (現有 Keyring 憑證將優先使用，傳入將覆蓋)
    #[arg(long)]
    api_key: Option<String>,

    /// 啟用 LLM 通訊日誌
    #[arg(long)]
    log_llm: bool,

    /// 批次量 (預設: 150)
    #[arg(long)]
    batch_size: Option<u32>,

    /// 批次字數上限 (預設: 3500)
    #[arg(long)]
    batch_max_chars: Option<u32>,

    /// API 逾時秒數 (預設: 60)
    #[arg(long)]
    timeout: Option<u32>,

    /// 術語優先級 (official 或 user)
    #[arg(long)]
    glossary_priority: Option<String>,

    /// 來源語言 (預設: en_us)
    #[arg(long)]
    source_lang: Option<String>,

    /// 目標語言 (預設: zh_tw)
    #[arg(long)]
    target_lang: Option<String>,

    /// 跳過 .json
    #[arg(long)]
    skip_json: bool,

    /// 跳過 .js
    #[arg(long)]
    skip_js: bool,

    /// 跳過 .jar
    #[arg(long)]
    skip_jar: bool,

    /// 跳過手冊
    #[arg(long)]
    skip_book: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 確保 langs 目錄與語言檔案存在
    let _ = CliLabels::ensure_langs_exists();

    let mut config = AppConfig::load();
    let mut i18n = CliLabels::load_or_default(&config.ui_lang);

    println!("{}", i18n.cli_banner_title);

    // --- 參數覆蓋設定檔 (Parity Mapping) ---
    if let Some(o) = args.output.clone() {
        config.output_dir = o;
    }
    if let Some(p) = args.provider.clone() {
        config.api_provider = p;
    }
    if let Some(m) = args.model.clone() {
        config.model = m;
    }
    if let Some(k) = args.api_key.clone() {
        config.api_key = k;
    }
    if let Some(b) = args.batch_size {
        config.batch_size = b;
    }
    if let Some(c) = args.batch_max_chars {
        config.batch_max_chars = c;
    }
    if let Some(t) = args.timeout {
        config.timeout = t;
    }
    if let Some(g) = args.glossary_priority.clone() {
        config.glossary_priority = g;
    }
    if let Some(s) = args.source_lang.clone() {
        config.source_lang = s;
    }
    if let Some(t) = args.target_lang.clone() {
        config.target_lang = t;
    }
    if args.skip_json {
        config.skip_json = true;
    }
    if args.skip_js {
        config.skip_js = true;
    }
    if args.skip_jar {
        config.skip_jar = true;
    }
    if args.skip_book {
        config.skip_book = true;
    }
    if args.log_llm {
        config.enable_llm_log = true;
    }

    let is_headless = args.input.is_some();

    if is_headless {
        // --- 1. 靜態 Headless 參數模式 ---
        let input_path_str = args.input.unwrap();
        let input_path = PathBuf::from(&input_path_str);

        if !input_path.exists() {
            println!("❌ 錯誤: 輸入路徑不存在 ({})", input_path_str);
            return Ok(());
        }

        // 參數已於全域套用

        println!("-> 偵測到指令參數，進入靜態 Headless 模式...");
        println!("   提供商: {}", config.api_provider);
        println!("   模型: {}", config.model);
        println!("   輸入: {}", input_path_str);
        println!(
            "   輸出: {}",
            if config.output_dir.is_empty() {
                "LLMTranslator/ (預設)"
            } else {
                &config.output_dir
            }
        );
        println!();

        run_translation(config, input_path).await?;
    } else {
        println!("{}", i18n.cli_mode_interactive);

        let mut next_step = 1;
        let mut initial_history: Vec<usize> = Vec::new();

        loop {
            let mut status_history = initial_history.clone();
            let mut step = next_step;
            let mut input_path = PathBuf::new();

            while step <= 7 {
                match step {
                    1 => {
                        // --- Step 1: 介面語言 ---
                        let langs = CliLabels::get_available_ui_langs();
                        let default_idx =
                            langs.iter().position(|l| l == &config.ui_lang).unwrap_or(0);

                        let idx = Select::new()
                            .with_prompt(&i18n.cli_select_ui_lang)
                            .items(&langs)
                            .default(default_idx)
                            .interact()?;

                        config.ui_lang = langs[idx].clone();
                        i18n = CliLabels::load_or_default(&config.ui_lang);
                        config.save(); // 即時存入

                        print!("\x1B[2J\x1B[1;1H"); // 清空畫面
                        println!("{}", i18n.cli_banner_title);
                        println!("{}", i18n.cli_mode_interactive);

                        status_history.push(1);
                        step = 2;
                    }
                    2 => {
                        // --- Step 2: API 提供商 ---
                        let providers = [
                            "Gemini",
                            "OpenAI",
                            "DeepSeek",
                            "Mistral",
                            "Ollama",
                            "DeepL",
                            "Google Free",
                        ];
                        let default_idx = providers
                            .iter()
                            .position(|&p| p == config.api_provider)
                            .unwrap_or(0);
                        let mut items: Vec<String> =
                            providers.iter().map(|s| s.to_string()).collect();
                        items.push(i18n.label_back_to_prev_cli.clone()); // 修正重覆 <-

                        let idx = Select::new()
                            .with_prompt(&i18n.prompt_select_provider_cli)
                            .items(&items)
                            .default(default_idx)
                            .interact()?;

                        if idx == items.len() - 1 {
                            step = status_history.pop().unwrap_or(1);
                            continue;
                        }

                        config.api_provider = providers[idx].to_string();
                        config.save(); // 即時存入
                        status_history.push(2);
                        step = 3;
                    }
                    3 => {
                        // --- Step 3: API Key ---
                        if config.api_provider == "Ollama" || config.api_provider == "Google Free" {
                            status_history.push(3);
                            step = 4;
                            continue;
                        }

                        let has_saved_key = !config.api_key.is_empty();
                        let key_prompt = if has_saved_key {
                            format!(
                                "{} (鍵入 '<' 或 Enter 代表跳過使用舊金鑰)",
                                i18n.common.label_api_key
                            )
                        } else {
                            format!("{} (鍵入 '<' 回到上一步)", i18n.common.label_api_key)
                        };

                        let key: String = Password::new()
                            .with_prompt(&key_prompt)
                            .allow_empty_password(true)
                            .interact()?;

                        if key == "<" {
                            step = status_history.pop().unwrap_or(1);
                            continue;
                        }

                        if !key.trim().is_empty() {
                            config.api_key = key.trim().to_string();
                            config.save(); // 即時存入
                        }
                        status_history.push(3);
                        step = 4;
                    }
                    4 => {
                        // --- Step 4: 模型名稱 ---
                        if config.api_provider == "Google Free" {
                            status_history.push(4);
                            step = 5;
                            continue;
                        }

                        println!(
                            "{}",
                            i18n.cli_fetching_models.replace("{}", &config.api_provider)
                        );
                        let mut items =
                            mc_translator::translation::api::models::fetch_dynamic_models(
                                &config.api_provider,
                                &config.api_key,
                                &config.ollama_url,
                            )
                            .await
                            .unwrap_or_else(|_| Vec::new());

                        let is_dynamic = !items.is_empty();
                        let mut prompt_text = i18n.common.label_model.clone();
                        if !is_dynamic
                            && config.api_provider != "DeepL"
                            && config.api_provider != "Ollama"
                        {
                            prompt_text = format!(
                                "{}{}",
                                i18n.common.label_model, i18n.cli_model_fetch_failed
                            );
                        }

                        items.push(i18n.label_custom_input_cli.clone());
                        items.push(i18n.label_back_to_prev_cli.clone()); // 修正重覆 <-

                        let default_idx = if !config.model.is_empty() {
                            items.iter().position(|m| m == &config.model).unwrap_or(0)
                        } else {
                            0
                        };

                        let idx = Select::new()
                            .with_prompt(&prompt_text)
                            .items(&items)
                            .default(default_idx)
                            .interact()?;

                        if idx == items.len() - 1 {
                            if config.api_provider == "Ollama"
                                || config.api_provider == "Google Free"
                            {
                                step = 2; // 跳過不適用的 APIKey 步驟
                            } else {
                                step = status_history.pop().unwrap_or(1);
                            }
                            continue;
                        }

                        if items[idx] == i18n.label_custom_input_cli {
                            let model: String = Input::new()
                                .with_prompt(&i18n.cli_custom_model_prompt)
                                .allow_empty(true)
                                .interact()?;
                            if model == "<" {
                                continue;
                            }
                            if !model.trim().is_empty() {
                                config.model = model.trim().to_string();
                            }
                        } else {
                            config.model = items[idx].to_string();
                        }
                        config.save(); // 即時存入
                        status_history.push(4);
                        step = 5;
                    }
                    5 => {
                        // --- Step 5: 輸入路徑 ---
                        let input_prompt = &i18n.cli_input_path_prompt;
                        let input_path_str: String =
                            Input::new().with_prompt(input_prompt).interact()?;

                        if input_path_str == "<" {
                            if config.api_provider == "Google Free" {
                                step = 2; // Google Free 同時跳過 3 與 4
                            } else {
                                step = status_history.pop().unwrap_or(1);
                            }
                            continue;
                        }

                        let path = PathBuf::from(input_path_str.trim());
                        if !path.exists() {
                            println!("{}", i18n.cli_error_path_not_exist);
                            continue;
                        }

                        input_path = path;
                        status_history.push(5);
                        step = 6;
                    }
                    6 => {
                        // --- Step 6: 輸出資料夾 ---
                        let default_output = if config.output_dir.is_empty() {
                            "LLMTranslator"
                        } else {
                            &config.output_dir
                        };
                        let output_prompt = i18n
                            .cli_output_path_prompt
                            .replace("{}", &i18n.common.label_output_path)
                            .replace("{}", default_output);
                        let output_dir: String = Input::new()
                            .with_prompt(&output_prompt)
                            .allow_empty(true)
                            .interact()?;

                        if output_dir == "<" {
                            step = status_history.pop().unwrap_or(1);
                            continue;
                        }

                        if !output_dir.trim().is_empty() {
                            config.output_dir = output_dir.trim().to_string();
                        }
                        config.save(); // 即時存入
                        status_history.push(6);
                        step = 7;
                    }
                    7 => {
                        // --- Step 7: 確定 / 進階與取消 ---
                        let items = vec![
                            i18n.label_yes_confirm_cli.clone(),
                            i18n.prompt_advanced_settings_cli.clone(),
                            i18n.label_no_cancel_cli.clone(),
                            i18n.label_back_to_prev_cli.to_string(), // 修正重覆 <-
                        ];

                        let start = Select::new()
                            .with_prompt(&i18n.prompt_confirm_start_cli)
                            .items(&items)
                            .default(0)
                            .interact()?;

                        if start == 3 {
                            // 回上一步
                            step = status_history.pop().unwrap_or(1);
                            continue;
                        } else if start == 2 {
                            // 取消離開
                            println!("{}", i18n.cli_op_cancelled);
                            return Ok(());
                        } else if start == 1 {
                            // 進階參數
                            // 簡易進階切換，此處可以彈出單次輸入或不彈，若使用者只要跟GUI對等，
                            // 這邊可加入簡單 skips 或批次量 Input，為了保持結構先讓它一律過!
                            println!("{}", i18n.cli_adv_settings_synced);
                        }

                        break; // 離開 while 循環發起 Pipeline
                    }
                    _ => break,
                }
            }

            println!("{}", i18n.cli_starting_pipeline);
            config.save();
            let _ = run_translation(config.clone(), input_path).await; // 忽略單次錯誤

            let choice = Select::new()
                .with_prompt(&i18n.prompt_task_finished_cli)
                .items(&[
                    i18n.prompt_new_task_cli.clone(),
                    i18n.label_no_cancel_cli.clone(),
                ])
                .default(0)
                .interact()?;

            if choice == 1 {
                break;
            }

            println!("\n=========================================");
            println!("===          {}           ===", i18n.prompt_new_task_cli);
            println!("=========================================\n");

            next_step = 5;
            if config.api_provider == "Google Free" {
                initial_history = vec![1, 2];
            } else if config.api_provider == "Ollama" {
                initial_history = vec![1, 2, 4];
            } else {
                initial_history = vec![1, 2, 3, 4]; // 確保 Step 5 按上一步確實回到模型 (4)
            }
        }
    }

    Ok(())
}

async fn run_translation(
    config: AppConfig,
    input_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let i18n = CliLabels::load_or_default(&config.ui_lang);

    // 包裹成管線需要的 Tuple 格式 (Path Buf, Rel Path)
    let rel_path = input_path.to_string_lossy().to_string();
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

    let progress_updater =
        |current: f32, total: f32, batch_curr: f32, batch_tot: f32, status: &str| {
            let pct = if total > 0.0 { current / total } else { 0.0 };
            let bar_len = 25;
            let filled = (pct * bar_len as f32).max(0.0) as usize;
            let filled = filled.min(bar_len);
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_len - filled));

            let sub_info = if batch_tot > 0.0 {
                format!(" | 條目: {}/{}", batch_curr as u32, batch_tot as u32)
            } else {
                String::new()
            };

            print!(
                "\r\x1B[K[{}] {:.0}%{} | {}",
                bar,
                (pct * 100.0).clamp(0.0, 100.0),
                sub_info,
                status
            );
            let _ = std::io::stdout().flush();
        };

    let res = start_translation_workflow(config, paths, logger, progress_updater).await;

    println!("{}", i18n.cli_pipeline_ended);

    match res {
        Ok(_) => println!("{}", i18n.cli_pipeline_success),
        Err(e) => println!("{}", i18n.cli_pipeline_failed.replace("{}", &e.to_string())),
    }

    Ok(())
}
