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
                .unwrap_or_else(|_| "trestle_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = load_config()?;
    let providers = load_providers()?;
    let routes = load_routes()?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    tracing::info!("🚀 Trestle Server starting on http://{}", addr);
    tracing::info!("📖 OpenAI API: POST http://{}/v1/chat/completions", addr);
    tracing::info!("📊 Status: GET http://{}/api/status", addr);

    // 创建应用状态
    let state = Arc::new(AppState::new(config, providers, routes)?);

    // 构建路由
    let app = Router::new()
        .route("/v1/chat/completions", post(handlers::chat_completions))
        .route("/v1/models", get(handlers::list_models))
        .route("/v1/messages", post(handlers::anthropic_messages))
        .route("/api/status", get(handlers::get_status))
        .route("/api/config", get(handlers::get_config).put(handlers::update_config))
        .route("/api/providers", get(handlers::list_providers))
        .route("/api/routes", get(handlers::list_routes))
        .route("/api/logs", get(handlers::get_logs))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("✅ Server is running!");
    axum::serve(listener, app).await?;

    Ok(())
}

fn load_config() -> anyhow::Result<Config> {
    let config_path = Config::default_path();
    if config_path.exists() {
        tracing::info!("Loading config from {:?}", config_path);
        Ok(Config::from_file(&config_path)?)
    } else {
        tracing::info!("Using default config");
        Ok(Config::default())
    }
}

fn load_providers() -> anyhow::Result<Vec<trestle_core::Provider>> {
    let providers_path = Config::default_path()
        .parent()
        .map(|p| p.join("providers.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("providers.toml"));

    if providers_path.exists() {
        tracing::info!("Loading providers from {:?}", providers_path);
        let content = std::fs::read_to_string(&providers_path)?;
        
        #[derive(serde::Deserialize)]
        struct ProvidersFile {
            providers: Vec<trestle_core::Provider>,
        }
        
        let file: ProvidersFile = toml::from_str(&content)?;
        tracing::info!("Loaded {} providers", file.providers.len());
        Ok(file.providers)
    } else {
        tracing::warn!("No providers.toml found");
        Ok(Vec::new())
    }
}

fn load_routes() -> anyhow::Result<Vec<trestle_core::Route>> {
    let routes_path = Config::default_path()
        .parent()
        .map(|p| p.join("routes.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("routes.toml"));

    if routes_path.exists() {
        tracing::info!("Loading routes from {:?}", routes_path);
        let content = std::fs::read_to_string(&routes_path)?;
        
        #[derive(serde::Deserialize)]
        struct RoutesFile {
            routes: Vec<trestle_core::Route>,
        }
        
        let file: RoutesFile = toml::from_str(&content)?;
        tracing::info!("Loaded {} routes", file.routes.len());
        Ok(file.routes)
    } else {
        tracing::warn!("No routes.toml found");
        Ok(Vec::new())
    }
}
