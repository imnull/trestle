# Trestle - AI 代理管理工具

> 一键启动和管理 AI 代理服务

## 技术栈

- **GUI**: egui (Rust)
- **后端**: Axum + Tokio
- **架构**: 单机应用（内嵌 HTTP 服务）

## 开发

```bash
# 构建
cargo build --release

# 运行
cargo run --release -p trestle-client

# 或者直接运行二进制文件
./target/release/trestle
```

## 项目结构

```
trestle/
├── crates/
│   ├── core/           # 共享类型和配置
│   ├── server/         # HTTP 服务 (Axum)
│   └── client/         # GUI 应用 (egui)
├── config.example.toml # 配置示例
├── providers.example.toml
└── routes.example.toml
```

## 功能

- ✅ 服务商管理 (OpenAI, Claude, 等)
- ✅ 路由规则配置
- ✅ 请求日志查看
- ✅ 单机运行，无需额外服务
- ✅ 跨平台 (Windows, macOS, Linux)

## 运行原理

应用启动时会自动启动内嵌 HTTP 服务（默认端口 31415），GUI 通过本地 API 与服务通信。同时提供 OpenAI 兼容的 API 端点供外部使用。
