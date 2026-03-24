mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 確保語言文件刷新釋放至硬碟
            let _ = mc_translator::i18n::GuiLabels::ensure_langs_exists();

            // 🟢 確保字典目錄與預設檔案存在
            use mc_translator::config::dictionary::{ensure_dicts_dir, get_user_dict_path};
            use mc_translator::config::AppConfig;
            ensure_dicts_dir();

            let config = AppConfig::load();
            let user_dict = get_user_dict_path(&config.ui_lang);

            if !user_dict.exists() {
                if let Some(p) = user_dict.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                let _ = std::fs::write(&user_dict, "{}");
            }
            // 🟢 背景啟動推論詞庫預生成 (量子提取)
            tauri::async_runtime::spawn(async move {
                let config = AppConfig::load();
                // 讀取本地字典檔跑一次分析 (強制定錨 en_us 以計算量子提取)
                if let Ok((_files, exact, _unfiltered)) =
                    mc_translator::translation::glossary::mc_lang::load_mc_dicts(
                        "en_us",
                        &config.ui_lang,
                    )
                    .await
                {
                    let inferred = mc_translator::translation::glossary::analyze_dictionary(&exact);
                    let official_dict =
                        mc_translator::config::get_official_dict_path(&config.ui_lang);

                    // 確保父目錄存在
                    if let Some(p) = official_dict.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    mc_translator::config::save_dict(&official_dict, &inferred);
                }
            });

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 讀取視窗幾何記錄並應用
            use tauri::Manager;

            if let Some(window) = app.get_webview_window("main") {
                let config = AppConfig::load();
                if config.main_width > 200.0 && config.main_height > 200.0 {
                    let _ = window.set_size(tauri::PhysicalSize::new(
                        config.main_width as u32,
                        config.main_height as u32,
                    ));
                }

                if config.main_x != 0.0 || config.main_y != 0.0 {
                    let _ = window.set_position(tauri::PhysicalPosition::new(
                        config.main_x as i32,
                        config.main_y as i32,
                    ));
                } else {
                    // 初次啟動強制置中
                    if let Ok(Some(monitor)) = window.current_monitor() {
                        let monitor_size = monitor.size();
                        let monitor_pos = monitor.position();

                        if let Ok(window_size) = window.outer_size() {
                            let x = monitor_pos.x
                                + (monitor_size.width as i32 - window_size.width as i32) / 2;
                            let y = monitor_pos.y
                                + (monitor_size.height as i32 - window_size.height as i32) / 2;

                            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                        }
                    }
                }

                // 監聽視窗關閉事件並回寫座標與尺寸
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        if let Ok(pos) = window_clone.outer_position() {
                            if let Ok(size) = window_clone.outer_size() {
                                let mut config = AppConfig::load();
                                config.main_x = pos.x as f32;
                                config.main_y = pos.y as f32;
                                config.main_width = size.width as f32;
                                config.main_height = size.height as f32;
                                config.save();
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_default_config,
            commands::get_api_key_cmd,
            commands::save_api_key_cmd,
            commands::start_translation,
            commands::get_models_from_provider,
            commands::get_i18n_labels,
            commands::get_style_config,
            commands::save_style_config,
            commands::get_default_style_config,
            commands::query_dictionary,
            commands::open_dictionary_location,
            commands::edit_dictionary_item,
            commands::clear_user_dictionary,
            commands::import_user_dictionary,
            commands::export_user_dictionary,
            commands::pause_translation,
            commands::resume_translation,
            commands::stop_translation,
            commands::open_path_dialog,
            commands::open_folder,
            commands::get_available_langs,
            commands::open_dict_window,
            commands::show_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
