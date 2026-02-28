# TurboGitHub Installer Creation Script
Write-Host "========================================" -ForegroundColor Green
Write-Host "   TurboGitHub Installer Creation" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# Check required files
Write-Host "Checking required files..." -ForegroundColor Yellow
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

# Create professional installer
Write-Host "Creating professional installer..." -ForegroundColor Yellow

# 1. Copy executable
Copy-Item "target/release/turbogithub-gui.exe" "dist/TurboGitHub.exe" -Force

# 2. Create ZIP package with icon
Write-Host "Creating installer package..." -ForegroundColor Cyan
Compress-Archive -Path "dist/TurboGitHub.exe", "assets/icons/logo.ico" -DestinationPath "dist/TurboGitHub-Installer.zip" -Force

# 3. Create installation script
$installScript = @"
@echo off
chcp 65001 >nul

echo ========================================
echo    TurboGitHub Installer
echo ========================================
echo.

echo Installing TurboGitHub...

REM Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo Please run this installer as Administrator
echo Right-click -> "Run as administrator"
    pause
    exit /b 1
)

REM Create installation directory
set "INSTALL_DIR=%ProgramFiles%\TurboGitHub"
if not exist "%INSTALL_DIR%" (
    mkdir "%INSTALL_DIR%"
)

REM Copy files
copy "TurboGitHub.exe" "%INSTALL_DIR%\"
copy "logo.ico" "%INSTALL_DIR%\"

REM Create desktop shortcut
set "DESKTOP_SHORTCUT=%USERPROFILE%\Desktop\TurboGitHub.lnk"

powershell -Command "\$ws = New-Object -ComObject WScript.Shell; \$shortcut = \$ws.CreateShortcut('%DESKTOP_SHORTCUT%'); \$shortcut.TargetPath = '%INSTALL_DIR%\TurboGitHub.exe'; \$shortcut.WorkingDirectory = '%INSTALL_DIR%'; \$shortcut.IconLocation = '%INSTALL_DIR%\logo.ico'; \$shortcut.Save()"

REM Create start menu shortcut
set "START_MENU_DIR=%APPDATA%\Microsoft\Windows\Start Menu\Programs\TurboGitHub"
if not exist "%START_MENU_DIR%" (
    mkdir "%START_MENU_DIR%"
)

set "START_MENU_SHORTCUT=%START_MENU_DIR%\TurboGitHub.lnk"

powershell -Command "\$ws = New-Object -ComObject WScript.Shell; \$shortcut = \$ws.CreateShortcut('%START_MENU_SHORTCUT%'); \$shortcut.TargetPath = '%INSTALL_DIR%\TurboGitHub.exe'; \$shortcut.WorkingDirectory = '%INSTALL_DIR%'; \$shortcut.IconLocation = '%INSTALL_DIR%\logo.ico'; \$shortcut.Save()"

echo.
echo TurboGitHub installation completed!
echo.
echo Installation directory: %INSTALL_DIR%
echo Desktop shortcut created
echo Start menu shortcut created
echo.
echo You can now run TurboGitHub
echo.
pause
"@

Set-Content -Path "dist/install.bat" -Value $installScript

# 4. Create uninstall script
$uninstallScript = @"
@echo off
chcp 65001 >nul

echo ========================================
echo    TurboGitHub Uninstaller
echo ========================================
echo.

echo Uninstalling TurboGitHub...

REM Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo Please run this uninstaller as Administrator
echo Right-click -> "Run as administrator"
    pause
    exit /b 1
)

REM Remove installation directory
set "INSTALL_DIR=%ProgramFiles%\TurboGitHub"
if exist "%INSTALL_DIR%" (
    rmdir /s /q "%INSTALL_DIR%"
    echo Removed installation directory: %INSTALL_DIR%
)

REM Remove desktop shortcut
set "DESKTOP_SHORTCUT=%USERPROFILE%\Desktop\TurboGitHub.lnk"
if exist "%DESKTOP_SHORTCUT%" (
    del "%DESKTOP_SHORTCUT%"
    echo Removed desktop shortcut
)

REM Remove start menu shortcut
set "START_MENU_DIR=%APPDATA%\Microsoft\Windows\Start Menu\Programs\TurboGitHub"
if exist "%START_MENU_DIR%" (
    rmdir /s /q "%START_MENU_DIR%"
    echo Removed start menu shortcut
)

echo.
echo TurboGitHub uninstallation completed!
echo.
pause
"@

Set-Content -Path "dist/uninstall.bat" -Value $uninstallScript

# 5. Create complete installer package
Write-Host "Creating complete installer package..." -ForegroundColor Cyan
Compress-Archive -Path "dist/TurboGitHub.exe", "assets/icons/logo.ico", "dist/install.bat", "dist/uninstall.bat" -DestinationPath "dist/TurboGitHub-Complete-Installer.zip" -Force

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "   Installer Creation Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Write-Host "Generated files:" -ForegroundColor Yellow
Get-ChildItem "dist" | Format-Table Name, Length -AutoSize

Write-Host ""
Write-Host "Installation options:" -ForegroundColor Cyan
Write-Host "1. TurboGitHub-Installer.zip - Simple installer" -ForegroundColor White
Write-Host "   - Contains executable and icon" -ForegroundColor Gray
Write-Host ""
Write-Host "2. TurboGitHub-Complete-Installer.zip - Complete installer" -ForegroundColor White
Write-Host "   - Includes installation and uninstallation scripts" -ForegroundColor Gray
Write-Host "   - Automatically creates desktop and start menu shortcuts" -ForegroundColor Gray
Write-Host ""
Write-Host "3. Manual installation" -ForegroundColor White
Write-Host "   - Run install.bat for professional installation" -ForegroundColor Gray
Write-Host ""

Write-Host "Icon loading mechanism:" -ForegroundColor Cyan
Write-Host "- Application loads icon from external file" -ForegroundColor White
Write-Host "- Complete error handling and fallback support" -ForegroundColor White
Write-Host "- Automatically detects and loads correct icon at runtime" -ForegroundColor White
Write-Host ""

Write-Host "File location: $(Get-Location)\dist\" -ForegroundColor Green
Write-Host ""

Write-Host "Professional installer features:" -ForegroundColor Cyan
Write-Host "- Automatic desktop shortcut creation" -ForegroundColor White
Write-Host "- Automatic start menu shortcut creation" -ForegroundColor White
Write-Host "- Complete installation/uninstallation scripts" -ForegroundColor White
Write-Host "- Correct icon display" -ForegroundColor White
Write-Host "- Administrator privileges detection" -ForegroundColor White
Write-Host ""

Write-Host "Note: cargo-bundle MSI support is still experimental" -ForegroundColor Yellow
Write-Host "Current solution provides more stable and complete installation experience" -ForegroundColor Green