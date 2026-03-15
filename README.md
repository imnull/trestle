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

### 构建服务端

```bash
cd crates/server
cargo build --release
./target/release/trestle-server
```

### 构建客户端

```bash
cd crates/client
cargo build --release
./target/release/trestle
```

### 配置

配置文件位于 `~/.config/trestle/config.toml`

```toml
[server]
host = "localhost"
port = 31415

[ui]
theme = "system"
language = "zh-CN"

[logging]
level = "info"
retention_days = 7
```

## API 端点

### OpenAI 兼容

- `POST /v1/chat/completions` — 聊天补全
- `GET /v1/models` — 模型列表

### Anthropic 兼容

- `POST /v1/messages` — Anthropic 消息

### 管理

- `GET /api/status` — 服务状态
- `GET/PUT /api/config` — 配置管理
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
├── research.md     # 市场调研
└── README.md
```

## 许可证

MIT
