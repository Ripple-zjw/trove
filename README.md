# Trove 🛠️

**极致性能的跨平台工具集合**

Trove 是一个基于 Rust 后端的跨平台工具集合软件，追求极致性能、极低内存占用与极快启动速度。
所有工具自动获得 CLI 命令、REST API、WebSocket 三种调用方式。

## 安装

### macOS（推荐）

从 [Releases](https://github.com/Ripple-zjw/trove/releases) 下载最新 `.dmg` 安装包。

安装后 CLI 也可用：

```bash
# 桌面应用
open /Applications/Trove.app

# CLI（浏览器访问 http://localhost:8080）
/Applications/Trove.app/Contents/MacOS/trove serve --port 8080

# 直接执行工具
/Applications/Trove.app/Contents/MacOS/trove exec uuid-gen --input '{"count":3}'
```

### 从源码构建

```bash
# 前提
rustup toolchain install stable
brew install node   # 前端构建需要

# 构建 CLI
cargo build --release
./target/release/trove serve --port 8080

# 构建桌面应用（macOS）
bash scripts/copy-sidecar.sh
cd gui && npx tauri build
```

## 功能

| 工具 | 类型 | 说明 |
|------|------|------|
| json-format | JSON | 格式化/压缩 JSON |
| json-validate | JSON | 校验 JSON 合法性，定位错误行列 |
| base64-encode/decode | 编码 | Base64 编解码 |
| hash | 编码 | BLAKE3 哈希计算 |
| ts-to-date / date-to-ts | 日期 | 时间戳与日期互转 |
| url-encode/decode | 网络 | URL 百分号编码/解码 |
| uuid-gen | 通用 | UUID v4 生成（批量） |
| text-stats | 文本 | 文本统计分析（字频柱状图） |
| string-case | 文本 | 大小写/驼峰/蛇形等命名格式互转 |

## 一键开发

```bash
git clone https://github.com/Ripple-zjw/trove.git
cd trove
bash scripts/dev.sh
# → Core API: http://127.0.0.1:8080
# → Web UI:   http://localhost:1420
```

## 架构

```
┌──────────────────────────────────────────────┐
│                   Tauri (桌面壳)               │
│  ┌────────────────┐  ┌────────────────────┐  │
│  │   Web GUI      │  │  CLI (trove)       │  │
│  │ React+Vite+TS  │  │  clap 子命令       │  │
│  └───────┬────────┘  └─────────┬──────────┘  │
│          │ HTTP/WS             │ 直接调用      │
│          └──────────┬──────────┘              │
│                     ▼                        │
│  ┌──────────────────────────────────────┐    │
│  │           Core (Rust/axum)           │    │
│  │  ToolRegistry · ExecuteEngine · MCP  │    │
│  └──────────────────────────────────────┘    │
└──────────────────────────────────────────────┘
```

## 技术栈

| 层 | 技术 |
|----|------|
| 后端 | Rust (axum, clap, tokio, serde) |
| 桌面壳 | Tauri v2（Rust 原生，~5MB） |
| 前端 | React 19 + Vite 6 + TypeScript |
| 通信 | REST（控制面）+ WebSocket（数据面） |

## 许可证

[MIT](LICENSE)
