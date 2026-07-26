use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    /// 工具不存在
    #[error("工具 '{0}' 未找到")]
    NotFound(String),

    /// 输入参数不合法（包含具体的校验错误信息）
    #[error("输入参数错误: {0}")]
    InvalidInput(String),

    /// 工具执行超时
    #[error("工具执行超时 (限制: {0}s)")]
    Timeout(u64),

    /// 输入超出大小限制（单位: 字节）
    #[error("输入大小 ({0} bytes) 超过限制 ({1} bytes)")]
    InputTooLarge(usize, usize),

    /// 工具执行出错
    #[error("执行错误: {0}")]
    ExecutionError(String),

    /// 内部错误（如线程 panicked）
    #[error("内部错误: {0}")]
    InternalError(String),

    /// 序列化/反序列化错误
    #[error("JSON 错误: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type ToolResult<T> = Result<T, ToolError>;

impl ToolError {
    /// 返回 HTTP 状态码对应的数字
    pub fn status_code(&self) -> u16 {
        match self {
            ToolError::NotFound(_) => 404,
            ToolError::InvalidInput(_) => 400,
            ToolError::Timeout(_) => 408,
            ToolError::InputTooLarge(_, _) => 413,
            ToolError::ExecutionError(_) => 500,
            ToolError::InternalError(_) => 500,
            ToolError::JsonError(_) => 400,
        }
    }
}
