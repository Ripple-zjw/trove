use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use trove_core::{ExecuteEngine, ToolRegistry};
use trove_server::start_server;

// ANSI 颜色常量
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

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

fn init_engine() -> ExecuteEngine {
    let mut registry = ToolRegistry::new();
    trove_tools::register_all(&mut registry);
    ExecuteEngine::new(Arc::new(registry))
}

/// 读取工具输出中的一个值，如果缺失则返回默认字符串
fn get_str(value: &Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or(default.to_string())
}

fn get_u64(value: &Value, key: &str, default: u64) -> u64 {
    value.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

/// 打印一行带标记的信息
fn info_line(label: &str, value: impl std::fmt::Display) {
    println!("  {}▪ {} {} {}", DIM, label, RESET, value);
}

/// 智能格式化工具输出
fn format_output(tool_id: &str, result: &Value) {
    match tool_id {
        "json-format" => {
            let text = get_str(result, "result", "");
            println!("\n  \x1b[1m\x1b[36m{}\x1b[0m\n", text);
        }

        "json-validate" => {
            if result.get("valid").and_then(|v| v.as_bool()).unwrap_or(false) {
                let t = get_str(result, "type", "unknown");
                println!("\n  \x1b[32m✔  JSON 校验通过\x1b[0m\n");
                info_line("类型", &t);
                info_line("深度", get_u64(result, "depth", 0));
                info_line("大小", format!("{} bytes", get_u64(result, "size", 0)));
            } else {
                let err = get_str(result, "error", "未知错误");
                println!("\n  \x1b[31m✘  JSON 校验失败\x1b[0m\n");
                println!("  \x1b[31m错误:\x1b[0m {}", err);
                info_line("行", get_u64(result, "line", 0));
                info_line("列", get_u64(result, "column", 0));
            }
        }

        "string-case" => {
            let cases = [
                ("lowercase", "全小写"),
                ("uppercase", "全大写"),
                ("capitalize", "首字母大写"),
                ("camelCase", "驼峰命名 camelCase"),
                ("snake_case", "蛇形命名 snake_case"),
                ("PascalCase", "帕斯卡命名 PascalCase"),
                ("kebab-case", "短横线命名 kebab-case"),
                ("CONSTANT_CASE", "常量命名 CONSTANT_CASE"),
            ];
            let to_case = get_str(result, "to_case", "");
            let label = cases.iter().find(|(k, _)| *k == to_case).map(|(_, v)| *v).unwrap_or(&to_case);
            let original = get_str(result, "original", "");
            let converted = get_str(result, "result", "");
            println!("\n  \x1b[1m🔤 字符串格式转换 → \x1b[36m{}\x1b[0m\n", label);
            println!("  {}原文本:{} {}", DIM, RESET, original);
            println!("  {}转换后:{} {}", CYAN, RESET, converted);
        }

        "base64-encode" => {
            let text = get_str(result, "result", "");
            println!("\n  {}🔐 Base64 编码结果{}\n", BOLD, RESET);
            println!("  {}{}{}", CYAN, text, RESET);
        }

        "base64-decode" => {
            if let Some(hex) = result.get("result_hex").and_then(|v| v.as_str()) {
                println!("\n  {}🔓 Base64 解码（十六进制）{}\n", BOLD, RESET);
                println!("  {}", hex);
            } else {
                let text = get_str(result, "result", "");
                println!("\n  {}🔓 Base64 解码结果{}\n", BOLD, RESET);
                println!("  {}{}{}", CYAN, text, RESET);
            }
        }

        "ts-to-date" => {
            let date = get_str(result, "result", "");
            let iso = get_str(result, "iso_8601", "");
            println!("\n  {}📅 时间戳 → 日期{}\n", BOLD, RESET);
            println!("  {}日期时间:{} {}", GREEN, RESET, date);
            info_line("ISO 8601", &iso);
            info_line("秒级时间戳", get_u64(result, "timestamp_secs", 0));
        }

        "date-to-ts" => {
            println!("\n  {}📅 日期 → 时间戳{}\n", BOLD, RESET);
            info_line("秒级时间戳", get_u64(result, "timestamp_secs", 0));
            info_line("毫秒时间戳", get_u64(result, "timestamp_ms", 0));
        }

        "url-encode" | "url-decode" => {
            let text = get_str(result, "result", "");
            let action = if tool_id == "url-encode" { "编码" } else { "解码" };
            println!("\n  {}🔗 URL {}{}\n", BOLD, action, RESET);
            println!("  {}{}{}", CYAN, text, RESET);
        }

        "uuid-gen" => {
            let count = get_u64(result, "count", 1);
            println!("\n  {}🆔 已生成 {} 个 UUID{}\n", BOLD, count, RESET);
            if let Some(arr) = result.get("uuids").and_then(|v| v.as_array()) {
                for (i, v) in arr.iter().enumerate() {
                    println!("  {}{}.{} {}", DIM, i + 1, RESET, v.as_str().unwrap_or(""));
                }
            }
        }

        "text-stats" => {
            println!("\n  {}📊 文本统计{}\n", BOLD, RESET);
            info_line("字符数", get_u64(result, "char_count", 0));
            info_line("单词数", get_u64(result, "word_count", 0));
            info_line("行数", get_u64(result, "line_count", 0));
            info_line("字节数", get_u64(result, "byte_count", 0));
            info_line("CJK 字符", get_u64(result, "cjk_char_count", 0));

            if let Some(freq) = result.get("word_frequency").and_then(|v| v.as_array()) {
                if !freq.is_empty() {
                    println!("\n  {}词频:{}", DIM, RESET);
                    for item in freq.iter().take(10) {
                        let word = item.get("word").and_then(|v| v.as_str()).unwrap_or("");
                        let count = item.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                        let bar = "▇".repeat(count as usize);
                        println!("    {:<12} {} {}", word, bar, count);
                    }
                }
            }
        }

        "hash" => {
            let hash = get_str(result, "result", "");
            let algo = get_str(result, "algorithm", "");
            println!("\n  {}🔑 {} 哈希值{}\n", BOLD, algo, RESET);
            println!("  {}{}{}", CYAN, hash, RESET);
            info_line("输入长度", format!("{} 字符", get_u64(result, "input_length", 0)));
        }

        "video-concat" => {
            let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
            if success {
                let out = get_str(result, "output_path", "");
                let count = get_u64(result, "input_count", 0);
                let strategy = get_str(result, "strategy", "");
                let ffmpeg_ver = get_str(result, "ffmpeg_version", "");
                let ffmpeg_path = get_str(result, "ffmpeg_path", "");

                println!("\n  {}🎬 视频拼接完成{}\n", BOLD, RESET);
                println!("  {}输出文件:{} {}", GREEN, RESET, out);
                info_line("输入文件数", count);
                info_line("拼接策略", &strategy);
                if let Some(size) = result.get("output_size_bytes").and_then(|v| v.as_u64()) {
                    let size_str = if size > 1024 * 1024 {
                        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
                    } else {
                        format!("{} bytes", size)
                    };
                    info_line("输出大小", &size_str);
                }
                info_line("ffmpeg 版本", &ffmpeg_ver);
                info_line("ffmpeg 路径", &ffmpeg_path);
            } else {
                let cancelled = result.get("cancelled").and_then(|v| v.as_bool()).unwrap_or(false);
                if cancelled {
                    let msg = get_str(result, "message", "用户取消了操作");
                    println!("\n  {}⏹  视频拼接已取消{}\n", YELLOW, RESET);
                    info_line("信息", &msg);
                } else {
                    println!("\n  {}❌ 视频拼接失败{}\n", RED, RESET);
                    println!("{}", serde_json::to_string_pretty(result).unwrap());
                }
            }
        }

        _ => {
            // 通用兜底：打印完整 JSON
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
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
                        tracing::error!("读取文件失败: {}", e);
                        std::process::exit(1);
                    });
                serde_json::from_str(&content).unwrap_or_else(|_| json!({ "input": content }))
            } else if let Some(json_str) = &input {
                serde_json::from_str(json_str).unwrap_or_else(|_| json!({ "input": json_str }))
            } else {
                let mut buffer = String::new();
                use std::io::Read;
                std::io::stdin().read_to_string(&mut buffer).unwrap_or_default();
                let trimmed = buffer.trim().to_string();
                serde_json::from_str(&trimmed).unwrap_or_else(|_| json!({ "input": trimmed }))
            };

            match engine.execute(&tool_id, input_value).await {
                Ok(result) => format_output(&tool_id, &result),
                Err(e) => {
                    tracing::error!("工具执行失败: {}", e);
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
            println!("\n  {}🛠  可用工具 ({}){}\n", BOLD, tools.len(), RESET);
            let mut current_cat: Option<&str> = None;
            for tool in &tools {
                let cat = tool.category.label();
                if current_cat != Some(cat) {
                    println!("  {}[{}]{}", YELLOW, cat, RESET);
                    current_cat = Some(cat);
                }
                println!("    {}{:<22}{} {}", CYAN, tool.id, RESET, tool.name);
            }
            println!();
        }
    }
}
