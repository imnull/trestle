//! Tauri Commands - 前端调用的 API

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_secs: u64,
    pub active_connections: usize,
}

/// Provider 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub api_base: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub enabled: bool,
}

/// Route 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub id: String,
    pub path: String,
    pub target_provider: String,
    pub target_model: String,
    pub enabled: bool,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub request_id: Option<String>,
}

/// 应用状态
pub struct AppState {
    pub server_status: Mutex<Option<ServerStatus>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            server_status: Mutex::new(None),
        }
    }
}

// ============ Commands ============

#[tauri::command]
pub async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    // 调用内嵌服务器 API
    let client = reqwest::Client::new();
    match client.get("http://127.0.0.1:31415/api/status").send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json().await.map_err(|e| e.to_string())
        }
        _ => Err("Server not responding".to_string())
    }
}

#[tauri::command]
pub async fn get_providers() -> Result<Vec<Provider>, String> {
    let client = reqwest::Client::new();
    match client.get("http://127.0.0.1:31415/api/providers").send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json().await.map_err(|e| e.to_string())
        }
        _ => Err("Failed to fetch providers".to_string())
    }
}

#[tauri::command]
pub async fn get_routes() -> Result<Vec<Route>, String> {
    let client = reqwest::Client::new();
    match client.get("http://127.0.0.1:31415/api/routes").send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json().await.map_err(|e| e.to_string())
        }
        _ => Err("Failed to fetch routes".to_string())
    }
}

#[tauri::command]
pub async fn get_logs(limit: Option<usize>) -> Result<Vec<LogEntry>, String> {
    let limit = limit.unwrap_or(100);
    let client = reqwest::Client::new();
    match client.get(&format!("http://127.0.0.1:31415/api/logs?limit={}", limit)).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json().await.map_err(|e| e.to_string())
        }
        _ => Err("Failed to fetch logs".to_string())
    }
}

#[tauri::command]
pub async fn add_provider(provider: Provider) -> Result<(), String> {
    let client = reqwest::Client::new();
    match client.post("http://127.0.0.1:31415/api/providers")
        .json(&provider)
        .send().await
    {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Server error: {}", resp.status())),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_provider(id: String, provider: Provider) -> Result<(), String> {
    let client = reqwest::Client::new();
    match client.patch(&format!("http://127.0.0.1:31415/api/providers/{}", id))
        .json(&provider)
        .send().await
    {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Server error: {}", resp.status())),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_provider(id: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    match client.delete(&format!("http://127.0.0.1:31415/api/providers/{}", id))
        .send().await
    {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Server error: {}", resp.status())),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn add_route(route: Route) -> Result<(), String> {
    let client = reqwest::Client::new();
    match client.post("http://127.0.0.1:31415/api/routes")
        .json(&route)
        .send().await
    {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Server error: {}", resp.status())),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_route(id: String, route: Route) -> Result<(), String> {
    let client = reqwest::Client::new();
    match client.patch(&format!("http://127.0.0.1:31415/api/routes/{}", id))
        .json(&route)
        .send().await
    {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Server error: {}", resp.status())),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_route(id: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    match client.delete(&format!("http://127.0.0.1:31415/api/routes/{}", id))
        .send().await
    {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Server error: {}", resp.status())),
        Err(e) => Err(e.to_string()),
    }
}
