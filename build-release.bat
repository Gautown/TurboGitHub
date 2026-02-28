@echo off
echo ========================================
echo    TurboGitHub 独立发布版本构建脚本
echo ========================================
echo.

REM 检查是否在正确的目录
if not exist "Cargo.toml" (
    echo 错误：请在项目根目录运行此脚本
    pause
    exit /b 1
)

echo [1/5] 清理构建缓存...
call cargo clean

REM 检查是否安装了Rust
rustc --version >nul 2>&1
if errorlevel 1 (
    echo 错误：未检测到Rust编译器，请先安装Rust
    echo 访问 https://rustup.rs/ 安装Rust
    pause
    exit /b 1
)

echo [2/5] 检查依赖项...
call cargo check --release

if errorlevel 1 (
    echo 错误：依赖项检查失败
    pause
    exit /b 1
)

echo [3/5] 构建发布版本...
call cargo build --release

if errorlevel 1 (
    echo 错误：构建失败
    pause
    exit /b 1
)

echo [4/5] 创建发布目录...
if not exist "release" mkdir release

REM 复制可执行文件
echo [5/5] 准备发布文件...
copy "target\release\turbogithub-gui.exe" "release\TurboGitHub.exe"
copy "config.toml" "release\"

REM 创建资源目录
if not exist "release\assets" mkdir release\assets
if not exist "release\assets\icons" mkdir release\assets\icons
copy "assets\icons\logo.svg" "release\assets\icons\"
copy "assets\icons\logo.png" "release\assets\icons\"
copy "assets\icons\logo.ico" "release\assets\icons\"

REM 创建启动脚本（无控制台窗口）
echo @echo off > "release\启动TurboGitHub.bat"
echo echo 正在启动 TurboGitHub... >> "release\启动TurboGitHub.bat"
echo echo ======================================== >> "release\启动TurboGitHub.bat"
echo echo GitHub加速工具 - DNS优化和流量监控 >> "release\启动TurboGitHub.bat"
echo echo 版本 0.0.1 >> "release\启动TurboGitHub.bat"
echo echo ======================================== >> "release\启动TurboGitHub.bat"
echo echo. >> "release\启动TurboGitHub.bat"
echo echo 注意：GUI窗口将直接打开，不会显示控制台窗口 >> "release\启动TurboGitHub.bat"
echo echo. >> "release\启动TurboGitHub.bat"
echo timeout /t 2 /nobreak >nul >> "release\启动TurboGitHub.bat"
echo start "" "TurboGitHub.exe" >> "release\启动TurboGitHub.bat"

REM 创建无控制台窗口启动脚本
echo Set WshShell = CreateObject("WScript.Shell") > "release\启动TurboGitHub.vbs"
echo WshShell.Run "TurboGitHub.exe", 0, False >> "release\启动TurboGitHub.vbs"

REM 创建守护进程启动脚本
echo @echo off > "release\启动守护进程.bat"
echo echo 正在启动 TurboGitHub 守护进程... >> "release\启动守护进程.bat"
echo echo ======================================== >> "release\启动守护进程.bat"
echo echo DNS服务器 - 端口 53535 >> "release\启动守护进程.bat"
echo echo ======================================== >> "release\启动守护进程.bat"
echo echo. >> "release\启动守护进程.bat"
echo echo 注意：此进程将在后台运行DNS服务 >> "release\启动守护进程.bat"
echo echo 按 Ctrl+C 停止服务 >> "release\启动守护进程.bat"
echo echo. >> "release\启动守护进程.bat"
echo timeout /t 2 /nobreak >nul >> "release\启动守护进程.bat"
echo echo 启动DNS服务器... >> "release\启动守护进程.bat"
echo cd /d "%~dp0" >> "release\启动守护进程.bat"
echo if exist "turbogithub-core.exe" ( >> "release\启动守护进程.bat"
echo     turbogithub-core.exe >> "release\启动守护进程.bat"
echo ) else ( >> "release\启动守护进程.bat"
echo     echo 错误：未找到守护进程可执行文件 >> "release\启动守护进程.bat"
echo     echo 请先构建核心组件 >> "release\启动守护进程.bat"
echo     pause >> "release\启动守护进程.bat"
echo ) >> "release\启动守护进程.bat"

echo.
echo ========================================
echo   构建完成！
echo ========================================
echo.
echo 发布文件位置：release 目录
echo.
echo 包含的文件：
echo   - TurboGitHub.exe (主程序)
if exist "target\release\turbogithub-core.exe" (
    copy "target\release\turbogithub-core.exe" "release\"
    echo   - turbogithub-core.exe (守护进程)
)
echo   - config.toml (配置文件)
echo   - assets/icons/ (图标文件)
echo   - 启动TurboGitHub.bat (启动脚本)
echo   - 启动守护进程.bat (守护进程启动脚本)
echo.
echo 使用说明：
echo   1. 运行 启动守护进程.bat 启动DNS服务（可选）
echo   2. 运行 启动TurboGitHub.bat 启动GUI界面
echo.
echo 文件大小信息：
for %%f in ("release\*.exe") do (
    for %%i in (%%~zf) do set size=%%i
    echo   %%~nxf: !size! 字节
)

pause