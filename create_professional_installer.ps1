# TurboGitHub Professional Installer Creation Script
Write-Host "========================================" -ForegroundColor Green
Write-Host "   TurboGitHub Professional Installer" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# 检查必要的文件
Write-Host "检查必要文件..." -ForegroundColor Yellow
if (!(Test-Path "target/release/turbogithub-gui.exe")) {
    Write-Host "错误: 未找到 turbogithub-gui.exe" -ForegroundColor Red
    Write-Host "请先运行 cargo build --release" -ForegroundColor Yellow
    exit 1
}

if (!(Test-Path "assets/icons/logo.ico")) {
    Write-Host "错误: 未找到图标文件 assets/icons/logo.ico" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 找到可执行文件: target/release/turbogithub-gui.exe" -ForegroundColor Green
Write-Host "✅ 找到图标文件: assets/icons/logo.ico" -ForegroundColor Green
Write-Host ""

# 确保dist目录存在
if (!(Test-Path "dist")) {
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null
}

# 创建专业安装包
Write-Host "创建专业安装包..." -ForegroundColor Yellow

# 1. 复制可执行文件
Copy-Item "target/release/turbogithub-gui.exe" "dist/TurboGitHub.exe" -Force

# 2. 创建包含图标的ZIP包
Write-Host "创建包含图标的安装包..." -ForegroundColor Cyan
Compress-Archive -Path "dist/TurboGitHub.exe", "assets/icons/logo.ico" -DestinationPath "dist/TurboGitHub-Installer.zip" -Force

# 3. 创建安装脚本
$installScript = @"
@echo off
chcp 65001 >nul

echo ========================================
echo    TurboGitHub 安装程序
echo ========================================
echo.

echo 正在安装 TurboGitHub...

REM 检查是否以管理员权限运行
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo 请以管理员权限运行此安装程序
echo 右键点击 -> "以管理员身份运行"
    pause
    exit /b 1
)

REM 创建安装目录
set "INSTALL_DIR=%ProgramFiles%\TurboGitHub"
if not exist "%INSTALL_DIR%" (
    mkdir "%INSTALL_DIR%"
)

REM 复制文件
copy "TurboGitHub.exe" "%INSTALL_DIR%\"
copy "logo.ico" "%INSTALL_DIR%\"

REM 创建桌面快捷方式
set "DESKTOP_SHORTCUT=%USERPROFILE%\Desktop\TurboGitHub.lnk"

powershell -Command "\$ws = New-Object -ComObject WScript.Shell; \$shortcut = \$ws.CreateShortcut('%DESKTOP_SHORTCUT%'); \$shortcut.TargetPath = '%INSTALL_DIR%\TurboGitHub.exe'; \$shortcut.WorkingDirectory = '%INSTALL_DIR%'; \$shortcut.IconLocation = '%INSTALL_DIR%\logo.ico'; \$shortcut.Save()"

REM 创建开始菜单快捷方式
set "START_MENU_DIR=%APPDATA%\Microsoft\Windows\Start Menu\Programs\TurboGitHub"
if not exist "%START_MENU_DIR%" (
    mkdir "%START_MENU_DIR%"
)

set "START_MENU_SHORTCUT=%START_MENU_DIR%\TurboGitHub.lnk"

powershell -Command "\$ws = New-Object -ComObject WScript.Shell; \$shortcut = \$ws.CreateShortcut('%START_MENU_SHORTCUT%'); \$shortcut.TargetPath = '%INSTALL_DIR%\TurboGitHub.exe'; \$shortcut.WorkingDirectory = '%INSTALL_DIR%'; \$shortcut.IconLocation = '%INSTALL_DIR%\logo.ico'; \$shortcut.Save()"

echo.
echo ✅ TurboGitHub 安装完成！
echo.
echo 安装位置: %INSTALL_DIR%
echo 桌面快捷方式已创建
echo 开始菜单快捷方式已创建
echo.
echo 现在可以运行 TurboGitHub 了
echo.
pause
"@

