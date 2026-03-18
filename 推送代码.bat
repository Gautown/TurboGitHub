@echo off
chcp 65001 >nul
setlocal EnableDelayedExpansion

echo ========================================
echo   TurboGitHub 推送工具
echo ========================================
echo.

echo [信息] 此脚本将推送本地提交到 GitHub
echo.

REM 检查网络连接
echo [检查] 测试 GitHub 连接...
ping -n 1 github.com >nul 2>&1
if errorlevel 1 (
    echo [警告] 无法连接到 GitHub，请检查网络连接
    echo.
    echo [提示] 您可以：
    echo   1. 检查网络连接
    echo   2. 使用加速器或代理
    echo   3. 稍后再试
    echo.
    set /p continue="是否继续尝试？(Y/N): "
    if /i not "%continue%"=="Y" (
        echo [取消] 推送已取消
        pause
        exit /b 0
    )
)

echo.
echo ========================================
echo [1/3] 查看当前状态...
echo ========================================
echo.

git status
echo.

echo ========================================
echo [2/3] 推送到 GitHub...
echo ========================================
echo.

git push TurboGitHub main

if errorlevel 1 (
    echo.
    echo [错误] 推送失败
    echo.
    echo [可能的原因]:
    echo   1. 网络连接问题
    echo   2. GitHub 服务不可用
    echo   3. 需要身份验证
    echo.
    echo [建议]:
    echo   1. 检查网络连接
    echo   2. 使用 GitHub 加速器
    echo   3. 配置 Git 凭证
    echo   4. 稍后重试
    echo.
    echo [手动推送命令]:
    echo   git push TurboGitHub main
    echo.
    pause
    exit /b 1
)

echo.
echo ========================================
echo [3/3] 推送标签...
echo ========================================
echo.

set /p push_tags="是否推送所有标签？(Y/N): "
if /i "%push_tags%"=="Y" (
    git push TurboGitHub --tags
    if errorlevel 1 (
        echo [警告] 标签推送失败
    ) else (
        echo [✓] 标签推送成功
    )
) else (
    echo [跳过] 标签推送
)

echo.
echo ========================================
echo   推送完成！
echo ========================================
echo.

echo [总结]:
echo   ✓ 代码已推送到 GitHub
echo   ✓ 检查 GitHub 仓库确认
echo.

echo [下一步]:
echo   1. 访问 https://github.com/Gautown/TurboGitHub
echo   2. 确认提交已推送
echo   3. 创建 Release 版本
echo.

pause
