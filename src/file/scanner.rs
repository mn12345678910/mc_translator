use std::fs;

pub fn scan_files_recursive(
    dir: &std::path::Path,
    base_dir: &std::path::Path,
) -> Vec<(std::path::PathBuf, String)> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        // 規範化基準路徑以避免 Windows 磁碟機代號大小寫造成的 strip_prefix 失敗
        let base_norm = base_dir.canonicalize().unwrap_or(base_dir.to_path_buf());
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(scan_files_recursive(&path, base_dir));
            } else if path
                .extension()
                .is_some_and(|ext| ext == "jar" || ext == "json" || ext == "js")
            {
                let path_norm = path.canonicalize().unwrap_or(path.clone());
                let rel = match path_norm.strip_prefix(&base_norm) {
                    Ok(p) => p.to_string_lossy().to_string(),
                    Err(_) => {
                        // 跨磁碟機或規範化失敗時的安全回退：僅保留檔名，防止絕對路徑造成輸出漂移
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown_file".to_string())
                    }
                };
                files.push((path, rel.replace('\\', "/")));
            }
        }
    }
    files
}






