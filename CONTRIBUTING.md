# Contributing to Trestle

感谢你对 Trestle 的关注！

## 开发

### 环境要求

- Rust 1.70+
- macOS / Linux / Windows

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行
cargo run --bin trestle
```

### 测试

```bash
cargo test
```

### 代码风格

```bash
cargo fmt
cargo clippy
```

## 发布流程

1. 更新版本号
2. 创建 tag: `git tag v0.1.0`
3. 推送 tag: `git push origin v0.1.0`
4. CI 自动构建并发布

## 提交规范

- `feat:` 新功能
- `fix:` 修复 bug
- `docs:` 文档更新
- `refactor:` 重构
- `test:` 测试
- `chore:` 杂项
