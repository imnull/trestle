# CodePlan Proxy 市场调研报告

## 目标
将用户的 code plan (AI 编程助手) 包装成通用代理服务器，兼容多种下游应用场景。

---

## 一、主流 API 标准对比

### 1. OpenAI Chat Completions API (行业标准)

**覆盖率**: ⭐⭐⭐⭐⭐ 最广泛

**支持的客户端**:
- OpenAI 官方 SDK
- 几乎所有 AI 应用 (Cursor, Continue.dev, VSCode Copilot, Aider 等)
- Ollama, LM Studio, vLLM 等本地推理引擎
- LiteLLM, OneAPI 等网关

**请求格式**:
```json
POST /v1/chat/completions
{
  "model": "gpt-4",
  "messages": [{"role": "user", "content": "Hello"}],
  "temperature": 0.7,
  "stream": true,
  "tools": [{
    "type": "function",
    "function": {
      "name": "run_code",
      "description": "Execute code",
      "parameters": {...}
    }
  }],
  "tool_choice": "auto"
}
```

**响应格式 (非流式)**:
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "model": "gpt-4",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "...",
      "tool_calls": [{
        "id": "call_xxx",
        "type": "function",
        "function": {"name": "run_code", "arguments": "{...}"}
      }]
    },
    "finish_reason": "stop" | "tool_calls"
  }],
  "usage": {"prompt_tokens": 10, "completion_tokens": 20}
}
```

**流式响应 (SSE)**:
```
data: {"id":"chatcmpl-xxx","choices":[{"delta":{"content":"Hi"},"finish_reason":null}]}
data: {"id":"chatcmpl-xxx","choices":[{"delta":{"tool_calls":[{"function":{"name":"run_code"}}]},"finish_reason":null}]}
data: [DONE]
```

### 2. Anthropic Messages API

**覆盖率**: ⭐⭐⭐⭐ 增长中

**特点**:
- 结构化 content blocks (text, tool_use, tool_result)
- 支持 computer_use (2024.10 新功能)
- 显式 `system` 参数 (不放在 messages 里)
- 必须指定 `max_tokens`

**请求格式**:
```json
POST /v1/messages
{
  "model": "claude-3-opus-20240229",
  "max_tokens": 4096,
  "messages": [{"role": "user", "content": "Hello"}],
  "system": "You are helpful",
  "tools": [{
    "name": "run_code",
    "description": "Execute code",
    "input_schema": {...}
  }]
}
```

**响应格式**:
```json
{
  "id": "msg_xxx",
  "type": "message",
  "role": "assistant",
  "content": [
    {"type": "text", "text": "..."},
    {"type": "tool_use", "name": "run_code", "input": {...}}
  ],
  "model": "claude-3-opus",
  "stop_reason": "end_turn" | "tool_use",
  "usage": {"input_tokens": 10, "output_tokens": 20}
}
```

### 3. A2A Protocol (Google Agent-to-Agent)

**覆盖率**: ⭐⭐ 新兴标准

**特点**:
- 专为 AI Agent 通信设计
- 支持多轮对话 + 工具调用
- 标准 JSON-RPC 风格

---

## 二、主流代码助手兼容性矩阵

| 工具 | OpenAI API | Anthropic API | 自定义 |
|------|-----------|---------------|--------|
| **Cursor** | ✅ 主要 | ✅ | - |
| **Continue.dev** | ✅ 主要 | ✅ | ✅ 多种 |
| **Aider** | ✅ 主要 | ✅ | - |
| **VSCode Copilot** | ⚠️ 专有 | - | ✅ |
| **Windsurf** | ✅ | ✅ | - |
| **Trae (字节)** | ✅ | ✅ | 可能 |
| **Claude Code CLI** | - | ✅ 主要 | - |
| **OpenClaw** | ✅ | ✅ | ✅ ACP |

---

## 三、关键发现

### 1. OpenAI Chat Completions = 事实标准

**结论**: 你的代理**必须**以 OpenAI Chat Completions API 作为主要出口格式。

原因:
- 90%+ 的 AI 编程工具支持
- 工具调用 (function calling) 生态成熟
- 流式响应标准 (SSE)
- 简单易实现

### 2. Anthropic Messages = 重要补充

**建议**: 作为第二优先级支持。

原因:
- Claude Code CLI 只支持 Anthropic API
- Claude 3.5/4 在编程任务上表现优秀
- computer_use 能力独特

### 3. 关键功能需求

你的代理需要支持:

| 功能 | 优先级 | 说明 |
|------|--------|------|
| `/v1/chat/completions` | P0 | 核心接口 |
| `messages` 数组 | P0 | 多轮对话 |
| `stream: true` (SSE) | P0 | 流式响应 |
| `tools` (function calling) | P0 | 代码执行 |
| `tool_calls` 响应 | P0 | 工具调用结果 |
| `/v1/models` | P1 | 模型列表 |
| `vision` (图片输入) | P2 | 多模态 |
| `/v1/messages` (Anthropic) | P1 | Claude 兼容 |

---

## 四、竞品分析

### LiteLLM (Python)
- **定位**: 统一 100+ LLM 调用
- **优点**: 成熟、支持全面
- **缺点**: Python、重量级、面向服务端部署
- **GitHub**: 25k+ stars

### OneAPI (Go)
- **定位**: API 网关 + 管理
- **优点**: Go、高性能
- **缺点**: 面向 SaaS、功能复杂
- **GitHub**: 20k+ stars

### Ollama (Go)
- **定位**: 本地推理 + OpenAI 兼容
- **优点**: 轻量、OpenAI 兼容
- **缺点**: 只支持本地模型

### 你的机会
- **Rust 实现**: 高性能 + 低资源占用
- **本地运行**: 用户隐私 + 无需服务器
- **Code Plan 专用**: 针对编程场景优化
- **轻量级**: 单 binary、开箱即用

---

## 五、推荐架构

```
用户配置 (code plan 配置文件)
    ↓
