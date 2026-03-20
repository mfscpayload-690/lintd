mod commands;
mod db;
mod distro_detect;
mod pmal;
mod sysinfo_collector;

use commands::AppState;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let db = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime")
                .block_on(async { db::Database::new().await })
                .expect("Failed to initialize database");

            let distro = distro_detect::detect_distro();
            let managers = distro_detect::detect_package_managers();

            let state = AppState {
                managers,
                distro,
                db,
            };

            app.manage(Arc::new(Mutex::new(state)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::get_all_packages,
            commands::get_orphans,
            commands::get_package_files,
            commands::get_reverse_deps,
            commands::preview_removal,
            commands::execute_removal,
            commands::get_removal_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
