# Trove 外部依赖清单

> 记录项目中直接声明的所有外部库及其用途。
> 间接依赖（子依赖）不在此列出，仅列我们在 Cargo.toml / package.json 中显式引用的。

---

## Rust 依赖（后端）

### 核心运行时代理

| 库 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **tokio** | ^1, features=[full] | 异步运行时 | Rust 生态标准，提供 async/await、线程池、timeout、IO 多路复用。Trove 的所有并发执行都基于它 |
| **async-trait** | ^0.1 | 异步 Trait 支持 | 让 Tool trait 的 `execute` 方法可以是 `async fn`。Rust 稳定版尚未原生支持 async trait |

### Web 服务器

| 库 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **axum** | ^0.7, features=[ws] | Web 框架 + WebSocket | Rust 生态中与 tokio 集成最深的 Web 框架，支持同一端口同时处理 HTTP 和 WS，性能与 actix-web 相近但更简洁 |
| **tower-http** | ^0.6, features=[cors] | HTTP 中间件 | 用于添加 CORS 头，让浏览器端的 SPA 可以跨域访问 Core API |

### 序列化

| 库 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **serde** | ^1, features=[derive] | 序列化/反序列化框架 | Rust 事实标准，零开销抽象。用派生宏自动生成序列化代码 |
| **serde_json** | ^1 | JSON 操作 | serde 生态的 JSON 实现，所有工具的输入输出都用它 |

### CLI

| 库 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **clap** | ^4, features=[derive] | 命令行参数解析 | 声明式派生宏，几行代码定义 serve/exec/list 子命令和参数，自动生成 help 文档 |

### 工具实现

| 库 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **uuid** | ^1, features=[v4] | UUID 生成 | uuid-gen 工具的核心依赖，纯 Rust 实现 |
| **chrono** | ^0.4, features=[serde] | 日期时间处理 | 提供时间戳解析、格式化、时区转换，是 ts-to-date 和 date-to-ts 工具的底层 |
| **base64** | ^0.22 | Base64 编解码 | Rust 官方推荐的 base64 实现，纯 Rust、无 unsafe |
| **blake3** | ^1 | BLAKE3 哈希 | hash 工具的核心依赖。相比 SHA-256 快约 5 倍，纯 Rust + 汇编优化 |

### 错误处理

| 库 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **thiserror** | ^2 | 错误类型定义 | 用派生宏自动生成 Display + Error 实现，简洁定义 NotFound/InvalidInput/Timeout 等错误变体 |

### 日志

| 库 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **tracing** | ^0.1 | 诊断日志 | tokio 团队出品，与 async 运行时深度集成，支持结构化日志 |
| **tracing-subscriber** | ^0.3, features=[env-filter] | 日志输出 | 读取 `RUST_LOG` 环境变量控制日志级别 |

### 工具链工具（Cargo.toml workspace 成员）

| 库 | 路径 | 用途 |
|----|------|------|
| **trove-core** | crates/core | Tool trait 定义、注册表、执行引擎 |
| **trove-tools** | crates/tools | 所有工具的实现集合 |
| **trove-server** | crates/server | axum HTTP + WS 服务器 |
| **trove-mcp-server** | crates/mcp-server | MCP 服务器（Phase 3，目前占位） |
| **trove** (cli) | crates/cli | 二进制入口（CLI） |

---

## Rust 间接依赖（主要）

以下是从 Cargo.lock 中提取的值得注意的间接依赖：

| 库 | 什么库带来的 | 用途 |
|----|-------------|------|
| **hyper** ^1.11 | axum 底层 | HTTP 协议实现，axum 的 HTTP 服务器和客户端都构建在它之上 |
| **matchit** ^0.7 | axum 路由 | URL 路由匹配引擎，支持 `:param` 路径参数 |
| **http** ^1.4 | hyper / axum | HTTP 核心类型（StatusCode、HeaderMap 等） |
| **http-body** ^1.1 | hyper / axum | HTTP body 流式处理 |
| **futures** ^0.3 | tokio / axum | 提供 StreamExt、SinkExt 等异步流式工具（WS handler 使用） |
| **tokio-tungstenite** ^0.24 | — | WebSocket 协议实现（我们没直接引用，被 axum 依赖携带） |
| **regex** ^1 | — | 正则表达式引擎（预留在工具依赖中，后续正则测试器工具会用） |
| **crypto-common** + **digest** + **sha1** | blake3 | 哈希算法基础设施 |
| **camino** / **fs-err** | tauri | 文件系统操作 |

---

## npm 依赖（前端）

### 运行时

| 包 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **react** | ^19.0.0 | UI 框架 | 生态最大的前端框架，组件化开发 |
| **react-dom** | ^19.0.0 | DOM 渲染 | React 的浏览器渲染器 |
| **react-router-dom** | ^7.0.0 | 路由 | 支持客户端路由，用于 /（工具列表）和 /tool/:id（工具执行）页面的切换 |

### 开发工具

| 包 | 版本 | 用途 | 为什么选它 |
|----|------|------|-----------|
| **vite** | ^6.0.0 | 构建工具 + 开发服务器 | 极快的 ESBuild 预构建和 HMR（<200ms 热更新） |
| **typescript** | ^5.6.0 | 类型检查 | 前端代码的类型安全保障 |
| **@vitejs/plugin-react** | ^4.3.0 | Vite React 插件 | 支持 React 的 JSX 编译和 HMR |
| **@types/react** | ^19.0.0 | React 类型定义 | TypeScript 需要 |
| **@types/react-dom** | ^19.0.0 | ReactDOM 类型定义 | TypeScript 需要 |
| **@tauri-apps/cli** | ^2.0.0 | Tauri 命令行 | 用于 `npx tauri dev/build` 构建桌面壳 |

### 前端零运行时依赖

值得注意：我们**没有**使用以下常见库，这是刻意的：

| 没用 | 理由 |
|------|------|
| axios | 浏览器原生 fetch 已足够，Trove 不是复杂 SPA |
| zustand / redux | 页面状态简单（工具列表 + 执行表单），React 内置 useState 就够 |
| shadcn/ui / antd | 不想引入大组件库，自制 CSS 更轻量灵活（最终 JS ≈ 76KB gzipped） |
| tailwindcss | 自制 CSS 对 3 页应用够用，少一层构建 |
| tanstack-query | 没有复杂缓存/重试需求，native fetch 足够 |
| framer-motion | MVP 不需要动画 |

---

## 各端依赖体量

```
Rust 编译缓存（target/debug/）：     ~1.8 GB
  ├── 直接依赖：                     11 个
  ├── 间接依赖（Cargo.lock）：       ~140 个
  └── 合计（crates.io 下载）：       152 个

npm node_modules/：                 ~230 MB / 305 个包
  ├── 直接依赖：                     6 个
  └── 间接依赖：                    ~300 个

Tauri CLI（独立安装）：              ~50 MB
```

---

## 为什么选择这些库的关键标准

1. **纯 Rust / 零 unsafe 优先** — blake3 除外（有汇编优化），其他核心库均避免 unsafe
2. **tokio 生态优先** — axum、tracing、tower-http 都是 tokio 团队出品，集成无缝
3. **最小化前端依赖** — 只 6 个直接 npm 依赖，不引入 UI 框架库，保持 76KB gzip
4. **无运行时开销** — serde 是编译期代码生成，clap 是编译期宏，不增加运行时负担
