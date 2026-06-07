#!/bin/bash
# Lobotomy Danmaku GUI 启动脚本 (macOS / Linux)

cd "$(dirname "$0")"

# 检查 Python
if command -v python3 &>/dev/null; then
    PYTHON=python3
elif command -v python &>/dev/null; then
    PYTHON=python
else
    echo "错误：未找到 Python，请先安装 Python 3"
    exit 1
fi

# 检查依赖
if ! $PYTHON -c "import PIL" 2>/dev/null; then
    echo "正在安装依赖..."
    $PYTHON -m pip install -r requirements.txt
fi

# 检查字体目录
if [ ! -d "fonts_proper" ] || [ -z "$(ls -A fonts_proper 2>/dev/null)" ]; then
    echo "警告：fonts_proper 目录为空"
    echo "请将字体文件放入 fonts_proper/ 目录"
    echo "macOS: cp /System/Library/Fonts/PingFang.ttc fonts_proper/"
    echo ""
fi

# 启动 GUI
echo "启动弹幕编辑器..."
$PYTHON lobotomy_gui.py
