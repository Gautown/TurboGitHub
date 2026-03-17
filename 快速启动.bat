@echo off
REM TurboGitHub 快速启动脚本
REM 解决 cargo run 缓慢的问题

cd /d "%~dp0"

echo ========================================
echo   TurboGitHub 快速启动
echo ========================================
echo.

REM 检查是否存在编译产物
if exist "target\debug\turbogithub-gui.exe" (
    echo [✓] 检测到已编译的可执行文件
    echo [ℹ] 直接运行已编译的程序...
    echo.
    start "" "target\debug\turbogithub-gui.exe"
    echo [✓] 应用程序已启动
) else (
    echo [⚠] 未检测到已编译的可执行文件
    echo [ℹ] 首次运行，开始编译...
    echo.
    echo [提示] 首次编译可能需要较长时间（5-15 分钟）
    echo [提示] 编译完成后将自动启动应用程序
    echo.
    cargo run --package turbogithub-gui
)

echo.
echo ========================================
echo 提示：
echo - 首次编译后，后续运行会快很多
echo - 可以使用 cargo run --release 获得更好性能
echo - 运行 cargo check 可以快速检查代码
echo ========================================

pause
