# TurboGitHub 国内源配置验证脚本
# 用于检查和验证所有国内源配置是否正确

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  TurboGitHub 国内源配置验证工具" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$projectRoot = $PSScriptRoot
$cargoConfig = "$projectRoot\.cargo\config.toml"

# ========================================
# 1. 检查 Rust 工具链
# ========================================
Write-Host "[1/6] 检查 Rust 工具链..." -ForegroundColor Yellow

try {
    $rustVersion = rustc --version
    $cargoVersion = cargo --version
    $rustupVersion = rustup --version
    
    Write-Host "  ✓ Rust: $rustVersion" -ForegroundColor Green
    Write-Host "  ✓ Cargo: $cargoVersion" -ForegroundColor Green
    Write-Host "  ✓ Rustup: $rustupVersion" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Rust 工具链未安装或配置错误" -ForegroundColor Red
    Write-Host "  提示：请运行 .\setup-rust-mirror.ps1 安装和配置" -ForegroundColor Yellow
    pause
    exit 1
}

Write-Host ""

# ========================================
# 2. 检查项目级 Cargo 配置
# ========================================
Write-Host "[2/6] 检查项目级 Cargo 配置..." -ForegroundColor Yellow

if (Test-Path $cargoConfig) {
    Write-Host "  ✓ 配置文件存在：$cargoConfig" -ForegroundColor Green
    
    $configContent = Get-Content $cargoConfig -Raw
    
    # 检查镜像源配置
    if ($configContent -match "mirrors\.ustc\.edu\.cn") {
        Write-Host "  ✓ 中科大镜像源已配置" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ 中科大镜像源未配置" -ForegroundColor Yellow
    }
    
    if ($configContent -match "mirrors\.tuna\.tsinghua\.edu\.cn") {
        Write-Host "  ✓ 清华镜像源已配置（备选）" -ForegroundColor Green
    }
    
    if ($configContent -match "mirrors\.aliyun\.com") {
        Write-Host "  ✓ 阿里云镜像源已配置（备选）" -ForegroundColor Green
    }
    
    # 检查网络配置
    if ($configContent -match "git-fetch-with-cli") {
        Write-Host "  ✓ Git CLI 模式已启用" -ForegroundColor Green
    }
    
    if ($configContent -match "retry") {
        Write-Host "  ✓ 重试机制已配置" -ForegroundColor Green
    }
} else {
    Write-Host "  ✗ 项目级 Cargo 配置文件不存在" -ForegroundColor Red
    Write-Host "  提示：配置文件位于 .cargo\config.toml" -ForegroundColor Yellow
}

Write-Host ""

# ========================================
# 3. 检查全局 Cargo 配置
# ========================================
Write-Host "[3/6] 检查全局 Cargo 配置..." -ForegroundColor Yellow

