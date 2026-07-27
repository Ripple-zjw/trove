use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::ToolContext;
use crate::error::ToolResult;

/// 工具分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCategory {
    /// JSON 相关（格式化、压缩、校验等）
    Json,
    /// 文本/字符串处理（编解码、转义、正则等）
    Text,
    /// 加密/编码（Base64、Hash、JWT 等）
    Crypto,
    /// 日期/时间（时间戳转换、时区等）
    DateTime,
    /// 网络/URL（URL 编解码、HTTP 工具等）
    Network,
    /// 颜色（HEX/RGB/HSL 转换等）
    Color,
    /// 图片（后期 B 类工具）
    Image,
    /// 媒体（音视频处理工具）
    Media,
    /// 其他/通用
    Utility,
    /// 生产力工具（后期 B 类工具）
    Productivity,
}

impl ToolCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ToolCategory::Json => "JSON",
            ToolCategory::Text => "文本",
            ToolCategory::Crypto => "加密/编码",
            ToolCategory::DateTime => "日期/时间",
            ToolCategory::Network => "网络",
            ToolCategory::Color => "颜色",
            ToolCategory::Image => "图片",
            ToolCategory::Media => "媒体",
            ToolCategory::Utility => "通用",
            ToolCategory::Productivity => "生产力",
        }
    }

    pub fn order(&self) -> u32 {
        match self {
            ToolCategory::Json => 1,
            ToolCategory::Text => 2,
            ToolCategory::Crypto => 3,
            ToolCategory::DateTime => 4,
            ToolCategory::Network => 5,
            ToolCategory::Color => 6,
            ToolCategory::Image => 7,
            ToolCategory::Media => 8,
            ToolCategory::Utility => 9,
            ToolCategory::Productivity => 10,
        }
    }
}

/// 工具元数据（通过 REST API 暴露给前端）
#[derive(Debug, Clone, Serialize)]
pub struct ToolMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: ToolCategory,
    pub input_schema: Value,
    pub is_cpu_intensive: bool,
}

/// Tool trait — 所有工具的核心抽象
///
/// 每个工具只需要实现这个 trait，就会自动获得：
/// - CLI 命令（通过 clap 子命令）
/// - HTTP API 端点（通过 axum 路由）
/// - WebSocket 流式执行
/// - 未来 MCP 工具暴露
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具唯一标识（snake_case，用于 URL/CLI 路由）
    fn id(&self) -> &'static str;

    /// 人类可读的工具名称
    fn name(&self) -> &'static str;

    /// 工具描述
    fn description(&self) -> &'static str;

    /// 输入参数的 JSON Schema（用于前端动态生成表单）
    fn input_schema(&self) -> Value;

    /// 工具分类（用于前端分组展示）
    fn category(&self) -> ToolCategory;

    /// 是否是 CPU 密集型工具（默认 false）
    /// CPU 密集型工具会被 spawn_blocking 到阻塞线程池执行
    fn is_cpu_intensive(&self) -> bool {
        false
    }

    /// 获取工具元数据（自动实现）
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id(),
            name: self.name(),
            description: self.description(),
            category: self.category(),
            input_schema: self.input_schema(),
            is_cpu_intensive: self.is_cpu_intensive(),
        }
    }

    /// 执行工具
    ///
    /// 对于 CPU 密集型的工具，框架会自动使用 spawn_blocking
    /// 子类通常只需要实现这个方法
    async fn execute(&self, input: Value, ctx: ToolContext) -> ToolResult<Value>;
}
