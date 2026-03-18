@echo off
chcp 65001 >nul
setlocal EnableDelayedExpansion

echo ========================================
echo   TurboGitHub 清理工具
echo ========================================
echo.

echo [信息] 此脚本将清理以下文件：
echo   - target/ 目录（构建产物）
echo   - release/ 目录（发布包）
echo   - *.log 文件
echo   - *.tmp 文件
echo   - *.bak 文件
echo   - .DS_Store 文件
echo   - Thumbs.db 文件
echo.

set /p confirm="是否继续？(Y/N): "
if /i not "%confirm%"=="Y" (
    echo [取消] 清理已取消
    pause
    exit /b 0
)

echo.
echo ========================================
echo [1/6] 清理 target 目录...
echo ========================================
echo.

if exist "target" (
    echo [信息] 删除 target 目录...
    rmdir /s /q target
    if exist "target" (
        echo [警告] target 目录删除失败，可能需要管理员权限
        echo [提示] 请手动删除或运行：cargo clean
    ) else (
        echo [✓] target 目录已删除
    )
) else (
    echo [信息] target 目录不存在
)
echo.

echo ========================================
echo [2/6] 清理 release 目录...
echo ========================================
echo.

if exist "release" (
    echo [信息] 删除 release 目录...
    rmdir /s /q release
    if exist "release" (
        echo [警告] release 目录删除失败
    ) else (
        echo [✓] release 目录已删除
    )
) else (
    echo [信息] release 目录不存在
)
echo.

echo ========================================
echo [3/6] 清理日志文件...
echo ========================================
echo.

set "deleted=0"
for /r %%f in (*.log) do (
    if exist "%%f" (
        del /q "%%f"
        echo [删除] %%f
        set /a deleted+=1
    )
)
if %deleted%==0 (
    echo [信息] 未发现日志文件
) else (
    echo [✓] 已删除 %deleted% 个日志文件
)
echo.

echo ========================================
echo [4/6] 清理临时文件...
echo ========================================
echo.

set "deleted=0"
for /r %%f in (*.tmp *.bak *.old) do (
    if exist "%%f" (
        del /q "%%f"
        echo [删除] %%f
        set /a deleted+=1
    )
)
if %deleted%==0 (
    echo [信息] 未发现临时文件
) else (
    echo [✓] 已删除 %deleted% 个临时文件
)
echo.

echo ========================================
echo [5/6] 清理系统文件...
echo ========================================
echo.

set "deleted=0"
for /r %%f in (.DS_Store Thumbs.db) do (
    if exist "%%f" (
        del /q "%%f"
        echo [删除] %%f
        set /a deleted+=1
    )
)
if %deleted%==0 (
    echo [信息] 未发现系统文件
) else (
    echo [✓] 已删除 %deleted% 个系统文件
)
echo.

echo ========================================
echo [6/6] 清理 Cargo 缓存...
echo ========================================
echo.

echo [提示] Cargo 缓存位于：%USERPROFILE%\.cargo\registry
echo [提示] 如需清理，请运行：cargo clean --release
echo [跳过] 保留 Cargo 缓存以加速下次编译
echo.

echo ========================================
echo   清理完成！
echo ========================================
echo.

echo [总结]:
if not exist "target" (
    echo   ✓ target 目录已清理
)
if not exist "release" (
    echo   ✓ release 目录已清理
)
echo   ✓ 日志文件已清理
echo   ✓ 临时文件已清理
echo   ✓ 系统文件已清理
echo.

echo [提示] 现在可以运行以下命令：
echo   cargo build --release    - 重新构建
echo   .\打包发布.bat           - 打包发布
echo.

pause
