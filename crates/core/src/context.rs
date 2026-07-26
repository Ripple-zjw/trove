use serde_json::Value;

/// 工具执行上下文
///
/// 包含工具执行时的环境信息，在 execute 方法中传递。
/// 可以根据需要扩展——比如后续加入日志收集器、进度回调等。
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// 附加参数（从 CLI/HTTP/WS 传入的额外上下文）
    pub extras: Option<Value>,

    /// 超时时间（秒）
    pub timeout_secs: u64,
}

impl ToolContext {
    pub fn new() -> Self {
        Self {
            extras: None,
            timeout_secs: 30,
        }
    }

    pub fn with_extras(mut self, extras: Value) -> Self {
        self.extras = Some(extras);
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new()
    }
}
