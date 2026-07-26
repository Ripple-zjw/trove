# Trove — 极致性能的跨平台工具集合

## 项目概述

Trove 是一个跨平台工具集合软件，使用 Rust 后端 + React/Tauri 前端。
核心设计理念：极致性能、极低内存占用、极快启动速度。

## 技术栈

| 层 | 技术 | 理由 |
|----|------|------|
| 后端 Core | **Rust** (axum, clap, tokio) | 零运行时、无 GC、单二进制、超快启动 |
| 桌面壳 | **Tauri** | Rust 原生、~5MB、极致轻量 |
| 前端 | **React + Vite + TypeScript** | 生态最大、开发高效 |
| 通信 | **REST + WebSocket 混合** | REST 控制面，WS 数据面，MCP 对位 |

## 项目结构

```
trove/
├── Cargo.toml              # workspace root
├── crates/
│   ├── core/               # Tool trait 定义、注册表、执行引擎
│   ├── tools/              # 所有工具的实现
│   ├── server/             # axum HTTP + WS 服务器
│   ├── cli/                # CLI 入口 (clap)
│   └── mcp-server/         # MCP 服务器（Phase 2）
├── gui/                    # React + Vite + TypeScript 前端
│   ├── src/
│   ├── src-tauri/          # Tauri 壳
│   └── package.json
└── scripts/                # 构建/开发脚本
```

## 核心架构决策

### Tool Trait 系统
- 每个工具实现 `Tool` trait（`id`, `name`, `description`, `input_schema`, `category`, `execute`）
- 自动获得 CLI 命令 + HTTP API + WS 命令三种暴露方式
- 注册式：`Vec<Box<dyn Tool>>` 线性查找（工具数 < 200 时 O(n) 足够）

### 执行模型
- 同进程执行，tokio async task 调度
- CPU 密集型工具（`is_cpu_intensive()`）通过 `tokio::spawn` 隔离
- 三重安全防线：`timeout` + 输入大小校验 + 异常捕获

### 通信协议

> axum 0.7 使用 matchit 0.7，路径参数用 `:id` 语法（非 `{id}`，那是 axum 0.8+）

- **REST**: `GET /api/tools` 查询工具列表，`GET /api/tools/:id` 获取元数据，`POST /api/tools/:id/execute` 同步执行
- **WebSocket**: `WS /api/ws` 支持流式执行和推送
- **配置**: `GET/PUT /api/config`

### 结果展示模式

CLI 和 Web UI 的结果展示都按工具 ID 做 switch 分发：

- CLI: `format_output()` 在 `crates/cli/src/main.rs`，按 `tool_id` match
- Web UI: `renderResult()` 在 `gui/src/pages/ToolExecute.tsx`，按 `id` switch
- 原则：REST API 返回机器可读的结构化 JSON，展示层负责格式化

### 前端枚举选项描述

JSON Schema 不支持 per-enum-item 描述。做法：在前端定义 `ENUM_DESCRIPTIONS` 常量映射，
渲染 `<select>` 时在 `<option>` 上加 `title` 属性实现 hover 提示。参见
`gui/src/pages/ToolExecute.tsx` 中的 `ENUM_DESCRIPTIONS`。

## 开发指南

```bash
# 启动后端
cargo run -- serve --port 8080

# 启动前端
cd gui && npm run dev

# 一键开发环境
bash scripts/dev.sh

# 直接 CLI 执行
cargo run -- exec uuid-gen --input '{"count":1}'

# 构建
cargo build --release
```

## 构建与打包

```bash
# Tauri 打包（需要先复制 sidecar 二进制）
bash scripts/copy-sidecar.sh       # 先编译并复制 Core 二进制到 src-tauri/binaries/
cd gui && npx tauri build          # 再构建 Tauri 安装包

# 产物路径:
#   gui/src-tauri/target/release/bundle/macos/Trove.app
#   gui/src-tauri/target/release/bundle/dmg/Trove_0.1.0_aarch64.dmg
```

> 注意：`gui/src-tauri/` 不是 workspace 成员，它的 `Cargo.toml` 含有独立的 `[workspace]` 表。

## 添加新工具

详细步骤见 [`docs/add-tool.md`](docs/add-tool.md)。快速概览：

1. **创建文件** `crates/tools/src/your_tool.rs`，实现 `Tool` trait
2. **注册**：在 `crates/tools/src/lib.rs` 的 `register_all()` 加一行 `.register(your_tool::YourTool)`
3. **验证**：`cargo build && ./target/debug/trove list | grep your-tool-id`

前端会根据 `input_schema` 自动生成表单——无需额外编码。

## 验证方式

```bash
# 验证 API
curl http://127.0.0.1:8080/api/tools
curl -X POST http://127.0.0.1:8080/api/tools/uuid-gen/execute \
  -H 'Content-Type: application/json' -d '{"input":{"count":1}}'
```

## 依赖

### Rust (Cargo workspace)
- `crates/core`: tokio, serde, serde_json, async-trait, thiserror
- `crates/tools`: uuid, chrono, base64, blake3
- `crates/server`: axum (ws), tower-http (cors)
- `crates/cli`: clap

### 前端 (npm)
- react 19, react-router-dom 7
- vite 6, typescript 5
- @tauri-apps/cli 2

## 外部依赖

全部依赖清单见 [`docs/dependencies.md`](docs/dependencies.md)。

### 直接依赖速览

| 层 | 核心库 | 数量 |
|----|--------|------|
| Rust 运行时 | tokio, async-trait | 2 |
| Web 服务器 | axum, tower-http | 2 |
| 序列化 | serde, serde_json | 2 |
| CLI | clap | 1 |
| 工具实现 | uuid, chrono, base64, blake3 | 4 |
| 错误/日志 | thiserror, tracing, tracing-subscriber | 3 |
| 前端运行时 | react, react-dom, react-router-dom | 3 |
| 前端工具 | vite, typescript, @vitejs/plugin-react | 3 |
| 桌面壳 | @tauri-apps/cli | 1 |

共计 **21 个直接依赖**（11 Rust + 10 npm），间接约 450 个。

## 实现阶段

1. ✅ **MVP**: Rust Core + 12 初始工具 + REST API + WS + CLI + React Web UI + Tauri 壳
2. 📋 **完善**: 30-50 工具，PWA，配置系统，工具分类/搜索/收藏，主题系统
3. 📋 **高级**: MCP 服务器，移动端，插件系统

## 网络环境

在中国大陆开发时，注意设置 git 和 npm 代理：
```bash
git config --global http.proxy http://127.0.0.1:7897
npm config set proxy http://127.0.0.1:7897
```
测试 API 时使用 `--noproxy '*'` 或 `curl --noproxy '*'` 绕过代理。
