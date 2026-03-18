@echo off
chcp 65001 >nul
setlocal EnableDelayedExpansion

echo ========================================
echo   TurboGitHub 打包发布工具
echo ========================================
echo.

REM 设置版本号
set VERSION=0.0.1
set BUILD_DIR=target\release
set OUTPUT_DIR=release\TurboGitHub-%VERSION%
set ASSETS_DIR=assets

REM 检查 Rust 是否安装
rustc --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未检测到 Rust，请先安装 Rust
    echo 下载地址：https://www.rust-lang.org/zh-CN/tools/install
    pause
    exit /b 1
)

echo [信息] Rust 版本:
rustc --version
cargo --version
echo.

echo ========================================
echo [1/6] 构建 Release 版本...
echo ========================================
echo.

cargo build --release --verbose
if errorlevel 1 (
    echo [错误] 构建失败
    pause
    exit /b 1
)
echo [✓] 构建完成
echo.

echo ========================================
echo [2/6] 检查构建产物...
echo ========================================
echo.

if not exist "%BUILD_DIR%\turbogithub-gui.exe" (
    echo [错误] 未找到可执行文件
    pause
    exit /b 1
)

for %%A in ("%BUILD_DIR%\turbogithub-gui.exe") do set "EXE_SIZE=%%~zA"
set /a EXE_SIZE_MB=%EXE_SIZE%/1048576
echo [✓] 可执行文件大小：%EXE_SIZE_MB% MB
echo.

echo ========================================
echo [3/6] 创建发布目录...
echo ========================================
echo.

if exist "%OUTPUT_DIR%" (
    echo [信息] 删除旧版本目录...
    rmdir /s /q "%OUTPUT_DIR%"
)

mkdir "%OUTPUT_DIR%"
mkdir "%OUTPUT_DIR%\assets\icons"
mkdir "%OUTPUT_DIR%\assets\images"
echo [✓] 目录创建完成
echo.

echo ========================================
echo [4/6] 复制文件...
echo ========================================
echo.

echo 复制主程序...
copy "%BUILD_DIR%\turbogithub-gui.exe" "%OUTPUT_DIR%\"
if errorlevel 1 (
    echo [错误] 复制主程序失败
    pause
    exit /b 1
)

echo 复制配置文件...
copy "config.toml" "%OUTPUT_DIR%\"
if exist "core\config.toml" (
    mkdir "%OUTPUT_DIR%\core"
    copy "core\config.toml" "%OUTPUT_DIR%\core\"
)

echo 复制启动脚本...
copy "启动 TurboGitHub.bat" "%OUTPUT_DIR%\"

echo 复制文档...
copy "README.md" "%OUTPUT_DIR%\"
copy "LICENSE" "%OUTPUT_DIR%\"

echo 复制资源文件...
xcopy /E /I /Y "assets\icons" "%OUTPUT_DIR%\assets\icons"
xcopy /E /I /Y "assets\images" "%OUTPUT_DIR%\assets\images"

echo [✓] 文件复制完成
echo.

echo ========================================
echo [5/6] 创建版本信息文件...
echo ========================================
echo.

(
echo TurboGitHub v%VERSION%
echo.
echo 构建时间：%date% %time%
echo Rust 版本：
rustc --version
cargo --version
echo.
echo 文件列表:
dir /b "%OUTPUT_DIR%"
) > "%OUTPUT_DIR%\version.txt"

echo [✓] 版本信息文件创建完成
echo.

echo ========================================
echo [6/6] 创建压缩包...
echo ========================================
echo.

cd release

REM 检查 7-Zip 是否安装
where 7z >nul 2>&1
if errorlevel 1 (
    echo [警告] 未找到 7-Zip，使用 Windows 内置压缩
    echo [提示] 建议安装 7-Zip 以获得更好的压缩效果
    echo 下载地址：https://www.7-zip.org/
    echo.
    
    REM 使用 Windows 内置压缩
    powershell -Command "Compress-Archive -Path 'TurboGitHub-%VERSION%' -DestinationPath 'TurboGitHub-%VERSION%-portable.zip' -Force"
    echo [✓] ZIP 压缩包创建完成
) else (
    echo 使用 7-Zip 压缩...
    "C:\Program Files\7-Zip\7z.exe" a -t7z -mmt=on -mx=9 "TurboGitHub-%VERSION%-portable.7z" "TurboGitHub-%VERSION%"
    "C:\Program Files\7-Zip\7z.exe" a -tzip -mmt=on -mx=9 "TurboGitHub-%VERSION%-portable.zip" "TurboGitHub-%VERSION%"
    echo [✓] 7Z 和 ZIP 压缩包创建完成
)

cd ..

echo.
echo ========================================
echo   发布完成！
echo ========================================
echo.

echo 输出文件:
echo   - release\TurboGitHub-%VERSION%-portable.7z
echo   - release\TurboGitHub-%VERSION%-portable.zip
echo   - release\TurboGitHub-%VERSION%\ (便携版目录)
echo.

REM 显示目录结构
echo 发布目录结构:
tree /F /A "%OUTPUT_DIR%"
echo.

REM 计算文件大小
for %%A in ("%OUTPUT_DIR%\turbogithub-gui.exe") do set "EXE_SIZE=%%~zA"
set /a EXE_SIZE_MB=%EXE_SIZE%/1048576
echo 可执行文件大小：%EXE_SIZE_MB% MB
echo.

echo [提示] 测试程序运行：
echo   cd "%OUTPUT_DIR%"
echo   .\turbogithub-gui.exe
echo.

echo [提示] 创建安装包：
echo   使用 Inno Setup 或其他工具创建安装程序
echo   参考：打包发布建议.md
echo.

pause
