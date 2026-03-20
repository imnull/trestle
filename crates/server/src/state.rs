//! 应用状态

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Instant;
use trestle_core::{Config, Provider, RequestLog, Route};
use chrono::{DateTime, Utc};

/// 健康检查信息
#[derive(Debug, Clone)]
pub struct HealthInfo {
    pub healthy: bool,
    pub latency_ms: Option<u64>,
    pub last_check: Option<DateTime<Utc>>,
}

pub struct AppState {
    pub config: RwLock<Config>,
    pub providers: RwLock<Vec<Provider>>,
    pub routes: RwLock<Vec<Route>>,
    pub logs: RwLock<Vec<RequestLog>>,
    pub start_time: Instant,
    pub total_requests: AtomicU64,
    pub total_tokens: AtomicU64,
    pub http_client: reqwest::Client,
    pub health_cache: RwLock<HashMap<String, HealthInfo>>,
}

impl AppState {
    pub fn new(
        config: Config,
        providers: Vec<Provider>,
        routes: Vec<Route>,
    ) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        tracing::info!("Providers loaded: {:?}", providers.iter().map(|p| &p.name).collect::<Vec<_>>());
        tracing::info!("Routes loaded: {} rules", routes.len());

        Ok(Self {
            config: RwLock::new(config),
            providers: RwLock::new(providers),
            routes: RwLock::new(routes),
            logs: RwLock::new(Vec::new()),
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            http_client,
            health_cache: RwLock::new(HashMap::new()),
        })
    }

    pub fn inc_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_tokens(&self, count: u64) {
        self.total_tokens.fetch_add(count, Ordering::Relaxed);
    }

    pub fn add_log(&self, log: RequestLog) {
        let mut logs = self.logs.write().unwrap();
        logs.push(log);
        if logs.len() > 10000 {
            logs.drain(0..1000);
        }
    }

    pub fn update_health(&self, name: String, info: HealthInfo) {
        let mut cache = self.health_cache.write().unwrap();
        cache.insert(name, info);
    }
}
