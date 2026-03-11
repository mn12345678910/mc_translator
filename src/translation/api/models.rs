use super::client::CLIENT;

/// 從 Ollama 伺服器取得可用模型列表
pub async fn fetch_ollama_models(ollama_url: &str) -> Vec<String> {
    let url = format!("{}/api/tags", ollama_url.trim_end_matches('/'));

    match CLIENT
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(models) = json["models"].as_array() {
                    let mut model_names: Vec<String> = Vec::new();
                    for m in models {
                        if let Some(name) = m["name"].as_str() {
                            model_names.push(name.to_string());
                        }
                    }
                    return model_names;
                }
            }
            Vec::new()
        }
        Err(_) => Vec::new(),
    }
}

pub async fn fetch_dynamic_models(
    provider: &str,
    api_key: &str,
    ollama_url: &str,
) -> Result<Vec<String>, String> {
    if api_key.is_empty() && provider != "Ollama" {
        return Ok(Vec::new());
    }
    let res = match provider {
        "Gemini" => fetch_gemini_models(api_key).await,
        "OpenAI" => {
            fetch_openai_compatible_models(api_key, "https://api.openai.com/v1/models").await
        }
        "DeepSeek" => {
            fetch_openai_compatible_models(api_key, "https://api.deepseek.com/v1/models").await
        }
        "Mistral" => {
            fetch_openai_compatible_models(api_key, "https://api.mistral.ai/v1/models").await
        }
        "Ollama" => fetch_ollama_models(ollama_url).await,
        "DeepL" => vec!["deepl-free".to_string(), "deepl-standard".to_string()],
        _ => Vec::new(),
    };
    Ok(res)
}

async fn fetch_gemini_models(api_key: &str) -> Vec<String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );
    if let Ok(resp) = CLIENT.get(url).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(models) = json["models"].as_array() {
                return models
                    .iter()
                    .filter_map(|m| {
                        let name = m["name"].as_str()?;
                        if name.contains("gemini")
                            && m["supportedGenerationMethods"]
                                .as_array()?
                                .iter()
                                .any(|v| v == "generateContent")
                        {
                            Some(name.replace("models/", ""))
                        } else {
                            None
                        }
                    })
                    .collect();
            }
        }
    }
    Vec::new()
}

async fn fetch_openai_compatible_models(api_key: &str, url: &str) -> Vec<String> {
    if let Ok(resp) = CLIENT
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
    {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(models) = json["data"].as_array() {
                return models
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
    }
    Vec::new()
}

pub async fn fetch_mc_versions() -> Vec<(String, u32)> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
    if let Ok(resp) = CLIENT.get(url).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(versions) = json["versions"].as_array() {
                let mut results: Vec<(String, u32)> = Vec::new();
                for v in versions {
                    if v["type"].as_str() == Some("release") {
                        if let Some(id) = v["id"].as_str() {
                            results.push((id.to_string(), version_to_pack_format(id)));
                        }
                    }
                }
                return results;
            }
        }
    }
    get_static_mc_versions()
}

pub fn version_to_pack_format(version: &str) -> u32 {
    match version {
        "1.6.1" | "1.6.2" | "1.6.4" | "1.7.2" | "1.7.4" | "1.7.5" | "1.7.6" | "1.7.7" | "1.7.8"
        | "1.7.9" | "1.7.10" | "1.8" | "1.8.1" | "1.8.2" | "1.8.3" | "1.8.4" | "1.8.5"
        | "1.8.6" | "1.8.7" | "1.8.8" | "1.8.9" => 1,
        "1.9" | "1.9.1" | "1.9.2" | "1.9.3" | "1.9.4" | "1.10" | "1.10.1" | "1.10.2" => 2,
        "1.11" | "1.11.1" | "1.11.2" | "1.12" | "1.12.1" | "1.12.2" => 3,
        "1.13" | "1.13.1" | "1.13.2" | "1.14" | "1.14.1" | "1.14.2" | "1.14.3" | "1.14.4" => 4,
        "1.15" | "1.15.1" | "1.15.2" | "1.16" | "1.16.1" => 5,
        "1.16.2" | "1.16.3" | "1.16.4" | "1.16.5" => 6,
        "1.17" | "1.17.1" => 7,
        "1.18" | "1.18.1" | "1.18.2" => 8,
        "1.19" | "1.19.1" | "1.19.2" => 9,
        "1.19.3" => 12,
        "1.19.4" => 13,
        "1.20" | "1.20.1" => 15,
        "1.20.2" => 18,
        "1.20.3" | "1.20.4" => 22,
        "1.20.5" | "1.20.6" => 32,
        "1.21" | "1.21.1" => 34,
        "1.21.2" | "1.21.3" => 42,
        "1.21.4" => 46,
        _ => 46,
    }
}

fn get_static_mc_versions() -> Vec<(String, u32)> {
    let known_versions = [
        "1.21.4", "1.21.3", "1.21.2", "1.21.1", "1.21", "1.20.6", "1.20.5", "1.20.4", "1.20.3",
        "1.20.2", "1.20.1", "1.20", "1.19.4", "1.19.3", "1.19.2", "1.19.1", "1.19", "1.18.2",
        "1.18.1", "1.18",
    ];
    known_versions
        .into_iter()
        .map(|v| (v.to_string(), version_to_pack_format(v)))
        .collect()
}
