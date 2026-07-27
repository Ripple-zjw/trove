use std::path::PathBuf;

use serde::Serialize;

/// ffmpeg 检测信息
#[derive(Debug, Clone, Serialize)]
pub struct FfmpegInfo {
    /// 是否可用
    pub available: bool,
    /// ffmpeg 的路径
    pub path: String,
    /// ffmpeg 版本字符串
    pub version: String,
}

/// 检测系统中的 ffmpeg
///
/// 搜索顺序：
/// 1. `$FFMPEG_PATH` 环境变量
/// 2. `$PATH` 中的 `ffmpeg`
/// 3. Windows 平台特定路径
pub fn detect_ffmpeg() -> FfmpegInfo {
    let path = find_ffmpeg();
    match path {
        Some(path) => {
            let version = get_ffmpeg_version(&path);
            FfmpegInfo {
                available: version.is_some(),
                path: path.to_string_lossy().to_string(),
                version: version.unwrap_or_default(),
            }
        }
        None => FfmpegInfo {
            available: false,
            path: String::new(),
            version: String::new(),
        },
    }
}

fn find_ffmpeg() -> Option<PathBuf> {
    // 1. 环境变量 $FFMPEG_PATH
    if let Ok(env_path) = std::env::var("FFMPEG_PATH") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Some(p);
        }
        // 也可能是目录，拼接 ffmpeg 可执行文件名
        let exe = if cfg!(target_os = "windows") {
            p.join("ffmpeg.exe")
        } else {
            p.join("ffmpeg")
        };
        if exe.is_file() {
            return Some(exe);
        }
    }

    // 2. $PATH 搜索
    if let Some(path) = which("ffmpeg") {
        return Some(path);
    }

    // 3. Windows 平台特定路径
    #[cfg(target_os = "windows")]
    {
        let candidates = vec![
            r"C:\ffmpeg\bin\ffmpeg.exe",
            r"C:\Program Files\ffmpeg\bin\ffmpeg.exe",
        ];
        for c in candidates {
            let p = PathBuf::from(c);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    None
}

/// 在 PATH 中查找可执行文件
fn which(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };

    for dir in std::env::split_paths(&path_var) {
        let full_path = dir.join(&exe_name);
        if full_path.is_file() {
            return Some(full_path);
        }
    }
    None
}

/// 执行 ffmpeg -version 获取版本信息
fn get_ffmpeg_version(path: &PathBuf) -> Option<String> {
    let output = std::process::Command::new(path)
        .arg("-version")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 取第一行，如 "ffmpeg version 6.1.1 Copyright (c) 2000-2023 the FFmpeg developers"
    let first_line = stdout.lines().next()?;
    Some(first_line.to_string())
}
