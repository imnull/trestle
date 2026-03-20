//! Trestle - 一键启动 AI 代理

mod app;
mod pages;
mod api;

use std::sync::Mutex;

use trestle_server::ServerRuntime;

static SERVER_RUNTIME: once_cell::sync::Lazy<Mutex<Option<ServerRuntime>>> = 
    once_cell::sync::Lazy::new(|| Mutex::new(None));

fn main() -> eframe::Result<()> {
    // 启动内嵌服务端
    match start_embedded_server() {
        Ok(url) => println!("OK Embedded server started at {}", url),
        Err(e) => {
            eprintln!("ERR Failed to start embedded server: {}", e);
            eprintln!("   Client will run without server functionality");
        }
    }

    // 等待服务端完全启动
    println!("Waiting for server to be ready...");
    for i in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if let Ok(resp) = reqwest::blocking::get("http://127.0.0.1:31415/api/status") {
            if resp.status().is_success() {
                println!("Server is ready!");
                break;
            }
        }
        println!("Attempt {} - server not ready yet", i + 1);
    }

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

    // 退出时清理服务端
    stop_server();

    result
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
            println!("Server stopped");
        }
    }
}
