

## v1.1.0 (2026-07-27)

### 新增

- 🎬 **video-concat 视频拼接工具**：基于 ffmpeg 的高性能视频拼接，支持流拷贝（同格式）和重编码（跨格式）两种策略，实时进度推送与取消
- 📄 新增 video-concat 设计文档和进度/取消 ADR 记录
- 🛠️ 改进 CLAUDE.md 文档结构

# Changelog

## v1.0.0 (2026-07-26)

### 初始发布

**核心能力：**

- 🦀 Rust 后端：Tool trait 系统 + 执行引擎 + 三重安全防线
- 🔧 12 个内置工具：JSON 格式化/校验、Base64 编解码、时间戳转换、UUID 生成、Hash 计算、URL 编解码、文本统计、字符串格式转换
- 🎨 React 前端：自动表单生成、定制化结果展示
- 🖥️ Tauri 桌面壳：macOS .dmg 安装包
- ⌨️ CLI：serve / exec / list 子命令

**架构特性：**

- REST + WebSocket 混合通信协议
- Tauri sidecar 架构（Core 二进制自动捆绑）
- CLI 和桌面应用共用同一份二进制
- CPU 密集型工具自动隔离执行
- JSON Schema 驱动的前端表单生成
