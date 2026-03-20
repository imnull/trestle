#![allow(dead_code)]

//! API 客户端

use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        Self { base_url, client }
    }

    // ============================================================================
    // GET 请求
    // ============================================================================
    
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.get(&url).send()?;
        let data = resp.json()?;
        Ok(data)
    }

    // ============================================================================
    // POST 请求
    // ============================================================================
    
    pub fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.post(&url).json(body).send()?;
        let data = resp.json()?;
        Ok(data)
    }

    // ============================================================================
    // PUT 请求
    // ============================================================================
    
    pub fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.put(&url).json(body).send()?;
        let data = resp.json()?;
        Ok(data)
    }

    // ============================================================================
    // DELETE 请求
    // ============================================================================
    
    pub fn delete<T: DeserializeOwned>(&self,
        path: &str,
    ) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.delete(&url).send()?;
        let data = resp.json()?;
        Ok(data)
    }

    // ============================================================================
    // 状态 API
    // ============================================================================
    
    pub fn get_status(&self) -> anyhow::Result<ServerStatus> {
        self.get("/api/status")
    }

    // ============================================================================
    // 配置 API
    // ============================================================================
    
    pub fn get_config(&self) -> anyhow::Result<trestle_core::Config> {
        self.get("/api/config")
    }

    pub fn update_config(&self,
        config: &trestle_core::Config,
    ) -> anyhow::Result<SuccessResponse> {
        self.put("/api/config", config)
    }

    pub fn save_config(&self) -> anyhow::Result<SuccessResponse> {
        self.post("/api/config/save", &serde_json::json!({}))
    }

    // ============================================================================
    // 服务商 API
    // ============================================================================
    
    pub fn get_providers(&self) -> anyhow::Result<Vec<Provider>> {
        self.get("/api/providers")
    }

    pub fn create_provider(&self,
        provider: &Provider,
    ) -> anyhow::Result<Provider> {
        self.post("/api/providers", provider)
    }

    pub fn update_provider(&self,
        name: &str,
        provider: &Provider,
    ) -> anyhow::Result<Provider> {
        self.put(&format!("/api/providers/{}", name), provider)
    }

    pub fn delete_provider(&self,
        name: &str,
    ) -> anyhow::Result<SuccessResponse> {
        self.delete(&format!("/api/providers/{}", name))
    }

    // ============================================================================
    // 路由 API
    // ============================================================================
    
    pub fn get_routes(&self) -> anyhow::Result<Vec<Route>> {
        self.get("/api/routes")
    }

    pub fn create_route(&self,
        route: &Route,
    ) -> anyhow::Result<Route> {
        self.post("/api/routes", route)
    }

    pub fn update_route(&self,
        pattern: &str,
        route: &Route,
    ) -> anyhow::Result<Route> {
        self.put(&format!("/api/routes/{}", urlencoding::encode(pattern)), route)
    }

    pub fn delete_route(&self,
        pattern: &str,
    ) -> anyhow::Result<SuccessResponse> {
        self.delete(&format!("/api/routes/{}", urlencoding::encode(pattern)))
    }

    // ============================================================================
    // 日志 API
    // ============================================================================
    
    pub fn get_logs(&self) -> anyhow::Result<Vec<RequestLog>> {
        self.get("/api/logs")
    }
}

// ============================================================================
// 响应类型
// ============================================================================

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ============================================================================
// 数据类型
// ============================================================================

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerStatus {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub total_tokens: u64,
    #[allow(dead_code)]
    pub active_connections: u32,
    pub providers: Vec<ProviderStatus>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub healthy: bool,
    pub latency_ms: Option<u64>,
    #[allow(dead_code)]
    pub last_check: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Provider {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Route {
    pub pattern: String,
    pub provider: String,
    #[serde(default)]
    pub model: Option<String>,
    pub priority: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RequestLog {
    pub id: String,
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub model: String,
    pub status: u16,
    pub latency_ms: u64,
}
