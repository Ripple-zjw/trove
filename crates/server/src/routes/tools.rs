use std::sync::Arc;

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use trove_core::ToolError;

use crate::AppState;

#[derive(Serialize)]
pub struct ToolListResponse {
    pub tools: Vec<trove_core::ToolMetadata>,
    pub total: usize,
}

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub input: Value,
}

#[derive(Serialize)]
pub struct ExecuteResponse {
    pub result: Value,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

fn error_response(err: ToolError) -> (StatusCode, Json<ErrorResponse>) {
    let code = err.status_code();
    (
        StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        Json(ErrorResponse {
            error: err.to_string(),
            code,
        }),
    )
}

/// 获取所有工具列表
async fn list_tools(
    State(state): State<Arc<AppState>>,
) -> Json<ToolListResponse> {
    let tools = state.engine.registry().list_metadata();
    let total = tools.len();
    Json(ToolListResponse { tools, total })
}

/// 获取单个工具元数据
async fn get_tool(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.engine.registry().get(&id) {
        Ok(tool) => Ok::<_, (StatusCode, Json<ErrorResponse>)>(Json(tool.metadata())),
        Err(e) => Err(error_response(e)),
    }
}

/// 同步执行工具
async fn execute_tool(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteRequest>,
) -> impl IntoResponse {
    match state.engine.execute(&id, req.input).await {
        Ok(result) => Ok(Json(ExecuteResponse { result })),
        Err(e) => Err(error_response(e)),
    }
}

/// WebSocket 端点 - 升级连接
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| crate::ws::handle_socket(socket, state.engine.clone()))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tools", get(list_tools))
        .route("/tools/:id", get(get_tool))
        .route("/tools/:id/execute", post(execute_tool))
        .route("/ws", get(ws_handler))
}
