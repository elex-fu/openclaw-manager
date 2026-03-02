// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use openclaw_manager::commands::openclaw::InstallerState;
use openclaw_manager::installer::OpenClawInstaller;
use openclaw_manager::models::openclaw::InstallStatus;
use openclaw_manager::services::installer::InstallerService;
use std::sync::Arc;
use tauri::{Manager, App, Emitter};
use tokio::sync::Mutex;

// Bring modules into scope for invoke_handler and auto_initialize
use openclaw_manager::{commands, system, installer, services};

/// 自动初始化 OpenClaw
/// 在应用启动时后台检查并自动解压
async fn auto_initialize(app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    log::info!("开始自动初始化检查...");

    // 检查是否已安装
    let installer = OpenClawInstaller::new()?;

    match installer.check_installation()? {
        InstallStatus::Installed { version, .. } => {
            log::info!("OpenClaw 已安装 (版本: {}), 跳过初始化", version);
            return Ok(());
        }
        InstallStatus::Installing { stage, .. } => {
            log::info!("OpenClaw 正在安装中 ({}), 跳过初始化", stage);
            return Ok(());
        }
        InstallStatus::NotInstalled | InstallStatus::Error { .. } => {
            log::info!("OpenClaw 未安装，开始自动初始化...");
        }
    }

    // 发送进度事件到前端
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "Initializing",
        "percentage": 0.0,
        "message": "正在准备自动初始化..."
    }));

    // 步骤 1: 解压嵌入式 Runtime
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "ExtractingRuntime",
        "percentage": 5.0,
        "message": "正在准备运行环境..."
    }));

    let runtime_bundle = installer::RuntimeBundle::new()?;
    runtime_bundle.install_required(|msg, _pct| {
        log::info!("Runtime 安装: {}", msg);
    }).await?;

    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "ExtractingRuntime",
        "percentage": 20.0,
        "message": "运行环境准备完成"
    }));

    // 步骤 2: 解压 OpenClaw
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "Installing",
        "percentage": 25.0,
        "message": "正在解压 OpenClaw..."
    }));

    use services::offline_installer::OfflineInstaller;
    let offline_installer = OfflineInstaller::for_current_platform()?;

    let install_dir = dirs::home_dir()
        .map(|h| h.join(".openclaw"))
        .ok_or_else(|| anyhow::anyhow!("无法获取主目录"))?;

    offline_installer.install(&install_dir).await?;

    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "Installing",
        "percentage": 70.0,
        "message": "OpenClaw 解压完成"
    }));

    // 步骤 3: 应用预设配置
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "Configuring",
        "percentage": 75.0,
        "message": "正在应用本土化配置..."
    }));

    installer.apply_china_presets().await?;

    // 步骤 4: 设置环境变量
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "Configuring",
        "percentage": 85.0,
        "message": "正在配置环境..."
    }));

    runtime_bundle.setup_environment().await?;

    // 完成
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "stage": "Complete",
        "percentage": 100.0,
        "message": "初始化完成！OpenClaw 已准备就绪"
    }));

    let _ = app_handle.emit("auto-init-complete", serde_json::json!({
        "success": true,
        "message": "OpenClaw 自动初始化成功"
    }));

    log::info!("自动初始化完成！");
    Ok(())
}

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

            // 后台自动初始化检查
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // 延迟 3 秒等待前端加载完成
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                if let Err(e) = auto_initialize(&app_handle).await {
                    log::error!("自动初始化失败: {}", e);
                    // 发送通知到前端
                    let _ = app_handle.emit("auto-init-error", e.to_string());
                }
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
            commands::openclaw::get_all_agents,
            commands::openclaw::save_agent,
            commands::openclaw::delete_agent,
            commands::openclaw::set_current_agent,
            commands::openclaw::get_current_agent,
            commands::openclaw::is_openclaw_running,
            commands::openclaw::get_openclaw_process_info,
            commands::openclaw::update_openclaw_config,
            commands::openclaw::start_openclaw_service,
            commands::openclaw::check_system_environment,
            commands::openclaw::execute_openclaw_command,
            commands::openclaw::check_for_updates,
            commands::openclaw::perform_update,
            commands::openclaw::perform_offline_update,
            commands::openclaw::get_backup_list,
            commands::openclaw::restore_from_backup,
            commands::secure::save_api_key,
            commands::secure::get_api_key,
            commands::secure::delete_api_key,
            commands::secure::has_api_key,
            commands::service::start_service,
            commands::service::stop_service,
            commands::service::get_service_status,
            commands::service::health_check_service,
            commands::service::run_diagnostics,
            commands::service::auto_fix_issues,
            commands::service::fix_issue,
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
