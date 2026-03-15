# ⚡ Trestle

AI 代理管理工具 — 一个界面管理所有 AI 编程助手

## 特性

- 🚀 **高性能** — Rust 实现，低资源占用
- 🖥️ **纯 Rust GUI** — egui 原生桌面客户端
- 🔌 **统一入口** — OpenAI & Anthropic API 兼容
- 🛤️ **灵活路由** — 按需选择模型
- 🔒 **本地运行** — 隐私安全，数据不出机器
- 📊 **实时监控** — 请求统计、日志查看
- 🎯 **一键启动** — Server 自动启动， 无需手动管理

## 快速开始

### 枇方式一：启动应用

```bash
./trestle  # 自动启动 Server + 打开 GUI
```

**就是这么简单！** 

### 方式二、只启动 Server（高级用户/无头环境）

```bash
./trestle-server  # 只启动服务端
```

### 配置

1. **复制示例配置到配置目录**：

```bash
mkdir -p ~/.config/trestle
cp config.example.toml ~/.config/trestle/config.toml
cp providers.example.toml ~/.config/trestle/providers.toml
cp routes.example.toml ~/.config/trestle/routes.toml
```

2. **编辑 `providers.toml` 填入你的 API Key：

```tom
[[providers]]
name = "openai"
type = "openai"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-actual-key"  # 替换为真实 Key
enabled = true

[[providers]]
name = "anthropic"
type = "anthropic"
base_url = "https://api.anthropic.com/v1"
api_key = "sk-ant-your-actual-key"  # 替换为真实 Key
enabled = true

[[providers]]
name = "ollama"
type = "openai-compatible"
base_url = "http://localhost:11434/v1"
enabled = true
```

3. **启动应用**

```bash
./trestle
```

GUI 会自动打开，连接到 Server。

## 架构

```
┌─────────────┐        ┌─────────────────┐
│   Client    │  HTTP  │     Server      │
│  (egui GUI) │ ◄────► │  (Axum Proxy)   │
│             │  :31415 │                 │
└─────────────┘        └─────────────────┘
                               │
                               ▼
                        上游 AI 服务
```

## 使用代理

将你的 AI 工具配置为使用 Trestle：

```bash
# Cursor / Continue / Aider
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

```tom
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

```om
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

```om
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
cargo run --bin trestle

# 启用日志
RUST_LOG=debug cargo run --bin trestle

# 只启动 Server
cargo run --bin trestle-server
```

## 发布

项目使用 GitHub Actions 自动构建多平台版本：

### 支持的平台

| 平台 | 架构 | 文件格式 |
|------|------|----------|
| **Linux** | x86_64 | tar.gz |
| **Windows** | x86_64 | zip |
| **macOS** | x86_64 (Intel) | dmg / zip |
| **macOS** | ARM64 (Apple Silicon) | dmg / zip |
| **macOS** | Universal (Intel + ARM) | dmg / zip ⭐ 推荐 |

### 发布流程

```bash
# 1. 更新版本号
vim Cargo.toml  # 修改 version = "0.2.0"

# 2. 创建 tag
git tag v0.2.0
git push origin v0.2.0

# 3. 等待 CI 构建完成
# 访问 https://github.com/imnull/trestle/actions 查看进度

# 4. 发布完成！
# 在 https://github.com/imnull/trestle/releases 下载
```

### macOS 通用版本

**Universal Binary** 同时支持 Intel 和 Apple Silicon Mac，推荐分发此版本：

- 单个 `.dmg` 文件，适用于所有 Mac
- 自动检测 CPU 架构，无需用户选择
- 文件稍大（约 30MB），但兼容性最好

## 许可证

MIT
