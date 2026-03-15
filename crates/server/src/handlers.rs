//! HTTP 处理器

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use trestle_core::{
    ChatCompletionRequest, ModelInfo, ServerStatus, ProviderStatus,
};
use crate::state::AppState;
use crate::proxy;

/// POST /v1/chat/completions
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    state.inc_requests();

    tracing::debug!("Chat completion request: model={}", req.model);

    let stream = req.stream.unwrap_or(false);

    if stream {
        // 流式响应
        match proxy::stream_chat_completion(&state, req).await {
            Ok(sse) => {
                (StatusCode::OK, sse).into_response()
            }
            Err(e) => {
                tracing::error!("Stream error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": {
                        "message": e.to_string(),
                        "type": "internal_error"
                    }
                }))).into_response()
            }
        }
    } else {
        // 非流式响应
        match proxy::chat_completion(&state, req).await {
            Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
            Err(e) => {
                tracing::error!("Request error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": {
                        "message": e.to_string(),
                        "type": "internal_error"
                    }
                }))).into_response()
            }
        }
    }
}

/// POST /v1/messages (Anthropic)
pub async fn anthropic_messages(
    State(state): State<Arc<AppState>>,
    Json(req): Json<trestle_core::AnthropicRequest>,
) -> impl IntoResponse {
    state.inc_requests();

    tracing::debug!("Anthropic messages request: model={}", req.model);

    // TODO: 实现 Anthropic 消息处理
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": {
                "message": "Anthropic API not yet implemented",
                "type": "not_implemented"
            }
        }))
    )
}

/// GET /v1/models
pub async fn list_models(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let models: Vec<ModelInfo> = state.providers.read().unwrap().iter()
        .flat_map(|p| {
            // TODO: 从上游获取实际模型列表
            vec![ModelInfo {
                id: format!("{}/models", p.name),
                object: "model".to_string(),
                created: chrono::Utc::now().timestamp(),
                owned_by: p.name.clone(),
            }]
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
    let status = ServerStatus {
        uptime_secs: state.start_time.elapsed().as_secs(),
        total_requests: state.total_requests.load(std::sync::atomic::Ordering::Relaxed),
        total_tokens: state.total_tokens.load(std::sync::atomic::Ordering::Relaxed),
        active_connections: 0, // TODO
        providers: state.providers.read().unwrap().iter().map(|p| {
            ProviderStatus {
                name: p.name.clone(),
                healthy: true, // TODO: 实际健康检查
                latency_ms: None,
                last_check: None,
            }
        }).collect(),
    };

    Json(status)
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

    // TODO: 保存到文件

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
