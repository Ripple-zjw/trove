use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State, WebSocketUpgrade, Multipart, DefaultBodyLimit, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

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

#[derive(Serialize)]
pub struct UploadResponse {
    pub path: String,
    pub name: String,
    pub size: u64,
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
        Err(e) => {
            warn!("工具未找到: id={}", id);
            Err(error_response(e))
        }
    }
}

/// 同步执行工具
async fn execute_tool(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteRequest>,
) -> impl IntoResponse {
    info!("执行工具: id={}", id);
    match state.engine.execute(&id, req.input).await {
        Ok(result) => Ok(Json(ExecuteResponse { result })),
        Err(e) => Err(error_response(e)),
    }
}

/// 检测 ffmpeg 信息（用于视频相关工具的前置检查）
async fn video_concat_deps() -> Json<Value> {
    let info = trove_tools::ffmpeg_detector::detect_ffmpeg();
    Json(serde_json::to_value(info).unwrap_or_default())
}

/// 上传文件到服务器临时目录（用于视频拼接等工具）
async fn upload_file(
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    use tokio::io::AsyncWriteExt;

    let debug_id: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64 % 100000)
        .unwrap_or(0);
    tracing::info!("[upload#{}] 收到上传请求", debug_id);

    let field = multipart
        .next_field()
        .await
        .map_err(|e| {
            tracing::error!("[upload#{}] 读取 multipart 失败: {}", debug_id, e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("读取上传失败: {}", e),
                    code: 400,
                }),
            )
        })?
        .ok_or_else(|| {
            tracing::error!("[upload#{}] multipart 中没有文件字段", debug_id);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "没有上传文件".to_string(),
                    code: 400,
                }),
            )
        })?;

    let file_name = field
        .file_name()
        .unwrap_or("uploaded_file")
        .to_string();
    let content_type = field.content_type().map(|s| s.to_string()).unwrap_or_default();
    tracing::info!(
        "[upload#{}] 收到文件: name={}, content_type={}, name_in_form={}",
        debug_id,
        file_name,
        content_type,
        field.name().unwrap_or("?")
    );

    let data = field
        .bytes()
        .await
        .map_err(|e| {
            tracing::error!("[upload#{}] 读取文件数据失败: {}", debug_id, e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("读取文件数据失败: {}", e),
                    code: 400,
                }),
            )
        })?;

    tracing::info!("[upload#{}] 文件大小: {} bytes", debug_id, data.len());

    // 保存到系统临时目录
    let tmp_dir = std::env::temp_dir().join("trove_uploads");
    tokio::fs::create_dir_all(&tmp_dir).await.map_err(|e| {
        tracing::error!("[upload#{}] 创建临时目录失败: {}", debug_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("创建临时目录失败: {}", e),
                code: 500,
            }),
        )
    })?;

    // 用时间戳+原始文件名避免重名
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let safe_name = format!("{}_{}", ts, sanitize_filename(&file_name));
    let save_path = tmp_dir.join(&safe_name);

    let mut file = tokio::fs::File::create(&save_path).await.map_err(|e| {
        tracing::error!("[upload#{}] 创建文件失败: {} -> {}", debug_id, save_path.display(), e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("创建文件失败: {}", e),
                code: 500,
            }),
        )
    })?;
    file.write_all(&data).await.map_err(|e| {
        tracing::error!("[upload#{}] 写入文件失败: {}", debug_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("写入文件失败: {}", e),
                code: 500,
            }),
        )
    })?;

    tracing::info!(
        "[upload#{}] ✅ 上传成功: {} -> {} ({} bytes)",
        debug_id,
        file_name,
        save_path.display(),
        data.len()
    );

    Ok(Json(UploadResponse {
        path: save_path.to_string_lossy().to_string(),
        name: file_name,
        size: data.len() as u64,
    }))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[derive(Deserialize)]
struct DownloadQuery {
    path: String,
}

/// 下载拼接完成的文件（流式传输）
async fn download_file(Query(q): Query<DownloadQuery>) -> Response<Body> {
    use tokio_util::io::ReaderStream;

    let path = std::path::PathBuf::from(&q.path);
    let tmp_uploads = std::env::temp_dir().join("trove_uploads");

    let error_res = |status: StatusCode, msg: &str| -> Response<Body> {
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::json!({ "error": msg, "code": status.as_u16() }).to_string()))
            .unwrap()
    };

    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return error_res(StatusCode::NOT_FOUND, "文件不存在"),
    };

    // macOS /tmp → /private/tmp，所以要 canonicalize 两边
    let canonical_tmp = tmp_uploads.canonicalize().unwrap_or_else(|_| tmp_uploads.clone());
    if !canonical.starts_with(&canonical_tmp) {
        return error_res(StatusCode::FORBIDDEN, "禁止的路径");
    }

    let content_type = match path.extension().and_then(|e| e.to_str()) {
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    };
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("output.mp4");

    match tokio::fs::File::open(&canonical).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_name))
                .body(Body::from_stream(stream))
                .unwrap()
        }
        Err(_) => error_res(StatusCode::NOT_FOUND, "无法打开文件"),
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
        .route("/tools/video-concat/deps", get(video_concat_deps))
        .route("/tools/:id", get(get_tool))
        .route("/tools/:id/execute", post(execute_tool))
        .route("/upload", post(upload_file))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 * 1024))
        .route("/download", get(download_file))
        .route("/ws", get(ws_handler))
}
