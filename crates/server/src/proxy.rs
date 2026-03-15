//! 代理核心逻辑

use crate::state::AppState;
use axum::response::{IntoResponse, sse::{Event, KeepAlive, Sse}};
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use std::time::Instant;
use trestle_core::{ChatCompletionRequest, ChatCompletionResponse, TrestleError};

/// 非流式聊天补全
pub async fn chat_completion(
    state: &Arc<AppState>,
    req: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, TrestleError> {
    let start = Instant::now();

    // 1. 路由匹配
    let (provider, model) = match_route(&state, &req.model)?;

    tracing::info!("Route: {} -> {} ({})", req.model, provider.name, model);

    // 2. 构建上游请求
    let mut upstream_req = req.clone();
    upstream_req.model = model.clone();

    // 3. 发送请求
    let client = &state.http_client;
    let resp = client
        .post(format!("{}/chat/completions", provider.base_url))
        .header("Authorization", format!("Bearer {}", provider.api_key.as_deref().unwrap_or("")))
        .json(&upstream_req)
        .send()
        .await
        .map_err(|e| TrestleError::Upstream(e.to_string()))?;

    // 4. 处理响应
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(TrestleError::Upstream(format!("HTTP {}: {}", status, body)));
    }

    let completion: ChatCompletionResponse = resp
        .json()
        .await
        .map_err(|e| TrestleError::Upstream(e.to_string()))?;

    // 5. 更新统计
    if let Some(usage) = &completion.usage {
        state.add_tokens(usage.total_tokens as u64);
    }

    // 6. 记录日志
    let log = trestle_core::RequestLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        method: "POST".to_string(),
        path: "/v1/chat/completions".to_string(),
        model: req.model,
        status: 200,
        latency_ms: start.elapsed().as_millis() as u64,
        prompt_tokens: completion.usage.as_ref().map(|u| u.prompt_tokens),
        completion_tokens: completion.usage.as_ref().map(|u| u.completion_tokens),
    };
    state.add_log(log);

    Ok(completion)
}

/// 流式聊天补全
pub async fn stream_chat_completion(
    state: &Arc<AppState>,
    req: ChatCompletionRequest,
) -> Result<impl IntoResponse, TrestleError> {
    // 1. 路由匹配
    let (provider, model) = match_route(&state, &req.model)?;

    tracing::info!("Stream route: {} -> {} ({})", req.model, provider.name, model);

    // 2. 构建上游请求
    let mut upstream_req = req.clone();
    upstream_req.model = model.clone();
    upstream_req.stream = Some(true);

    // 3. 发送请求
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

    // 4. 流式转发
    let stream = resp.bytes_stream();

    let event_stream = stream
        .map(|result| result.map_err(axum::Error::new))
        .flat_map(|chunk_result| {
            match chunk_result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let events: Vec<Result<Event, axum::Error>> = text
                        .lines()
                        .filter(|line| line.starts_with("data: "))
                        .map(|line| {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                Ok(Event::default().data("[DONE]"))
                            } else {
                                Ok(Event::default().data(data))
                            }
                        })
                        .collect();
                    futures::stream::iter(events)
                }
                Err(e) => futures::stream::iter(vec![Err(e)]),
            }
        });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

/// 路由匹配
fn match_route(state: &AppState, model: &str) -> Result<(trestle_core::Provider, String), TrestleError> {
    let routes = state.routes.read().unwrap();
    let providers = state.providers.read().unwrap();

    // 按优先级排序的路由
    let mut matched_routes: Vec<_> = routes.iter()
        .filter(|r| matches_pattern(&r.pattern, model))
        .collect();
    matched_routes.sort_by_key(|r| r.priority);

    if let Some(route) = matched_routes.first() {
        // 找到对应的服务商
        if let Some(provider) = providers.iter().find(|p| p.name == route.provider) {
            let target_model = route.model.as_ref()
                .map(|m| m.replace("${model}", model))
                .unwrap_or_else(|| model.to_string());
            return Ok((provider.clone(), target_model));
        }
    }

    // 默认: 使用第一个启用的服务商
    if let Some(provider) = providers.iter().find(|p| p.enabled) {
        return Ok((provider.clone(), model.to_string()));
    }

    Err(TrestleError::Route(format!("No provider found for model: {}", model)))
}

/// 通配符匹配
fn matches_pattern(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len()-1];
        return value.starts_with(prefix);
    }

    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        return value.ends_with(suffix);
    }

    pattern == value
}
