use std::collections::{HashMap, HashSet};

/// CJK 字符判斷
pub fn is_cjk(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{3400}'..='\u{4dbf}').contains(&c)
}

/// 推論結果黑名單
pub const INFERENCE_BLACKLIST: &[&str] = &["色", "的", "了", "是", "在", "有", "為", "牆上"];

pub fn clean_inferred_zh(s: &str) -> Option<String> {
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

pub fn analyze_dictionary(map: &HashMap<String, String>) -> HashMap<String, String> {
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

pub fn find_common_hanzi(texts: Vec<&String>) -> Option<String> {
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
        .max_by_key(|(s, count)| (*count, s.chars().count(), s.clone()));
    let best_suffix = suffix_counts
        .into_iter()
        .filter(|(s, count)| *count >= min_freq && s.chars().count() <= max_allowed_len)
        .max_by_key(|(s, count)| (*count, s.chars().count(), s.clone()));
    let best_ngram = ngram_counts
        .into_iter()
        .filter(|(s, count)| *count >= min_freq && s.chars().count() <= max_allowed_len)
        .max_by_key(|(s, count)| (*count, s.chars().count(), s.clone()));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cjk_check() {
        assert!(is_cjk('中'));
        assert!(is_cjk('文'));
        assert!(!is_cjk('A'));
        assert!(!is_cjk('1'));
    }

    #[test]
    fn test_clean_inferred_zh_cases() {
        assert_eq!(clean_inferred_zh("紅色"), Some("紅".to_string()));
        assert_eq!(clean_inferred_zh("色床"), Some("床".to_string()));
        assert_eq!(clean_inferred_zh("木牌"), Some("牌".to_string()));
        assert_eq!(clean_inferred_zh("的"), None); // 黑名單
    }

    #[test]
    fn test_find_common_hanzi_structure() {
        let s1 = "紅色的床".to_string();
        let s2 = "藍色的床".to_string();
        let s3 = "綠色的床".to_string();
        let texts = vec![&s1, &s2, &s3];
        let common = find_common_hanzi(texts);
        // 的床 頻率 3/3，長度 2。
        assert_eq!(common, Some("的床".to_string()));
    }

    #[test]
    fn test_analyze_dictionary_inferring() {
        let mut map = HashMap::new();
        map.insert("Red Bed".to_string(), "紅色的床".to_string());
        map.insert("Blue Bed".to_string(), "藍色的床".to_string());
        map.insert("Green Bed".to_string(), "綠色的床".to_string());

        let inferred = analyze_dictionary(&map);
        assert!(inferred.contains_key("bed"));
        assert_eq!(inferred.get("bed").unwrap(), "的床");
    }
}
