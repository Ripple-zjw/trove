# Trove 设计文档

> 极致性能、极低内存占用、极快启动速度的跨平台工具集合软件
>
> 项目创建日期：2026-07-25

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术决策](#2-技术决策)
3. [架构设计](#3-架构设计)
4. [项目结构](#4-项目结构)
5. [API 设计](#5-api-设计)
6. [工具系统](#6-工具系统)
7. [目前进度](#7-目前进度)
8. [TODO —— 总待办列表](#8-todo--总待办列表)

---

## 1. 项目概述

### 1.1 目标

构建一个跨五平台（macOS / Windows / Linux / iOS / Android）的工具集合软件。核心是一个高性能的 Core 服务，负责实现所有工具的逻辑；用户可以通过浏览器或桌面 GUI 访问 Core，也可以通过 CLI 直接调用。

### 1.2 定位

- **A 类工具（首期）**：开发者工具 —— JSON 格式化、Base64 编解码、时间戳转换、UUID 生成、Hash 计算、URL 编解码、文本统计、字符串格式转换、正则测试、颜色转换等
- **B 类工具（扩展）**：生产力工具 —— Markdown 编辑器、思维导图、待办事项、画图工具、文件格式转换等
- **MCP 扩展（远期）**：大模型通过 MCP 服务器访问 Core 中的工具

## 2. 技术决策

| 决策点 | 结论 | 理由 |
|--------|------|------|
| Core 语言 | **Rust** 🦀 | 零运行时、无 GC、单二进制部署、启动 <10ms、内存 <15MB RSS |
| GUI 形态 | **Web GUI (SPA)** | 一套代码跑所有平台，开发效率高 |
| 前端框架 | **React + Vite + TypeScript** | 生态最大、开发效率高、Vite HMR 极快 |
| 桌面壳 | **Tauri** | Rust 原生、~5MB、与 Core 同语言深度集成 |
| 通信协议 | **REST + WebSocket 混合** | REST 控制面（查询/配置），WS 数据面（工具执行流式输出） |
| 工具注册 | **Vec 线性查找** | 工具数 <200 时 O(n) ≈ 1-3μs，够用不提前优化 |
| Core 生命周期 | **混合模式** | CLI 启动为 daemon，GUI 可启动/连接已有 Core |
| 执行模型 | **同进程 async task** | CPU 密集型工具用 `tokio::spawn` 隔离 |
| 安全措施 | timeout + 输入校验 + catch_unwind | 三道防线覆盖 99% 异常 |
| 配置系统 | JSON 文件，三层覆盖 | CLI 参数 > 配置文件 > 默认值 |
| 移动端策略 | 桌面 + PWA 先行 | 开发者工具在移动端场景少，后期用 Tauri Mobile |
| 项目名 | **Trove** | 意喻"宝库"——一堆有用的小工具 |

## 3. 架构设计

### 3.1 核心抽象：Tool Trait

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> serde_json::Value;  // JSON Schema — 前端动态生成表单
    fn category(&self) -> ToolCategory;
    fn is_cpu_intensive(&self) -> bool { false }
    async fn execute(&self, input: Value, ctx: ToolContext) -> ToolResult<Value>;
}
```

**设计理由**：每个工具只关注自己的逻辑实现。框架自动为每个工具提供三种入口：
1. CLI 命令（clap 子命令）
2. HTTP API 端点（axum 路由）
3. WebSocket 命令
4. 未来：MCP 工具暴露

### 3.2 执行引擎

```
请求到达 axum
  → REST: axum handler 创建 async task
  → WS:   axum handler 创建 async task 处理 WS 消息
      → 轻量工具（<1ms）：直接在当前 async task 执行
      → CPU 密集型：tokio::spawn 隔离执行
      → 超时控制：tokio::time::timeout 兜底
```

### 3.3 Core 生命周期

```
用户打开 Tauri 桌面应用
  → Tauri 检查 localhost:<port> 是否存活
  → 否 → Tauri 启动 trove serve 子进程，等待 ready
  → 是 → 直接连接
  → Web GUI 加载 → REST 获取工具列表 → 用户选择工具 → WS 或 REST 执行

用户关闭 GUI 窗口
  → 默认：Core 继续运行（后台托盘）
  → 可选：关闭 GUI 时同时关闭 Core
```

### 3.4 配置层次

```
CLI 参数（最高优先级）
  ↓ 覆盖
JSON 配置文件（~/.config/trove/config.json）
  ↓ 覆盖
代码内默认值
```

## 4. 项目结构

```
trove/
├── Cargo.toml                    # Cargo workspace 根
├── CLAUDE.md                     # 项目文档（AI 辅助用）
├── docs/
│   └── design.md                 # 本文档
│
├── crates/
│   ├── core/                     # Tool trait、注册表、执行引擎、错误类型、上下文
│   │   └── src/
│   │       ├── lib.rs            # 公开导出
│   │       ├── tool.rs           # Tool trait + ToolCategory + ToolMetadata
│   │       ├── registry.rs       # ToolRegistry (Vec<Arc<dyn Tool>>)
│   │       ├── execute.rs        # ExecuteEngine（校验 + 超时 + spawn）
│   │       ├── context.rs        # ToolContext
│   │       └── error.rs          # ToolError + ToolResult
│   │
│   ├── tools/                    # 所有工具实现
│   │   ├── src/
│   │   │   ├── lib.rs            # register_all() 注册入口
│   │   │   ├── json_formatter.rs
│   │   │   ├── json_validator.rs
│   │   │   ├── base64_codec.rs
│   │   │   ├── timestamp.rs
│   │   │   ├── uuid_gen.rs
│   │   │   ├── url_codec.rs
│   │   │   ├── text_stats.rs
│   │   │   ├── string_case.rs
│   │   │   └── hash_tool.rs
│   │   └── Cargo.toml
│   │
│   ├── server/                   # axum HTTP + WebSocket 服务器
│   │   └── src/
│   │       ├── lib.rs            # AppState + create_app() + start_server()
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── tools.rs      # GET/POST /api/tools, WS /api/ws
│   │       │   └── config.rs     # GET/PUT /api/config
│   │       └── ws/
│   │           └── handler.rs    # WebSocket 消息处理
│   │
│   ├── cli/                      # CLI 入口（二进制 crate）
│   │   └── src/
│   │       └── main.rs           # clap: serve / exec / list
│   │
│   └── mcp-server/               # [TODO Phase 3] MCP 服务器
│       └── src/
│           └── lib.rs            # 占位
│
├── gui/                          # 前端项目
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── index.html
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx               # 路由 + 布局
│   │   ├── App.css
│   │   ├── styles.css            # 全局样式
│   │   ├── vite-env.d.ts         # Vite 类型声明
│   │   ├── api/
│   │   │   ├── rest.ts           # REST 客户端
│   │   │   └── ws.ts             # WebSocket 客户端
│   │   └── pages/
│   │       ├── ToolList.tsx      # 工具列表页（分类 + 搜索）
│   │       └── ToolExecute.tsx   # 工具执行页（动态表单 + 结果展示）
│   └── src-tauri/                # Tauri 桌面壳
│       ├── tauri.conf.json
│       ├── Cargo.toml
│       ├── build.rs
│       ├── capabilities/
│       │   └── default.json
│       ├── icons/                # 应用图标
│       └── src/
│           └── main.rs           # Tauri 入口：管理 Core 子进程
│
└── scripts/
    └── dev.sh                    # 一键启动开发环境
```

## 5. API 设计

### 5.1 REST 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/tools` | 获取所有工具列表（含元数据 + JSON Schema） |
| `GET` | `/api/tools/:id` | 获取单个工具元数据 |
| `POST` | `/api/tools/:id/execute` | 同步执行工具 |
| `GET` | `/api/config` | 获取配置 |
| `PUT` | `/api/config` | 更新配置 |
| `WS` | `/api/ws` | WebSocket 连接（流式执行 + 推送） |

### 5.2 WebSocket 协议

**客户端 → 服务器：**
```json
{ "type": "execute", "id": "uuid-gen", "input": { "count": 5 } }
{ "type": "ping" }
```

**服务器 → 客户端：**
```json
{ "type": "result", "id": "uuid-gen", "data": { "uuids": [...], "count": 5 } }
{ "type": "error",  "id": "uuid-gen", "error": "...", "code": 400 }
{ "type": "pong" }
```

## 6. 工具系统

### 6.1 添加新工具的流程

1. 在 `crates/tools/src/` 下创建新文件，实现 `Tool` trait
2. 在 `lib.rs` 的 `register_all()` 中添加一行 `.register(YourTool)`
3. 自动获得 CLI 命令、HTTP API、WS 调用三种入口

### 6.2 工具分类

```
ToolCategory::Json       → JSON 相关（格式化、校验、压缩）
ToolCategory::Text       → 文本处理（统计、转换、正则）
ToolCategory::Crypto     → 加密编码（Base64、Hash、JWT）
ToolCategory::DateTime   → 日期时间（时间戳、时区）
ToolCategory::Network    → 网络（URL 编解码）
ToolCategory::Color      → 颜色（HEX/RGB/HSL）
ToolCategory::Image      → 图片处理（B 类）
ToolCategory::Utility    → 通用（UUID 等）
ToolCategory::Productivity → 生产力工具（B 类）
```

## 7. 目前进度

### 已完成 ✅

- [x] Cargo workspace 项目结构搭建
- [x] Core crate：Tool trait + ToolRegistry + ExecuteEngine
- [x] Tools crate：首批 12 个开发者工具
- [x] Server crate：axum HTTP + WebSocket 服务器
- [x] CLI crate：serve / exec / list 命令
- [x] MCP server 占位 crate
- [x] React + Vite + TypeScript 前端（工具列表 + 工具执行页面）
- [x] Tauri 桌面壳配置（tauri.conf.json + Cargo.toml + main.rs）
- [x] REST API 全链路可用
- [x] WebSocket 端点可用
- [x] CLI 直接执行工具
- [x] 开发脚本 `scripts/dev.sh`
- [x] 项目文档 `CLAUDE.md` + `docs/design.md`

---

## 8. TODO —— 总待办列表

### Phase 1 · MVP 完善

#### 工具扩展

- [ ] **新增工具：颜色转换** — HEX / RGB / HSL 互转（`ToolCategory::Color`）
- [ ] **新增工具：JWT 解码** — 解析 JWT token 的 header 和 payload（`ToolCategory::Crypto`）
- [ ] **新增工具：HTML 转义/反转义** — HTML 实体编码与解码（`ToolCategory::Text`）
- [ ] **新增工具：正则测试器** — 输入正则和测试文本，输出匹配结果（`ToolCategory::Text`）
- [ ] **新增工具：文本差异比较** — 两段文本的 diff 对比（`ToolCategory::Text`）
- [ ] **新增工具：JSON 压缩** — JSON 输出为单行（`ToolCategory::Json`）
- [ ] **新增工具：JSON 转 YAML/TOML** — 格式互转（`ToolCategory::Json`）
- [ ] **新增工具：IP 信息查询** — 归属地、运营商（`ToolCategory::Network`）
- [ ] **新增工具：进制转换** — 2/8/10/16 进制互转（`ToolCategory::Utility`）
- [ ] **新增工具：密码生成器** — 可配置长度和字符集（`ToolCategory::Utility`）

#### 工具系统优化

- [ ] **Proc-macro 自动注册**（可选）— 用 `#[tool]` 注解自动收集工具，减少 `register_all()` 手动维护
- [ ] **工具输入校验** — 执行前根据 JSON Schema 做类型/必填校验，返回清晰的错误提示
- [ ] **工具执行历史** — 记录最近执行的工具和输入参数

#### 配置系统

- [ ] **配置文件读写** — 实现 JSON 配置文件的持久化存储
- [ ] **配置路径规范** — 按平台实现标准路径（macOS: `~/Library/Application Support/trove/config.json`，Linux: `~/.config/trove/config.json`，Windows: `%APPDATA%/trove/config.json`）
- [ ] **三层覆盖完整实现** — CLI 参数 > 配置文件 > 默认值，精确合并逻辑
- [ ] **`trove config` 子命令** — `trove config set/get/list` CLI 管理配置

#### 前端完善

- [ ] **PWA 支持** — Service Worker 注册、`manifest.json`、离线缓存核心 UI
- [ ] **主题系统（亮/暗）** — CSS 变量切换，持久化用户偏好到配置
- [ ] **工具搜索优化** — 搜索词高亮、模糊匹配、最近使用优先
- [ ] **工具收藏** — 收藏常用工具到首页快捷入口
- [ ] **响应式布局** — 适配小屏幕和平板
- [ ] **输入/输出历史** — 保留上次输入，可在历史间切换
- [ ] **大 JSON 自动折叠** — 结果超过一定大小自动折叠，点击展开
- [ ] **快捷键** — Cmd+Enter 执行、Cmd+K 搜索、Esc 返回
- [ ] **国际化（i18n）** — 至少中英文切换

### Phase 2 · 桌面体验

- [ ] **Tauri 壳完善** — 托盘图标、全局快捷键、系统通知
- [ ] **Tauri 自动管理 Core 进程** — 检测 Core 是否在运行，自动启动/停止
- [ ] **应用更新机制** — 内建更新检查（GitHub Releases 或其他渠道）
- [ ] **原生菜单栏** — Tauri 原生菜单（文件、编辑、帮助）
- [ ] **文件拖放** — 拖拽文件到工具输入区域
- [ ] **系统代理集成** — 自动检测系统代理设置

### Phase 3 · 高级功能

#### MCP 服务器

> 核心 idea：用一个统一的 MCP 服务器 crate，将 Trove 中的每个工具自动暴露为一个 MCP Tool。这样任何支持 MCP 协议的 LLM 客户端（Claude Desktop、VS Code 等）都能直接调用 Trove 的工具。

- [ ] **实现 `crates/mcp-server`** — 基于 MCP 协议的 JSON-RPC 服务器
- [ ] **自动工具映射** — 将 ToolRegistry 中的每个工具转为 MCP Tool 定义
- [ ] **传输层** — 支持 stdio（子进程通信）和 SSE（HTTP 流）两种传输模式
- [ ] **会话管理** — 处理 LLM 发起的多次工具调用
- [ ] **错误处理** — 将 ToolError 映射为 MCP 错误码
- [ ] **安装脚本** — 一键注册到 Claude Desktop 的 MCP 配置

#### 插件系统

- [ ] **WASM 沙箱** — 用户可编写 WASM 插件作为自定义工具
- [ ] **插件 SDK** — 提供 Rust 端的 SDK crate，简化插件编写
- [ ] **插件市场** — 插件发现和安装（简单列出）
- [ ] **权限管理** — 插件可访问系统资源的权限声明和控制

#### 移动端

- [ ] **Tauri Mobile** — 用 Tauri v2 Mobile 编译 iOS/Android 壳
- [ ] **触控优化** — 工具输入控件适配触屏操作
- [ ] **Core 内嵌** — 移动端 Core 编译为 native library 在 app 进程中运行
- [ ] **离线支持** — 全部工具离线可用（核心工具不需要网络）

#### 云同步

- [ ] **配置同步** — iCloud / WebDAV 同步配置文件
- [ ] **收藏同步** — 工具收藏列表跨设备同步
- [ ] **历史同步** — 工具使用历史跨设备同步

### Phase 4 · 生态与发布

- [ ] **CI/CD** — GitHub Actions 自动化构建、测试、发布
- [ ] **跨平台构建** — macOS（x86 + ARM）、Windows（x86）、Linux（x86 + ARM）的自动化构建
- [ ] **安装包** — macOS `.dmg`、Windows `.msi`、Linux `.deb`/`.AppImage`
- [ ] **Homebrew Tap** — 提供 `brew install trove`
- [ ] **官方网站** — 项目介绍页面
- [ ] **用户文档** — 在线文档站（工具使用指南、开发指南）

---

## 附录：设计决策详解

### 为什么选 Rust 而不是 Go / Node.js / C++？

| 维度 | Rust | Go | Node.js/Bun | C++ |
|------|------|-----|-------------|-----|
| 启动速度 | ~5ms | ~10ms | ~100ms | ~5ms |
| 内存 (idle) | ~5MB | ~10MB | ~30-50MB | ~5MB |
| 二进制大小 | ~5MB | ~10MB | ~50MB+ | ~5MB |
| 跨平台编译 | ✅ 极佳 | ✅ 极佳 | ❌ 需要运行时 | ❌ 痛苦 |
| 安全（内存） | ✅ 所有权模型 | ✅ GC | ✅ GC | ❌ 手动管理 |
| Web生态 | ⚠️ 有但少 | ⚠️ 一般 | ✅ 最丰富 | ❌ 少 |
| WASM | ✅ 原生支持 | ⚠️ 一般 | ❌ 不支持 | ⚠️ Emscripten |

Rust 在性能、内存、跨平台三方面同时做到极致，和你明确要求的"极致性能和内存、启动速度极快"完全吻合。

### 为什么选 REST + WebSocket 混合，而不是纯 REST 或 gRPC？

- **REST 适合控制面**：工具列表查询、配置管理——天然请求-响应模式，curl 即可调试，DevTools 可直接查看
- **WS 适合数据面**：工具执行可能是长时间的计算或流式输出，WS 的全双工特性让 Core 可以持续推送输出片段
- MCP 协议本身基于 JSON-RPC + 流式通信，和 WS 天然对位，后续 MCP 服务器实现时协议转换成本最低
- 对比 gRPC：需要 protobuf 编译 + gRPC-Web proxy，复杂度远高于 REST + WS，但对你的场景（本地工具执行）没有额外收益

### 为什么工具注册用 Vec 而不是 HashMap 或 phf？

- 工具总数预计 50-150 个
- Vec 线性扫描 150 个元素 ≈ 1-3μs，一次 axum 路由匹配 ≈ 20-50μs
- 工具查找不占主导，HashMap 的 SipHash 对短字符串可能反而比线性扫描慢
- 如果工具数超过 200 且分析确认为热点：第一优化用 `phf::Map`（编译期完美哈希），第二优化用 enum dispatch（编译为 jump table）
