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

    /// 啟用偵錯日誌持久化 (debug.log)
    #[arg(long)]
    log_debug: bool,

    /// 排除路徑 (例如: --exclude "secret_folder" -e "ignore_this") [僅追加]
    #[arg(short = 'e', long)]
    exclude: Vec<String>,

    /// 啟用快速簡繁轉換 (zh_cn↔zh_tw 繞過 LLM)
    #[arg(long)]
    fast_convert: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    run_main_with_args(args, &RealCliInteract).await
}

async fn run_main_with_args(
    args: Args,
    interact: &dyn CliInteract,
) -> Result<(), Box<dyn std::error::Error>> {
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
    if args.log_debug {
        config.enable_debug_log = true;
    }
    if !args.exclude.is_empty() {
        config.excluded_paths.extend(args.exclude);
        println!("{}", i18n.cli_hint_config_exclude);
    }
    if args.fast_convert {
        config.fast_convert = true;
    }

    let is_headless = args.input.is_some();

    if is_headless {
        // --- 1. 靜態 Headless 參數模式 ---
        let input_path_str = args.input.unwrap();
        let input_path = PathBuf::from(&input_path_str);

        if !input_path.exists() {
            println!(
                "{}",
                i18n.cli_error_input_not_exist
                    .replace("{}", &input_path_str)
            );
            return Ok(());
        }

        // 參數已於全域套用

        println!("{}", i18n.cli_detect_headless);
        println!(
            "   {}: {}",
            i18n.cli_label_provider.replace(": {}", ""),
            config.api_provider
        );
        println!(
            "   {}: {}",
            i18n.cli_label_model.replace(": {}", ""),
            config.model
        );
        println!(
            "   {}: {}",
            i18n.cli_label_input.replace(": {}", ""),
            input_path_str
        );
        println!(
            "   {}: {}",
            i18n.cli_label_output.replace(": {}", ""),
            if config.output_dir.is_empty() {
                format!("LLMTranslator/ ({})", i18n.cli_label_default)
            } else {
                config.output_dir.clone()
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

                        let idx = interact.select(&i18n.cli_select_ui_lang, &langs, default_idx)?;

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
                        items.push(i18n.label_back_to_prev_cli.clone()); // 避免重複返回項

                        let idx = interact.select(
                            &i18n.prompt_select_provider_cli,
                            &items,
                            default_idx,
                        )?;

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
                                "{} ({})",
                                i18n.common.label_api_key, i18n.cli_api_key_old_hint
                            )
                        } else {
                            format!(
                                "{} ({})",
                                i18n.common.label_api_key, i18n.label_back_to_prev_cli
                            )
                        };

                        let key = interact.password(&key_prompt)?;

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
                                &config.api_base_url,
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
                        items.push(i18n.label_back_to_prev_cli.clone()); // 避免重複返回項

                        let default_idx = if !config.model.is_empty() {
                            items.iter().position(|m| m == &config.model).unwrap_or(0)
                        } else {
                            0
                        };

                        let idx = interact.select(&prompt_text, &items, default_idx)?;

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
                            let model = interact.input(&i18n.cli_custom_model_prompt, true)?;
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
                        let input_path_str = interact.input(input_prompt, false)?;

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
                        let output_dir = interact.input(&output_prompt, true)?;

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
                            i18n.label_back_to_prev_cli.to_string(), // 避免重複返回項
                        ];

                        let start = interact.select(&i18n.prompt_confirm_start_cli, &items, 0)?;

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

            let choice = interact.select(
                &i18n.prompt_task_finished_cli,
                &[
                    i18n.prompt_new_task_cli.clone(),
                    i18n.label_no_cancel_cli.clone(),
                ],
                0,
            )?;

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

    let logger = |entry: mc_translator::translation::LogEntry| {
        // 去除換行避免與進度條衝突
        let clean_msg = entry.message.replace("\n", " ").trim().to_string();
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
            let filled_raw = (pct * bar_len as f32).max(0.0) as usize;
            let filled = filled_raw.min(bar_len);
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

pub trait CliInteract {
    fn select(
        &self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, dialoguer::Error>;
    fn input(&self, prompt: &str, allow_empty: bool) -> Result<String, dialoguer::Error>;
    fn password(&self, prompt: &str) -> Result<String, dialoguer::Error>;
}

pub struct RealCliInteract;

impl CliInteract for RealCliInteract {
    fn select(
        &self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, dialoguer::Error> {
        Select::new()
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
    }

    fn input(&self, prompt: &str, allow_empty: bool) -> Result<String, dialoguer::Error> {
        Input::<String>::new()
            .with_prompt(prompt)
            .allow_empty(allow_empty)
            .interact()
    }

    fn password(&self, prompt: &str) -> Result<String, dialoguer::Error> {
        Password::new()
            .with_prompt(prompt)
            .allow_empty_password(true)
            .interact()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_args_parsing_input() {
        let args = Args::try_parse_from(["mc_translator_cli", "-i", "/path/to/mod.jar"]).unwrap();
        assert_eq!(args.input.unwrap(), "/path/to/mod.jar");
        assert_eq!(args.output, None);
        assert!(!args.skip_json);
    }

    #[test]
    fn test_args_parsing_flags() {
        let args = Args::try_parse_from([
            "mc_translator_cli",
            "--input",
            "test_dir",
            "--skip-json",
            "--skip-jar",
            "-p",
            "Ollama",
        ])
        .unwrap();

        assert_eq!(args.input.unwrap(), "test_dir");
        assert!(args.skip_json);
        assert!(args.skip_jar);
        assert!(!args.skip_js); // 預設應為 false
        assert_eq!(args.provider.unwrap(), "Ollama");
    }

    #[test]
    fn test_args_parsing_integers() {
        let args = Args::try_parse_from([
            "mc_translator_cli",
            "--batch-size",
            "50",
            "--timeout",
            "120",
        ])
        .unwrap();

        assert_eq!(args.batch_size.unwrap(), 50);
        assert_eq!(args.timeout.unwrap(), 120);
    }

    pub struct MockCliInteract {
        pub select_answers: std::cell::RefCell<Vec<usize>>,
        pub input_answers: std::cell::RefCell<Vec<String>>,
        pub password_answers: std::cell::RefCell<Vec<String>>,
    }

    impl CliInteract for MockCliInteract {
        fn select(
            &self,
            prompt: &str,
            items: &[String],
            _default: usize,
        ) -> Result<usize, dialoguer::Error> {
            if prompt.contains("Model") || prompt.contains("模型") {
                let ans = self.select_answers.borrow_mut().remove(0);
                if ans == 1 {
                    return Ok(items.len() - 1);
                }
                return Ok(ans);
            }
            Ok(self.select_answers.borrow_mut().remove(0))
        }
        fn input(&self, _prompt: &str, _allow_empty: bool) -> Result<String, dialoguer::Error> {
            Ok(self.input_answers.borrow_mut().remove(0))
        }
        fn password(&self, _prompt: &str) -> Result<String, dialoguer::Error> {
            Ok(self.password_answers.borrow_mut().remove(0))
        }
    }

    #[test]
    fn test_mock_cli_interact_select_coverage() {
        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![1, 2, 3]),
            input_answers: std::cell::RefCell::new(vec![]),
            password_answers: std::cell::RefCell::new(vec![]),
        };
        let items = vec!["A".to_string(), "B".to_string(), "Back".to_string()];

        // 1. Model & ans = 1 -> items.len() - 1 = 2
        assert_eq!(mock.select("Select Model", &items, 0).unwrap(), 2);

        // 2. Model & ans = 2 -> 2
        assert_eq!(mock.select("Select Model", &items, 0).unwrap(), 2);

        // 3. Other & ans = 3 -> 3
        assert_eq!(mock.select("Something Else", &items, 0).unwrap(), 3);
    }

    #[tokio::test]
    async fn test_run_main_interactive_cancel() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 2]), // Lang, Provider, Model, Cancel
            input_answers: std::cell::RefCell::new(vec![
                "my_custom_model".to_string(),
                abs_path_str,
                "output_dir".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec!["test_key".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_headless_invalid_input() {
        let args = Args::try_parse_from(["mc_translator_cli", "-i", "non_existent_file_path_123"])
            .unwrap();
        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![]),
            input_answers: std::cell::RefCell::new(vec![]),
            password_answers: std::cell::RefCell::new(vec![]),
        };
        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_run_main_interactive_back_navigation() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_back.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 0, 2]), // Lang, Provider, Provider(again), Model, Cancel
            input_answers: std::cell::RefCell::new(vec![
                "my_custom_model".to_string(),
                abs_path_str,
                "output_dir".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec![
                "<".to_string(),
                "test_key".to_string(),
            ]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_headless_valid_input() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_headless.json");
        let _ = std::fs::write(&temp_file, "{}");

        let args = Args::try_parse_from([
            "mc_translator_cli",
            "-i",
            temp_file.to_string_lossy().as_ref(),
            "-o",
            "output_dir_test",
            "-p",
            "Gemini",
            "-m",
            "gemini-1.5-pro",
            "--api-key",
            "test_key",
            "--log-llm",
            "--batch-size",
            "100",
            "--batch-max-chars",
            "2000",
            "--timeout",
            "30",
            "--glossary-priority",
            "official",
            "--source-lang",
            "en",
            "--target-lang",
            "es",
            "--skip-json",
            "--skip-js",
            "--skip-jar",
            "--skip-book",
        ])
        .unwrap();
        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![]),
            input_answers: std::cell::RefCell::new(vec![]),
            password_answers: std::cell::RefCell::new(vec![]),
        };
        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_ollama() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_ollama.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![0, 4, 0, 2]), // Lang, Ollama, Model(Custom), Cancel
            input_answers: std::cell::RefCell::new(vec![
                "llama3".to_string(),
                abs_path_str,
                "output_dir".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec![]), // Skipped Step 3
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_google_free() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_google_free.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![0, 6, 2]), // Lang, Google Free, Cancel
            input_answers: std::cell::RefCell::new(vec![abs_path_str, "output_dir".to_string()]), // Skipped 4 Model, so input starts at Path
            password_answers: std::cell::RefCell::new(vec![]), // Skipped 3 APIKey
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_backoffs() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_backoff.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            // Select Answers Flow:
            // 1. Lang: 0 (zh_tw) -> Step 2
            // 2. Provider: 7 (Back) -> Pop -> Step 1 again (line 227 cover)
            // 3. Lang again: 0 -> Step 2
            // 4. Provider: 4 (Ollama) -> Step 4 (Key skip)
            // 5. Model: 1 (Back) -> inside Ollama back to 2 (line 316 cover)
            // 6. Provider again: 0 (Gemini) -> Step 3
            // 7. Key: "test_key" -> Step 4
            // 8. Model: 0 (Custom) -> Input "mod_a" -> Step 5
            // 9. Path: Input -> Step 6
            // 10. Output: Input -> Step 7
            // 11. Confirm: 3 (Back) -> Step 6 (line 400 cover)
            // 12. Output again: Input -> Step 7
            // 13. Confirm: 2 (Cancel) -> Exit
            select_answers: std::cell::RefCell::new(vec![0, 7, 0, 4, 1, 0, 0, 3, 2]),
            input_answers: std::cell::RefCell::new(vec![
                "mod_a".to_string(),
                abs_path_str.clone(),
                "out".to_string(),
                "out2".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec!["test_key".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_advanced() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_adv.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            // Select Answers Flow:
            // 1. Lang: 0 -> Step 2
            // 2. Provider: 0 (Gemini) -> Step 3
            // 3. Key: "key" -> Step 4
            // 4. Model: 0 (Custom) -> Input -> Step 5
            // 5. Path: abs_path_str -> Step 6
            // 6. Output: "out" -> Step 7
            // 7. Confirm: 1 (Advanced) -> triggering loop break (line 407 cover)
            // 8. Finished Choice: 1 (Cancel/Exit)
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 0, 1, 1]),
            input_answers: std::cell::RefCell::new(vec![
                "mod_b".to_string(),
                abs_path_str.clone(),
                "out".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec!["key".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_confirm_success() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_confirm.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            // Select Answers Flow:
            // 1. Lang: 0 -> Step 2
            // 2. Provider: 0 (Gemini) -> Step 3
            // 3. Key -> input password -> Step 4
            // 4. Model: 0 (Custom) -> Input name -> Step 5
            // 5. Path: Input -> Step 6
            // 6. Output: Input -> Step 7
            // 7. Confirm: 0 (Confirm) -> Breaking while loop (line 440 cover)
            // 8. Finished Choice: 1 (No/Cancel to exit outer loop) (line 452 cover)
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 0, 1]),
            input_answers: std::cell::RefCell::new(vec![
                "mod_confirm".to_string(),
                abs_path_str.clone(),
                "out_confirm".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec!["test_key".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_headless_def_output() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_headless_def.json");
        let _ = std::fs::write(&temp_file, "{}");

        // Omit output -o option to trigger defaults line 176
        let args = Args::try_parse_from([
            "mc_translator_cli",
            "-i",
            temp_file.to_string_lossy().as_ref(),
            "-p",
            "Gemini",
            "-m",
            "gemini-1.5-pro",
        ])
        .unwrap();

        let mock = MockCliInteract {
            select_answers: std::cell::RefCell::new(vec![]),
            input_answers: std::cell::RefCell::new(vec![]),
            password_answers: std::cell::RefCell::new(vec![]),
        };
        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_step5_back() {
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_step5_back.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            // Select Answers Flow:
            // 1. Lang: 0 -> Step 2
            // 2. Provider: 0 (Gemini) -> Step 3
            // 3. Key -> Step 4
            // 4. Model: 0 (Custom) -> Step 5
            // 5. Path: "<" (Back) -> Pop -> Back to Step 4 Model
            // 6. Model again: 0 (Custom) -> Step 5
            // 7. Path: abs_path_str -> Step 6
            // 8. Output: "out" -> Step 7
            // 9. Confirm: 2 (Cancel) -> Exit
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 0, 2]),
            input_answers: std::cell::RefCell::new(vec![
                "mod_step5_1".to_string(),
                "<".to_string(),
                "mod_step5_2".to_string(),
                abs_path_str.clone(),
                "out_dir".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec!["test_key".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_advanced_branch() {
        // This test hits start == 1 (Advanced) branch at line 407
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_adv_b.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            // 1 Lang -> 2 Provider(Gemini) -> 3 Key -> 4 Model(Custom) -> 5 Path -> 6 Output
            // -> 7 Confirm:1 (Advanced) -> break -> run_translation -> finished choice: 1 (Exit)
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 1, 1]),
            input_answers: std::cell::RefCell::new(vec![
                "m_adv_b".to_string(),
                abs_path_str.clone(),
                "out_adv_b".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec!["key_adv_b".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_new_task_loop() {
        // This test hits new_task loop (lines 429-440) when Confirm=0 (Start), finished choice=0 (New), then Cancel
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_newloop.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            // Round 1:
            //   1 Lang -> 2 Provider(Gemini) -> 3 Key -> 4 Model(Custom) -> 5 Path -> 6 Output
            //   -> 7 Confirm:0 (Yes, Start) -> break while -> run_translation
            //   -> finished choice: 0 (New task) -> next_step=5, initial_history=[1,2,3,4]
            // Round 2 (next_step=5, initial_history=[1,2,3,4]):
            //   5 Path -> 6 Output -> 7 Confirm:2 (Cancel) -> Exit
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 0, 0, 2]),
            input_answers: std::cell::RefCell::new(vec![
                "m_loop".to_string(),    // Round1: Step4 model custom input
                abs_path_str.clone(),    // Round1: Step5 path
                "out_loop".to_string(),  // Round1: Step6 output
                abs_path_str.clone(),    // Round2: Step5 path
                "out_loop2".to_string(), // Round2: Step6 output
            ]),
            password_answers: std::cell::RefCell::new(vec!["key_loop".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_has_saved_key() {
        // Test empty password input (hits line 261: !key.trim().is_empty() = false)
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_hskey.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        let mock = MockCliInteract {
            // 1 Lang -> 2 Provider(Gemini=0) -> 3 Key(empty) -> 4 Model(Custom=0) -> 5 Path -> 6 Out -> 7 Cancel
            select_answers: std::cell::RefCell::new(vec![0, 0, 0, 2]),
            input_answers: std::cell::RefCell::new(vec![
                "m_hskey".to_string(),
                abs_path_str.clone(),
                "out_hskey".to_string(),
            ]),
            // empty password hits the !key.trim().is_empty() = false branch
            password_answers: std::cell::RefCell::new(vec!["".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_run_main_interactive_non_custom_model() {
        // Cover the else branch at line 331: model = items[idx].to_string() when idx != custom
        let args = Args::try_parse_from(["mc_translator_cli"]).unwrap();

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("temp_test_input_noncustom.json");
        let _ = std::fs::write(&temp_file, "{}");
        let abs_path_str = temp_file.to_string_lossy().to_string();

        // Use DeepL (provider idx=5) which populates items. idx=0 picks first model (non-custom)
        // Actually fetch_dynamic_models returns empty for DeepL, so items = [Custom, Back]
        // To hit non-custom, we need a dynamic model. Since network is unavailable in tests,
        // items will always be [Custom, Back]. We target DeepL which skips the prompt_text fallback (Ollama/DeepL case)
        let mock = MockCliInteract {
            // 1 Lang -> 2 Provider(DeepL=5) -> 3 Key -> 4 Model: 0(Custom) -> 5 Path -> 6 Out -> 7 Cancel
            select_answers: std::cell::RefCell::new(vec![0, 5, 0, 2]),
            input_answers: std::cell::RefCell::new(vec![
                "deepl_mod".to_string(),
                abs_path_str.clone(),
                "out_dl".to_string(),
            ]),
            password_answers: std::cell::RefCell::new(vec!["key_dl".to_string()]),
        };

        let res = run_main_with_args(args, &mock).await;
        assert!(res.is_ok());

        let _ = std::fs::remove_file(&temp_file);
    }
}
