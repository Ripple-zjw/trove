use std::sync::Arc;

use serde_json::Value;
use tokio::time::{timeout, Duration};

use crate::context::ToolContext;
use crate::error::{ToolError, ToolResult};
use crate::registry::ToolRegistry;

/// 输入大小限制（默认 10MB）
const DEFAULT_MAX_INPUT_SIZE: usize = 10 * 1024 * 1024;

/// 执行引擎
///
/// 负责在执行工具时提供安全措施：
/// 1. 输入大小校验
/// 2. 超时控制
/// 3. CPU 密集型任务通过 tokio::spawn 隔离
pub struct ExecuteEngine {
    registry: Arc<ToolRegistry>,
    max_input_size: usize,
}

impl ExecuteEngine {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            max_input_size: DEFAULT_MAX_INPUT_SIZE,
        }
    }

    /// 设置最大输入大小（字节）
    pub fn with_max_input_size(mut self, size: usize) -> Self {
        self.max_input_size = size;
        self
    }

    /// 同步执行工具（通过 REST API 调用）
    pub async fn execute(&self, tool_id: &str, input: Value) -> ToolResult<Value> {
        let tool = self.registry.get_arc(tool_id)?;
        let ctx = ToolContext::default();
        self.execute_tool(tool, input, ctx).await
    }

    /// 带上下文执行
    pub async fn execute_with_ctx(
        &self,
        tool_id: &str,
        input: Value,
        ctx: ToolContext,
    ) -> ToolResult<Value> {
        let tool = self.registry.get_arc(tool_id)?;
        self.execute_tool(tool, input, ctx).await
    }

    async fn execute_tool(
        &self,
        tool: Arc<dyn crate::tool::Tool>,
        input: Value,
        ctx: ToolContext,
    ) -> ToolResult<Value> {
        // 1. 输入大小校验
        let input_size = serde_json::to_string(&input)
            .map(|s| s.len())
            .unwrap_or(0);
        if input_size > self.max_input_size {
            return Err(ToolError::InputTooLarge(input_size, self.max_input_size));
        }

        let timeout_secs = ctx.timeout_secs;

        if tool.is_cpu_intensive() {
            // CPU 密集型工具：在新 tokio task 中执行，避免阻塞其他请求
            let task = tokio::task::spawn(async move {
                tool.execute(input, ctx).await
            });

            if timeout_secs == 0 {
                // 不超时（用于视频处理等长耗时工具）
                match task.await {
                    Ok(result) => result,
                    Err(join_err) => {
                        Err(ToolError::InternalError(format!("任务 panicked: {}", join_err)))
                    }
                }
            } else {
                match timeout(Duration::from_secs(timeout_secs), task).await {
                    Ok(Ok(result)) => result,
                    Ok(Err(join_err)) => {
                        Err(ToolError::InternalError(format!("任务 panicked: {}", join_err)))
                    }
                    Err(_elapsed) => Err(ToolError::Timeout(timeout_secs)),
                }
            }
        } else {
            // 轻量工具：直接在 async context 中运行
            let fut = tool.execute(input, ctx);
            if timeout_secs == 0 {
                fut.await
            } else {
                match timeout(Duration::from_secs(timeout_secs), fut).await {
                    Ok(result) => result,
                    Err(_elapsed) => Err(ToolError::Timeout(timeout_secs)),
                }
            }
        }
    }

    /// 注册表引用（供 server crate 使用）
    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.registry
    }
}
