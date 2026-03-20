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
    let providers = state.providers.read().unwrap();
    let models: Vec<ModelInfo> = providers.iter()
        .filter(|p| p.enabled)
        .flat_map(|_p| {
            // TODO: 从上游获取实际模型列表
            vec![]
        })
        .collect();

    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
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

/// GET /api/logs
pub async fn get_logs(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let logs = state.logs.read().unwrap().clone();
    Json(logs)
}
