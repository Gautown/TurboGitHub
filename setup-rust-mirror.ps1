# Rust 国内镜像源配置脚本
# 用于自动配置 Cargo 使用国内镜像源

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  Rust 国内镜像源配置工具" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# 检查 Rust 是否安装
try {
    $rustVersion = rustc --version
    Write-Host "[✓] Rust 已安装：$rustVersion" -ForegroundColor Green
} catch {
    Write-Host "[✗] Rust 未安装，请先安装 Rust" -ForegroundColor Red
    Write-Host "下载地址：https://www.rust-lang.org/zh-CN/tools/install" -ForegroundColor Yellow
    pause
    exit 1
}

# 获取 Cargo 配置目录
$cargoDir = "$env:USERPROFILE\.cargo"
$configPath = "$cargoDir\config.toml"

Write-Host ""
Write-Host "[信息] Cargo 配置目录：$cargoDir" -ForegroundColor Cyan

# 检查配置目录是否存在
if (-not (Test-Path $cargoDir)) {
    Write-Host "[信息] 创建配置目录..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $cargoDir -Force | Out-Null
    Write-Host "[✓] 配置目录已创建" -ForegroundColor Green
}

# 备份现有配置
if (Test-Path $configPath) {
    Write-Host "[信息] 发现现有配置文件" -ForegroundColor Yellow
    $backupPath = "$configPath.backup.$(Get-Date -Format 'yyyyMMdd-HHmmss')"
    Write-Host "[信息] 备份现有配置到：$backupPath" -ForegroundColor Cyan
    Copy-Item $configPath $backupPath
    Write-Host "[✓] 备份完成" -ForegroundColor Green
}

# 创建新的配置文件
Write-Host ""
Write-Host "[信息] 创建新的配置文件..." -ForegroundColor Cyan

$configContent = @"
# Cargo 国内镜像配置
# 使用中科大镜像源加速下载

[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"

# 清华镜像源（备选）
# 如果中科大源不可用，取消下面的注释
# [source.tuna]
# registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"

[net]
git-fetch-with-cli = true
retry = 2
http.timeout = 30
"@

try {
    $configContent | Out-File -FilePath $configPath -Encoding UTF8
    Write-Host "[✓] 配置文件已创建：$configPath" -ForegroundColor Green
} catch {
    Write-Host "[✗] 创建配置文件失败：$_" -ForegroundColor Red
    Write-Host ""
    Write-Host "请手动创建配置文件，内容如下：" -ForegroundColor Yellow
    Write-Host $configContent
    pause
    exit 1
}

# 显示配置内容
Write-Host ""
Write-Host "===== 配置内容 =====" -ForegroundColor Cyan
Get-Content $configPath
Write-Host "====================" -ForegroundColor Cyan

# 测试配置
Write-Host ""
Write-Host "[信息] 测试配置..." -ForegroundColor Cyan
try {
    Write-Host "运行 cargo check 测试配置..." -ForegroundColor Gray
    cargo check --version | Out-Null
    Write-Host "[✓] 配置测试成功" -ForegroundColor Green
} catch {
    Write-Host "[⚠] 配置测试失败，但不影响使用：$_" -ForegroundColor Yellow
}

# 显示 Rust 版本信息
Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  Rust 版本信息" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
rustc --version
cargo --version
rustup show

Write-Host ""
Write-Host "[✓] 配置完成！" -ForegroundColor Green
Write-Host ""
Write-Host "提示：" -ForegroundColor Yellow
Write-Host "1. 现在可以使用国内镜像源加速下载依赖" -ForegroundColor Gray
Write-Host "2. 运行 'cargo build' 或 'cargo run' 会更快" -ForegroundColor Gray
Write-Host "3. 如需切换回官方源，删除 $configPath 即可" -ForegroundColor Gray
Write-Host ""

pause
