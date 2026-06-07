@echo off
REM Lobotomy Danmaku GUI 启动脚本 (Windows)

cd /d "%~dp0"

REM 检查 Python
where python3 >nul 2>&1
if %errorlevel% equ 0 (
    set PYTHON=python3
) else (
    where python >nul 2>&1
    if %errorlevel% equ 0 (
        set PYTHON=python
    ) else (
        echo 错误：未找到 Python，请先安装 Python 3
        pause
        exit /b 1
    )
)

REM 检查依赖
%PYTHON% -c "import PIL" 2>nul
if %errorlevel% neq 0 (
    echo 正在安装依赖...
    %PYTHON% -m pip install -r requirements.txt
)

REM 检查字体目录
if not exist "fonts_proper" (
    mkdir fonts_proper
    echo 警告：fonts_proper 目录为空
    echo 请将字体文件放入 fonts_proper\ 目录
    echo 例如：将 PingFang.ttc 复制到 fonts_proper\
    pause
)

REM 启动 GUI
echo 启动弹幕编辑器...
%PYTHON% lobotomy_gui.py
pause