$globalConfig = "$env:USERPROFILE\.cargo\config.toml"
if (Test-Path $globalConfig) {
    Write-Host "  ✓ 全局配置文件存在：$globalConfig" -ForegroundColor Green
    
    $globalContent = Get-Content $globalConfig -Raw
    if ($globalContent -match "mirrors\.ustc\.edu\.cn") {
        Write-Host "  ✓ 全局已配置中科大镜像源" -ForegroundColor Green
    } elseif ($globalContent -match "mirrors\.tuna\.tsinghua\.edu\.cn") {
        Write-Host "  ✓ 全局已配置清华镜像源" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ 全局配置可能未使用国内镜像源" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ℹ 全局配置文件不存在（使用项目配置即可）" -ForegroundColor Cyan
}

Write-Host ""

# ========================================
# 4. 检查环境变量
# ========================================
Write-Host "[4/6] 检查环境变量配置..." -ForegroundColor Yellow

if ($env:RUSTUP_DIST_SERVER) {
    Write-Host "  ✓ RUSTUP_DIST_SERVER: $env:RUSTUP_DIST_SERVER" -ForegroundColor Green
    if ($env:RUSTUP_DIST_SERVER -match "mirrors\.ustc\.edu\.cn") {
        Write-Host "    ✓ 使用中科大源" -ForegroundColor Green
    } elseif ($env:RUSTUP_DIST_SERVER -match "mirrors\.tuna\.tsinghua\.edu\.cn") {
        Write-Host "    ✓ 使用清华源" -ForegroundColor Green
    }
} else {
    Write-Host "  ℹ RUSTUP_DIST_SERVER 未设置（使用配置文件的设置）" -ForegroundColor Cyan
}

if ($env:RUSTUP_UPDATE_ROOT) {
    Write-Host "  ✓ RUSTUP_UPDATE_ROOT: $env:RUSTUP_UPDATE_ROOT" -ForegroundColor Green
} else {
    Write-Host "  ℹ RUSTUP_UPDATE_ROOT 未设置" -ForegroundColor Cyan
}

Write-Host ""

# ========================================
# 5. 测试依赖下载
# ========================================
Write-Host "[5/6] 测试依赖下载速度..." -ForegroundColor Yellow

$testStart = Get-Date
try {
    Write-Host "  运行 cargo check（首次可能较慢）..." -ForegroundColor Gray
    $result = cargo check --message-format=short 2>&1
    $testEnd = Get-Date
    $duration = $testEnd - $testStart
    
    Write-Host "  ✓ 依赖检查完成，耗时：$($duration.TotalSeconds.ToString("F2")) 秒" -ForegroundColor Green
    
    if ($duration.TotalSeconds -lt 10) {
        Write-Host "  ✓ 速度优秀！" -ForegroundColor Green
    } elseif ($duration.TotalSeconds -lt 30) {
        Write-Host "  ✓ 速度正常" -ForegroundColor Cyan
    } else {
        Write-Host "  ⚠ 速度较慢，请检查网络连接" -ForegroundColor Yellow
    }
} catch {
    Write-Host "  ⚠ 依赖检查出现问题：$_" -ForegroundColor Yellow
    Write-Host "  提示：这可能是正常的，如果是首次运行" -ForegroundColor Cyan
}

Write-Host ""

# ========================================
# 6. 检查 Git 配置
# ========================================
Write-Host "[6/6] 检查 Git 配置..." -ForegroundColor Yellow

try {
    $gitConfig = git config --global --get-regexp "url\." 2>$null
    if ($gitConfig) {
        Write-Host "  ✓ Git 镜像配置：" -ForegroundColor Green
        foreach ($line in $gitConfig) {
            Write-Host "    - $line" -ForegroundColor Cyan
        }
    } else {
        Write-Host "  ℹ 未发现 Git 镜像配置（使用默认配置）" -ForegroundColor Cyan
    }
} catch {
    Write-Host "  ℹ Git 未安装或配置" -ForegroundColor Cyan
}

Write-Host ""

# ========================================
# 总结
# ========================================
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  验证完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 显示配置摘要
Write-Host "配置摘要:" -ForegroundColor Yellow
Write-Host "  • Rust 版本：$rustVersion" -ForegroundColor Gray
Write-Host "  • 项目配置：$(if (Test-Path $cargoConfig) { "✓" } else { "✗" })" -ForegroundColor Gray
Write-Host "  • 全局配置：$(if (Test-Path $globalConfig) { "✓" } else { "ℹ" })" -ForegroundColor Gray
Write-Host "  • 中科大源：$(if ((Get-Content $cargoConfig -Raw) -match "mirrors\.ustc\.edu\.cn") { "✓" } else { "✗" })" -ForegroundColor Gray
Write-Host ""

# 提供建议
Write-Host "建议:" -ForegroundColor Yellow

if (-not (Test-Path $cargoConfig)) {
    Write-Host "  1. 创建项目配置文件：.cargo\config.toml" -ForegroundColor Red
}

if ($env:RUSTUP_DIST_SERVER -notmatch "mirrors\.(ustc|tuna|aliyun)\.edu\.cn") {
    Write-Host "  2. 配置 Rustup 使用国内源" -ForegroundColor Yellow
    Write-Host "     运行：.\setup-rust-mirror.ps1" -ForegroundColor Gray
}

Write-Host "  3. 定期更新依赖：cargo update" -ForegroundColor Gray
Write-Host "  4. 查看详细说明：国内源配置说明.md" -ForegroundColor Gray

Write-Host ""
Write-Host "验证完成！" -ForegroundColor Green
Write-Host ""

pause
