#!/bin/bash
# Lobotomy Danmaku GUI 启动脚本 (macOS / Linux)

cd "$(dirname "$0")"

# 检查已有可用的虚拟环境
for VENV in .venv /tmp/unityextract; do
    if [ -x "$VENV/bin/python" ] && "$VENV/bin/python" -c "import PIL" 2>/dev/null; then
        PYTHON="$VENV/bin/python"
        break
    fi
done

# 没有可用环境，创建新的
if [ -z "$PYTHON" ]; then
    PYTHON=python3
    if ! command -v python3 &>/dev/null; then
        echo "错误：未找到 Python，请先安装 Python 3"
        exit 1
    fi
    VENV_DIR=".venv"
    if [ ! -d "$VENV_DIR" ]; then
        echo "首次运行，正在创建虚拟环境..."
        $PYTHON -m venv "$VENV_DIR"
    fi
    source "$VENV_DIR/bin/activate"
    pip install -r requirements.txt
    PYTHON="$VENV_DIR/bin/python"
fi

# 检查字体目录
if [ ! -d "fonts_proper" ] || [ -z "$(ls -A fonts_proper 2>/dev/null)" ]; then
    echo "提示：fonts_proper 目录为空，将尝试使用系统字体"
    echo "推荐：cp /System/Library/Fonts/PingFang.ttc fonts_proper/"
    echo ""
fi

# 启动 GUI
$PYTHON lobotomy_gui.py
