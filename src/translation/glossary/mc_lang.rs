use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

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
pub async fn load_mc_dicts(source_lang: &str, target_lang: &str) -> Result<
    (McLangFiles, HashMap<String, String>, Vec<(String, String)>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let dict_dir = Path::new("dicts");
    if !dict_dir.exists() {
        let _ = fs::create_dir_all(dict_dir);
    }

    let mut files = McLangFiles::default();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("mc_translator_rs") // GitHub API 必要 Header
        .build()?;

    // 1. 取得目錄下的檔案清單 (優先透過 API 獲取所有官方支援字典)
    let api_url = "https://api.github.com/repos/SkyEye-FAST/mc_lang/contents/valid";
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
                        if let Ok(json_map) = serde_json::from_str::<HashMap<String, String>>(&content) {
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

