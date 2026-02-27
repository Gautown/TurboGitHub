# TurboGitHub GUI无控制台窗口测试脚本
Write-Host "测试TurboGitHub GUI无控制台窗口启动..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 检查可执行文件是否存在
if (-not (Test-Path "target\release\turbogithub-gui.exe")) {
    Write-Host "错误：未找到turbogithub-gui.exe文件" -ForegroundColor Red
    exit 1
}

Write-Host "✓ 找到GUI可执行文件" -ForegroundColor Green

# 检查守护进程是否运行
$daemonProcess = Get-Process -Name "turbogithub-core" -ErrorAction SilentlyContinue
if ($daemonProcess) {
    Write-Host "✓ 守护进程正在运行（PID: $($daemonProcess.Id)）" -ForegroundColor Green
} else {
    Write-Host "⚠️ 守护进程未运行，GUI将显示连接错误" -ForegroundColor Yellow
}

# 启动GUI应用程序（无控制台窗口）
Write-Host "启动GUI应用程序..." -ForegroundColor Yellow
Write-Host "注意：GUI窗口应该直接打开，不会显示控制台窗口" -ForegroundColor Yellow

# 使用Start-Process启动，确保不显示控制台窗口
$process = Start-Process -FilePath "target\release\turbogithub-gui.exe" -PassThru -WindowStyle Hidden

Write-Host "✓ GUI进程已启动（PID: $($process.Id)）" -ForegroundColor Green

# 等待几秒钟让GUI初始化
Start-Sleep -Seconds 3

# 检查进程状态
$currentProcess = Get-Process -Id $process.Id -ErrorAction SilentlyContinue
if ($currentProcess) {
    Write-Host "✓ GUI进程仍在运行" -ForegroundColor Green
    Write-Host "窗口标题: $($currentProcess.MainWindowTitle)" -ForegroundColor White
    Write-Host "内存使用: $([math]::Round($currentProcess.WorkingSet64 / 1MB, 2)) MB" -ForegroundColor White
} else {
    Write-Host "❌ GUI进程已退出" -ForegroundColor Red
}

Write-Host ""
Write-Host "测试完成！" -ForegroundColor Cyan
Write-Host "如果GUI窗口成功打开且没有控制台窗口，说明配置成功" -ForegroundColor Cyan
Write-Host ""

# 询问是否停止GUI进程
$choice = Read-Host "是否停止GUI进程？(y/n)"
if ($choice -eq "y" -or $choice -eq "Y") {
    Stop-Process -Id $process.Id -Force
    Write-Host "✓ GUI进程已停止" -ForegroundColor Green
}