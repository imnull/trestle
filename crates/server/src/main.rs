//! Trestle Server - 独立服务端入口
//! 用于无头环境或需要独立运行服务端的场景

use trestle_server::ServerRuntime;

fn main() -> anyhow::Result<()> {
    // 初始化日志
    let _ = tracing_subscriber::fmt::try_init();

    println!(">> Trestle Server starting...");
    
    // 启动服务端
    let runtime = ServerRuntime::start()?;
    
    println!("OK Server is running at {}", runtime.url());
    println!("Press Ctrl+C to stop");

    // 等待中断信号
    ctrlc::set_handler(|| {
        println!("\nShutting down...");
        std::process::exit(0);
    })?;

    // 阻塞主线程
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
