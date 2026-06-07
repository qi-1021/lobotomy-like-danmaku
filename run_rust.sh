#!/bin/bash
# Lobotomy Danmaku - Rust 版本构建 & 运行 (macOS / Linux)

cd "$(dirname "$0")"

# 检查 Rust
if ! command -v cargo &>/dev/null; then
    echo "错误：未找到 Rust，请先安装: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 检查字体目录
if [ ! -d "fonts_proper" ] || [ -z "$(ls -A fonts_proper 2>/dev/null)" ]; then
    echo "提示：fonts_proper 目录为空，将尝试使用系统字体"
    echo "推荐：cp /System/Library/Fonts/PingFang.ttc fonts_proper/"
    echo ""
fi

# 检查 ffmpeg
if ! command -v ffmpeg &>/dev/null; then
    echo "错误：未找到 ffmpeg，请先安装"
    echo "macOS: brew install ffmpeg"
    echo "Linux: sudo apt install ffmpeg"
    exit 1
fi

# 构建
echo "构建中..."
cargo build --release 2>&1
if [ $? -ne 0 ]; then
    echo "构建失败"
    exit 1
fi

# 运行
echo ""
if [ "$1" = "--cli" ]; then
    shift
    echo "运行 danmaku CLI..."
    echo "用法: ./target/release/danmaku -i input.mp4 -o output.mp4 --text \"控制部\" --text \"WARNING\""
    echo ""
    if [ $# -eq 0 ]; then
        echo "示例："
        echo "  ./target/release/danmaku -i video.mp4 -o output.mp4 --texts texts.json"
        echo "  ./target/release/danmaku -i video.mp4 -o output.mp4 --text \"控制部\" --text \"WARNING\""
    else
        ./target/release/danmaku "$@"
    fi
else
    echo "启动 GUI..."
    ./target/release/danmaku-gui
fi
