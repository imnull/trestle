# Trestle 技术架构文档

> 版本: v0.1
> 日期: 2026-03-15

---

## 一、系统概述

Trestle 是一个高性能 AI 代理管理系统，采用 **纯 Rust** 实现，包含：
- **Server**: 代理服务核心 (Axum)
- **Client**: 桌面 GUI 客户端 (egui)
- **Core**: 共享类型和配置库

### 设计目标

1. **高性能** — 零拷贝、异步 IO、低延迟
2. **低资源** — 内存占用 < 50MB，CPU 空闲时 < 1%
3. **易用性** — 开箱即用，配置简单
4. **可扩展** — 插件化上游适配器

---

## 二、系统架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                          用户机器                                │
│                                                                  │
│  ┌─────────────────────┐        ┌────────────────────────────┐  │
│  │   Client (GUI)      │        │    Server (Proxy)          │  │
│  │   ┌───────────────┐ │  HTTP  │   ┌────────────────────┐   │  │
│  │   │   egui UI     │ │◄──────►│   │   Axum HTTP        │   │  │
│  │   │               │ │ :31415 │   │   Server           │   │  │
│  │   │ - Dashboard   │ │        │   └────────────────────┘   │  │
│  │   │ - Providers   │ │        │            │               │  │
│  │   │ - Routes      │ │        │            ▼               │  │
│  │   │ - Logs        │ │        │   ┌────────────────────┐   │  │
│  │   │ - Settings    │ │        │   │   Router Engine    │   │  │
│  │   └───────────────┘ │        │   │   - 模式匹配        │   │  │
│  │         │           │        │   │   - 负载均衡        │   │  │
│  │         ▼           │        │   └────────────────────┘   │  │
│  │   ┌───────────────┐ │        │            │               │  │
│  │   │  API Client   │ │        │            ▼               │  │
│  │   │  (reqwest)    │ │        │   ┌────────────────────┐   │  │
│  │   └───────────────┘ │        │   │   Provider Adapters│   │  │
│  └─────────────────────┘        │   │   - OpenAI         │   │  │
│                                 │   │   - Anthropic      │   │  │
│                                 │   │   - Ollama         │   │  │
│                                 │   │   - Custom         │   │  │
│                                 │   └────────────────────┘   │  │
│                                 │            │               │  │
│                                 └────────────┼───────────────┘  │
│                                              │                  │
│                                              ▼                  │
│                                       ┌─────────────┐           │
│                                       │  互联网     │           │
│                                       │  OpenAI API │           │
│                                       │  Claude API │           │
│                                       │  本地模型   │           │
│                                       └─────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 模块划分

```
trestle/
├── crates/
│   ├── core/           # 共享核心库
│   │   ├── config.rs   # 配置管理
│   │   ├── types.rs    # API 类型定义
│   │   └── error.rs    # 错误处理
│   │
│   ├── server/         # 服务端
│   │   ├── main.rs     # 入口
│   │   ├── handlers.rs # HTTP 处理器
│   │   ├── proxy.rs    # 代理逻辑
│   │   └── state.rs    # 应用状态
│   │
│   └── client/         # 客户端
│       ├── main.rs     # 入口
│       ├── app.rs      # 应用主逻辑
│       ├── api.rs      # API 客户端
│       └── pages/      # UI 页面
│           ├── dashboard.rs
│           ├── providers.rs
│           ├── routes.rs
│           ├── logs.rs
│           └── settings.rs
```

---

## 三、核心组件

### 3.1 Server (代理服务)

#### 技术栈

| 组件 | 库 | 用途 |
|------|-----|------|
| HTTP 框架 | Axum 0.8 | 高性能异步 HTTP |
| 运行时 | Tokio 1.x | 异步 IO |
| HTTP 客户端 | reqwest 0.12 | 上游请求 |
| 流式处理 | async-stream | SSE 流 |
| 日志 | tracing | 结构化日志 |

#### 端点设计

```rust
// OpenAI 兼容
POST /v1/chat/completions    // 聊天补全 (支持流式)
GET  /v1/models              // 模型列表

// Anthropic 兼容
POST /v1/messages            // Anthropic 消息

// 管理接口
GET  /api/status             // 服务状态
GET  /api/config             // 获取配置
PUT  /api/config             // 更新配置
GET  /api/providers          // 服务商列表
POST /api/providers          // 添加服务商
GET  /api/routes             // 路由规则
GET  /api/logs               // 请求日志
```

#### 请求处理流程

```
请求进入
    │
    ▼
┌─────────────┐
│ 路由匹配    │ ─── 根据 model 匹配路由规则
└─────────────┘
    │
    ▼
┌─────────────┐
│ Provider    │ ─── 选择上游服务商
│ Selection   │
└─────────────┘
    │
    ▼
┌─────────────┐
│ 请求转换    │ ─── 格式适配 (OpenAI/Anthropic)
└─────────────┘
    │
    ▼
┌─────────────┐
│ 上游请求    │ ─── HTTP 请求上游 API
└─────────────┘
    │
    ▼
┌─────────────┐
│ 响应转换    │ ─── 统一为客户端格式
└─────────────┘
    │
    ▼
返回客户端
```

