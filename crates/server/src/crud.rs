//! CRUD 处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;

use trestle_core::{Provider, Route};
use crate::state::AppState;

// ============================================================================
// Response Types
// ============================================================================

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Data(T),
    Error { error: String },
    Success { success: bool, #[serde(skip_serializing_if = "Option::is_none")] message: Option<String> },
}

impl<T: serde::Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

// ============================================================================
// Provider CRUD
// ============================================================================

/// POST /api/providers - 添加服务商
pub async fn create_provider(
    State(state): State<Arc<AppState>>,
    Json(provider): Json<Provider>,
) -> impl IntoResponse {
    let mut providers = state.providers.write().unwrap();
    
    // 检查是否已存在
    if providers.iter().any(|p| p.name == provider.name) {
        return (StatusCode::CONFLICT, Json(ApiResponse::<Provider>::Error {
            error: "Provider already exists".to_string(),
        }));
    }
    
    providers.push(provider.clone());
    (StatusCode::CREATED, Json(ApiResponse::Data(provider)))
}

/// PUT /api/providers/:name - 更新服务商
pub async fn update_provider(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(mut provider): Json<Provider>,
) -> impl IntoResponse {
    let mut providers = state.providers.write().unwrap();
    
    if let Some(existing) = providers.iter_mut().find(|p| p.name == name) {
        // 保持名称一致
        provider.name = name;
        *existing = provider.clone();
        (StatusCode::OK, Json(ApiResponse::Data(provider)))
    } else {
        (StatusCode::NOT_FOUND, Json(ApiResponse::<Provider>::Error {
            error: format!("Provider '{}' not found", name),
        }))
    }
}

/// DELETE /api/providers/:name - 删除服务商
pub async fn delete_provider(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let mut providers = state.providers.write().unwrap();
    let len_before = providers.len();
    providers.retain(|p| p.name != name);
    
    if providers.len() < len_before {
        (StatusCode::OK, Json(ApiResponse::<()>::Success { success: true, message: None }))
    } else {
        (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::Error {
            error: format!("Provider '{}' not found", name),
        }))
    }
}

// ============================================================================
// Route CRUD
// ============================================================================

/// POST /api/routes - 添加路由
pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Json(route): Json<Route>,
) -> impl IntoResponse {
    let mut routes = state.routes.write().unwrap();
    routes.push(route.clone());
    
    // 按优先级排序
    routes.sort_by(|a, b| a.priority.cmp(&b.priority));
    
    (StatusCode::CREATED, Json(ApiResponse::Data(route)))
}

/// PUT /api/routes/:pattern - 更新路由
pub async fn update_route(
    State(state): State<Arc<AppState>>,
    Path(pattern): Path<String>,
    Json(mut route): Json<Route>,
) -> impl IntoResponse {
    let mut routes = state.routes.write().unwrap();
    
    if let Some(existing) = routes.iter_mut().find(|r| r.pattern == pattern) {
        // 保持 pattern 一致
        route.pattern = pattern;
        *existing = route.clone();
        routes.sort_by(|a, b| a.priority.cmp(&b.priority));
        (StatusCode::OK, Json(ApiResponse::Data(route)))
    } else {
        (StatusCode::NOT_FOUND, Json(ApiResponse::<Route>::Error {
            error: format!("Route '{}' not found", pattern),
        }))
    }
}

/// DELETE /api/routes/:pattern - 删除路由
pub async fn delete_route(
    State(state): State<Arc<AppState>>,
    Path(pattern): Path<String>,
) -> impl IntoResponse {
    let mut routes = state.routes.write().unwrap();
    let len_before = routes.len();
    routes.retain(|r| r.pattern != pattern);
    
    if routes.len() < len_before {
        (StatusCode::OK, Json(ApiResponse::<()>::Success { success: true, message: None }))
    } else {
        (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::Error {
            error: format!("Route '{}' not found", pattern),
        }))
    }
}

// ============================================================================
// Config Management
// ============================================================================

/// POST /api/config/save - 保存配置到文件
pub async fn save_config(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();
    let providers = state.providers.read().unwrap().clone();
    let routes = state.routes.read().unwrap().clone();
    
    let config_path = trestle_core::Config::default_path();
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::Error {
                error: format!("Failed to create config directory: {}", e),
            }));
        }
    }
    
    let providers_path = config_path.parent()
        .map(|p| p.join("providers.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("providers.toml"));
    let routes_path = config_path.parent()
        .map(|p| p.join("routes.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("routes.toml"));
    
    // 保存 config.toml
    if let Err(e) = config.to_file(&config_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::Error {
            error: format!("Failed to save config: {}", e),
        }));
    }
    
    // 保存 providers.toml
    #[derive(serde::Serialize)]
    struct ProvidersFile {
        providers: Vec<Provider>,
    }
    let providers_file = ProvidersFile { providers };
    if let Ok(content) = toml::to_string_pretty(&providers_file) {
        if let Err(e) = std::fs::write(&providers_path, content) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::Error {
                error: format!("Failed to save providers: {}", e),
            }));
        }
    }
    
    // 保存 routes.toml
    #[derive(serde::Serialize)]
    struct RoutesFile {
        routes: Vec<Route>,
    }
    let routes_file = RoutesFile { routes };
    if let Ok(content) = toml::to_string_pretty(&routes_file) {
        if let Err(e) = std::fs::write(&routes_path, content) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::Error {
                error: format!("Failed to save routes: {}", e),
            }));
        }
    }
    
    (StatusCode::OK, Json(ApiResponse::<()>::Success { 
        success: true, 
        message: Some("Configuration saved successfully".to_string()),
    }))
}
