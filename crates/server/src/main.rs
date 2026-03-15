//! Trestle Server - AI 代理服务

mod handlers;
mod proxy;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use trestle_core::config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trestle_server=debug,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = load_config()?;
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    tracing::info!("🚀 Trestle Server starting on {}", addr);

    // 创建应用状态
    let state = Arc::new(AppState::new(config)?);

    // 构建路由
    let app = Router::new()
        // OpenAI 兼容接口
        .route("/v1/chat/completions", post(handlers::chat_completions))
        .route("/v1/models", get(handlers::list_models))
        // Anthropic 兼容接口
        .route("/v1/messages", post(handlers::anthropic_messages))
        // 管理接口
        .route("/api/status", get(handlers::get_status))
        .route("/api/config", get(handlers::get_config).put(handlers::update_config))
        .route("/api/providers", get(handlers::list_providers))
        .route("/api/routes", get(handlers::list_routes))
        .route("/api/logs", get(handlers::get_logs))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 启动服务
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 加载配置
fn load_config() -> anyhow::Result<Config> {
    let config_path = Config::default_path();

    if config_path.exists() {
        tracing::info!("Loading config from {:?}", config_path);
        Ok(Config::from_file(&config_path)?)
    } else {
        tracing::info!("Using default config (no config file found)");
        Ok(Config::default())
    }
}
