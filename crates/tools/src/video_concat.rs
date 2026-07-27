use std::path::Path;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use trove_core::{CancelToken, Tool, ToolCategory, ToolContext, ToolError, ToolResult, ProgressEvent};
use crate::ffmpeg_detector;

/// 视频拼接工具
pub struct VideoConcat;

#[async_trait]
impl Tool for VideoConcat {
    fn id(&self) -> &'static str {
        "video-concat"
    }

    fn name(&self) -> &'static str {
        "视频拼接"
    }

    fn description(&self) -> &'static str {
        "将多个视频文件拼接成一个视频文件。自动检测 ffmpeg 并选择最优策略：同格式流拷贝（无损、极快），不同格式 H.264 重编码。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["files"],
            "properties": {
                "files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "title": "视频文件列表",
                    "description": "要拼接的视频文件绝对路径，按此顺序拼接"
                },
                "output": {
                    "type": "string",
                    "title": "输出路径（可选）",
                    "description": "输出文件路径，默认为 ~/Downloads/video_concat_<时间戳>.mp4"
                },
                "quality": {
                    "type": "string",
                    "title": "画质",
                    "description": "转码时的输出画质：低(CRF28) 中(CRF23) 高(CRF18)",
                    "enum": ["low", "medium", "high"],
                    "default": "medium"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Media
    }

    fn is_cpu_intensive(&self) -> bool {
        true
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> ToolResult<Value> {
        let files = parse_files(&input)?;
        let output = determine_output(&input)?;
        let quality = parse_quality(&input);
        let crf = quality_crf(quality);

        // 检测 ffmpeg
        let ffmpeg_info = ffmpeg_detector::detect_ffmpeg();
        if !ffmpeg_info.available {
            return Err(ToolError::ExecutionError(
                "未找到 ffmpeg。请先安装 ffmpeg：\n"
                    .to_string()
                    + "  macOS: brew install ffmpeg\n"
                    + "  Ubuntu/Debian: sudo apt install ffmpeg\n"
                    + "  Windows: choco install ffmpeg\n"
                    + "也可通过设置 FFMPEG_PATH 环境变量指定自定义路径。",
            ));
        }

        // 检测容器格式是否一致
        let all_same = all_same_container(&files);
        let strategy = if all_same { "stream-copy" } else { "re-encode" };

        let ffmpeg_path = &ffmpeg_info.path;
        let cancel = ctx.cancel_token.clone();
        let progress_tx = ctx.progress_tx.clone();

        let result = if all_same {
            run_concat_demuxer(ffmpeg_path, &files, &output, cancel.as_ref()).await
        } else {
            run_concat_filter(
                ffmpeg_path, &files, &output, crf,
                progress_tx.as_ref(), cancel.as_ref(),
            ).await
        };

        match result {
            Ok(()) => {
                let size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
                let duration = get_media_duration(&output).await.unwrap_or(0.0);

                Ok(json!({
                    "success": true,
                    "output_path": output,
                    "input_count": files.len(),
                    "output_size_bytes": size,
                    "output_duration_secs": duration,
                    "strategy": strategy,
                    "ffmpeg_version": ffmpeg_info.version,
                    "ffmpeg_path": ffmpeg_info.path
                }))
            }
            Err(e) => {
                if let Some(ref c) = cancel {
                    if c.is_cancelled() {
                        let _ = std::fs::remove_file(&output);
                        return Ok(json!({
                            "success": false,
                            "cancelled": true,
                            "output_path": Value::Null,
                            "message": "用户取消了拼接操作"
                        }));
                    }
                }
                Err(e)
            }
        }
    }
}

// ─── 参数解析 ──────────────────────────────────────────

fn parse_files(input: &Value) -> ToolResult<Vec<String>> {
    let arr = input
        .get("files")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidInput("缺少 files 字段，请提供视频文件路径列表".to_string()))?;

    if arr.is_empty() {
        return Err(ToolError::InvalidInput("files 列表不能为空".to_string()));
    }

    if arr.len() > 200 {
        return Err(ToolError::InvalidInput(
            format!("文件数量过多（{}个），建议不超过 200 个", arr.len()),
        ));
    }

    let files: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    if files.len() != arr.len() {
        return Err(ToolError::InvalidInput(
            "files 列表中的项目必须是字符串（文件路径）".to_string(),
        ));
    }

    for f in &files {
        if !Path::new(f).exists() {
            return Err(ToolError::InvalidInput(format!("文件不存在: {}", f)));
        }
    }

    Ok(files)
}

