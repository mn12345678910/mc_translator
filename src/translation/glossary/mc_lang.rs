use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

/// Minecraft 官方語言檔案集合
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct McLangFiles {
    pub en_us: HashMap<String, String>,
    pub zh_cn: HashMap<String, String>,
    pub zh_tw: HashMap<String, String>,
}

/// 從本地快取或 GitHub 下載並建構 mc_lang 字典
/// 回傳: (語言檔案, 精確匹配表, 常規差異表)
pub async fn load_mc_dicts() -> Result<
    (McLangFiles, HashMap<String, String>, Vec<(String, String)>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let dict_dir = Path::new("dicts");
    let en_us_path = dict_dir.join("en_us.json");
    let zh_cn_path = dict_dir.join("zh_cn.json");
    let zh_tw_path = dict_dir.join("zh_tw.json");

    let mut files = McLangFiles::default();
    let mut use_local = false;

    // 嘗試從本地讀取
    if en_us_path.exists() && zh_cn_path.exists() && zh_tw_path.exists() {
        if let (Ok(en), Ok(cn), Ok(tw)) = (
            fs::read_to_string(&en_us_path),
            fs::read_to_string(&zh_cn_path),
            fs::read_to_string(&zh_tw_path),
        ) {
            if let (Ok(en_json), Ok(cn_json), Ok(tw_json)) = (
                serde_json::from_str(&en),
                serde_json::from_str(&cn),
                serde_json::from_str(&tw),
            ) {
                files.en_us = en_json;
                files.zh_cn = cn_json;
                files.zh_tw = tw_json;
                use_local = true;
            }
        }
    }

    // 若本地無有效快取，則從網路下載
    if !use_local {
        let base = "https://raw.githubusercontent.com/SkyEye-FAST/mc_lang/master/valid/";
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        files.en_us = client
            .get(format!("{}en_us.json", base))
            .send()
            .await?
            .json()
            .await?;
        files.zh_cn = client
            .get(format!("{}zh_cn.json", base))
            .send()
            .await?
            .json()
            .await?;
        files.zh_tw = client
            .get(format!("{}zh_tw.json", base))
            .send()
            .await?
            .json()
            .await?;

        // 儲存至本地快取
        if !dict_dir.exists() {
            let _ = fs::create_dir_all(dict_dir);
        }
        if let Ok(en_str) = serde_json::to_string(&files.en_us) {
            let _ = fs::write(&en_us_path, en_str);
        }
        if let Ok(cn_str) = serde_json::to_string(&files.zh_cn) {
            let _ = fs::write(&zh_cn_path, cn_str);
        }
        if let Ok(tw_str) = serde_json::to_string(&files.zh_tw) {
            let _ = fs::write(&zh_tw_path, tw_str);
        }
    }

    // 建構精確匹配表
    let mut exact = HashMap::new();
    for (k, v) in &files.en_us {
        if let Some(tw) = files.zh_tw.get(k) {
            exact.insert(v.to_lowercase(), tw.clone());
        }
    }

    // 1. 常規差異表 (Unfiltered)
    let mut unfiltered_diffs = Vec::new();
    for (k, cn) in &files.zh_cn {
        if let Some(tw) = files.zh_tw.get(k) {
            if cn != tw {
                let converted = hanconv::s2tw(cn);
                if converted != *tw {
                    unfiltered_diffs.push((cn.clone(), tw.clone()));
                }
            }
        }
    }
    unfiltered_diffs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    Ok((files, exact, unfiltered_diffs))
}
