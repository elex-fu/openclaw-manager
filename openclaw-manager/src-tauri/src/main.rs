// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod errors;
mod installer;
mod models;
mod plugins;
mod services;
mod system;
mod utils;

use commands::openclaw::InstallerState;
use installer::OpenClawInstaller;
use services::installer::InstallerService;
use std::sync::Arc;
use tauri::{Manager, AppHandle, App};
use tokio::sync::Mutex;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 初始化托盘图标
            setup_tray(app)?;
            
            // 管理状态
            let installer = OpenClawInstaller::new().unwrap_or_default();
            let service = InstallerService::new().unwrap_or_default();
            
            app.manage(InstallerState {
                installer: Arc::new(Mutex::new(installer)),
                service: Arc::new(Mutex::new(service)),
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::set_config,
            commands::config::delete_config,
            commands::plugin::get_plugins,
            commands::plugin::install_plugin,
            commands::plugin::uninstall_plugin,
            commands::plugin::enable_plugin,
            commands::plugin::disable_plugin,
            commands::openclaw::check_openclaw_installation,
            commands::openclaw::install_openclaw,
            commands::openclaw::install_openclaw_one_click,
            commands::openclaw::install_openclaw_offline,
            commands::openclaw::get_install_methods,
            commands::openclaw::uninstall_openclaw,
            commands::openclaw::get_openclaw_config_if_installed,
            commands::openclaw::get_openclaw_models,
            commands::openclaw::get_openclaw_agents,
            commands::openclaw::is_openclaw_running,
            commands::openclaw::get_openclaw_process_info,
            commands::openclaw::update_openclaw_config,
            commands::openclaw::start_openclaw_service,
            commands::openclaw::check_system_environment,
            commands::openclaw::execute_openclaw_command,
            commands::secure::save_api_key,
            commands::secure::get_api_key,
            commands::secure::delete_api_key,
            commands::secure::has_api_key,
            commands::service::start_service,
            commands::service::stop_service,
            commands::service::get_service_status,
            commands::service::health_check_service,
            system::get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};
    use tauri::menu::{Menu, MenuItem};
    
    // 创建菜单项
    let show_item = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    // 创建菜单
    let menu = Menu::with_items(app, &[
        &show_item,
        &quit_item,
    ])?;
    
    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                std::process::exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click { button, .. } => {
                if button == MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            _ => {}
        })
        .build(app)?;
    
    Ok(())
}
