mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // 主畫面置中功能 (手動計算，支援多螢幕)
      use tauri::Manager;
      if let Some(window) = app.get_webview_window("main") {
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

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_config,
      commands::save_config,
      commands::get_default_config,
      commands::get_api_key_cmd,
      commands::save_api_key_cmd,
      commands::start_translation,
      commands::get_style_config,
      commands::save_style_config,
      commands::get_default_style_config,
      commands::query_dictionary,
      commands::edit_dictionary_item,
      commands::pause_translation,
      commands::resume_translation,
      commands::stop_translation,
      commands::open_path_dialog,
      commands::get_available_langs
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
