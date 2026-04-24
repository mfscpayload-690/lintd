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
            let managers = distro_detect::detect_package_managers()
                .into_iter()
                .map(Arc::new)
                .collect();

            let state = AppState {
                managers,
                distro,
                db,
            };

            app.manage(Arc::new(Mutex::new(state)));

            // Set window icon so it appears in the taskbar/dock even in dev mode
            if let Some(window) = app.get_webview_window("main") {
                let icon_png = include_bytes!("../icons/128x128.png");
                let decoder = png::Decoder::new(icon_png.as_ref());
                if let Ok(mut reader) = decoder.read_info() {
                    let mut buf = vec![0u8; reader.output_buffer_size()];
                    if let Ok(info) = reader.next_frame(&mut buf) {
                        let rgba: Vec<u8> = match info.color_type {
                            png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
                            png::ColorType::Rgb => buf[..info.buffer_size()]
                                .chunks(3)
                                .flat_map(|c| [c[0], c[1], c[2], 255])
                                .collect(),
                            _ => vec![],
                        };
                        if !rgba.is_empty() {
                            let icon =
                                tauri::image::Image::new_owned(rgba, info.width, info.height);
                            let _ = window.set_icon(icon);
                        }
                    }
                }
            }

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
            commands::backfill_flatpak_history_sizes,
            commands::scan_packages_streaming,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
