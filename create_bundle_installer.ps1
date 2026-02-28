# TurboGitHub Bundle Installer Creation Script
Write-Host "========================================" -ForegroundColor Green
Write-Host "   TurboGitHub Bundle Installer Creation" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# Check if cargo-bundle is installed
Write-Host "Checking cargo-bundle installation..." -ForegroundColor Yellow
$bundleInstalled = cargo bundle --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "cargo-bundle not found, installing..." -ForegroundColor Yellow
    cargo install cargo-bundle
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install cargo-bundle" -ForegroundColor Red
        exit 1
    }
}

Write-Host "cargo-bundle is installed" -ForegroundColor Green
Write-Host ""

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "target/release") {
    Remove-Item -Recurse -Force "target/release"
}
if (Test-Path "bundle") {
    Remove-Item -Recurse -Force "bundle"
}

# Ensure dist directory exists
if (!(Test-Path "dist")) {
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null
}

# Build the application first
Write-Host "Building TurboGitHub..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed" -ForegroundColor Red
    exit 1
}

Write-Host "Build successful" -ForegroundColor Green
Write-Host ""

# Create bundle using cargo-bundle
Write-Host "Creating Windows MSI installer with cargo-bundle..." -ForegroundColor Yellow
Write-Host "This may take a few minutes..." -ForegroundColor Cyan

# Try different bundle formats
$bundleFormats = @("msi", "app")

foreach ($format in $bundleFormats) {
    Write-Host ""
    Write-Host "Attempting to create $format bundle..." -ForegroundColor Yellow
    
    cargo bundle --release --format $format
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "$format bundle created successfully" -ForegroundColor Green
        
        # Check what was created
        if (Test-Path "bundle") {
            Write-Host "Bundle contents:" -ForegroundColor Cyan
            Get-ChildItem "bundle" -Recurse | Format-Table Name, Length -AutoSize
            
            # Copy the created bundle to dist directory
            $bundleFiles = Get-ChildItem "bundle" -Recurse -File | Where-Object { $_.Name -like "*TurboGitHub*" }
            foreach ($file in $bundleFiles) {
                $destPath = Join-Path "dist" $file.Name
                Copy-Item $file.FullName $destPath -Force
                Write-Host "Copied: $($file.Name)" -ForegroundColor Green
            }
        }
        break
    } else {
        Write-Host "$format bundle creation failed" -ForegroundColor Red
    }
}

# If bundle creation failed, create a simple installer
if (!(Test-Path "bundle")) {
    Write-Host ""
    Write-Host "Bundle creation failed, creating simple installer..." -ForegroundColor Yellow
    
    # Copy executable
    Copy-Item "target/release/turbogithub-gui.exe" "dist/TurboGitHub.exe" -Force
    
    # Create ZIP package with icon
    Compress-Archive -Path "dist/TurboGitHub.exe", "assets/icons/logo.ico" -DestinationPath "dist/TurboGitHub-Simple.zip" -Force
    
    Write-Host "Simple installer created" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "   Installer Creation Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Write-Host "Generated files in dist directory:" -ForegroundColor Yellow
Get-ChildItem "dist" | Format-Table Name, Length -AutoSize

Write-Host ""
Write-Host "Installation options:" -ForegroundColor Cyan
if (Test-Path "dist\*.msi") {
    Write-Host "- MSI Installer: Professional Windows installer" -ForegroundColor White
}
if (Test-Path "dist\*.exe") {
    Write-Host "- Portable EXE: Direct executable" -ForegroundColor White
}
if (Test-Path "dist\*.zip") {
    Write-Host "- ZIP Package: Contains executable and icon" -ForegroundColor White
}

Write-Host ""
Write-Host "Files location: $(Get-Location)\dist\" -ForegroundColor Green
Write-Host ""

Write-Host "Icon loading mechanism:" -ForegroundColor Cyan
Write-Host "- Application loads icon from assets/icons/logo.ico" -ForegroundColor White
Write-Host "- If icon file not found, uses default blue circle" -ForegroundColor White
Write-Host "- Complete error handling and fallback" -ForegroundColor White