fn determine_output(input: &Value) -> ToolResult<String> {
    if let Some(out) = input.get("output").and_then(|v| v.as_str()) {
        if !out.is_empty() {
            return Ok(out.to_string());
        }
    }

    let download_dir = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let default_name = format!("video_concat_{}.mp4", timestamp);
    Ok(download_dir.join(default_name).to_string_lossy().to_string())
}

fn parse_quality(input: &Value) -> &str {
    input
        .get("quality")
        .and_then(|v| v.as_str())
        .unwrap_or("medium")
}

fn quality_crf(q: &str) -> u32 {
    match q {
        "low" => 28,
        "high" => 18,
        _ => 23, // medium
    }
}

fn all_same_container(files: &[String]) -> bool {
    if files.len() <= 1 {
        return true;
    }
    let first_ext = Path::new(&files[0])
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    files[1..].iter().all(|f| {
        Path::new(f)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
            == first_ext
    })
}

// ─── 方案 A: Concat Demuxer（同格式，流拷贝）───────────

async fn run_concat_demuxer(
    ffmpeg_path: &str,
    files: &[String],
    output: &str,
    cancel: Option<&CancelToken>,
) -> ToolResult<()> {
    // 创建临时文件列表
    let mut filelist_content = String::new();
    for f in files {
        // ffmpeg concat demuxer 要求 path 用单引号包裹，特殊字符需转义
        let escaped = f.replace('\'', "'\\''");
        filelist_content.push_str(&format!("file '{}'\n", escaped));
    }

    let tmp_dir = std::env::temp_dir();
    let filelist_name = format!("trove_concat_{}.txt", std::process::id());
    let filelist_path = tmp_dir.join(&filelist_name);
    std::fs::write(&filelist_path, &filelist_content)
        .map_err(|e| ToolError::ExecutionError(format!("无法创建临时文件列表: {}", e)))?;

    let status = Command::new(ffmpeg_path)
        .args([
            "-f", "concat",
            "-safe", "0",
            "-i", filelist_path.to_str().unwrap_or_default(),
            "-c", "copy",
            "-y",
            output,
        ])
        .status()
        .await
        .map_err(|e| ToolError::ExecutionError(format!("ffmpeg 执行失败: {}", e)))?;

    let _ = std::fs::remove_file(&filelist_path);

    if let Some(c) = cancel {
        if c.is_cancelled() {
            return Ok(());
        }
    }

    if !status.success() {
        return Err(ToolError::ExecutionError(
            "ffmpeg 拼接失败（流拷贝模式），请检查输入文件是否损坏或格式是否真的一致。".to_string(),
        ));
    }

    Ok(())
}

// ─── 方案 B: Concat Filter（不同格式，重编码）────────

