// Trove Desktop — Tauri 桌面应用壳
// 负责启动 Web GUI 并管理 Trove Core sidecar 子进程

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::Manager;
use tauri_plugin_shell::ShellExt;

struct CoreProcess(Mutex<Option<tauri_plugin_shell::process::CommandChild>>);

impl Drop for CoreProcess {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.0.lock() {
            if let Some(child) = guard.take() {
                let _ = child.kill();
                println!("🧩 Trove Core 已停止");
            }
        }
    }
}

/// 启动 Trove Core sidecar 进程
fn start_core(app: &tauri::AppHandle) -> Option<tauri_plugin_shell::process::CommandChild> {
    let shell = app.shell();

    match shell.sidecar("trove").map_err(|e| e.to_string()) {
        Ok(command) => {
            match command.args(["serve", "--port", "8080"]).spawn() {
                Ok((mut rx, child)) => {
                    println!("🧩 Trove Core 已启动 (PID: {})", child.pid());

                    // 异步读取 sidecar 输出（日志）
                    tauri::async_runtime::spawn(async move {
                        use tauri_plugin_shell::process::CommandEvent;
                        while let Some(event) = rx.recv().await {
                            if let CommandEvent::Stderr(line) = &event {
                                eprintln!("[core] {}", String::from_utf8_lossy(line));
                            } else if let CommandEvent::Stdout(line) = &event {
                                println!("[core] {}", String::from_utf8_lossy(line));
                            } else if let CommandEvent::Terminated(payload) = &event {
                                println!("🧩 Trove Core 已退出 (exit: {})", payload.code.unwrap_or(-1));
                                break;
                            }
                        }
                    });

                    Some(child)
                }
                Err(e) => {
                    eprintln!("⚠️ 无法启动 Trove Core: {}", e);
                    eprintln!("   开发时请手动启动: trove serve --port 8080");
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️ 无法创建 sidecar 命令: {}", e);
            eprintln!("   开发阶段请手动启动: trove serve --port 8080");
            None
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 启动 Core sidecar
            let child = start_core(app.handle());
            app.manage(CoreProcess(Mutex::new(child)));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<CoreProcess>() {
                    if let Ok(mut guard) = state.0.lock() {
                        if let Some(child) = guard.take() {
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