### 3.2 Client (GUI 客户端)

#### 技术栈

| 组件 | 库 | 用途 |
|------|-----|------|
| GUI 框架 | egui 0.30 | 即时模式 GUI |
| 应用框架 | eframe 0.30 | 桌面应用封装 |
| HTTP 客户端 | reqwest | API 调用 |
| 异步运行时 | Tokio | 异步操作 |

#### 页面结构

```
┌──────────────────────────────────────────────┐
│  Sidebar (180px)    │   Main Content        │
│  ┌────────────────┐ │   ┌────────────────┐  │
│  │ ⚡ Trestle     │ │   │                │  │
│  │                │ │   │   Current      │  │
│  │ 📊 仪表盘      │ │   │   Page         │  │
│  │ 🔌 服务商      │ │   │   Content      │  │
│  │ 🛤 路由        │ │   │                │  │
│  │ 📜 日志        │ │   │                │  │
│  │ ⚙ 设置        │ │   │                │  │
│  │                │ │   │                │  │
│  │ ─────────────  │ │   │                │  │
│  │ ● 运行中       │ │   └────────────────┘  │
│  │ 端口: 31415   │ │                       │
│  └────────────────┘ │                       │
└──────────────────────────────────────────────┘
```

#### 状态管理

```rust
pub struct TrestleApp {
    current_page: Page,           // 当前页面
    dashboard: DashboardPage,     // 仪表盘状态
    providers: ProvidersPage,     // 服务商状态
    routes: RoutesPage,           // 路由状态
    logs: LogsPage,               // 日志状态
    settings: SettingsPage,       // 设置状态
    api: ApiClient,               // API 客户端
    server_status: Option<ServerStatusInfo>, // 服务状态
}
```

### 3.3 Core (共享库)

#### 配置系统

```rust
pub struct Config {
    pub server: ServerConfig,   // 服务配置
    pub ui: UiConfig,           // UI 配置
    pub logging: LoggingConfig, // 日志配置
}

pub struct ServerConfig {
    pub host: String,    // 默认: "localhost"
    pub port: u16,       // 默认: 31415
}
```

#### 类型系统

```rust
// OpenAI 请求
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: Option<bool>,
    pub tools: Vec<Tool>,
    // ...
}

// OpenAI 响应
pub struct ChatCompletionResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

// 流式响应
pub struct ChatCompletionChunk {
    pub id: String,
    pub choices: Vec<StreamChoice>,
}
```

---

## 四、数据流

### 4.1 非流式请求

```
Client                    Server                    Upstream
  │                         │                          │
  │  POST /v1/chat/...      │                          │
  │ ───────────────────────►│                          │
  │                         │  路由匹配                │
  │                         │  Provider 选择           │
  │                         │                          │
  │                         │  POST /chat/completions  │
  │                         │ ────────────────────────►│
  │                         │                          │
  │                         │      Response (JSON)     │
  │                         │ ◄────────────────────────│
  │                         │                          │
  │     Response (JSON)     │                          │
  │ ◄───────────────────────│                          │
  │                         │                          │
```

### 4.2 流式请求 (SSE)

```
Client                    Server                    Upstream
  │                         │                          │
  │  POST /v1/chat/...      │                          │
  │  (stream: true)         │                          │
  │ ───────────────────────►│                          │
  │                         │                          │
  │                         │  POST /chat/completions  │
  │                         │  (stream: true)          │
  │                         │ ────────────────────────►│
  │                         │                          │
  │  data: {...}            │  data: {...}             │
  │ ◄───────────────────────│ ◄────────────────────────│
  │                         │                          │
  │  data: {...}            │  data: {...}             │
  │ ◄───────────────────────│ ◄────────────────────────│
  │                         │                          │
  │  data: [DONE]           │  data: [DONE]            │
  │ ◄───────────────────────│ ◄────────────────────────│
  │                         │                          │
```

---

## 五、路由系统

### 5.1 路由规则

```toml
[[routes]]
pattern = "gpt-4*"      # 通配符匹配
provider = "openai"     # 上游服务商
model = "gpt-4-turbo"   # 目标模型
priority = 1            # 优先级 (越小越优先)

[[routes]]
pattern = "claude-*"
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"
priority = 1

[[routes]]
pattern = "*"           # 默认规则
provider = "openai"
model = "gpt-4o"
priority = 99
```

### 5.2 匹配算法

```rust
fn match_route(model: &str, routes: &[Route]) -> Option<&Route> {
    let mut matched: Vec<_> = routes.iter()
        .filter(|r| matches_pattern(&r.pattern, model))
        .collect();
    
    // 按优先级排序
    matched.sort_by_key(|r| r.priority);
    
    matched.first()
}

fn matches_pattern(pattern: &str, value: &str) -> bool {
    match pattern {
        "*" => true,
        p if p.ends_with('*') => value.starts_with(&p[..p.len()-1]),
        p if p.starts_with('*') => value.ends_with(&p[1..]),
        p => p == value,
    }
}
```

