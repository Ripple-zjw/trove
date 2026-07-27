use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;
use tokio::sync::mpsc;

/// 进度事件（用于长耗时工具向客户端推送进度）
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    /// 完成百分比（0.0 ~ 1.0）
    pub percent: f64,
    /// ffmpeg 的 time= 值，如 "00:01:23.45"
    pub time: String,
    /// 当前处理的帧数
    pub frame: u64,
    /// 编码速度，如 "1.5x"
    pub speed: String,
}

/// 取消令牌，用于从外部取消正在执行的工具
#[derive(Debug, Clone)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }

    /// 触发取消
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// 检查是否已被取消
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// 工具执行上下文
///
/// 包含工具执行时的环境信息，在 execute 方法中传递。
/// 可以根据需要扩展——比如后续加入日志收集器、进度回调等。
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// 附加参数（从 CLI/HTTP/WS 传入的额外上下文）
    pub extras: Option<Value>,

    /// 超时时间（秒），设为 0 表示不超时
    pub timeout_secs: u64,

    /// 进度推送通道（可选）
    /// 设置后，长耗时工具可通过此通道推送进度事件
    pub progress_tx: Option<mpsc::UnboundedSender<ProgressEvent>>,

    /// 取消令牌（可选）
    /// 设置后，外部可通过调用 cancel() 来取消当前工具的执行
    pub cancel_token: Option<CancelToken>,
}

impl ToolContext {
    pub fn new() -> Self {
        Self {
            extras: None,
            timeout_secs: 30,
            progress_tx: None,
            cancel_token: None,
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

    /// 设置进度推送通道
    pub fn with_progress(mut self, tx: mpsc::UnboundedSender<ProgressEvent>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    /// 设置取消令牌
    pub fn with_cancel_token(mut self, token: CancelToken) -> Self {
        self.cancel_token = Some(token);
        self
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new()
    }
}
