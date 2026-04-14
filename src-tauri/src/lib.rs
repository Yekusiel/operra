mod adapters;
mod commands;
mod db;
mod models;
mod scanner;

use db::AppDb;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;
            let db = AppDb::init(&app_data_dir)?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project_commands::create_project,
            commands::project_commands::list_projects,
            commands::project_commands::get_project,
            commands::project_commands::delete_project,
            commands::scan_commands::start_scan,
            commands::scan_commands::get_scan_results,
            commands::scan_commands::list_scans_for_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
