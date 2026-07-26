use std::sync::Arc;

use clap::{Parser, Subcommand};
use trove_core::{ExecuteEngine, ToolRegistry};
use trove_server::start_server;

#[derive(Parser)]
#[command(
    name = "trove",
    version,
    about = "🛠️ Trove — 极致性能的跨平台工具集合",
    long_about = "Trove 是一款高性能跨平台工具集合软件。\n\
                  启动服务后，可通过浏览器访问 GUI 使用各种开发者工具。\n\
                  也可直接通过 CLI 执行工具。"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 Trove 服务（HTTP + WebSocket）
    Serve {
        /// 监听端口
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// 直接执行工具（不启动服务）
    Exec {
        /// 工具 ID
        tool_id: String,

        /// 输入 JSON 字符串
        #[arg(short, long)]
        input: Option<String>,

        /// 从文件读取输入
        #[arg(short = 'f', long)]
        file: Option<String>,
    },
    /// 列出所有可用工具
    List,
}

/// 初始化注册表和执行引擎
fn init_engine() -> ExecuteEngine {
    let mut registry = ToolRegistry::new();
    trove_tools::register_all(&mut registry);

    let registry = Arc::new(registry);
    ExecuteEngine::new(registry)
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port } => {
            tracing::info!("⏳ Trove 正在启动...");
            let engine = init_engine();
            tracing::info!("✅ 已加载 {} 个工具", engine.registry().len());

            if let Err(e) = start_server(Arc::new(engine), port).await {
                tracing::error!("❌ 服务启动失败: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Exec {
            tool_id,
            input,
            file,
        } => {
            let engine = init_engine();

            let input_value = if let Some(file_path) = &file {
                let content = std::fs::read_to_string(file_path)
                    .unwrap_or_else(|e| {
                        eprintln!("❌ 读取文件失败: {}", e);
                        std::process::exit(1);
                    });
                serde_json::from_str(&content).unwrap_or_else(|_| {
                    serde_json::json!({ "input": content })
                })
            } else if let Some(json_str) = &input {
                serde_json::from_str(json_str).unwrap_or_else(|_| {
                    serde_json::json!({ "input": json_str })
                })
            } else {
                // 从 stdin 读取
                let mut buffer = String::new();
                use std::io::Read;
                std::io::stdin().read_to_string(&mut buffer).unwrap_or_default();
                let trimmed = buffer.trim().to_string();
                serde_json::from_str(&trimmed).unwrap_or_else(|_| {
                    serde_json::json!({ "input": trimmed })
                })
            };

            match engine.execute(&tool_id, input_value).await {
                Ok(result) => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap());
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::List => {
            let engine = init_engine();
            let tools = engine.registry().list_metadata();

            if tools.is_empty() {
                println!("暂无可用工具");
                return;
            }

            println!("📦 可用工具 ({})", tools.len());
            println!("{}", "-".repeat(50));

            let mut current_category: Option<&str> = None;
            for tool in &tools {
                let cat_label = tool.category.label();
                if current_category != Some(cat_label) {
                    println!("\n  [{}]", cat_label);
                    current_category = Some(cat_label);
                }
                println!("    {:<20} {}", tool.id, tool.name);
                println!("    {:>20}{}", "", tool.description);
            }
        }
    }
}