async fn run_concat_filter(
    ffmpeg_path: &str,
    files: &[String],
    output: &str,
    crf: u32,
    progress_tx: Option<&tokio::sync::mpsc::UnboundedSender<ProgressEvent>>,
    cancel: Option<&CancelToken>,
) -> ToolResult<()> {
    let n = files.len();

    let mut args: Vec<String> = Vec::new();
    for f in files {
        args.push("-i".to_string());
        args.push(f.clone());
    }

    // 构建 filter_complex: [0:v][0:a][1:v][1:a]...concat=n=N:v=1:a=1
    let mut filter = String::new();
    for i in 0..n {
        filter.push_str(&format!("[{}:v][{}:a]", i, i));
    }
    filter.push_str(&format!("concat=n={}:v=1:a=1", n));

    args.push("-filter_complex".to_string());
    args.push(filter);
    args.push("-c:v".to_string());
    args.push("libx264".to_string());
    args.push("-crf".to_string());
    args.push(crf.to_string());
    args.push("-preset".to_string());
    args.push("medium".to_string());
    args.push("-c:a".to_string());
    args.push("aac".to_string());
    args.push("-b:a".to_string());
    args.push("192k".to_string());
    args.push("-y".to_string());
    args.push(output.to_string());

    let mut child = Command::new(ffmpeg_path)
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| ToolError::ExecutionError(format!("无法启动 ffmpeg: {}", e)))?;

    // 读取 stderr 获取进度和检查取消
    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    // 用 loop + select! 模式同时处理进度行读取和取消检查
    use tokio::select;

    let cancel_check_interval = tokio::time::Duration::from_millis(500);

    loop {
        select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(ref l)) if l.starts_with("frame=") => {
                        // 解析进度行: frame=  123 fps=30 time=00:00:04.10 speed=1.5x
                        parse_and_send_progress(l, progress_tx);
                    }
                    Ok(Some(_)) => {
                        // 非进度行，忽略
                    }
                    Ok(None) => {
                        // stderr 结束，等待子进程退出
                        break;
                    }
                    Err(e) => {
                        return Err(ToolError::ExecutionError(
                            format!("读取 ffmpeg 输出失败: {}", e)
                        ));
                    }
                }
            }
            _ = tokio::time::sleep(cancel_check_interval) => {
                if let Some(c) = cancel {
                    if c.is_cancelled() {
                        child.kill().await.ok();
                        return Ok(());
                    }
                }
            }
        }
    }

    let status = child.wait().await
        .map_err(|e| ToolError::ExecutionError(format!("ffmpeg 进程异常: {}", e)))?;

    if let Some(c) = cancel {
        if c.is_cancelled() {
            return Ok(());
        }
    }

    if status.success() {
        // 发送 100% 进度
        if let Some(tx) = progress_tx {
            let _ = tx.send(ProgressEvent {
                percent: 1.0,
                time: String::new(),
                frame: 0,
                speed: String::new(),
            });
        }
        Ok(())
    } else {
        Err(ToolError::ExecutionError(
            "ffmpeg 拼接失败（重编码模式），请检查输入文件是否有效。".to_string(),
        ))
    }
}

// ─── 进度解析 ──────────────────────────────────────────

fn parse_and_send_progress(
    line: &str,
    progress_tx: Option<&tokio::sync::mpsc::UnboundedSender<ProgressEvent>>,
) {
    let tx = match progress_tx {
        Some(tx) => tx,
        None => return,
    };

    let frame = extract_u64(line, "frame=");
    let time_str = extract_string(line, "time=");
    let speed = extract_string(line, "speed=");
    let percent = time_to_progress(&time_str);

    let _ = tx.send(ProgressEvent {
        percent,
        time: time_str,
        frame,
        speed,
    });
}

fn extract_u64(line: &str, key: &str) -> u64 {
    if let Some(pos) = line.find(key) {
        let rest = &line[pos + key.len()..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        num_str.parse().unwrap_or(0)
    } else {
        0
    }
}

fn extract_string<'a>(line: &'a str, key: &'a str) -> String {
    if let Some(pos) = line.find(key) {
        let rest = &line[pos + key.len()..];
        rest.split_whitespace().next().unwrap_or("").to_string()
    } else {
        String::new()
    }
}

/// 从 time= 格式解析进度百分比
/// time=00:01:23.45 → 无总时长信息，返回 0.0 表示未知
fn time_to_progress(_time_str: &str) -> f64 {
    0.0
}

// ─── 时长获取 ──────────────────────────────────────────

/// 通过 ffprobe 获取视频时长（秒）
async fn get_media_duration(path: &str) -> Option<f64> {
    let info = ffmpeg_detector::detect_ffmpeg();
    if !info.available {
        return None;
    }

    let ffprobe_path = {
        let p = Path::new(&info.path);
        let dir = p.parent()?;
        let name = if cfg!(target_os = "windows") {
            "ffprobe.exe"
        } else {
            "ffprobe"
        };
        dir.join(name)
    };

    if !ffprobe_path.exists() {
        return None;
    }

    let output = Command::new(ffprobe_path.to_str()?)
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "csv=p=0",
            path,
        ])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<f64>().ok()
}
