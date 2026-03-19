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

      // 主畫面置中功能
      use tauri::Manager;
      if let Some(window) = app.get_webview_window("main") {
          let _ = window.center();
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_config,
      commands::save_config,
      commands::start_translation
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
