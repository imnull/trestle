//! API 客户端

use reqwest::Client;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self { base_url, client }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.get(&url).send().await?;
        let data = resp.json().await?;
        Ok(data)
    }

    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(&self, path: &str, body: &B) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.post(&url).json(body).send().await?;
        let data = resp.json().await?;
        Ok(data)
    }

    // === 具体 API ===

    pub async fn get_status(&self) -> anyhow::Result<ServerStatus> {
        self.get("/api/status").await
    }

    pub async fn get_providers(&self) -> anyhow::Result<Vec<Provider>> {
        self.get("/api/providers").await
    }

    pub async fn get_routes(&self) -> anyhow::Result<Vec<Route>> {
        self.get("/api/routes").await
    }

    pub async fn get_logs(&self) -> anyhow::Result<Vec<RequestLog>> {
        self.get("/api/logs").await
    }
}

// === 类型定义 (从 core 复制，避免循环依赖) ===

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerStatus {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub active_connections: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Provider {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Route {
    pub pattern: String,
    pub provider: String,
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
