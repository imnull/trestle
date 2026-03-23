# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2026-03-23

### Changed
- **UI Framework Migration**: 从 egui 迁移到 Tauri 2.0 + React
  - Vite + React 18 + TypeScript 前端
  - Tailwind CSS 样式系统
  - Zustand 状态管理
  - 完美支持中文显示

### Added
- **New UI Components**
  - 仪表盘页面 - 服务器状态、统计卡片
  - 服务商管理 - CRUD 操作、启用/禁用
  - 路由管理 - 路径映射、模型选择
  - 日志查看 - 实时日志、级别过滤
  - 设置页面 - 端口、日志级别、主题配置

- **System Tray**
  - 显示/隐藏窗口
  - 服务状态
  - 退出应用

- **Icons**
  - 自定义闪电图标设计
  - 多平台图标资源 (PNG, ICO, ICNS)

### Removed
- 旧的 egui 客户端 (保留在 crates/client/ 但不再默认构建)

### Technical
- Tauri 2.0 框架
- React 18 + TypeScript 5
- Tailwind CSS 3.4
- Vite 6.0 构建工具
- 复用现有 Rust 后端 (crates/server, crates/core)

## [1.0.0] - 2026-03-21

### Added
- **Core Proxy Server**
  - OpenAI-compatible `/v1/chat/completions` (streaming + non-streaming)
  - Anthropic-compatible `/v1/messages` endpoint
  - `/v1/models` endpoint to list available models
  - Flexible routing with wildcard pattern matching
  - Multi-provider support (OpenAI, Anthropic, Ollama, custom)

- **GUI Client (egui)**
  - Dashboard with real-time stats
  - Provider management (CRUD operations)
  - Route configuration
  - Request logs viewer
  - Settings page
  - Import/Export configuration

- **System Features**
  - SQLite-based persistent logging
  - Provider health checks
  - Configuration hot-reload
  - System tray support (optional, requires `--features tray`)

- **Configuration**
  - TOML-based config files
  - Provider definitions in `providers.toml`
  - Routing rules in `routes.toml`
  - Sample configs included

- **Build & Distribution**
  - Multi-platform CI/CD (Linux, Windows, macOS)
  - Universal binary for macOS (Intel + ARM)
  - Automatic GitHub Releases

### Technical
- Rust 1.94.0+ required
- Axum HTTP framework
- egui native GUI
- SQLite for log persistence
- Full async with Tokio

## [0.1.0] - 2026-03-15

### Added
- Initial MVP
- Basic proxy functionality
- Simple egui client
