use crate::translation::job::JobConfig;
use crate::translation::glossary::TermType;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// 移除硬編碼 TECHNICAL_CONSTRAINTS 與重複的 DEFAULT_SYSTEM_PROMPT，改由設定模組統一管理

// 移除硬編碼 TECHNICAL_CONSTRAINTS，改為從 JobConfig 讀取

/// 建立包含術語表的系統提示詞
pub fn build_system_prompt(
    base_prompt: &str,
    glossary: Option<&[crate::translation::glossary::GlossaryEntry]>,
    technical_constraints: &str,
) -> String {
    let mut prompt = if base_prompt.is_empty() {
        crate::config::settings::DEFAULT_PROMPT.to_string()
    } else {
        base_prompt.to_string()
    };

    if let Some(terms) = glossary {
        if !terms.is_empty() {
            let mut official: Vec<_> = terms
                .iter()
                .filter(|t| t.source == TermType::Official)
                .collect();
            let mut inferred: Vec<_> = terms
                .iter()
                .filter(|t| t.source == TermType::Inferred)
                .collect();

            official.sort_by_key(|t| &t.original);
            inferred.sort_by_key(|t| &t.original);

            // 1. 處理術語建議 (Official + Inferred)
            if !official.is_empty() || !inferred.is_empty() {
                prompt.push_str("\n\n請根據以下【術語建議】進行翻譯（僅供參考，請靈活使用）：\n");
                let max_terms = 30;
                let mut count = 0;
                for t in official {
                    if count >= max_terms {
                        break;
                    }
                    prompt.push_str(&format!("- {} => {}\n", t.original, t.translated));
                    count += 1;
                }
                for t in inferred {
                    if count >= max_terms {
                        break;
                    }
                    prompt.push_str(&format!("- {} => {}\n", t.original, t.translated));
                    count += 1;
                }
            }
        }
    }

    prompt.push_str(technical_constraints);
    prompt
}

/// 全域共用的 HTTP 用戶端
pub(crate) static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
        .pool_idle_timeout(Some(std::time::Duration::from_secs(90)))
        .build()
        .unwrap_or_default()
});

/// 單筆翻譯路由
pub async fn translate_one(
    text: &str,
    config: &JobConfig,
    file_name: &str,
    glossary: Option<&[crate::translation::glossary::GlossaryEntry]>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match config.api_provider.as_str() {
        "Gemini" => {
            if config.api_key.is_empty() {
                translate_free_google_with_config(text, config).await
            } else {
                translate_with_gemini(text, config, file_name, glossary).await
            }
        }
        "OpenAI" | "DeepSeek" | "Mistral" => {
            if config.api_key.is_empty() {
                translate_free_google_with_config(text, config).await
            } else {
                translate_with_openai_compatible(text, config, file_name, glossary).await
            }
        }
        "Ollama" => translate_with_ollama(text, config, file_name, glossary).await,
        "DeepL" => translate_with_deepl(text, config).await,
        _ => translate_free_google_with_config(text, config).await,
    }
}

async fn call_google_api_raw(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut last_error = "未知錯誤".to_string();
    for i in 0..3 {
        match CLIENT
            .get(url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(t) = json[0][0][0].as_str() {
                                return Ok(t.to_string());
                            }
                        }
                        Err(e) => last_error = format!("JSON 解析失敗: {}", e),
                    }
                } else if resp.status().as_u16() == 429 {
                    last_error = "API 限制 (429 Too Many Requests)".to_string();
                } else {
                    last_error = format!("HTTP 錯誤: {}", resp.status());
                }
            }
            Err(e) => last_error = format!("網路連線失敗: {}", e),
        }

        if i < 2 {
            let backoff = (i + 1) * 2;
            tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
        }
    }
    Err(last_error.into())
}

