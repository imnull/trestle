//! Trestle - AI 代理管理工具 (Tauri 2.0)

use tauri::Manager;
use tauri_plugin_store::StoreBuilder;

mod commands;
mod tray;

pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // 启动内嵌服务器
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Err(e) = start_embedded_server(&handle) {
                    tracing::error!("Failed to start embedded server: {}", e);
                }
            });

            // 初始化系统托盘
            #[cfg(desktop)]
            {
                tray::setup_tray(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 服务器状态
            commands::get_server_status,
            commands::get_providers,
            commands::get_routes,
            commands::get_logs,
            // Provider 管理
            commands::add_provider,
            commands::update_provider,
            commands::delete_provider,
            // Route 管理
            commands::add_route,
            commands::update_route,
            commands::delete_route,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 启动内嵌服务器
fn start_embedded_server(_app: &tauri::AppHandle) -> anyhow::Result<()> {
    use std::sync::OnceLock;
    use tokio::runtime::Runtime;

    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    let rt = RUNTIME.get_or_init(|| Runtime::new().unwrap());

    rt.spawn(async move {
        // 使用现有的 trestle-server 启动服务
        // 服务会在 127.0.0.1:31415 运行
        tracing::info!("Starting embedded server on 127.0.0.1:31415");
        // TODO: 调用 trestle-server 的启动逻辑
    });

    Ok(())
}
