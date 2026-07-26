#!/bin/bash
# Trove sidecar 二进制复制脚本
# 把编译好的 trove CLI 复制到 Tauri 的 binaries 目录，
# 以便 Tauri 在开发和打包时作为 sidecar 捆绑。
#
# 用法:
#   bash scripts/copy-sidecar.sh              # 复制 release 版本
#   bash scripts/copy-sidecar.sh debug        # 复制 debug 版本

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Rust 目标三元组
TARGET_TRIPLE=$(rustc -vV | grep host | cut -d' ' -f2)
echo "🔍 检测到目标平台: $TARGET_TRIPLE"

# 构建模式
BUILD_MODE="${1:-release}"
if [ "$BUILD_MODE" = "debug" ]; then
  BUILD_DIR="debug"
else
  BUILD_DIR="release"
fi

SOURCE="$PROJECT_DIR/target/$BUILD_DIR/trove"
TARGET_DIR="$PROJECT_DIR/gui/src-tauri/binaries"

# 目标文件名：Tauri 要求 sidecar 名称为 binaries/{name}-{target_triple}
if [ "$BUILD_MODE" = "release" ]; then
  # 确保 release 版本已构建
  if [ ! -f "$SOURCE" ]; then
    echo "📦 构建 release 版本..."
    cd "$PROJECT_DIR"
    cargo build --release --bin trove
  fi
else
  # 确保 build 已构建
  if [ ! -f "$SOURCE" ]; then
    echo "📦 构建 debug 版本..."
    cd "$PROJECT_DIR"
    cargo build --bin trove
  fi
fi

# Windows 需要 .exe 后缀
if [[ "$TARGET_TRIPLE" == *"-windows-"* ]]; then
  TARGET_NAME="trove-$TARGET_TRIPLE.exe"
else
  TARGET_NAME="trove-$TARGET_TRIPLE"
fi

mkdir -p "$TARGET_DIR"
cp "$SOURCE" "$TARGET_DIR/$TARGET_NAME"
echo "✅ sidecar 已复制: $TARGET_DIR/$TARGET_NAME"
echo "   大小: $(du -h "$TARGET_DIR/$TARGET_NAME" | cut -f1)"
