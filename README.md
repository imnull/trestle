# ⚡ Trestle

AI 代理管理工具 — 一个界面管理所有 AI 编程助手

## 特性

- 🚀 **高性能** — Rust 实现，低资源占用
- 🖥️ **纯 Rust GUI** — egui 原生桌面客户端
- 🔌 **统一入口** — OpenAI & Anthropic API 兼容
- 🛤️ **灵活路由** — 按需选择模型
- 🔒 **本地运行** — 隐私安全，数据不出机器
- 📊 **实时监控** — 请求统计、日志查看

## 架构

```
┌─────────────┐        ┌─────────────────┐
│   Client    │  HTTP  │     Server      │
│  (egui GUI) │ ◄────► │  (Axum Proxy)   │
│             │ :31415 │                 │
└─────────────┘        └─────────────────┘
                               │
                               ▼
                        上游 AI 服务
```

## 快速开始

### 1. 构建服务端

```bash
cargo build --release -p trestle-server
```

### 2. 配置

复制示例配置到配置目录：

```bash
mkdir -p ~/.config/trestle
cp config.example.toml ~/.config/trestle/config.toml
cp providers.example.toml ~/.config/trestle/providers.toml
cp routes.example.toml ~/.config/trestle/routes.toml
```

编辑 `providers.toml` 填入你的 API Key：

```toml
[[providers]]
name = "openai"
type = "openai"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-actual-key"  # 替换为真实 Key
enabled = true
```

### 3. 启动服务

```bash
./target/release/trestle-server
```

### 4. 测试

```bash
# 查看状态
curl http://localhost:31415/api/status

# 查看 providers
curl http://localhost:31415/api/providers

# 查看 routes
curl http://localhost:31415/api/routes
```

### 5. 使用代理

将你的 AI 工具配置为使用 Trestle：

```bash
# OpenAI SDK
export OPENAI_API_BASE=http://localhost:31415/v1
export OPENAI_API_KEY=any-key  # 会被代理忽略

# 或在代码中
client = OpenAI(
    base_url="http://localhost:31415/v1",
    api_key="any-key"
)
```

## 配置文件

### config.toml

```toml
[server]
host = "127.0.0.1"
port = 31415

[ui]
theme = "system"
language = "zh-CN"

[logging]
level = "info"
retention_days = 7
```

### providers.toml

```toml
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

### routes.toml

```toml
[[routes]]
pattern = "gpt-4*"
provider = "openai"
model = "gpt-4-turbo"
priority = 1

[[routes]]
pattern = "claude-*"
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"
priority = 1

[[routes]]
pattern = "*"
provider = "openai"
model = "gpt-4o"
priority = 99  # 默认规则
```

## API 端点

### OpenAI 兼容

- `POST /v1/chat/completions` — 聊天补全
- `GET /v1/models` — 模型列表

### Anthropic 兼容

- `POST /v1/messages` — Anthropic 消息

### 管理

- `GET /api/status` — 服务状态
- `GET /api/config` — 获取配置
- `PUT /api/config` — 更新配置
- `GET /api/providers` — 服务商列表
- `GET /api/routes` — 路由规则
- `GET /api/logs` — 请求日志

## 项目结构

```
trestle/
├── crates/
│   ├── core/       # 共享类型和配置
│   ├── server/     # 代理服务
│   └── client/     # GUI 客户端
├── PRD.md          # 产品文档
├── ARCHITECTURE.md # 技术架构
├── research.md     # 市场调研
└── README.md
```

## 开发

```bash
# 开发模式
cargo run --bin trestle-server

# 启用日志
RUST_LOG=debug cargo run --bin trestle-server

# 构建 GUI 客户端
cargo run --bin trestle
```

## 许可证

MIT
