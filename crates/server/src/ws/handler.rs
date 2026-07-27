use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use trove_core::ExecuteEngine;
use trove_core::CancelToken;

/// WebSocket 消息（客户端 → 服务器）
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsRequest {
    #[serde(rename = "execute")]
    Execute {
        id: String,
        input: Value,
    },
    #[serde(rename = "cancel")]
    Cancel {
        id: String,
    },
    #[serde(rename = "ping")]
    Ping,
}

/// WebSocket 消息（服务器 → 客户端）
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum WsResponse {
    #[serde(rename = "result")]
    Result {
        id: String,
        data: Value,
    },
    #[serde(rename = "error")]
    Error {
        id: String,
        error: String,
        code: u16,
    },
    #[serde(rename = "progress")]
    Progress {
        id: String,
        percent: f64,
        time: String,
        frame: u64,
        speed: String,
    },
    #[serde(rename = "pong")]
    Pong,
}

/// 处理 WebSocket 连接
///
/// 每个 WS 连接独立管理其执行任务的取消和进度推送。
pub async fn handle_socket(socket: WebSocket, engine: Arc<ExecuteEngine>) {
    let (mut sender, mut receiver) = socket.split();

    // 使用 unbounded channel 避免进度事件导致背压
    let (response_tx, mut response_rx) = mpsc::unbounded_channel::<WsResponse>();

    // 该连接内所有执行任务的取消标志（keyed by tool id）
    let cancel_flags: Arc<tokio::sync::Mutex<HashMap<String, CancelToken>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    // 接收任务：解析 WS 消息 → 执行工具 → 通过 channel 发回结果/进度
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsRequest>(&text) {
                        Ok(WsRequest::Execute { id, input }) => {
                            let engine = engine.clone();
                            let response_tx = response_tx.clone();
                            let flags = cancel_flags.clone();

                            tokio::spawn(async move {
                                // 为此执行创建取消令牌
                                let cancel_token = CancelToken::new();
                                {
                                    let mut map = flags.lock().await;
                                    map.insert(id.clone(), cancel_token.clone());
                                }

                                // 创建带进度和取消支持的 ToolContext
                                let (progress_tx, mut progress_rx) =
                                    mpsc::unbounded_channel::<trove_core::ProgressEvent>();

                                // 转发进度事件到 WS
                                let ws_tx = response_tx.clone();
                                let exec_id = id.clone();
                                tokio::spawn(async move {
                                    while let Some(evt) = progress_rx.recv().await {
                                        let msg = WsResponse::Progress {
                                            id: exec_id.clone(),
                                            percent: evt.percent,
                                            time: evt.time,
                                            frame: evt.frame,
                                            speed: evt.speed,
                                        };
                                        if ws_tx.send(msg).is_err() {
                                            break;
                                        }
                                    }
                                });

                                let ctx = trove_core::ToolContext::default()
                                    .with_timeout(0) // 不超时
                                    .with_progress(progress_tx)
                                    .with_cancel_token(cancel_token.clone());

                                let result = engine.execute_with_ctx(&id, input, ctx).await;

                                // 移除取消标志
                                {
                                    let mut map = flags.lock().await;
                                    map.remove(&id);
                                }

                                let response = match result {
                                    Ok(data) => WsResponse::Result { id: id.clone(), data },
                                    Err(e) => WsResponse::Error {
                                        id: id.clone(),
                                        error: e.to_string(),
                                        code: e.status_code(),
                                    },
                                };
                                let _ = response_tx.send(response);
                            });
                        }
                        Ok(WsRequest::Cancel { id }) => {
                            // 查找并取消对应的执行任务
                            let flags = cancel_flags.clone();
                            tokio::spawn(async move {
                                let map = flags.lock().await;
                                if let Some(token) = map.get(&id) {
                                    token.cancel();
                                }
                            });
                        }
                        Ok(WsRequest::Ping) => {
                            let _ = response_tx.send(WsResponse::Pong);
                        }
                        Err(e) => {
                            let _ = response_tx.send(WsResponse::Error {
                                id: "parse".to_string(),
                                error: format!("消息解析失败: {}", e),
                                code: 400,
                            });
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    // 发送循环：从 channel 取结果/进度 → 发送给 WS 客户端
    while let Some(response) = response_rx.recv().await {
        let text = serde_json::to_string(&response).unwrap_or_default();
        if sender.send(Message::Text(text)).await.is_err() {
            break;
        }
    }

    recv_task.abort();
}
