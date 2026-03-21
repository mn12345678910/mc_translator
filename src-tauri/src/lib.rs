mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // 確保語言文件刷新釋放至硬碟
      let _ = mc_translator::i18n::GuiLabels::ensure_langs_exists();

      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // 讀取視窗幾何記錄並應用
      use tauri::Manager;
      use mc_translator::config::AppConfig;

      if let Some(window) = app.get_webview_window("main") {
          let config = AppConfig::load();
          if config.main_width > 200.0 && config.main_height > 200.0 {
              let _ = window.set_size(tauri::PhysicalSize::new(config.main_width as u32, config.main_height as u32));
          }
          
          if config.main_x != 0.0 || config.main_y != 0.0 {
              let _ = window.set_position(tauri::PhysicalPosition::new(config.main_x as i32, config.main_y as i32));
          } else {
              // 初次啟動強制置中
              if let Ok(Some(monitor)) = window.current_monitor() {
                  let monitor_size = monitor.size();
                  let monitor_pos = monitor.position();
                  
                  if let Ok(window_size) = window.outer_size() {
                      let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
                      let y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;
                      
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
      commands::show_window
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
