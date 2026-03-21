# Changelog

All notable changes to this project will be documented in this file.

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