Set-Content -Path "dist/install.bat" -Value $installScript -Encoding UTF8

# 4. 创建卸载脚本
$uninstallScript = @"
@echo off
chcp 65001 >nul

echo ========================================
echo    TurboGitHub 卸载程序
echo ========================================
echo.

echo 正在卸载 TurboGitHub...

REM 检查是否以管理员权限运行
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo 请以管理员权限运行此卸载程序
echo 右键点击 -> "以管理员身份运行"
    pause
    exit /b 1
)

REM 删除安装目录
set "INSTALL_DIR=%ProgramFiles%\TurboGitHub"
if exist "%INSTALL_DIR%" (
    rmdir /s /q "%INSTALL_DIR%"
    echo ✅ 删除安装目录: %INSTALL_DIR%
)

REM 删除桌面快捷方式
set "DESKTOP_SHORTCUT=%USERPROFILE%\Desktop\TurboGitHub.lnk"
if exist "%DESKTOP_SHORTCUT%" (
    del "%DESKTOP_SHORTCUT%"
    echo ✅ 删除桌面快捷方式
)

REM 删除开始菜单快捷方式
set "START_MENU_DIR=%APPDATA%\Microsoft\Windows\Start Menu\Programs\TurboGitHub"
if exist "%START_MENU_DIR%" (
    rmdir /s /q "%START_MENU_DIR%"
    echo ✅ 删除开始菜单快捷方式
)

echo.
echo ✅ TurboGitHub 卸载完成！
echo.
pause
"@

Set-Content -Path "dist/uninstall.bat" -Value $uninstallScript -Encoding UTF8

# 5. 创建完整的安装包
Write-Host "创建完整安装包..." -ForegroundColor Cyan
Compress-Archive -Path "dist/TurboGitHub.exe", "assets/icons/logo.ico", "dist/install.bat", "dist/uninstall.bat" -DestinationPath "dist/TurboGitHub-Full-Installer.zip" -Force

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "   专业安装包创建完成!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Write-Host "生成的文件:" -ForegroundColor Yellow
Get-ChildItem "dist" | Format-Table Name, Length -AutoSize

Write-Host ""
Write-Host "安装选项:" -ForegroundColor Cyan
Write-Host "1. TurboGitHub-Installer.zip - 简单安装包" -ForegroundColor White
Write-Host "   - 包含可执行文件和图标" -ForegroundColor Gray
Write-Host ""
Write-Host "2. TurboGitHub-Full-Installer.zip - 完整安装包" -ForegroundColor White
Write-Host "   - 包含安装脚本、卸载脚本和所有必要文件" -ForegroundColor Gray
Write-Host "   - 自动创建桌面和开始菜单快捷方式" -ForegroundColor Gray
Write-Host ""
Write-Host "3. 手动安装" -ForegroundColor White
Write-Host "   - 运行 install.bat 进行专业安装" -ForegroundColor Gray
Write-Host ""

Write-Host "图标加载机制:" -ForegroundColor Cyan
Write-Host "- 应用程序从外部文件加载图标" -ForegroundColor White
Write-Host "- 支持完整的错误处理和备用方案" -ForegroundColor White
Write-Host "- 运行时自动检测并加载正确图标" -ForegroundColor White
Write-Host ""

Write-Host "文件位置: $(Get-Location)\dist\" -ForegroundColor Green
Write-Host ""

Write-Host "专业安装包特点:" -ForegroundColor Cyan
Write-Host "✅ 自动创建桌面快捷方式" -ForegroundColor White
Write-Host "✅ 自动创建开始菜单快捷方式" -ForegroundColor White
Write-Host "✅ 完整的安装/卸载脚本" -ForegroundColor White
Write-Host "✅ 图标正确显示" -ForegroundColor White
Write-Host "✅ 管理员权限检测" -ForegroundColor White
Write-Host ""

Write-Host "注意: cargo-bundle的MSI支持仍在实验阶段" -ForegroundColor Yellow
Write-Host "当前解决方案提供更稳定和完整的安装体验" -ForegroundColor Green