/// 批量翻譯
#[allow(dead_code)]
pub async fn translate_batch(
    texts: &[String],
    config: &JobConfig,
    file_name: &str,
    glossary: Option<&[crate::translation::glossary::GlossaryEntry]>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    if texts.is_empty() {
        return Ok(HashMap::new());
    }

    let numbered: Vec<String> = texts
        .iter()
        .enumerate()
        .map(|(i, t)| format!("[{}] {}", i + 1, t))
        .collect();
    let batch_instruction = format!(
        "以下是需要翻譯的多行字串，請按照相同的編號格式輸出翻譯結果，每行一個：\n{}",
        numbered.join("\n")
    );

    let result = match config.api_provider.as_str() {
        "Gemini" => translate_with_gemini(&batch_instruction, config, file_name, glossary).await?,
        "OpenAI" | "DeepSeek" | "Mistral" => {
            translate_with_openai_compatible(&batch_instruction, config, file_name, glossary).await?
        }
        "Ollama" => {
            let ollama_user_prompt = format!(
                "以下是需要翻譯的多行字串，請按照相同的編號格式輸出翻譯結果，每行一個：\n{}",
                numbered.join("\n")
            );
            call_ollama_raw(&ollama_user_prompt, config, file_name, glossary).await?
        }
        "DeepL" => {
            translate_with_deepl(&batch_instruction, config).await?
        }
        _ => return Err("UNSUPPORTED:批量翻譯不支援免費 Google 翻譯".into()),
    };

    let result = crate::utils::text_processing::validate_and_cleanup(&result);
    let mut map = HashMap::new();

    if config.api_provider == "Ollama" || result.trim().starts_with('{') || result.contains("```") {
        let json_str_opt = if serde_json::from_str::<serde_json::Value>(result.trim()).is_ok() {
            Some(result.trim().to_string())
        } else {
            extract_json_from_text(&result)
        };

        if let Some(json_str) = json_str_opt {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(obj) = value.as_object() {
                    for (idx_str, trans_val) in obj {
                        if let (Ok(idx), Some(trans_s)) =
                            (idx_str.parse::<usize>(), trans_val.as_str())
                        {
                            if idx >= 1 && idx <= texts.len() {
                                let cleaned = crate::utils::text_processing::validate_and_cleanup(trans_s);
                                if !cleaned.is_empty() && cleaned != "{}" && cleaned != "{ }" {
                                    map.insert(texts[idx - 1].clone(), cleaned);
                                }
                            }
                        }
                    }
                    if !map.is_empty() {
                        return Ok(map);
                    }
                }
            }
        }
    }

    for cap in BATCH_INDEX_RE.captures_iter(&result) {
        if let (Ok(idx), Some(translated)) = (cap[1].parse::<usize>(), cap.get(2)) {
            if idx >= 1 && idx <= texts.len() {
                let translated_text =
                    crate::utils::text_processing::validate_and_cleanup(translated.as_str());
                map.insert(texts[idx - 1].clone(), translated_text);
            }
        }
    }

    Ok(map)
}

pub fn map_lang_google(lang: &str) -> &str {
    match lang {
        "en_us" | "en_gb" => "en",
        "zh_tw" => "zh-TW",
        "zh_cn" => "zh-CN",
        "ja_jp" => "ja",
        "ko_kr" => "ko",
        "fr_fr" => "fr",
        "de_de" => "de",
        "es_es" => "es",
        "ru_ru" => "ru",
        _ => "en",
    }
}

pub fn map_lang_deepl(lang: &str) -> &str {
    match lang {
        "en_us" => "EN-US",
        "en_gb" => "EN-GB",
        "zh_tw" | "zh_cn" => "ZH", // DeepL only supports ZH for target, handled by engine for TW
        "ja_jp" => "JA",
        "ko_kr" => "KO",
        "fr_fr" => "FR",
        "de_de" => "DE",
        "es_es" => "ES",
        "ru_ru" => "RU",
        _ => "ZH",
    }
}

async fn translate_free_google_with_config(
    text: &str,
    config: &JobConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let sl = map_lang_google(&config.source_lang);
    let tl = map_lang_google(&config.target_lang);
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&tl={}&dt=t&q={}",
        sl, tl,
        urlencoding::encode(text)
    );
    call_google_api_raw(&url).await
}

/// Gemini API 翻譯
async fn translate_with_gemini(
    text: &str,
    config: &JobConfig,
    file_name: &str,
    glossary: Option<&[crate::translation::glossary::GlossaryEntry]>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        config.selected_model, config.api_key
    );

    let sys_prompt = build_system_prompt(&config.user_prompt, glossary, &config.system_prompt);
    let body = serde_json::json!({
        "systemInstruction": {
            "parts": [{"text": sys_prompt}]
        },
        "contents": [{"parts": [{"text": text}]}],
        "generationConfig": {
            "temperature": 0.2,
            "topP": 0.95,
            "topK": 40,
            "maxOutputTokens": 4096,
        }
    });

    let resp: reqwest::Response = CLIENT
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let err_text = resp
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        if config.enable_llm_log {
            log_llm_communication(
                &format!("(System): {}\n(User): {}", sys_prompt, text),
                &format!("API_ERROR: {}", err_text),
                config,
                text.len(),
                file_name,
            );
        }
        return Err(format!("API_ERROR:Gemini API Error: {}", err_text).into());
    }

    let json: serde_json::Value = resp.json().await?;
    let translated = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| format!("解析 Gemini 回傳失敗: {:?}", json))?
        .trim();

    if config.enable_llm_log {
        log_llm_communication(
            &format!("(System): {}\n(User): {}", sys_prompt, text),
            translated,
            config,
            text.len(),
            file_name,
        );
    }

    Ok(translated.trim_matches('"').to_string())
}

