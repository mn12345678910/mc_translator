use walkdir::WalkDir;

pub fn scan_files_recursive(
    dir: &std::path::Path,
    base_dir: &std::path::Path,
) -> Vec<(std::path::PathBuf, String)> {
    let mut files = Vec::new();
    // 規範化基準路徑以避免 Windows 磁碟機代號大小寫造成的 strip_prefix 失敗
    let base_norm = base_dir.canonicalize().unwrap_or(base_dir.to_path_buf());

    for entry in WalkDir::new(dir).into_iter().flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jar" || ext == "json" || ext == "js" {
                    let path_norm = path.canonicalize().unwrap_or(path.to_path_buf());

                    let rel = match path_norm.strip_prefix(&base_norm) {
                        Ok(p) => p.to_string_lossy().to_string(),
                        Err(_) => {
                            // 跨磁碟機或規範化失敗時的安全回退：僅保留檔名，防止絕對路徑造成輸出漂移
                            path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown_file".to_string())
                        }
                    };
                    files.push((path.to_path_buf(), rel.replace('\\', "/")));
                }
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_scan_files_recursive_mixed() {
        let temp = env::temp_dir().join("mc_translator_test_scanner_mixed");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();

        let sub = temp.join("sub");
        fs::create_dir_all(&sub).unwrap();

        fs::write(temp.join("a.json"), "{}").unwrap();
        fs::write(sub.join("b.js"), "console.log()").unwrap();
        fs::write(sub.join("c.jar"), "PK").unwrap();
        fs::write(sub.join("d.txt"), "hello").unwrap(); // 應被過濾

        let res = scan_files_recursive(&temp, &temp);
        assert_eq!(res.len(), 3);

        let files: Vec<String> = res.iter().map(|(_, rel)| rel.clone()).collect();
        assert!(files.contains(&"a.json".to_string()));
        assert!(files.contains(&"sub/b.js".to_string()));
        assert!(files.contains(&"sub/c.jar".to_string()));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_scan_files_recursive_strip_prefix_fail() {
        let temp1 = env::temp_dir().join("mc_translator_test_scanner_fail1");
        let temp2 = env::temp_dir().join("mc_translator_test_scanner_fail2");

        let _ = fs::remove_dir_all(&temp1);
        let _ = fs::remove_dir_all(&temp2);
        fs::create_dir_all(&temp1).unwrap();
        fs::create_dir_all(&temp2).unwrap();

        fs::write(temp1.join("test.json"), "{}").unwrap();

        // 使用 temp2 做為 base_dir，迫使 strip_prefix 失敗
        let res = scan_files_recursive(&temp1, &temp2);
        assert_eq!(res.len(), 1);

        let (_, rel) = &res[0];
        assert_eq!(rel, "test.json");

        let _ = fs::remove_dir_all(&temp1);
        let _ = fs::remove_dir_all(&temp2);
    }
}
