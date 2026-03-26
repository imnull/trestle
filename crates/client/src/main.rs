//! Trestle - 一键启动 AI 代理

mod app;
mod pages;
mod api;
mod ui_theme;

use std::sync::Mutex;
use std::time::Duration;

use trestle_server::ServerRuntime;

#[cfg(feature = "tray")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "tray")]
use tray_item::{IconSource, TrayItem};

static SERVER_RUNTIME: once_cell::sync::Lazy<Mutex<Option<ServerRuntime>>> = 
    once_cell::sync::Lazy::new(|| Mutex::new(None));

#[cfg(feature = "tray")]
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

fn main() -> eframe::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 启动内嵌服务端
    match start_embedded_server() {
        Ok(url) => tracing::info!("Embedded server started at {}", url),
        Err(e) => {
            tracing::error!("Failed to start embedded server: {}", e);
            tracing::warn!("Client will run without server functionality");
        }
    }

    // 等待服务端完全启动
    wait_for_server();

    // 启动系统托盘（如果启用）
    #[cfg(feature = "tray")]
    let _tray_handle = start_system_tray();

    // 创建窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Trestle - AI 代理管理"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "Trestle",
        options,
        Box::new(|cc| Ok(Box::new(app::TrestleApp::new(cc)))),
    );

    // 清理
    stop_server();

    result
}

fn wait_for_server() {
    tracing::info!("Waiting for server to be ready...");
    for i in 0..10 {
        std::thread::sleep(Duration::from_millis(500));
        if let Ok(resp) = reqwest::blocking::get("http://127.0.0.1:31415/api/status") {
            if resp.status().is_success() {
                tracing::info!("Server is ready!");
                return;
            }
        }
        tracing::debug!("Attempt {} - server not ready yet", i + 1);
    }
    tracing::warn!("Server may not be fully ready");
}

fn start_embedded_server() -> anyhow::Result<String> {
    let runtime = ServerRuntime::start()?;
    let url = runtime.url();
    
    *SERVER_RUNTIME.lock().unwrap() = Some(runtime);
    
    Ok(url)
}

fn stop_server() {
    if let Ok(mut state) = SERVER_RUNTIME.lock() {
        if state.take().is_some() {
            tracing::info!("Server stopped");
        }
    }
}

// ============ 系统托盘 ============

#[cfg(feature = "tray")]
fn start_system_tray() -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {
        // 延迟创建托盘，确保主窗口已启动
        std::thread::sleep(Duration::from_millis(500));

        match create_tray() {
            Ok(mut tray) => {
                tracing::info!("System tray created");
                // 保持托盘线程运行
                while !SHOULD_EXIT.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(100));
                }
                drop(tray);
            }
            Err(e) => {
                tracing::error!("Failed to create system tray: {}", e);
                tracing::info!("Continuing without system tray");
            }
        }
    })
}

#[cfg(feature = "tray")]
fn create_tray() -> Result<TrayItem, tray_item::TIError> {
    let mut tray = TrayItem::new("Trestle", IconSource::Resource("trestle-icon"))?;
    
    tray.add_menu_item("显示窗口", || {
        tracing::info!("Show window requested");
        // TODO: 需要与 eframe 窗口集成
    })?;
    
    tray.add_menu_item("服务状态", || {
        tracing::info!("Server status: running");
    })?;
    
    tray.add_menu_item("退出", || {
        tracing::info!("Quit requested from tray");
        SHOULD_EXIT.store(true, Ordering::Relaxed);
    })?;

    Ok(tray)
}

#[cfg(not(feature = "tray"))]
fn start_system_tray() -> () {
    // 托盘功能未启用
}
