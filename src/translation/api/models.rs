use crate::translation::api::client::CLIENT;

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
                    for model in models {
                        if let Some(name) = model["name"].as_str() {
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
    fetch_gemini_models_with_url(api_key, "https://generativelanguage.googleapis.com").await
}

async fn fetch_gemini_models_with_url(api_key: &str, base_url: &str) -> Vec<String> {
    let url = format!(
        "{}/v1beta/models?key={}",
        base_url.trim_end_matches('/'),
        api_key
    );
    if let Ok(resp) = CLIENT.get(&url).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(models) = json["models"].as_array() {
                return models
                    .iter()
                    .filter_map(|model| {
                        let name = model["name"].as_str()?;
                        if name.contains("gemini")
                            && model["supportedGenerationMethods"]
                                .as_array()?
                                .iter()
                                .any(|method| method == "generateContent")
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
                    .filter_map(|model| model["id"].as_str().map(|id| id.to_string()))
                    .collect();
            }
        }
    }
    Vec::new()
}

pub async fn fetch_mc_versions() -> Vec<(String, u32)> {
    fetch_mc_versions_with_url("https://launchermeta.mojang.com").await
}

pub async fn fetch_mc_versions_with_url(base_url: &str) -> Vec<(String, u32)> {
    let url = format!(
        "{}/mc/game/version_manifest_v2.json",
        base_url.trim_end_matches('/')
    );
    if let Ok(resp) = CLIENT.get(&url).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(versions) = json["versions"].as_array() {
                let mut results: Vec<(String, u32)> = Vec::new();
                for version in versions {
                    if version["type"].as_str() == Some("release") {
                        if let Some(id) = version["id"].as_str() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_fetch_ollama_models_success() {
        let server = MockServer::start().await;
        let mock_response = serde_json::json!({
            "models": [{"name": "llama3"}, {"name": "mistral"}]
        });

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        let models = fetch_ollama_models(&server.uri()).await;
        assert_eq!(models.len(), 2);
        assert_eq!(models[0], "llama3");
    }

    #[tokio::test]
    async fn test_fetch_gemini_models_with_url_success() {
        let server = MockServer::start().await;
        let mock_response = serde_json::json!({
            "models": [
                {
                    "name": "models/gemini-1.5",
                    "supportedGenerationMethods": ["generateContent"]
                },
                {
                    "name": "models/gemini-pro",
                    "supportedGenerationMethods": ["embedContent"]
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/v1beta/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        let models = fetch_gemini_models_with_url("test_key", &server.uri()).await;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], "gemini-1.5");
    }

    #[tokio::test]
    async fn test_fetch_openai_compatible_models_success() {
        let server = MockServer::start().await;
        let mock_response = serde_json::json!({
            "data": [{"id": "gpt-4"}, {"id": "gpt-3.5-turbo"}]
        });

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        let models =
            fetch_openai_compatible_models("test_key", &format!("{}/v1/models", server.uri()))
                .await;
        assert_eq!(models.len(), 2);
        assert_eq!(models[0], "gpt-4");
    }

    #[tokio::test]
    async fn test_fetch_mc_versions_with_url_success() {
        let server = MockServer::start().await;
        let mock_response = serde_json::json!({
            "versions": [
                {"type": "release", "id": "1.21.4"},
                {"type": "snapshot", "id": "1.21.5-pre1"}
            ]
        });

        Mock::given(method("GET"))
            .and(path("/mc/game/version_manifest_v2.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        let versions = fetch_mc_versions_with_url(&server.uri()).await;
        assert!(!versions.is_empty());
        assert_eq!(versions[0].0, "1.21.4");
        assert_eq!(versions[0].1, 46);
    }

    #[test]
    fn test_version_to_pack_format_branches() {
        assert_eq!(version_to_pack_format("1.21.4"), 46);
        assert_eq!(version_to_pack_format("1.20.1"), 15);
        assert_eq!(version_to_pack_format("1.12.2"), 3);
        assert_eq!(version_to_pack_format("1.9"), 2); // 新增
        assert_eq!(version_to_pack_format("1.6.1"), 1); // 新增
        assert_eq!(version_to_pack_format("9.9.9"), 46);
    }

    #[tokio::test]
    async fn test_fetch_dynamic_models_all_providers() {
        let list = fetch_dynamic_models("DeepL", "dummy_key", "")
            .await
            .unwrap();
        assert_eq!(list.len(), 2);

        // 擴充測試其他 Provider 路由
        let _ = fetch_dynamic_models("OpenAI", "dummy_key", "").await;
        let _ = fetch_dynamic_models("DeepSeek", "dummy_key", "").await;
        let _ = fetch_dynamic_models("Mistral", "dummy_key", "").await;

        let list_empty = fetch_dynamic_models("Gemini", "", "").await.unwrap();
        assert!(list_empty.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_ollama_models_errors() {
        // 1. 測網故障 (連接一個不存在的埠口)
        let models_err = fetch_ollama_models("http://localhost:1").await;
        assert!(models_err.is_empty());

        // 2. 測解析故障 (JSON 不是陣列)
        let server = MockServer::start().await;
        let mock_response = serde_json::json!({ "models": "not_array" });
        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        let models_parse = fetch_ollama_models(&server.uri()).await;
        assert!(models_parse.is_empty());
    }

    #[tokio::test]
    async fn test_original_wrappers_fallback() {
        // 原裝 Wrapper 的網路調用覆蓋，呼叫即可（會因為 real key 無效或 network 走入 Err/Empty 分支）
        let _ = fetch_gemini_models("invalid_key").await;

        // fetch_mc_versions 會打 launchermeta.mojang.com 並返回結果或靜態備份
        let versions = fetch_mc_versions().await;
        assert!(!versions.is_empty());
    }
}
