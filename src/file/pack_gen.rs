use crate::translation::job::JobConfig;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn write_to_temp_or_output(
    config: &JobConfig,
    translated_files: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base_output = if config.output_dir.is_empty() {
        Path::new("LLMTranslator")
    } else {
        Path::new(&config.output_dir)
    };
    
    // 統一輸出至 LLMTranslator 子目錄 (需求 1)
    let output_path = if config.output_dir.is_empty() {
        base_output.to_path_buf()
    } else {
        base_output.join("LLMTranslator")
    };

    if !output_path.exists() {
        fs::create_dir_all(&output_path).unwrap_or(());
    }

    let temp_dir = output_path.join("temp_translator");

    for (name, content) in translated_files {
        let mut clean_name = name.clone();

        if let Some(stripped) = name.strip_prefix("[BUNDLE]") {
            clean_name = stripped.to_string();
        }

        let name_unix = clean_name.replace('\\', "/");
        let mut final_path = if name_unix.ends_with("en_us.json") {
            name_unix.replace("en_us.json", "zh_tw.json")
        } else {
            name_unix.clone()
        };

        let is_absolute = Path::new(&clean_name).is_absolute();
        let has_dirs = name_unix.contains("/");

        if is_absolute || !has_dirs {
            let path_obj = Path::new(&clean_name);
            if let (Some(parent), Some(fname)) = (path_obj.parent(), path_obj.file_name()) {
                let fname_str = fname.to_string_lossy();
                let target_fname = if fname_str == "en_us.json" {
                    "zh_tw.json"
                } else {
                    &fname_str
                };

                if let Some(modid) = parent.file_name() {
                    final_path =
                        format!("assets/{}/lang/{}", modid.to_string_lossy(), target_fname);
                } else {
                    final_path = format!("assets/unknown/lang/{}", target_fname);
                }
            } else {
                let fname = name_unix.split('/').next_back().unwrap_or("zh_tw.json");
                let target_fname = if fname == "en_us.json" {
                    "zh_tw.json"
                } else {
                    fname
                };
                final_path = format!("assets/unknown/lang/{}", target_fname);
            }
        }

        if final_path.contains("patchouli_books/") {
            final_path = final_path.replace("/en_us/", "/zh_tw/");
        } else if !final_path.ends_with(".js") {
            if let Some(pos) = final_path.rfind('/') {
                let dir = &final_path[..=pos];
                if dir.ends_with("lang/") {
                    final_path = format!("{}zh_tw.json", dir);
                }
            }
        }

        if final_path.starts_with("assets/") || final_path.contains("/lang/") || final_path.contains("patchouli_books/") {
            let fs_path = temp_dir.join(&final_path);
            if let Some(parent) = fs_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&fs_path, content);
        } else {
            // 防禦性檢查：防止 final_path 為絕對路徑導致 Path::join 置換掉基礎路徑 (Path Escape Fix)
            let safe_final_path = if Path::new(&final_path).is_absolute() {
                Path::new(&final_path).file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(final_path)
            } else {
                final_path
            };

            let fs_path = output_path.join(&safe_final_path);
            if let Some(parent) = fs_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&fs_path, content);
        }
    }
    Ok(())
}

pub async fn output_resource_pack(
    _src_path: &Path,
    _translated_files: HashMap<String, String>,
    config: JobConfig,
    log: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::task::spawn_blocking(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let base_output = if config.output_dir.is_empty() {
                Path::new("LLMTranslator")
            } else {
                Path::new(&config.output_dir)
            };
            
            let output_path = if config.output_dir.is_empty() {
                base_output.to_path_buf()
            } else {
                base_output.join("LLMTranslator")
            };

            let temp_dir = output_path.join("temp_translator");

            if !temp_dir.exists() {
                return Ok(());
            }

            let pack_mcmeta = serde_json::json!({
                "pack": {
                    "pack_format": config.pack_format,
                    "description": "LLMTranslator 資源翻譯包"
                }
            });
            fs::write(
                temp_dir.join("pack.mcmeta"),
                serde_json::to_string_pretty(&pack_mcmeta)?,
            )?;

            let zip_filename = "LLMTranslator.zip";
            let zip_path = output_path.join(zip_filename);

            if zip_path.exists() {
                log.lock().unwrap().push(format!(
                    "警告：已存在相同的資源包檔案 {}，將會被直接覆蓋。",
                    zip_filename
                ));
            }

            let zip_file = fs::File::create(&zip_path)?;
            let mut zip_out = zip::ZipWriter::new(zip_file);
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            for entry in walkdir::WalkDir::new(&temp_dir) {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    let relative_path = path
                        .strip_prefix(&temp_dir)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    zip_out.start_file(relative_path, options)?;
                    let mut file = fs::File::open(path)?;
                    std::io::copy(&mut file, &mut zip_out)?;
                }
            }
            zip_out.finish()?;

            let _ = fs::remove_dir_all(&temp_dir);

            Ok(())
        },
    )
    .await??;

    Ok(())
}
