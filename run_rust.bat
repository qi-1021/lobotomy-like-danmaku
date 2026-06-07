@echo off
REM Lobotomy Danmaku - Rust 版本构建 & 运行 (Windows)

cd /d "%~dp0"

REM 检查 Rust
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo 错误：未找到 Rust，请先安装
    echo 下载：https://rustup.rs/
    pause
    exit /b 1
)

REM 检查 ffmpeg
where ffmpeg >nul 2>&1
if %errorlevel% neq 0 (
    echo 错误：未找到 ffmpeg，请先安装
    echo 下载：https://ffmpeg.org/download.html
    pause
    exit /b 1
)

REM 构建
echo 构建中...
cargo build --release
if %errorlevel% neq 0 (
    echo 构建失败
    pause
    exit /b 1
)

REM 运行
echo.
if "%~1"=="--cli" (
    shift
    echo 运行 danmaku CLI...
    echo 用法: target\release\danmaku.exe -i input.mp4 -o output.mp4 --text "控制部" --text "WARNING"
    echo.
    if "%~1"=="" (
        echo 示例：
        echo   target\release\danmaku.exe -i video.mp4 -o output.mp4 --texts texts.json
        echo   target\release\danmaku.exe -i video.mp4 -o output.mp4 --text "控制部" --text "WARNING"
    ) else (
        target\release\danmaku.exe %*
    )
) else (
    echo 启动 GUI...
    target\release\danmaku-gui.exe
)
pause
