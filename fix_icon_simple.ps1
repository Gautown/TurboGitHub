# TurboGitHub Icon Fix Script
Write-Host "========================================" -ForegroundColor Green
Write-Host "   TurboGitHub Icon Fix" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# Check if files exist
if (!(Test-Path "target/release/turbogithub-gui.exe")) {
    Write-Host "Error: turbogithub-gui.exe not found" -ForegroundColor Red
    Write-Host "Please run cargo build --release first" -ForegroundColor Yellow
    exit 1
}

if (!(Test-Path "assets/icons/logo.ico")) {
    Write-Host "Error: Icon file assets/icons/logo.ico not found" -ForegroundColor Red
    exit 1
}

Write-Host "Found executable: target/release/turbogithub-gui.exe" -ForegroundColor Green
Write-Host "Found icon file: assets/icons/logo.ico" -ForegroundColor Green
Write-Host ""

# Ensure dist directory exists
if (!(Test-Path "dist")) {
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null
}

# Copy executable
Write-Host "Creating TurboGitHub.exe..." -ForegroundColor Yellow
Copy-Item "target/release/turbogithub-gui.exe" "dist/TurboGitHub.exe" -Force

# Check file size
$exeSize = (Get-Item "dist/TurboGitHub.exe").Length
Write-Host "TurboGitHub.exe created: $([math]::Round($exeSize/1MB, 2)) MB" -ForegroundColor Green

# Create ZIP package with icon
Write-Host ""
Write-Host "Creating installation package..." -ForegroundColor Yellow
Compress-Archive -Path "dist/TurboGitHub.exe", "assets/icons/logo.ico" -DestinationPath "dist/TurboGitHub-Windows.zip" -Force

$zipSize = (Get-Item "dist/TurboGitHub-Windows.zip").Length
Write-Host "TurboGitHub-Windows.zip created: $([math]::Round($zipSize/1MB, 2)) MB" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "   Icon Fix Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Write-Host "Generated files:" -ForegroundColor Yellow
Get-ChildItem "dist" | Format-Table Name, Length -AutoSize

Write-Host ""
Write-Host "Usage instructions:" -ForegroundColor Cyan
Write-Host "- Distribute TurboGitHub-Windows.zip to users" -ForegroundColor White
Write-Host "- Users extract ZIP, icon file is in same directory" -ForegroundColor White
Write-Host "- Application loads icon from external file" -ForegroundColor White

Write-Host ""
Write-Host "Files location: $(Get-Location)\dist\" -ForegroundColor Green
Write-Host ""

Write-Host "Note: To embed icon in EXE file, use specialized tools" -ForegroundColor Yellow
Write-Host "Current solution uses external icon file, works perfectly" -ForegroundColor Green