/// Ollama 本地 API 翻譯
pub async fn translate_with_ollama(
    text: &str,
    config: &JobConfig,
    file_name: &str,
    glossary: Option<&[crate::translation::glossary::GlossaryEntry]>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let enhanced_prompt = if text.len() < 500 {
        format!("{}\nReturn a JSON object with a \"translated\" key containing ONLY the translated text.", text)
    } else {
        text.to_string()
    };

    let response_str = call_ollama_raw(&enhanced_prompt, config, file_name, glossary).await?;
    let response_trimmed = response_str.trim();

    let translated = if let Some(json_str) = extract_json_from_text(response_trimmed) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(obj) = v.as_object() {
                obj.get("translated")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        obj.values()
                            .next()
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| response_trimmed.to_string())
            } else {
                response_trimmed.to_string()
            }
        } else {
            response_trimmed.to_string()
        }
    } else {
        response_trimmed.to_string()
    };

    let cleaned = translated
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();
    if cleaned == "{}" || cleaned == "{ }" {
        return Err("EMPTY_RESPONSE:Ollama 回傳了空 JSON 物件".into());
    }

    Ok(cleaned)
}

fn extract_json_from_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return Some(trimmed.to_string());
    }

    // 1. 嘗試從 Markdown 代碼塊提取
    if let Some(cap) = MARKDOWN_JSON_RE.captures(trimmed) {
        let content = cap[1].trim();
        if serde_json::from_str::<serde_json::Value>(content).is_ok() {
            return Some(content.to_string());
        }
    }

    // 2. 嘗試從大括號開始提取 (Self-healing 支持)
    if let Some(m) = JSON_BRACE_RE.find(trimmed) {
        let mut candidate = m.as_str().trim().to_string();
        
        // 優先嘗試直接解析 (處理完整的 JSON)
        // 注意：這裡需要處理可能存在的後續雜質，所以我們嘗試找到最後一個 }
        if let Some(last_brace) = candidate.rfind('}') {
            let possible_json = &candidate[..=last_brace];
            if serde_json::from_str::<serde_json::Value>(possible_json).is_ok() {
                return Some(possible_json.to_string());
            }
        }

        // --- 自我修復邏輯 (Self-healing) ---
        // 3. 如果解析失敗，嘗試補齊結尾
        if !candidate.ends_with('}') {
            candidate.push('}');
            if serde_json::from_str::<serde_json::Value>(&candidate).is_ok() {
                return Some(candidate);
            }
        }
    }
    None
}

lazy_static::lazy_static! {
    static ref BATCH_INDEX_RE: regex::Regex = regex::Regex::new(r"\[(\d+)\]\s*(.+)").unwrap();
    static ref MARKDOWN_JSON_RE: regex::Regex = regex::Regex::new(r"(?s)```(?:json)?\s*([\s\S]*?)```").unwrap();
    static ref JSON_BRACE_RE: regex::Regex = regex::Regex::new(r"(?s)\{([\s\S]*)").unwrap();
}

async fn call_ollama_raw(
    text: &str,
    config: &JobConfig,
    file_name: &str,
    glossary: Option<&[crate::translation::glossary::GlossaryEntry]>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/api/generate", config.ollama_url.trim_end_matches('/'));
    let sys_prompt = build_system_prompt(&config.user_prompt, glossary, &config.system_prompt);
    let body = serde_json::json!({
        "model": config.selected_model,
        "system": sys_prompt,
        "prompt": text,
        "stream": false,
        "keep_alive": "-1h",
        "options": {
            "temperature": 0.1,
            "num_predict": 8192
        }
    });

    let full_future = async {
        let resp = CLIENT
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err::<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>(
                format!("Ollama API Error: {}", err_text).into(),
            );
        }
        Ok(resp.json::<serde_json::Value>().await?)
    };

    let json: serde_json::Value = match tokio::time::timeout(
        std::time::Duration::from_secs(config.ollama_timeout),
        full_future,
    )
    .await
    {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err(format!("OLLAMA_TIMEOUT:{}", config.ollama_timeout).into()),
    };

    let response = json["response"]
        .as_str()
        .ok_or("Ollama 回傳回應為空")?
        .to_string();
    if config.enable_llm_log {
        log_llm_communication(
            &format!("(System): {}\n(User): {}", sys_prompt, text),
            &response,
            config,
            text.len(),
            file_name,
        );
    }
    Ok(response)
}

