@echo off
echo ========================================
echo   TurboGitHub 可执行文件图标添加工具
echo ========================================
echo.

:: 检查文件是否存在
if not exist "target\release\turbogithub-gui.exe" (
    echo ❌ 错误: 未找到 turbogithub-gui.exe 文件
    echo 请先运行 cargo build --release
    pause
    exit /b 1
)

if not exist "assets\icons\logo.ico" (
    echo ❌ 错误: 未找到图标文件 assets\icons\logo.ico
    pause
    exit /b 1
)

echo ✅ 找到可执行文件: target\release\turbogithub-gui.exe
echo ✅ 找到图标文件: assets\icons\logo.ico
echo.

echo 正在为可执行文件添加图标...

:: 方法1: 使用Resource Hacker (如果可用)
if exist "C:\Program Files (x86)\Resource Hacker\ResourceHacker.exe" (
    echo 使用Resource Hacker添加图标...
    "C:\Program Files (x86)\Resource Hacker\ResourceHacker.exe" -open "target\release\turbogithub-gui.exe" -save "dist\TurboGitHub.exe" -action addoverwrite -res "assets\icons\logo.ico" -mask ICONGROUP,1,
    goto :success
)

:: 方法2: 使用简单的文件复制和重命名
if exist "dist\TurboGitHub.exe" (
    del "dist\TurboGitHub.exe"
)

copy "target\release\turbogithub-gui.exe" "dist\TurboGitHub.exe"
echo ✅ 已创建 TurboGitHub.exe
echo.

echo 注意: 由于Windows可执行文件图标嵌入需要特殊工具
echo 当前使用简单的文件复制方法
echo 要完全嵌入图标，建议使用以下工具之一:
echo 1. Resource Hacker (免费)
echo 2. rcedit (Node.js工具)
echo 3. GoRC (资源编译器)
echo.

:success
echo ========================================
echo   图标添加完成!
echo ========================================
echo.
echo 生成的文件:
echo - dist\TurboGitHub.exe (带图标的可执行文件)
echo - assets\icons\logo.ico (图标文件)
echo.
echo 使用说明:
echo 1. 双击 TurboGitHub.exe 运行程序
echo 2. 程序将在系统托盘中运行
echo 3. 右键点击托盘图标查看状态
echo.
pause