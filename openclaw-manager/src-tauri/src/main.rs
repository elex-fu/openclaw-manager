// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod plugins;
mod utils;

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

fn main() {
    env_logger::init();

    let tray_menu = SystemTrayMenu::new()
        .add_item(tauri::CustomMenuItem::new("show", "显示"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(tauri::CustomMenuItem::new("quit", "退出"));

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        // .plugin(tauri_plugin_shell::init())
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle();
            let app_dir = app_handle
                .path_resolver()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("openclaw.db");

            db::init_database(db_path.to_str().unwrap())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::set_config,
            commands::config::delete_config,
            commands::file::scan_files,
            commands::file::get_files,
            commands::file::get_file_by_id,
            commands::file::update_file,
            commands::file::delete_file,
            commands::file::parse_file_info,
            commands::group::get_groups,
            commands::group::create_group,
            commands::group::update_group,
            commands::group::delete_group,
            commands::group::add_file_to_group,
            commands::group::remove_file_from_group,
            commands::plugin::get_plugins,
            commands::plugin::install_plugin,
            commands::plugin::uninstall_plugin,
            commands::plugin::enable_plugin,
            commands::plugin::disable_plugin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
