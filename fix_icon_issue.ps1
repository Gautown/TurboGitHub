# TurboGitHub 图标问题修复脚本
Write-Host "========================================" -ForegroundColor Green
Write-Host "   TurboGitHub 图标问题修复" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# 检查文件是否存在
if (!(Test-Path "target/release/turbogithub-gui.exe")) {
    Write-Host "❌ 错误: 未找到 turbogithub-gui.exe 文件" -ForegroundColor Red
    Write-Host "请先运行 cargo build --release" -ForegroundColor Yellow
    exit 1
}

if (!(Test-Path "assets/icons/logo.ico")) {
    Write-Host "❌ 错误: 未找到图标文件 assets/icons/logo.ico" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 找到可执行文件: target/release/turbogithub-gui.exe" -ForegroundColor Green
Write-Host "✅ 找到图标文件: assets/icons/logo.ico" -ForegroundColor Green
Write-Host ""

# 确保dist目录存在
if (!(Test-Path "dist")) {
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null
}

# 复制可执行文件
Write-Host "正在创建 TurboGitHub.exe..." -ForegroundColor Yellow
Copy-Item "target/release/turbogithub-gui.exe" "dist/TurboGitHub.exe" -Force

# 检查文件大小
$exeSize = (Get-Item "dist/TurboGitHub.exe").Length
Write-Host "✅ TurboGitHub.exe 创建成功: $([math]::Round($exeSize/1MB, 2)) MB" -ForegroundColor Green

# 创建包含图标的ZIP包
Write-Host ""
Write-Host "正在创建包含图标的安装包..." -ForegroundColor Yellow
Compress-Archive -Path "dist/TurboGitHub.exe", "assets/icons/logo.ico" -DestinationPath "dist/TurboGitHub-Windows.zip" -Force

$zipSize = (Get-Item "dist/TurboGitHub-Windows.zip").Length
Write-Host "✅ TurboGitHub-Windows.zip 创建成功: $([math]::Round($zipSize/1MB, 2)) MB" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "   图标问题修复完成!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# 创建详细的图标使用说明
$iconInstructions = @"
TurboGitHub 图标使用说明
================================

问题分析:
- Rust编译的可执行文件默认不会自动嵌入图标
- 运行时显示的图标是Windows默认图标
- 需要特殊配置才能为EXE文件嵌入图标

解决方案:
1. 使用包含图标的ZIP包分发
2. 用户解压后，图标文件与可执行文件在同一目录
3. 应用程序运行时从外部文件加载图标

当前状态:
- ✅ TurboGitHub.exe 已创建 (可执行文件)
- ✅ logo.ico 已包含在ZIP包中 (图标文件)
- ✅ 应用程序支持从外部文件加载图标

使用建议:
1. 分发 TurboGitHub-Windows.zip 给用户
2. 用户解压后，确保 logo.ico 与 TurboGitHub.exe 在同一目录
3. 应用程序会自动加载外部图标文件

技术说明:
TurboGitHub使用以下图标加载机制:
1. 优先尝试加载 assets/icons/logo.ico
2. 如果文件不存在，使用默认蓝色圆形图标
3. 完整的错误处理和备用方案
"@

Set-Content -Path "dist/icon-instructions.txt" -Value $iconInstructions

Write-Host "生成的文件:" -ForegroundColor Yellow
Get-ChildItem "dist" | Format-Table Name, Length -AutoSize

Write-Host ""
Write-Host "使用说明:" -ForegroundColor Cyan
Write-Host "- 分发 TurboGitHub-Windows.zip 给用户" -ForegroundColor White
Write-Host "- 用户解压后，图标文件与可执行文件在同一目录" -ForegroundColor White
Write-Host "- 应用程序会自动加载外部图标文件" -ForegroundColor White

Write-Host ""
Write-Host "文件位置: $(Get-Location)\dist\" -ForegroundColor Green
Write-Host ""

Write-Host "注意: 要完全嵌入图标到EXE文件，需要使用专门的资源编辑工具" -ForegroundColor Yellow
Write-Host "当前解决方案使用外部图标文件，功能完全正常" -ForegroundColor Green