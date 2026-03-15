//! Trestle - 一键启动 AI 代理

mod app;
mod pages;
mod api;

use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

static SERVER_PROCESS: once_cell::sync::Lazy<Mutex<Option<Child>>> = 
    once_cell::sync::Lazy::new(|| Mutex::new(None));

fn main() -> eframe::Result<()> {
    // 启动 server
    start_server();
    
    // 等待 server 启动
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // 设置退出处理
    ctrlc::set_handler(|| {
        stop_server();
        std::process::exit(0);
    }).ok();

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
    
    // 退出时停止 server
    stop_server();
    
    result
}

fn start_server() {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let exe_dir = match exe_path.parent() {
        Some(d) => d,
        None => return,
    };
    
    // 查找 server binary
    let server_paths = vec![
        exe_dir.join("trestle-server"),
        exe_dir.join("trestle-server.exe"),
        exe_dir.join("../Resources/trestle-server"),
        exe_dir.join("../../release/trestle-server"),
        exe_dir.join("../../debug/trestle-server"),
    ];
    
    let server_path = server_paths.into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| exe_dir.join("trestle-server"));

    println!("Starting server from: {:?}", server_path);
    
    match Command::new(&server_path)
        .env("RUST_LOG", "info")
        .spawn()
    {
        Ok(child) => {
            println!("✅ Server started (PID: {})", child.id());
            *SERVER_PROCESS.lock().unwrap() = Some(child);
        }
        Err(e) => {
            eprintln!("❌ Failed to start server: {}", e);
            eprintln!("   Looking for: {:?}", server_path);
            eprintln!("   Server will not be available, but client will still run");
        }
    }
}

fn stop_server() {
    if let Ok(mut state) = SERVER_PROCESS.lock() {
        if let Some(ref mut child) = *state {
            println!("🛑 Stopping server (PID: {})...", child.id());
            let _ = child.kill();
            let _ = child.wait();
            println!("✅ Server stopped");
        }
        *state = None;
    }
}
