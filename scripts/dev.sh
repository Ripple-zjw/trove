#!/bin/bash
# Trove 开发环境启动脚本
# 启动 Core 后端和 Vite 前端开发服务器

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🚀 Trove 开发环境启动"
echo "===================="

# 启动后端
echo ""
echo "📦 启动 Trove Core 服务..."
cd "$PROJECT_DIR"
cargo run -- serve --port 8080 &
CORE_PID=$!
echo "   Core PID: $CORE_PID"

# 等待 Core 就绪
sleep 2

# 启动前端
echo ""
echo "🎨 启动 Web GUI (Vite)..."
cd "$PROJECT_DIR/gui"
npx vite --port 1420 &
VITE_PID=$!
echo "   Vite PID: $VITE_PID"

echo ""
echo "✅ Trove 已启动!"
echo "   Core API:  http://127.0.0.1:8080"
echo "   Web GUI:   http://127.0.0.1:1420"
echo ""
echo "按 Ctrl+C 停止所有服务"

# 捕获退出信号
trap "echo '正在停止...'; kill $CORE_PID $VITE_PID 2>/dev/null; exit 0" SIGINT SIGTERM

# 等待子进程
wait
