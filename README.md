# Trove 🧰

**极致性能的跨平台工具集合**

Trove 是一个基于 Rust 后端的跨平台工具集合，追求极致性能、极低内存占用与极快启动速度。

## 功能特性

- **多功能工具箱** — UUID 生成、哈希计算、Base64 编解码、JSON 格式化/校验、时间戳转换、字符串变换、文本统计、URL 编解码
- **三合一接口** — 所有工具自动获得 CLI 命令、REST API、WebSocket 三种调用方式
- **极致性能** — Rust 零运行时，秒级启动，毫秒级响应
- **轻量桌面** — Tauri 壳，打包体积 ~5MB
- **自动表单** — 前端根据 `input_schema` 自动生成工具表单，新增工具无需前端编码

## 快速开始

```bash
# 启动后端
cargo run -- serve --port 8080

# 启动前端（开发模式）
cd gui && npm run dev

# 一键启动
bash scripts/dev.sh

# CLI 直接使用
cargo run -- exec uuid-gen --input '{"count":1}'
```

## 架构

| 层 | 技术 |
|----|------|
| 后端 Core | Rust (axum, clap, tokio) |
| 桌面壳 | Tauri（Rust 原生，~5MB） |
| 前端 | React + Vite + TypeScript |
| 通信 | REST（控制面）+ WebSocket（数据面） |

## 构建

```bash
cargo build --release
```

## 许可证

[MIT](LICENSE)
