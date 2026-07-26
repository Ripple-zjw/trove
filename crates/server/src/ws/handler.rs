use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use trove_core::ExecuteEngine;

/// WebSocket 消息（客户端 → 服务器）
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsRequest {
    #[serde(rename = "execute")]
    Execute {
        id: String,
        input: Value,
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
    #[serde(rename = "pong")]
    Pong,
}

/// 处理 WebSocket 连接
pub async fn handle_socket(socket: WebSocket, engine: Arc<ExecuteEngine>) {
    let (mut sender, mut receiver) = socket.split();

    // 为这个连接创建一个工具执行 channel
    let (result_tx, mut result_rx) = mpsc::channel::<WsResponse>(32);

    // 一个来自 receiver 的任务：接收 WS 消息 → 执行工具 → 通过 channel 发回结果
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsRequest>(&text) {
                        Ok(WsRequest::Execute { id, input }) => {
                            let engine = engine.clone();
                            let result_tx = result_tx.clone();
                            tokio::spawn(async move {
                                let result = engine.execute(&id, input).await;
                                let response = match result {
                                    Ok(data) => WsResponse::Result { id: id.clone(), data },
                                    Err(e) => WsResponse::Error {
                                        id: id.clone(),
                                        error: e.to_string(),
                                        code: e.status_code(),
                                    },
                                };
                                let _ = result_tx.send(response).await;
                            });
                        }
                        Ok(WsRequest::Ping) => {
                            let _ = result_tx.send(WsResponse::Pong).await;
                        }
                        Err(e) => {
                            let _ = result_tx
                                .send(WsResponse::Error {
                                    id: "parse".to_string(),
                                    error: format!("消息解析失败: {}", e),
                                    code: 400,
                                })
                                .await;
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    // 一个 sender 的任务：从 channel 取结果 → 发送给客户端
    while let Some(response) = result_rx.recv().await {
        let text = serde_json::to_string(&response).unwrap_or_default();
        if sender.send(Message::Text(text)).await.is_err() {
            break;
        }
    }

    // 等待接收任务结束
    recv_task.abort();
}
