pub mod routes;
pub mod ws;

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    response::Html,
    routing::any,
    Router,
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

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

    let api_routes = Router::new()
        .merge(routes::tools::routes())
        .merge(routes::config::routes());

    let mut router = Router::new()
        .nest("/api", api_routes);

    // 将前端构建产物作为静态文件提供
    if let Some(dir) = find_static_dir() {
        tracing::info!("📁 提供前端页面: {}", dir.display());
        let assets_dir = dir.join("assets");
        let index_html = std::sync::Arc::new(
            std::fs::read_to_string(dir.join("index.html"))
                .unwrap_or_default()
        );

        // 路由优先级：
        // 1. /api/* → REST API（优先匹配）
        // 2. /assets/* → JS/CSS 构建产物
        // 3. /* → index.html（SPA 客户端路由 catch-all）
        router = router
            .nest_service("/assets", ServeDir::new(&assets_dir))
            .fallback_service(any(move || {
                let html = index_html.clone();
                async move { Html(html.to_string()) }
            }));
    }

    router.layer(cors).with_state(state)
}

/// 启动服务器
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

/// 从前端构建目录提供静态文件（SPA）
fn find_static_dir() -> Option<PathBuf> {
    let candidates = [
        "gui/dist",
        "../gui/dist",
        "dist",
        "../Resources/dist",
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.join("index.html").exists() {
            return Some(p);
        }
    }
    None
}
