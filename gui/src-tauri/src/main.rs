// Trove Desktop — Tauri 桌面应用壳
// 负责启动 Web GUI 并管理 Trove Core 子进程

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command};
use std::sync::Mutex;

use tauri::Manager;

struct CoreProcess(Mutex<Option<Child>>);

/// 启动 Trove Core 子进程
fn start_core() -> Option<Child> {
    // 尝试启动 trove serve
    // 在开发环境中，Core 可能已经在外部启动
    match Command::new("trove")
        .args(["serve", "--port", "8080"])
        .spawn()
    {
        Ok(child) => {
            println!("🧩 Trove Core 已启动 (PID: {})", child.id());
            Some(child)
        }
        Err(e) => {
            eprintln!("⚠️ 无法启动 Trove Core (请确保已安装): {}", e);
            eprintln!("   开发时请手动启动: trove serve --port 8080");
            None
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 启动 Core 进程
            let core = start_core();
            app.manage(CoreProcess(Mutex::new(core)));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // 关闭时结束 Core 进程
                if let Some(state) = window.try_state::<CoreProcess>() {
                    if let Ok(mut guard) = state.0.lock() {
                        if let Some(ref mut child) = *guard {
                            let _ = child.kill();
                            println!("🧩 Trove Core 已停止");
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Trove 桌面应用启动失败");
}
