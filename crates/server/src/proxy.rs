//! 代理核心逻辑

use crate::state::AppState;
use axum::response::{IntoResponse, sse::{Event, KeepAlive, Sse}};
use futures::stream::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use trestle_core::{
    ChatCompletionRequest, ChatCompletionResponse, ChatCompletionChunk,
    AnthropicRequest, AnthropicResponse, AnthropicContent, ResponseBlock, AnthropicUsage,
    Message, MessageContent, TrestleError, RequestLog
};

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

// ============================================================================
// Anthropic API 兼容
// ============================================================================

/// Anthropic Messages API
pub async fn anthropic_messages(
    state: &Arc<AppState>,
    req: AnthropicRequest,
) -> Result<AnthropicResponse, TrestleError> {
    let start = Instant::now();

    // 转换为 OpenAI 格式
    let openai_req = anthropic_to_openai(&req);

    // 路由匹配
    let (provider, model) = match_route(&state, &openai_req.model)?;
    tracing::info!("Anthropic route: {} -> {} ({})", req.model, provider.name, model);

    let mut upstream_req = openai_req.clone();
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

    // 转换为 Anthropic 响应
    let anthropic_resp = openai_to_anthropic(&completion);

    // 记录日志
    state.add_log(RequestLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        method: "POST".to_string(),
        path: "/v1/messages".to_string(),
        model: req.model,
        status: 200,
        latency_ms: start.elapsed().as_millis() as u64,
        prompt_tokens: completion.usage.as_ref().map(|u| u.prompt_tokens),
        completion_tokens: completion.usage.as_ref().map(|u| u.completion_tokens),
    });

    Ok(anthropic_resp)
}

/// Anthropic 请求转 OpenAI 请求
fn anthropic_to_openai(req: &AnthropicRequest) -> ChatCompletionRequest {
    let mut messages = Vec::new();

    // 添加 system 消息
    if let Some(ref system) = req.system {
        messages.push(Message {
            role: "system".to_string(),
            content: Some(MessageContent::Text(system.clone())),
            name: None,
            tool_calls: Vec::new(),
            tool_call_id: None,
        });
    }

    // 转换消息
    for msg in &req.messages {
        let content = match &msg.content {
            AnthropicContent::Text(text) => Some(MessageContent::Text(text.clone())),
            AnthropicContent::Blocks(blocks) => {
                let parts: Vec<_> = blocks.iter()
                    .filter_map(|b| {
                        if b.block_type == "text" {
                            b.text.as_ref().map(|t| trestle_core::ContentPart {
                                part_type: "text".to_string(),
                                text: Some(t.clone()),
                                image_url: None,
                            })
                        } else if b.block_type == "image" {
                            b.source.as_ref().map(|s| trestle_core::ContentPart {
                                part_type: "image_url".to_string(),
                                text: None,
                                image_url: Some(trestle_core::ImageUrl {
                                    url: format!("data:{};base64,{}", s.media_type, s.data),
                                    detail: None,
                                }),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(MessageContent::Parts(parts))
                }
            }
        };

        messages.push(Message {
            role: msg.role.clone(),
            content,
            name: None,
            tool_calls: Vec::new(),
            tool_call_id: None,
        });
    }

    ChatCompletionRequest {
        model: req.model.clone(),
        messages,
        temperature: None,
        top_p: None,
        max_tokens: Some(req.max_tokens),
        stream: req.stream,
        tools: Vec::new(),
        tool_choice: None,
        extra: std::collections::HashMap::new(),
    }
}

/// OpenAI 响应转 Anthropic 响应
fn openai_to_anthropic(resp: &ChatCompletionResponse) -> AnthropicResponse {
    let text = resp.choices.first()
        .and_then(|c| c.message.content.as_ref())
        .and_then(|c| match c {
            MessageContent::Text(t) => Some(t.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let stop_reason = resp.choices.first()
        .and_then(|c| c.finish_reason.as_ref())
        .map(|r| {
            match r.as_str() {
                "stop" => "end_turn",
                "length" => "max_tokens",
                _ => "end_turn",
            }.to_string()
        });

    AnthropicResponse {
        id: format!("msg_{}", resp.id),
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![ResponseBlock {
            block_type: "text".to_string(),
            text: Some(text),
        }],
        model: resp.model.clone(),
        stop_reason,
        usage: AnthropicUsage {
            input_tokens: resp.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0),
            output_tokens: resp.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0),
        },
    }
}

/// 路由匹配
fn match_route(state: &AppState, model: &str) -> Result<(trestle_core::Provider, String), TrestleError> {
    let routes = state.routes.read().unwrap();
    let providers = state.providers.read().unwrap();

    if providers.is_empty() {
        return Err(TrestleError::Route("No providers configured".to_string()));
    }

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
