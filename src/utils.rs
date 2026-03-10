//! # 通用工具模組
//! 包含 mc_lang 字典載入與共用資料結構。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

use aho_corasick::{AhoCorasick, MatchKind};
use std::fs;

/// Minecraft 官方語言檔案集合
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct McLangFiles {
    pub en_us: HashMap<String, String>,
    pub zh_cn: HashMap<String, String>,
    pub zh_tw: HashMap<String, String>,
}

/// 從本地緩存或 GitHub 下載並建構 mc_lang 字典
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

    // 若本地無有效緩存，則從網路下載
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

pub struct GlossaryAutomaton {
    pub ac: AhoCorasick,
    pub entries: Vec<GlossaryEntry>,
}

impl GlossaryAutomaton {
    /// 建構术語自動機，回傳新發現的推斷詞
    pub fn new_with_inferred(
        exact_map: &HashMap<String, String>,
        memory_map: &HashMap<String, String>,
        priority: &str, // "official" 或 "user"
    ) -> (Self, HashMap<String, String>) {
        let mut terms_map: HashMap<String, String> = HashMap::new();
        let mut source_types: HashMap<String, TermType> = HashMap::new();

        let load_official = |target_map: &mut HashMap<String, String>,
                             st: &mut HashMap<String, TermType>| {
            for (k, v) in exact_map {
                if k.chars().any(|c| c.is_alphabetic()) {
                    let key_low = k.to_lowercase();
                    if !target_map.contains_key(&key_low) {
                        target_map.insert(key_low.clone(), v.clone());
                        st.insert(key_low, TermType::Official);
                    }
                }
            }
        };

        let load_user = |target_map: &mut HashMap<String, String>,
                         st: &mut HashMap<String, TermType>| {
            for (k, v) in memory_map {
                if k.chars().any(|c| c.is_alphabetic()) {
                    let key_low = k.to_lowercase();
                    if !target_map.contains_key(&key_low) {
                        target_map.insert(key_low.clone(), v.clone());
                        st.insert(key_low, TermType::Official);
                    }
                }
            }
        };

        if priority == "user" {
            load_user(&mut terms_map, &mut source_types);
            load_official(&mut terms_map, &mut source_types);
        } else {
            load_official(&mut terms_map, &mut source_types);
            load_user(&mut terms_map, &mut source_types);
        }

        // 推論補充
        let dict_fingerprint = exact_map.len();
        let cache_path = std::path::Path::new("dicts").join("inferred_cache.json");
        let cached_inferred: Option<HashMap<String, String>> = if cache_path.exists() {
            std::fs::read_to_string(&cache_path)
                .ok()
                .and_then(|data| {
                    serde_json::from_str::<(usize, HashMap<String, String>)>(&data).ok()
                })
                .and_then(|(fp, map)| {
                    if fp == dict_fingerprint {
                        Some(map)
                    } else {
                        None
                    }
                })
        } else {
            None
        };

        let inferred = if let Some(cached) = cached_inferred {
            cached
        } else {
            let result = analyze_dictionary(exact_map);
            if let Ok(json) = serde_json::to_string(&(dict_fingerprint, &result)) {
                let _ = std::fs::write(&cache_path, json);
            }
            result
        };

        for (k, v) in inferred {
            let key_low = k.to_lowercase();
            if !terms_map.contains_key(&key_low) {
                terms_map.insert(key_low.clone(), v.clone());
                source_types.insert(key_low.clone(), TermType::Inferred);
            }
        }

        let automaton = Self::build_automaton(terms_map, source_types);
        (automaton, HashMap::new())
    }

    /// 建構术語自動機（簡化版，不返回推斷詞）
    pub fn new(exact_map: &HashMap<String, String>, memory_map: &HashMap<String, String>) -> Self {
        let (automaton, _) = Self::new_with_inferred(exact_map, memory_map, "official");
        automaton
    }