---

## 六、配置管理

### 6.1 配置文件

```
~/.config/trestle/
├── config.toml        # 主配置
├── providers.toml     # 服务商配置
├── routes.toml        # 路由规则
└── logs/
    └── requests.db    # SQLite 请求日志 (可选)
```

### 6.2 配置示例

```toml
# config.toml
[server]
host = "localhost"
port = 31415

[ui]
theme = "system"      # light / dark / system
language = "zh-CN"
auto_start = true
minimize_to_tray = true

[logging]
level = "info"        # debug / info / warn / error
retention_days = 7

# providers.toml
[[providers]]
name = "openai"
type = "openai"
base_url = "https://api.openai.com/v1"
api_key = "sk-xxx"
enabled = true

[[providers]]
name = "anthropic"
type = "anthropic"
base_url = "https://api.anthropic.com/v1"
api_key = "sk-ant-xxx"
enabled = true

[[providers]]
name = "ollama"
type = "openai-compatible"
base_url = "http://localhost:11434/v1"
enabled = true
```

---

## 七、性能设计

### 7.1 性能目标

| 指标 | 目标值 |
|------|--------|
| 代理延迟增量 | < 5ms |
| 内存占用 | < 50MB |
| CPU 空闲 | < 1% |
| 并发连接 | > 1000 |
| QPS | > 10000 |

### 7.2 优化策略

1. **零拷贝**: 尽量使用引用，避免序列化/反序列化
2. **连接池**: 复用 HTTP 连接
3. **异步 IO**: 全链路异步
4. **流式处理**: SSE 直接转发，不缓冲

```rust
// 流式转发示例
let stream = resp.bytes_stream();
let event_stream = stream! {
    for await chunk in stream {
        for line in parse_lines(&chunk) {
            yield Ok(Event::default().data(line));
        }
    }
};
```

---

## 八、安全设计

### 8.1 API Key 保护

- 存储在本地配置文件，权限 600
- GUI 中默认隐藏显示
- 不记录到日志

### 8.2 网络安全

- 默认监听 localhost，不暴露外网
- 可配置为 0.0.0.0 (用户自担风险)
- 支持 API Key 认证 (管理接口)

### 8.3 数据安全

- 请求日志可选 (默认关闭)
- 不存储请求/响应内容
- 支持日志自动清理

---

## 九、扩展性

### 9.1 Provider 适配器

```rust
pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn chat_completion(&self, req: Request) -> Result<Response>;
    async fn stream_completion(&self, req: Request) -> Result<Stream>;
}

// 实现
pub struct OpenAIAdapter { ... }
pub struct AnthropicAdapter { ... }
pub struct OllamaAdapter { ... }
```

### 9.2 中间件支持

```rust
// 请求中间件
pub trait Middleware: Send + Sync {
    async fn before_request(&self, req: &mut Request);
    async fn after_response(&self, resp: &mut Response);
}

// 示例: 日志中间件
pub struct LoggingMiddleware;
pub struct MetricsMiddleware;
pub struct RateLimitMiddleware;
```

---

## 十、部署

### 10.1 开发模式

```bash
# 启动服务端
cd crates/server
cargo run

# 启动客户端
cd crates/client
cargo run
```

### 10.2 生产构建

```bash
# 构建所有组件
cargo build --release

# 输出
target/release/trestle-server
target/release/trestle
```

### 10.3 打包分发

| 平台 | 格式 | 工具 |
|------|------|------|
| macOS | .dmg / .app | cargo-bundle |
| Windows | .exe | cargo-wix |
| Linux | .AppImage / .deb | cargo-deb |

---

## 十一、监控与调试

### 11.1 日志

```bash
# 启用调试日志
RUST_LOG=debug cargo run

# 日志输出示例
2026-03-15T12:00:00Z INFO  trestle_server: 🚀 Server starting on localhost:31415
2026-03-15T12:00:05Z DEBUG trestle_server::proxy: Route: gpt-4 -> openai (gpt-4-turbo)
2026-03-15T12:00:05Z INFO  trestle_server::proxy: Request completed in 234ms
```

### 11.2 状态接口

```bash
# 获取服务状态
curl http://localhost:31415/api/status

# 响应
{
  "uptime_secs": 3600,
  "total_requests": 1234,
  "total_tokens": 567890,
  "active_connections": 5,
  "providers": [
    {"name": "openai", "healthy": true, "latency_ms": 234}
  ]
}
```

---

## 十二、未来规划

### Phase 2
- [ ] Anthropic `/v1/messages` 完整支持
- [ ] SQLite 请求日志
- [ ] 模型自动发现
- [ ] 健康检查

### Phase 3
- [ ] 负载均衡
- [ ] 重试与降级
- [ ] 请求/响应转换管道
- [ ] 插件系统

### Phase 4
- [ ] 多用户支持
- [ ] 云端同步
- [ ] 团队协作
- [ ] 移动端

---

*文档结束*
