pub mod routes;
pub mod ws;

use std::sync::Arc;

use axum::Router;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use trove_core::ExecuteEngine;
use routes::config::{AppConfig, SharedConfig};

/// 共享应用状态
pub struct AppState {
    pub engine: Arc<ExecuteEngine>,
    pub config: SharedConfig,
}

/// 创建完整的应用 Router
pub fn create_app(engine: Arc<ExecuteEngine>, config: SharedConfig) -> Router {
    let state = Arc::new(AppState { engine, config });
    let cors = CorsLayer::permissive();

    // 合并 tools 和 config 路由到一个 Router，再嵌套到 /api 下
    let api_routes = Router::new()
        .merge(routes::tools::routes())
        .merge(routes::config::routes());

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .with_state(state)
}

/// 启动服务器，返回绑定地址
pub async fn start_server(
    engine: Arc<ExecuteEngine>,
    port: u16,
) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 Trove 服务已启动: http://{}", addr);

    let config = Arc::new(RwLock::new(AppConfig {
        port,
        ..Default::default()
    }));

    let app = create_app(engine, config);
    axum::serve(listener, app).await
}
