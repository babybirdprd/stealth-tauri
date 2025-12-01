mod state;
mod scripting;
mod commands;

use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            app.manage(Arc::new(Mutex::new(AppState::default())));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::execute_script,
            commands::script_callback,
            commands::get_profiles,
            commands::set_profile,
            commands::list_scripts,
            commands::save_script,
            commands::read_script
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
