use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Minecraft 官方語言檔案集合 (動態快取)
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct McLangFiles {
    pub langs: HashMap<String, HashMap<String, String>>, // "en_us" -> { "key": "value" }
}

#[derive(Deserialize)]
struct GithubContentItem {
    name: String,
    download_url: Option<String>,
}

/// 從本地快取或 GitHub 下載並建構 mc_lang 字典
/// 回傳: (語言檔案, 精確匹配表, 常規差異表)
pub async fn load_mc_dicts(
    source_lang: &str,
    target_lang: &str,
) -> Result<
    (McLangFiles, HashMap<String, String>, Vec<(String, String)>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    load_mc_dicts_with_args(
        source_lang,
        target_lang,
        "https://api.github.com/repos/SkyEye-FAST/mc_lang/contents/valid",
        std::path::Path::new("dicts"),
    )
    .await
}

pub async fn load_mc_dicts_with_args(
    source_lang: &str,
    target_lang: &str,
    api_url: &str,
    dict_dir: &std::path::Path,
) -> Result<
    (McLangFiles, HashMap<String, String>, Vec<(String, String)>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    if !dict_dir.exists() {
        let _ = fs::create_dir_all(dict_dir);
    }

    let mut files = McLangFiles::default();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("mc_translator") // GitHub API 必要 Header
        .build()?;

    // 1. 取得目錄下的檔案清單 (優先透過 API 獲取所有官方支援字典)
    let mut available_langs = Vec::new();

    if let Ok(resp) = client.get(api_url).send().await {
        if resp.status().is_success() {
            if let Ok(items) = resp.json::<Vec<GithubContentItem>>().await {
                for item in items {
                    if item.name.ends_with(".json") {
                        let code = item.name.replace(".json", "");
                        available_langs.push(code.clone());

                        // 確保下載至快取
                        let cache_path = dict_dir.join(&item.name);
                        if !cache_path.exists() {
                            if let Some(dl_url) = item.download_url {
                                if let Ok(dl_resp) = client.get(&dl_url).send().await {
                                    if let Ok(txt) = dl_resp.text().await {
                                        let _ = fs::write(&cache_path, txt);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. 載入本地所有快取檔 (作為 Fallback 或主要的載入來源)
    if let Ok(entries) = fs::read_dir(dict_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(json_map) =
                            serde_json::from_str::<HashMap<String, String>>(&content)
                        {
                            files.langs.insert(stem.to_string(), json_map);
                        }
                    }
                }
            }
        }
    }

    let mut exact = HashMap::new();
    let mut unfiltered_diffs = Vec::new();

    // 3. 僅當條件為 en_us -> zh_tw 時，進入術語系統處理
    if source_lang == "en_us" && target_lang == "zh_tw" {
        if let (Some(en), Some(tw)) = (files.langs.get("en_us"), files.langs.get("zh_tw")) {
            // 建構精確匹配表
            for (k, v) in en {
                if let Some(tw_val) = tw.get(k) {
                    exact.insert(v.to_lowercase(), tw_val.clone());
                }
            }
        }

        if let (Some(cn), Some(tw)) = (files.langs.get("zh_cn"), files.langs.get("zh_tw")) {
            // 建構常規差異表 (Unfiltered)
            for (k, cn_val) in cn {
                if let Some(tw_val) = tw.get(k) {
                    if cn_val != tw_val {
                        let converted = hanconv::s2tw(cn_val);
                        if converted != *tw_val {
                            unfiltered_diffs.push((cn_val.clone(), tw_val.clone()));
                        }
                    }
                }
            }
            unfiltered_diffs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        }
    }

    Ok((files, exact, unfiltered_diffs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_load_mc_dicts_local_fallback() {
        let temp_dir = std::env::temp_dir().join("mc_lang_test_local");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 1. 寫入本地 dummy data
        fs::write(
            temp_dir.join("en_us.json"),
            r#"{"item.apple": "Apple", "item.potato": "Potato"}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.join("zh_tw.json"),
            r#"{"item.apple": "蘋果", "item.potato": "洋芋"}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.join("zh_cn.json"),
            r#"{"item.apple": "苹果", "item.potato": "马铃薯"}"#,
        )
        .unwrap();

        // 2. 測試讀取 (en_us -> zh_tw)
        let (files, exact, unfiltered) = load_mc_dicts_with_args(
            "en_us",
            "zh_tw",
            "http://invalid_url_xyz", // 促使網路失敗 fallback
            &temp_dir,
        )
        .await
        .unwrap();

        assert!(!files.langs.is_empty());
        assert_eq!(exact.get("apple").unwrap(), "蘋果");
        assert!(!unfiltered.is_empty());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_load_mc_dicts_network_success_mock() {
        let mock_server = MockServer::start().await;

        let temp_dir = std::env::temp_dir().join("mc_lang_test_net");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Mock 1: Github Contents valid API
        let api_response = r#"[{"name": "en_us.json", "download_url": "PLACEHOLDER"}]"#;
        let api_response =
            api_response.replace("PLACEHOLDER", &format!("{}/en_us.json", mock_server.uri()));

        Mock::given(method("GET"))
            .and(path("/contents/valid"))
            .respond_with(ResponseTemplate::new(200).set_body_string(api_response))
            .mount(&mock_server)
            .await;

        // Mock 2: Download link
        Mock::given(method("GET"))
            .and(path("/en_us.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"item.bed": "Bed"}"#))
            .mount(&mock_server)
            .await;

        let api_url = format!("{}/contents/valid", mock_server.uri());

        let (files, _, _) = load_mc_dicts_with_args("en_us", "zh_tw", &api_url, &temp_dir)
            .await
            .unwrap();

        // 驗證網路下載覆寫成功，本地快取建立
        assert!(temp_dir.join("en_us.json").exists());
        assert!(!files.langs.get("en_us").unwrap().is_empty());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
