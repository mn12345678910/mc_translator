use crate::translation::job::JobConfig;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn get_output_dir(config: &JobConfig) -> std::path::PathBuf {
    let base = if config.output_dir.is_empty() {
        Path::new(".")
    } else {
        Path::new(&config.output_dir)
    };
    base.join("LLMTranslator")
}

fn to_extended_abs_path(path: &Path) -> std::path::PathBuf {
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(path)
    };
    #[cfg(target_os = "windows")]
    {
        let path_str = abs_path.to_string_lossy();
        if !path_str.starts_with(r"\\?\") {
            return std::path::PathBuf::from(format!(r"\\?\{}", path_str));
        }
    }
    abs_path
}

pub fn write_to_temp_or_output(
    config: &JobConfig,
    translated_files: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output_path = to_extended_abs_path(&get_output_dir(config));

    if !output_path.exists() {
        fs::create_dir_all(&output_path).unwrap_or(());
    }

    let temp_dir = output_path.join("temp_translator");

    for (name, content) in translated_files {
        let mut clean_name = name.clone();
        let is_bundle = name.starts_with("[BUNDLE]");
        if is_bundle {
            clean_name = name.strip_prefix("[BUNDLE]").unwrap().to_string();
        }

        let name_unix = clean_name.replace('\\', "/");
        let is_json = name_unix.ends_with(".json");

        let src_suffix = format!("{}.json", config.source_lang);
        let tgt_suffix = format!("{}.json", config.target_lang);

        let mut final_path = if is_json && name_unix.ends_with(&src_suffix) {
            name_unix.replace(&src_suffix, &tgt_suffix)
        } else {
            name_unix.clone()
        };

        let is_absolute = Path::new(&clean_name).is_absolute();
        let has_dirs = name_unix.contains("/");
        let is_originally_resource =
            name_unix.starts_with("assets/") || name_unix.contains("patchouli_books/");

        // 1. 路徑轉換：僅針對 BUNDLE 中的單純 JSON 或絕對路徑 JSON 進行資源包化補全
        if (is_absolute || !has_dirs) && is_json && is_bundle {
            let path_obj = Path::new(&clean_name);
            if let (Some(parent), Some(fname)) = (path_obj.parent(), path_obj.file_name()) {
                let fname_str = fname.to_string_lossy();
                let target_fname_owned = if fname_str == src_suffix {
                    tgt_suffix.clone()
                } else {
                    fname_str.to_string()
                };

                if let Some(modid) = parent.file_name() {
                    final_path = format!(
                        "assets/{}/lang/{}",
                        modid.to_string_lossy(),
                        target_fname_owned
                    );
                } else {
                    final_path = format!("assets/unknown/lang/{}", target_fname_owned);
                }
            } else {
                let default_tgt = format!("{}.json", config.target_lang);
                let fname = name_unix.split('/').next_back().unwrap_or(&default_tgt);
                let target_fname = if fname == src_suffix {
                    &tgt_suffix
                } else {
                    fname
                };
                final_path = format!("assets/unknown/lang/{}", target_fname);
            }
        }

        // 2. 語法/手冊路徑調整 (僅針對 JSON)
        if is_json {
            let src_book_match = format!("/{}", config.source_lang);
            let tgt_book_replace = format!("/{}", config.target_lang);
            if final_path.contains("patchouli_books/") {
                final_path = final_path.replace(&src_book_match, &tgt_book_replace);
            } else if let Some(pos) = final_path.rfind('/') {
                let dir = &final_path[..=pos];
                if dir.ends_with("lang/") {
                    final_path = format!("{}{}.json", dir, config.target_lang);
                }
            }
        }

        // 3. 路徑安全清理 (僅保留 Normal 組件，防止絕對路徑或 .. 引發路徑穿越)
        let mut safe_buf = std::path::PathBuf::new();
        for comp in std::path::Path::new(&final_path).components() {
            if let std::path::Component::Normal(c) = comp {
                safe_buf.push(c);
            }
        }
        final_path = safe_buf.to_string_lossy().to_string();

        // 4. 輸出分流：
        //    - 只有 BUNDLE (JAR/Mods) 或原本就位於資源結構中的 JSON 進入 ZIP 暫存區
        //    - 獨立檔案 (JS, 非 BUNDLE JSON) 直接鏡像輸出
        let should_zip = (is_bundle || is_originally_resource) && is_json;

        if should_zip {
            let zip_temp_path = temp_dir.join(&final_path);
            if let Some(parent) = zip_temp_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&zip_temp_path, &content);
        } else {
            // 獨立檔案鏡像 (保持相對路徑)
            let fs_path = output_path.join(&final_path);
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
    i18n: crate::i18n::CommonLabels,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::task::spawn_blocking(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let output_path = to_extended_abs_path(&get_output_dir(&config));

            let temp_dir = output_path.join("temp_translator");

            if !temp_dir.exists() {
                return Ok(());
            }

            // 檢查暫存目錄是否有實際檔案需要壓縮 (排除即將生成的 pack.mcmeta)
            let has_files = walkdir::WalkDir::new(&temp_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| e.file_type().is_file());

            if !has_files {
                let _ = fs::remove_dir_all(&temp_dir); // 清理空目錄
                return Ok(());
            }

            crate::utils::add_log(
                &log,
                &i18n.log_generating_pack,
                &config.source_lang,
                &config.target_lang,
                "",
            );

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
                log.lock()
                    .unwrap()
                    .push(i18n.log_pack_item_exists_warn.replace("{}", zip_filename));
            }

            let zip_file = fs::File::create(&zip_path)?;
            let mut zip_out = zip::ZipWriter::new(zip_file);
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            for dir_entry in walkdir::WalkDir::new(&temp_dir) {
                let entry = dir_entry.unwrap();
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

            crate::utils::add_log(
                &log,
                &i18n.log_pack_gen_finished,
                &config.source_lang,
                &config.target_lang,
                "",
            );

            Ok(())
        },
    )
    .await??;

    Ok(())
}
