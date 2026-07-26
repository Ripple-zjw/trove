use std::sync::Arc;

use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub theme: String,
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            theme: "light".to_string(),
            language: "zh-CN".to_string(),
        }
    }
}

/// 全局应用配置
pub type SharedConfig = Arc<RwLock<AppConfig>>;

/// 创建默认共享配置
pub fn default_shared_config() -> SharedConfig {
    Arc::new(RwLock::new(AppConfig::default()))
}

async fn get_config(State(state): State<Arc<AppState>>) -> Json<AppConfig> {
    let cfg = state.config.read().await;
    Json(cfg.clone())
}

async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<AppConfig>,
) -> Json<AppConfig> {
    let mut cfg = state.config.write().await;
    *cfg = new_config.clone();
    Json(cfg.clone())
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/config", get(get_config).put(update_config))
}
