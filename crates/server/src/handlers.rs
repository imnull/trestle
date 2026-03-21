//! HTTP 处理器

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use trestle_core::{ChatCompletionRequest, ModelInfo, ServerStatus, ProviderStatus};
use crate::state::{AppState, HealthInfo};
use crate::proxy;

/// POST /v1/chat/completions (OpenAI 兼容)
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    state.inc_requests();

    tracing::info!("Chat completion request: model={}", req.model);

    let stream = req.stream.unwrap_or(false);

    if stream {
        match proxy::stream_chat_completion(&state, req).await {
            Ok(sse) => (StatusCode::OK, sse).into_response(),
            Err(e) => {
                tracing::error!("Stream error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": {"message": e.to_string(), "type": "internal_error"}
                }))).into_response()
            }
        }
    } else {
        match proxy::chat_completion(&state, req).await {
            Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
            Err(e) => {
                tracing::error!("Request error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": {"message": e.to_string(), "type": "internal_error"}
                }))).into_response()
            }
        }
    }
}

/// POST /v1/messages (Anthropic 兼容)
pub async fn anthropic_messages(
    State(state): State<Arc<AppState>>,
    Json(req): Json<trestle_core::AnthropicRequest>,
) -> impl IntoResponse {
    state.inc_requests();
    tracing::info!("Anthropic messages request: model={}", req.model);
    
    match proxy::anthropic_messages(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => {
            tracing::error!("Anthropic request error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": {"type": "internal_error", "message": e.to_string()}
            }))).into_response()
        }
    }
}

/// GET /v1/models
pub async fn list_models(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 尝试从上游获取模型列表
    let models = fetch_models_from_providers(&state).await;
    
    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
}

async fn fetch_models_from_providers(state: &Arc<AppState>) -> Vec<ModelInfo> {
    let providers = state.providers.read().unwrap().clone();
    let mut all_models = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    for provider in providers.iter().filter(|p| p.enabled) {
        if let Ok(models) = fetch_provider_models(&state.http_client, &provider).await {
            all_models.extend(models.into_iter().map(|m| ModelInfo {
                id: m,
                object: "model".to_string(),
                created: now,
                owned_by: provider.name.clone(),
            }));
        }
    }
    
    all_models
}

async fn fetch_provider_models(client: &reqwest::Client, provider: &trestle_core::Provider) -> Result<Vec<String>, reqwest::Error> {
    let url = format!("{}/models", provider.base_url.trim_end_matches('/'));
    
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .header("Authorization", format!("Bearer {}", provider.api_key.as_deref().unwrap_or("")))
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    
    #[derive(serde::Deserialize)]
    struct ModelsResponse {
        #[serde(default)]
        data: Vec<ModelData>,
    }
    
    #[derive(serde::Deserialize)]
    struct ModelData {
        id: String,
    }
    
    let models: ModelsResponse = resp.json().await?;
    Ok(models.data.into_iter().map(|m| m.id).collect())
}

/// GET /api/status
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let providers = state.providers.read().unwrap();
    let health_cache = state.health_cache.read().unwrap();
    
    let status = ServerStatus {
        uptime_secs: state.start_time.elapsed().as_secs(),
        total_requests: state.total_requests.load(std::sync::atomic::Ordering::Relaxed),
        total_tokens: state.total_tokens.load(std::sync::atomic::Ordering::Relaxed),
        active_connections: 0,
        providers: providers.iter().map(|p| {
            let cached = health_cache.get(&p.name);
            ProviderStatus {
                name: p.name.clone(),
                healthy: cached.map(|h| h.healthy).unwrap_or(true),
                latency_ms: cached.and_then(|h| h.latency_ms),
                last_check: cached.and_then(|h| h.last_check),
            }
        }).collect(),
    };

    Json(status)
}

/// POST /api/health-check - 触发健康检查
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use std::time::Instant;
    
    let providers = state.providers.read().unwrap().clone();
    let mut results = Vec::new();
    
    for provider in providers.iter().filter(|p| p.enabled) {
        let start = Instant::now();
        
        // 发送简单请求检测连通性
        let healthy = check_provider_health(&state.http_client, &provider).await;
        
        let latency_ms = if healthy {
            Some(start.elapsed().as_millis() as u64)
        } else {
            None
        };
        
        results.push(ProviderStatus {
            name: provider.name.clone(),
            healthy,
            latency_ms,
            last_check: Some(chrono::Utc::now()),
        });
        
        // 更新缓存
        state.update_health(provider.name.clone(), HealthInfo {
            healthy,
            latency_ms,
            last_check: Some(chrono::Utc::now()),
        });
    }
    
    (StatusCode::OK, Json(results))
}

async fn check_provider_health(client: &reqwest::Client, provider: &trestle_core::Provider) -> bool {
    let url = format!("{}/models", provider.base_url.trim_end_matches('/'));
    
    let result = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .header("Authorization", format!("Bearer {}", provider.api_key.as_deref().unwrap_or("")))
        .send()
        .await;
    
    match result {
        Ok(resp) => resp.status().is_success() || resp.status().as_u16() == 401, // 401 也算连通
        Err(_) => false,
    }
}

/// GET /api/config
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();
    Json(config)
}

/// PUT /api/config
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<trestle_core::Config>,
) -> impl IntoResponse {
    let mut current = state.config.write().unwrap();
    *current = config;
    (StatusCode::OK, Json(serde_json::json!({"success": true})))
}

/// GET /api/providers
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let providers = state.providers.read().unwrap().clone();
    Json(providers)
}

/// GET /api/routes
pub async fn list_routes(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let routes = state.routes.read().unwrap().clone();
    Json(routes)
}

/// GET /api/export - 导出所有配置
pub async fn export_config(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();
    let providers = state.providers.read().unwrap().clone();
    let routes = state.routes.read().unwrap().clone();
    
    Json(serde_json::json!({
        "config": config,
        "providers": providers,
        "routes": routes,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// POST /api/import - 导入配置
pub async fn import_config(
    State(state): State<Arc<AppState>>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    // 解析配置
    let config: Option<trestle_core::Config> = data.get("config")
        .and_then(|c| serde_json::from_value(c.clone()).ok());
    
    let providers: Option<Vec<trestle_core::Provider>> = data.get("providers")
        .and_then(|p| serde_json::from_value(p.clone()).ok());
    
    let routes: Option<Vec<trestle_core::Route>> = data.get("routes")
        .and_then(|r| serde_json::from_value(r.clone()).ok());
    
    // 应用配置
    let mut applied = Vec::new();
    
    if let Some(c) = config {
        *state.config.write().unwrap() = c;
        applied.push("config");
    }
    
    if let Some(p) = providers {
        *state.providers.write().unwrap() = p;
        applied.push("providers");
    }
    
    if let Some(r) = routes {
        *state.routes.write().unwrap() = r;
        applied.push("routes");
    }
    
    if applied.is_empty() {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "error": "No valid configuration found in import data"
        })))
    } else {
        (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "imported": applied
        })))
    }
}
pub async fn get_logs(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 从 SQLite 获取最近 100 条日志
    match state.log_store.get_logs(100, 0) {
        Ok(logs) => Json(logs),
        Err(e) => {
            tracing::error!("Failed to get logs from SQLite: {}", e);
            Json(Vec::new())
        }
    }
}
