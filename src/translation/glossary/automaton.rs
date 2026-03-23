use aho_corasick::{AhoCorasick, MatchKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct GlossaryAutomaton {
    pub ac: AhoCorasick,
    pub entries: Vec<GlossaryEntry>,
}

impl GlossaryAutomaton {
    /// 建構术術自動機，使用預先分析好的地圖
    pub fn new(
        official_map: &HashMap<String, String>,
        user_map: &HashMap<String, String>,
        inferred_map: &HashMap<String, String>,
        priority: &str, // "official" 或 "user"
    ) -> Self {
        let mut terms_map: HashMap<String, String> = HashMap::new();
        let mut source_types: HashMap<String, TermType> = HashMap::new();

        let load_official = |target_map: &mut HashMap<String, String>,
                             st: &mut HashMap<String, TermType>| {
            for (k, v) in official_map {
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
            for (k, v) in user_map {
                if k.chars().any(|c| c.is_alphabetic()) {
                    let key_low = k.to_lowercase();
                    if !target_map.contains_key(&key_low) {
                        target_map.insert(key_low.clone(), v.clone());
                        st.insert(key_low, TermType::Official); // USERDICT is also treated as fixed official priority in engine
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

        // 加上推論的部分 (不重複分析，直接使用傳入的 map)
        for (k, v) in inferred_map {
            let key_low = k.to_lowercase();
            if !terms_map.contains_key(&key_low) {
                terms_map.insert(key_low.clone(), v.clone());
                source_types.insert(key_low.clone(), TermType::Inferred);
            }
        }

        Self::build_automaton(terms_map, source_types)
    }

    /// 建構术語自動機（簡化版，不帶推論）
    pub fn new_simple(
        exact_map: &HashMap<String, String>,
        memory_map: &HashMap<String, String>,
    ) -> Self {
        Self::new(exact_map, memory_map, &HashMap::new(), "official")
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 1. 正常路徑測試（正常流程）
    #[test]
    fn test_glossary_match_standard() {
        let mut exact = HashMap::new();
        exact.insert("Apple".to_string(), "蘋果".to_string());
        exact.insert("Stone".to_string(), "石頭".to_string());

        let automaton =
            GlossaryAutomaton::new(&exact, &HashMap::new(), &HashMap::new(), "official");
        let results = automaton.extract(&["I have an Apple and some Stone".to_string()]);

        assert!(results.contains_key("apple")); // match is lowercase
        assert_eq!(results.get("apple").unwrap().0, "蘋果");
        assert!(results.contains_key("stone"));
    }

    /// 2. 邊界值與 UTF-8 測試（邊界案例 / UTF-8）
    #[test]
    fn test_glossary_match_utf8() {
        let mut exact = HashMap::new();
        // 包含表情符號與複合字元的 UTF-8 術語
        exact.insert("❄️ Ice方塊".to_string(), "冰塊".to_string());

        let automaton =
            GlossaryAutomaton::new(&exact, &HashMap::new(), &HashMap::new(), "official");
        // 測試在句子中精確匹配 UTF-8 術語
        let results = automaton.extract(&["這是一個 ❄️ Ice方塊 在這裡".to_string()]);

        assert!(results.contains_key("❄️ ice方塊"));
        assert_eq!(results.get("❄️ ice方塊").unwrap().0, "冰塊");
    }

    /// 3. 強韌性與邏輯防錯（健壯性 / 邏輯防錯）
    #[test]
    fn test_glossary_circular_definition_safety() {
        let mut exact = HashMap::new();
        // 模擬循環定義：A -> B, B -> A
        exact.insert("A".to_string(), "B".to_string());
        exact.insert("B".to_string(), "A".to_string());

        let automaton =
            GlossaryAutomaton::new(&exact, &HashMap::new(), &HashMap::new(), "official");
        let results = automaton.extract(&["A and B".to_string()]);

        // 自動機應能正常提取，且不應陷入死循環（因為 Aho-Corasick 是單次掃描）
        assert!(results.contains_key("a"));
        assert!(results.contains_key("b"));
    }

    /// 4. 優先權合併測試 (Official vs User)
    #[test]
    fn test_glossary_priority_merging() {
        let mut official = HashMap::new();
        official.insert("Apple".to_string(), "官方蘋果".to_string());

        let mut user = HashMap::new();
        user.insert("Apple".to_string(), "使用者蘋果".to_string());

        // 測試 Official 優先
        let automaton_off = GlossaryAutomaton::new(&official, &user, &HashMap::new(), "official");
        let results_off = automaton_off.extract(&["Apple".to_string()]);
        assert_eq!(results_off.get("apple").unwrap().0, "官方蘋果");

        // 測試 User 優先
        let automaton_user = GlossaryAutomaton::new(&official, &user, &HashMap::new(), "user");
        let results_user = automaton_user.extract(&["Apple".to_string()]);
        assert_eq!(results_user.get("apple").unwrap().0, "使用者蘋果");
    }

    /// 5. 邊界匹配測試 (避免單字內嵌誤判)
    #[test]
    fn test_glossary_match_boundaries() {
        let mut exact = HashMap::new();
        exact.insert("Apple".to_string(), "蘋果".to_string());

        let automaton =
            GlossaryAutomaton::new(&exact, &HashMap::new(), &HashMap::new(), "official");

        // 包含在單字內，不應匹配 (Pineapple)
        let results = automaton.extract(&["Pineapple".to_string()]);
        assert!(!results.contains_key("apple"));

        // 作為獨立單字應匹配
        let results_ok = automaton.extract(&["I eat an Apple".to_string()]);
        assert!(results_ok.contains_key("apple"));
    }
}