async fn translate_with_openai_compatible(
    text: &str,
    config: &JobConfig,
    file_name: &str,
    glossary: Option<&[crate::translation::glossary::GlossaryEntry]>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = match config.api_provider.as_str() {
        "DeepSeek" => "https://api.deepseek.com/v1/chat/completions",
        "Mistral" => "https://api.mistral.ai/v1/chat/completions",
        _ => "https://api.openai.com/v1/chat/completions",
    };
    let system_content = build_system_prompt(&config.user_prompt, glossary, &config.system_prompt);
    let body = serde_json::json!({
        "model": config.selected_model,
        "messages": [
            {"role": "system", "content": system_content},
            {"role": "user", "content": text}
        ],
        "temperature": 0.3
    });

    let resp: reqwest::Response = CLIENT
        .post(url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!(
            "API_ERROR:{} API Error: {}",
            config.api_provider,
            resp.text().await?
        )
        .into());
    }

    let json: serde_json::Value = resp.json().await?;
    let translated = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("解析回傳失敗")?;
    if config.enable_llm_log {
        log_llm_communication(
            &format!("(System): {}\n(User): {}", system_content, text),
            translated,
            config,
            text.len(),
            file_name,
        );
    }
    Ok(translated.trim_matches('"').to_string())
}

async fn translate_with_deepl(
    text: &str,
    config: &JobConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let is_free = config.selected_model == "deepl-free";
    let url = if is_free {
        "https://api-free.deepl.com/v2/translate"
    } else {
        "https://api.deepl.com/v2/translate"
    };
    let target_lang = map_lang_deepl(&config.target_lang).to_string();
    let params = [
        ("auth_key", config.api_key.clone()),
        ("text", text.to_string()),
        ("target_lang", target_lang),
    ];
    let resp = CLIENT
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(format!("API_ERROR:DeepL API Error: {}", resp.text().await?).into());
    }
    let json: serde_json::Value = resp.json().await?;
    let translated = json["translations"][0]["text"]
        .as_str()
        .ok_or("解析 DeepL 失敗")?;
    Ok(translated.to_string())
}

pub fn log_llm_communication(prompt: &str, response: &str, config: &JobConfig, char_count: usize, file_name: &str) {
    use std::io::Write;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("llm_communication.log")
    {
        let settings = format!(
            "[設定]: {} / {} / <{}->{}> / 檔案: {} / 批次量: {} / 文字數量: {} / 逾時: {}s",
            config.api_provider,
            config.selected_model,
            config.source_lang,
            config.target_lang,
            if file_name.is_empty() { "無" } else { file_name },
            config.batch_size,
            char_count,
            config.ollama_timeout
        );
        let _ = file.write_all(format!("--- LLM 通訊紀錄 [{}] ---\n{}\n[發送內容]:\n{}\n\n[接收內容]:\n{}\n------------------\n\n", now, settings, prompt, response).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_self_healing() {
        // 場景 1: 完整的 JSON
        let raw1 = r#"{"1": "hello", "2": "world"}"#;
        assert!(extract_json_from_text(raw1).is_some());

        // 場景 2: Markdown 代碼塊
        let raw2 = r#"Here is result:
```json
{"1": "hello"}
```
"#;
        assert_eq!(extract_json_from_text(raw2), Some(r#"{"1": "hello"}"#.to_string()));

        // 場景 3: 自我修復 - 缺少結尾括號
        let raw3 = r#"{"1": "hello", "2": "world""#; // 缺少最後一個 }
        assert_eq!(extract_json_from_text(raw3), Some(r#"{"1": "hello", "2": "world"}"#.to_string()));

        // 場景 4: 前後有雜質但括號完整
        let raw4 = r#"The LLM says: {"key": "val"} hope you like it."#;
        assert_eq!(extract_json_from_text(raw4), Some(r#"{"key": "val"}"#.to_string()));
    }
}
