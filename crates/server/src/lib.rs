//! Trestle Server Library
//! 可被客户端直接调用，无需独立进程

mod handlers;
mod proxy;
mod state;
mod crud;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use trestle_core::config::Config;
use state::AppState;

pub use state::AppState as ServerState;

/// 服务端运行时
pub struct ServerRuntime {
    addr: SocketAddr,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ServerRuntime {
    /// 在后台启动服务端
    pub fn start() -> anyhow::Result<Self> {
        Self::start_with_config(None)
    }

    /// 使用指定配置启动服务端
    pub fn start_with_config(config_override: Option<Config>) -> anyhow::Result<Self> {
        // 初始化日志（如果还没有）
        let _ = tracing_subscriber::fmt::try_init();

        let config = config_override.unwrap_or_else(|| load_config().unwrap_or_default());
        let providers = load_providers().unwrap_or_default();
        let routes = load_routes().unwrap_or_default();

        let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

        // 创建应用状态
        let state = Arc::new(AppState::new(config, providers, routes)?);

        // 构建路由
        let app = Router::new()
            // OpenAI/Anthropic 兼容 API
            .route("/v1/chat/completions", post(handlers::chat_completions))
            .route("/v1/models", get(handlers::list_models))
            .route("/v1/messages", post(handlers::anthropic_messages))
            // 状态和配置 API
            .route("/api/status", get(handlers::get_status))
            .route("/api/config", get(handlers::get_config).put(handlers::update_config))
            .route("/api/config/save", post(crud::save_config))
            // Provider CRUD
            .route("/api/providers", get(handlers::list_providers).post(crud::create_provider))
            .route("/api/providers/:name", put(crud::update_provider).delete(crud::delete_provider))
            // Route CRUD
            .route("/api/routes", get(handlers::list_routes).post(crud::create_route))
            .route("/api/routes/{*pattern}", put(crud::update_route).delete(crud::delete_route))
            // 日志
            .route("/api/logs", get(handlers::get_logs))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // 创建 tokio runtime
        let rt = tokio::runtime::Runtime::new()?;
        let addr_clone = addr;

        rt.spawn(async move {
            let listener = match tokio::net::TcpListener::bind(addr_clone).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to bind server: {}", e);
                    return;
                }
            };

            println!(">> Server started on http://{}", addr_clone);
            println!("[ OpenAI API: http://{}/v1/chat/completions", addr_clone);
            println!("[ Status: http://{}/api/status", addr_clone);

            // 使用 shutdown signal
            let server = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                    println!("Server shutting down...");
                });

            if let Err(e) = server.await {
                eprintln!("Server error: {}", e);
            }
        });

        // 将 runtime 泄露到全局，避免被 drop
        // 这是一个简化的方案，生产环境应该用更好的方式管理
        std::mem::forget(rt);

        Ok(Self {
            addr,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// 获取服务端地址
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// 获取服务端 URL
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

impl Drop for ServerRuntime {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

fn load_config() -> anyhow::Result<Config> {
    let config_path = Config::default_path();
    if config_path.exists() {
        println!("Loading config from {:?}", config_path);
        Ok(Config::from_file(&config_path)?)
    } else {
        println!("Using default config");
        Ok(Config::default())
    }
}

fn load_providers() -> anyhow::Result<Vec<trestle_core::Provider>> {
    let providers_path = Config::default_path()
        .parent()
        .map(|p| p.join("providers.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("providers.toml"));

    if providers_path.exists() {
        println!("Loading providers from {:?}", providers_path);
        let content = std::fs::read_to_string(&providers_path)?;
        
        #[derive(serde::Deserialize)]
        struct ProvidersFile {
            providers: Vec<trestle_core::Provider>,
        }
        
        let file: ProvidersFile = toml::from_str(&content)?;
        println!("Loaded {} providers", file.providers.len());
        Ok(file.providers)
    } else {
        println!("No providers.toml found");
        Ok(Vec::new())
    }
}

fn load_routes() -> anyhow::Result<Vec<trestle_core::Route>> {
    let routes_path = Config::default_path()
        .parent()
        .map(|p| p.join("routes.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("routes.toml"));

    if routes_path.exists() {
        println!("Loading routes from {:?}", routes_path);
        let content = std::fs::read_to_string(&routes_path)?;
        
        #[derive(serde::Deserialize)]
        struct RoutesFile {
            routes: Vec<trestle_core::Route>,
        }
        
        let file: RoutesFile = toml::from_str(&content)?;
        println!("Loaded {} routes", file.routes.len());
        Ok(file.routes)
    } else {
        println!("No routes.toml found");
        Ok(Vec::new())
    }
}