代理服务器 (Rust)
    ├── OpenAI 格式出口 (主要)
    │   └── POST /v1/chat/completions
    │   └── POST /v1/models
    │
    ├── Anthropic 格式出口 (次要)
    │   └── POST /v1/messages
    │
    └── 上游适配器
        ├── OpenAI 适配
        ├── Claude 适配
        ├── 本地模型适配
        └── 用户自定义适配
```

---

## 六、MVP 建议

### Phase 1: OpenAI 兼容 (1-2 周)
- [ ] `/v1/chat/completions` 非流式
- [ ] `/v1/chat/completions` 流式 (SSE)
- [ ] `messages` 多轮对话
- [ ] 基础配置文件 (TOML/YAML)

### Phase 2: Tool Calling (1 周)
- [ ] `tools` 参数解析
- [ ] `tool_calls` 响应生成
- [ ] 内置工具: 代码执行、文件读写

### Phase 3: Anthropic 兼容 (1 周)
- [ ] `/v1/messages` 端点
- [ ] `content` blocks 转换
- [ ] Claude Code CLI 兼容测试

### Phase 4: 增强功能 (可选)
- [ ] `/v1/models` 动态模型列表
- [ ] 请求日志/调试
- [ ] 多上游负载均衡
- [ ] Vision 支持

---

## 七、结论

**代理出口标准**: **OpenAI Chat Completions API** (v1)

这是业界事实标准，覆盖:
- ✅ Cursor
- ✅ Continue.dev
- ✅ Aider
- ✅ Windsurf
- ✅ Trae (大概率)
- ✅ OpenClaw
- ✅ 几乎所有新工具

**第二优先**: Anthropic Messages API (为 Claude Code CLI 用户)

**技术栈**: Rust + Axum/Tokio + 高性能 SSE

---

生成时间: 2026-03-15