    fn build_automaton(
        terms_map: HashMap<String, String>,
        source_types: HashMap<String, TermType>,
    ) -> Self {
        let mut patterns = Vec::with_capacity(terms_map.len());
        let mut entries = Vec::with_capacity(terms_map.len());

        let mut keys: Vec<_> = terms_map.into_iter().collect();
        keys.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (k, v) in keys {
            let source = source_types.get(&k).cloned().unwrap_or(TermType::Official);
            patterns.push(k.clone());
            entries.push(GlossaryEntry {
                original: k,
                translated: v,
                source,
            });
        }

        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .ascii_case_insensitive(true)
            .build(&patterns)
            .unwrap_or_else(|_| AhoCorasick::new(["___dummy___"]).unwrap());

        Self { ac, entries }
    }

    pub fn extract(&self, texts: &[String]) -> HashMap<String, (String, TermType)> {
        let mut result = HashMap::new();
        for text in texts {
            for mat in self.ac.find_iter(text) {
                let entry = &self.entries[mat.pattern().as_usize()];
                let start = mat.start();
                let end = mat.end();

                let is_start_boundary = start == 0
                    || !text[..start]
                        .chars()
                        .last()
                        .unwrap_or(' ')
                        .is_ascii_alphabetic();
                let is_end_boundary = end == text.len()
                    || !text[end..]
                        .chars()
                        .next()
                        .unwrap_or(' ')
                        .is_ascii_alphabetic();

                if is_start_boundary && is_end_boundary {
                    result.insert(
                        entry.original.clone(),
                        (entry.translated.clone(), entry.source.clone()),
                    );
                }
            }
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TermType {
    Official,
    Inferred,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub original: String,
    pub translated: String,
    pub source: TermType,
}

/// CJK 字符判斷
fn is_cjk(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{3400}'..='\u{4dbf}').contains(&c)
}

/// 推論結果黑名單
const INFERENCE_BLACKLIST: &[&str] = &["色", "色床", "的", "了", "是", "在", "有", "為", "牆上"];

fn clean_inferred_zh(s: &str) -> Option<String> {
    let mut result = s.to_string();
    if result.starts_with('色') && result.chars().count() > 1 {
        result = result.chars().skip(1).collect();
    }
    if result.ends_with('色') && result.chars().count() > 1 {
        result = result.chars().take(result.chars().count() - 1).collect();
    }
    if result.starts_with('木') && result.chars().count() > 1 {
        result = result.chars().skip(1).collect();
    }
    if INFERENCE_BLACKLIST.contains(&result.as_str()) {
        return None;
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

pub(crate) fn analyze_dictionary(map: &HashMap<String, String>) -> HashMap<String, String> {
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    let mut word_translations: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let stops = [
        "the", "of", "and", "a", "an", "with", "in", "to", "for", "on", "from", "into", "by", "as",
        "at", "is", "its", "or", "some", "any", "each", "every",
    ];

    for (k, v) in map {
        let words: Vec<&str> = k.split_whitespace().collect();
        if words.len() <= 1 {
            continue;
        }
        let zh_chars: Vec<char> = v.chars().filter(|c| is_cjk(*c)).collect();
        if zh_chars.is_empty() {
            continue;
        }
        for word in &words {
            let word_low = word.to_lowercase();
            if word_low.len() < 3 || stops.contains(&word_low.as_str()) {
                continue;
            }
            if !word_low.chars().all(|c| c.is_alphabetic()) {
                continue;
            }
            *word_counts.entry(word_low.clone()).or_insert(0) += 1;
            let translations = word_translations.entry(word_low).or_default();
            *translations.entry(v.clone()).or_insert(0) += 1;
        }
    }

    let mut final_inferred = HashMap::new();
    for (word, count) in word_counts {
        if count < 3 {
            continue;
        }
        if let Some(trans_map) = word_translations.get(&word) {
            let common_zh = find_common_hanzi(trans_map.keys().collect());
            if let Some(zh) = common_zh {
                if let Some(cleaned) = clean_inferred_zh(&zh) {
                    if cleaned.chars().count() <= 6 {
                        final_inferred.insert(word, cleaned);
                    }
                }
            }
        }
    }
    final_inferred
}

fn find_common_hanzi(texts: Vec<&String>) -> Option<String> {
    if texts.len() < 3 {
        return None;
    }
    let total = texts.len();
    let min_freq = (total / 2) + 1;
    let mut prefix_counts: HashMap<String, usize> = HashMap::new();
    let mut suffix_counts: HashMap<String, usize> = HashMap::new();
    let mut ngram_counts: HashMap<String, usize> = HashMap::new();

    for text in &texts {
        let chars: Vec<char> = text.chars().filter(|c| is_cjk(*c)).collect();
        if chars.is_empty() {
            continue;
        }
        for len in 1..=chars.len() {
            let prefix: String = chars[..len].iter().collect();
            *prefix_counts.entry(prefix).or_insert(0) += 1;
        }
        for len in 1..=chars.len() {
            let suffix: String = chars[chars.len() - len..].iter().collect();
            *suffix_counts.entry(suffix).or_insert(0) += 1;
        }
        let mut seen_in_this_text: HashSet<String> = HashSet::new();
        for n in 1..=chars.len() {
            for start in 0..=chars.len() - n {
                let gram: String = chars[start..start + n].iter().collect();
                if seen_in_this_text.insert(gram.clone()) {
                    *ngram_counts.entry(gram).or_insert(0) += 1;
                }
            }
        }
    }

    let max_allowed_len = texts
        .iter()
        .map(|t| t.chars().filter(|c| is_cjk(*c)).count())
        .min()
        .unwrap_or(0);
    let best_prefix = prefix_counts
        .into_iter()
        .filter(|(s, count)| *count >= min_freq && s.chars().count() <= max_allowed_len)
        .max_by_key(|(s, count)| (*count, s.chars().count()));
    let best_suffix = suffix_counts
        .into_iter()
        .filter(|(s, count)| *count >= min_freq && s.chars().count() <= max_allowed_len)
        .max_by_key(|(s, count)| (*count, s.chars().count()));
    let best_ngram = ngram_counts
        .into_iter()
        .filter(|(s, count)| *count >= min_freq && s.chars().count() <= max_allowed_len)
        .max_by_key(|(s, count)| (*count, s.chars().count()));

    let candidates: Vec<(String, usize, usize)> = [
        best_prefix.map(|(s, c)| (s.clone(), c, s.chars().count())),
        best_suffix.map(|(s, c)| (s.clone(), c, s.chars().count())),
        best_ngram.map(|(s, c)| (s.clone(), c, s.chars().count())),
    ]
    .into_iter()
    .flatten()
    .collect();

    let result = candidates
        .into_iter()
        .max_by_key(|(_, freq, len)| (*freq, *len))
        .map(|(s, _, _)| s);
    result.and_then(|s| clean_inferred_zh(&s))
}

pub fn extract_display_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(pos) = path_str.find("assets") {
        let after_assets = &path_str[pos + 7..];
        let parts: Vec<&str> = after_assets
            .split(['/', '\\'])
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() >= 2 {
            return format!("{}/{}", parts[0], parts.last().unwrap_or(&"en_us.json"));
        }
    }
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub fn add_log(log_arc: &Arc<Mutex<Vec<String>>>, msg: &str) {
    let mut log = log_arc.lock().unwrap();
    let now = chrono::Local::now();
    let timestamp = now.format("%H:%M:%S").to_string();
    for line in msg.lines() {
        if !line.trim().is_empty() {
            log.push(format!("[{}] {}", timestamp, line));
        } else {
            log.push("".to_string());
        }
    }
}

pub fn format_log_message(msg: &str) -> Vec<String> {
    let mut log_entries = Vec::new();
    let now = chrono::Local::now();
    let timestamp = now.format("%H:%M:%S").to_string();
    for line in msg.lines() {
        if !line.trim().is_empty() {
            log_entries.push(format!("[{}] {}", timestamp, line));
        } else {
            log_entries.push("".to_string());
        }
    }
    log_entries
}

pub fn hashmap_to_entries(
    map: &std::collections::HashMap<String, (String, TermType)>,
) -> Vec<GlossaryEntry> {
    let mut entries: Vec<GlossaryEntry> = map
        .iter()
        .map(|(k, (v, t))| GlossaryEntry {
            original: k.clone(),
            translated: v.clone(),
            source: t.clone(),
        })
        .collect();
    entries.sort_by(|a, b| b.original.len().cmp(&a.original.len()));
    entries
}
