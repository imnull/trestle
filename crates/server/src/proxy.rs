//! 代理核心逻辑

use crate::state::AppState;
use axum::response::{IntoResponse, sse::{Event, KeepAlive, Sse}};
use futures::stream::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use trestle_core::{ChatCompletionRequest, ChatCompletionResponse, TrestleError, RequestLog};

/// 非流式聊天补全
pub async fn chat_completion(
    state: &Arc<AppState>,
    req: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, TrestleError> {
    let start = Instant::now();

    // 路由匹配
    let (provider, model) = match_route(&state, &req.model)?;
    tracing::info!("Route: {} -> {} ({})", req.model, provider.name, model);

    // 构建上游请求
    let mut upstream_req = req.clone();
    upstream_req.model = model.clone();

    // 发送请求
    let client = &state.http_client;
    let resp = client
        .post(format!("{}/chat/completions", provider.base_url))
        .header("Authorization", format!("Bearer {}", provider.api_key.as_deref().unwrap_or("")))
        .json(&upstream_req)
        .send()
        .await
        .map_err(|e| TrestleError::Upstream(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(TrestleError::Upstream(format!("HTTP {}: {}", status, body)));
    }

    let completion: ChatCompletionResponse = resp
        .json()
        .await
        .map_err(|e| TrestleError::Upstream(e.to_string()))?;

    // 更新统计
    if let Some(usage) = &completion.usage {
        state.add_tokens(usage.total_tokens as u64);
    }

    // 记录日志
    state.add_log(RequestLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        method: "POST".to_string(),
        path: "/v1/chat/completions".to_string(),
        model: req.model,
        status: 200,
        latency_ms: start.elapsed().as_millis() as u64,
        prompt_tokens: completion.usage.as_ref().map(|u| u.prompt_tokens),
        completion_tokens: completion.usage.as_ref().map(|u| u.completion_tokens),
    });

    Ok(completion)
}

/// 流式聊天补全
pub async fn stream_chat_completion(
    state: &Arc<AppState>,
    req: ChatCompletionRequest,
) -> Result<impl IntoResponse, TrestleError> {
    let (provider, model) = match_route(&state, &req.model)?;
    tracing::info!("Stream route: {} -> {} ({})", req.model, provider.name, model);

    let mut upstream_req = req.clone();
    upstream_req.model = model.clone();
    upstream_req.stream = Some(true);

    let client = &state.http_client;
    let resp = client
        .post(format!("{}/chat/completions", provider.base_url))
        .header("Authorization", format!("Bearer {}", provider.api_key.as_deref().unwrap_or("")))
        .json(&upstream_req)
        .send()
        .await
        .map_err(|e| TrestleError::Upstream(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(TrestleError::Upstream(format!("HTTP {}: {}", status, body)));
    }

    let stream = resp.bytes_stream();
    let event_stream = stream
        .map(|result| result.map_err(axum::Error::new))
        .flat_map(|chunk_result| match chunk_result {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                let events: Vec<Result<Event, axum::Error>> = text
                    .lines()
                    .filter(|line| line.starts_with("data: "))
                    .map(|line| {
                        let data = &line[6..];
                        Ok(Event::default().data(data))
                    })
                    .collect();
                futures::stream::iter(events)
            }
            Err(e) => futures::stream::iter(vec![Err(e)]),
        });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

/// 路由匹配
fn match_route(state: &AppState, model: &str) -> Result<(trestle_core::Provider, String), TrestleError> {
    let routes = state.routes.read().unwrap();
    let providers = state.providers.read().unwrap();

    // 检查是否有配置
    if providers.is_empty() {
        return Err(TrestleError::Route("No providers configured. Please create providers.toml".to_string()));
    }

    // 按优先级匹配路由
    let mut matched: Vec<_> = routes.iter()
        .filter(|r| matches_pattern(&r.pattern, model))
        .collect();
    matched.sort_by_key(|r| r.priority);

    if let Some(route) = matched.first() {
        if let Some(provider) = providers.iter().find(|p| p.name == route.provider && p.enabled) {
            let target_model = route.model.as_ref()
                .map(|m| m.replace("${model}", model))
                .unwrap_or_else(|| model.to_string());
            return Ok((provider.clone(), target_model));
        }
    }

    // 默认: 使用第一个启用的服务商
    providers.iter()
        .find(|p| p.enabled)
        .map(|p| (p.clone(), model.to_string()))
        .ok_or_else(|| TrestleError::Route("No enabled providers".to_string()))
}

/// 通配符匹配
fn matches_pattern(pattern: &str, value: &str) -> bool {
    match pattern {
        "*" => true,
        p if p.ends_with('*') => value.starts_with(&p[..p.len()-1]),
        p if p.starts_with('*') => value.ends_with(&p[1..]),
        p => p == value,
    }
}
