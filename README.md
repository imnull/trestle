# Trestle - AI 代理管理工具

> 一键启动和管理 AI 代理服务

## 技术栈

- **前端**: Vite + React 18 + TypeScript + Tailwind CSS
- **后端**: Tauri 2.0 + Rust
- **状态管理**: Zustand

## 开发

```bash
# 安装依赖
npm install

# 开发模式 (需要 Tauri 环境)
npm run tauri dev

# 构建
npm run tauri build
```

## 项目结构

```
trestle/
├── src/                    # 前端源码
│   ├── components/         # UI 组件
│   ├── pages/              # 页面组件
│   ├── stores/             # Zustand stores
│   └── lib/                # 工具函数
├── src-tauri/              # Tauri 后端
│   └── src/
│       ├── main.rs         # 入口
│       ├── lib.rs          # 主逻辑
│       ├── commands.rs     # Tauri 命令
│       └── tray.rs         # 系统托盘
└── crates/                 # Rust crates
    ├── core/               # 共享类型和逻辑
    └── server/             # 内嵌 HTTP 服务
```

## 功能

- ✅ 服务商管理 (OpenAI, Claude, 等)
- ✅ 路由规则配置
- ✅ 请求日志查看
- ✅ 系统托盘支持
- ✅ 跨平台 (Windows, macOS, Linux)
